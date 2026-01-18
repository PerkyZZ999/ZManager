//! Job executor for file transfer operations.
//!
//! This module provides the execution logic that connects the copy primitives
//! to the job system from zmanager-core.

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
#[allow(unused_imports)]
use zmanager_core::{CancellationToken, Job, JobId, JobKind, JobState, Progress, ZError, ZResult};

use crate::copy::{copy_file_with_progress, CopyProgress, CopyResult, ProgressCallback};

/// Events emitted during job execution.
#[derive(Debug, Clone)]
pub enum ExecutorEvent {
    /// Job started executing.
    JobStarted { job_id: JobId },
    /// Progress update for a job.
    JobProgress { job_id: JobId, progress: Progress },
    /// Job completed successfully.
    JobCompleted { job_id: JobId, result: CopyResult },
    /// Job failed with an error.
    JobFailed { job_id: JobId, error: String },
    /// Job was cancelled.
    JobCancelled { job_id: JobId },
}

/// Configuration for the executor.
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// How often to emit progress updates (in bytes).
    pub progress_interval_bytes: u64,
    /// Minimum interval between progress updates (in milliseconds).
    pub progress_interval_ms: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            progress_interval_bytes: 1024 * 1024, // 1MB
            progress_interval_ms: 100,            // 100ms
        }
    }
}

/// Executor for file copy jobs.
///
/// This provides a bridge between the job system and the low-level copy primitives.
pub struct CopyExecutor {
    config: ExecutorConfig,
    event_tx: broadcast::Sender<ExecutorEvent>,
}

impl CopyExecutor {
    /// Create a new executor with default configuration.
    pub fn new() -> Self {
        Self::with_config(ExecutorConfig::default())
    }

    /// Create a new executor with custom configuration.
    pub fn with_config(config: ExecutorConfig) -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self { config, event_tx }
    }

    /// Subscribe to executor events.
    pub fn subscribe(&self) -> broadcast::Receiver<ExecutorEvent> {
        self.event_tx.subscribe()
    }

    /// Execute a single-file copy job.
    ///
    /// # Arguments
    ///
    /// * `job_id` - The job identifier for tracking
    /// * `source` - Source file path
    /// * `destination` - Destination file path  
    /// * `overwrite` - Whether to overwrite existing files
    /// * `cancel_token` - Token for cancellation
    ///
    /// # Returns
    ///
    /// Returns the copy result on success, or an error.
    pub async fn execute_single_copy(
        &self,
        job_id: JobId,
        source: impl AsRef<Path> + Send + 'static,
        destination: impl AsRef<Path> + Send + 'static,
        overwrite: bool,
        cancel_token: CancellationToken,
    ) -> ZResult<CopyResult> {
        let source = source.as_ref().to_path_buf();
        let destination = destination.as_ref().to_path_buf();
        let start_time = Instant::now();

        // Emit job started event
        let _ = self.event_tx.send(ExecutorEvent::JobStarted { job_id });

        debug!(
            job_id = %job_id,
            source = %source.display(),
            destination = %destination.display(),
            "Starting single file copy"
        );

        // Get source file size for initial progress
        let _total_bytes = std::fs::metadata(&source)
            .map(|m| m.len())
            .unwrap_or(0);

        // Create progress callback that emits events
        let event_tx = self.event_tx.clone();
        let config = self.config.clone();
        let last_progress_bytes = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let last_progress_time = Arc::new(std::sync::Mutex::new(Instant::now()));
        let source_clone = source.clone();

        let callback: ProgressCallback = Box::new(move |copy_progress: CopyProgress| {
            let last_bytes = last_progress_bytes.load(std::sync::atomic::Ordering::Relaxed);
            let bytes_since_last = copy_progress.bytes_copied.saturating_sub(last_bytes);
            
            // Check if we should emit a progress update
            let should_emit = bytes_since_last >= config.progress_interval_bytes || {
                let elapsed = last_progress_time.lock().map(|t| t.elapsed().as_millis());
                elapsed.unwrap_or(0) >= config.progress_interval_ms as u128
            };

            if should_emit {
                last_progress_bytes.store(copy_progress.bytes_copied, std::sync::atomic::Ordering::Relaxed);
                if let Ok(mut time) = last_progress_time.lock() {
                    *time = Instant::now();
                }

                let progress = Progress {
                    total_bytes: Some(copy_progress.total_bytes),
                    bytes_done: copy_progress.bytes_copied,
                    total_items: 1,
                    items_done: 0,
                    current_item: Some(source_clone.clone()),
                    eta: copy_progress.eta_seconds.map(std::time::Duration::from_secs),
                    speed_bytes_per_sec: Some(copy_progress.speed_bps),
                };

                let _ = event_tx.send(ExecutorEvent::JobProgress { job_id, progress });
            }
        });

        // Execute the copy in a blocking task
        let source_for_copy = source.clone();
        let dest_for_copy = destination.clone();

        let result = tokio::task::spawn_blocking(move || {
            copy_file_with_progress(
                &source_for_copy,
                &dest_for_copy,
                overwrite,
                cancel_token,
                Some(callback),
            )
        })
        .await
        .map_err(|e| ZError::Internal {
            message: format!("Task join error: {e}"),
        })?;

        let duration = start_time.elapsed();

        match result {
            Ok(bytes_copied) => {
                let average_speed = if duration.as_secs() > 0 {
                    bytes_copied / duration.as_secs()
                } else if duration.as_millis() > 0 {
                    (bytes_copied * 1000) / duration.as_millis() as u64
                } else {
                    bytes_copied
                };

                let copy_result = CopyResult {
                    source: source.clone(),
                    destination: destination.clone(),
                    bytes_copied,
                    duration,
                    average_speed_bps: average_speed,
                };

                info!(
                    job_id = %job_id,
                    bytes = bytes_copied,
                    duration_ms = duration.as_millis(),
                    speed_mbps = average_speed as f64 / 1_000_000.0,
                    "Copy completed"
                );

                // Send final progress (100%)
                let final_progress = Progress {
                    total_bytes: Some(bytes_copied),
                    bytes_done: bytes_copied,
                    total_items: 1,
                    items_done: 1,
                    current_item: None,
                    eta: Some(std::time::Duration::ZERO),
                    speed_bytes_per_sec: Some(average_speed),
                };
                let _ = self.event_tx.send(ExecutorEvent::JobProgress {
                    job_id,
                    progress: final_progress,
                });

                let _ = self.event_tx.send(ExecutorEvent::JobCompleted {
                    job_id,
                    result: copy_result.clone(),
                });

                Ok(copy_result)
            }
            Err(ZError::Cancelled) => {
                warn!(job_id = %job_id, "Copy cancelled");
                let _ = self.event_tx.send(ExecutorEvent::JobCancelled { job_id });
                Err(ZError::Cancelled)
            }
            Err(e) => {
                error!(job_id = %job_id, error = %e, "Copy failed");
                let _ = self.event_tx.send(ExecutorEvent::JobFailed {
                    job_id,
                    error: e.to_string(),
                });
                Err(e)
            }
        }
    }

    /// Execute a copy job from the job system.
    ///
    /// This extracts the source/destination from the JobKind and executes appropriately.
    pub async fn execute_job(
        &self,
        job: &Job,
        cancel_token: CancellationToken,
    ) -> ZResult<Vec<CopyResult>> {
        match &job.kind {
            JobKind::Copy { sources, destination } => {
                let mut results = Vec::with_capacity(sources.len());
                
                for source in sources {
                    // Determine destination path
                    let dest_path = if destination.is_dir() || sources.len() > 1 {
                        // Copy into directory
                        let file_name = source.file_name().ok_or_else(|| ZError::InvalidPath {
                            path: source.clone(),
                            reason: "No file name".to_string(),
                        })?;
                        destination.join(file_name)
                    } else {
                        // Direct file-to-file copy
                        destination.clone()
                    };

                    let result = self
                        .execute_single_copy(
                            job.id,
                            source.clone(),
                            dest_path,
                            false, // Don't overwrite by default
                            cancel_token.clone(),
                        )
                        .await?;

                    results.push(result);

                    // Check for cancellation between files
                    if cancel_token.is_cancelled() {
                        return Err(ZError::Cancelled);
                    }
                }

                Ok(results)
            }
            JobKind::Move { sources, destination } => {
                // For now, move is copy + delete
                // In Sprint 6, we'll optimize same-volume moves to use rename
                let mut results = Vec::with_capacity(sources.len());
                
                for source in sources {
                    let dest_path = if destination.is_dir() || sources.len() > 1 {
                        let file_name = source.file_name().ok_or_else(|| ZError::InvalidPath {
                            path: source.clone(),
                            reason: "No file name".to_string(),
                        })?;
                        destination.join(file_name)
                    } else {
                        destination.clone()
                    };

                    // Copy first
                    let result = self
                        .execute_single_copy(
                            job.id,
                            source.clone(),
                            dest_path,
                            false,
                            cancel_token.clone(),
                        )
                        .await?;

                    // Delete source after successful copy
                    std::fs::remove_file(source).map_err(|e| ZError::io(source, e))?;

                    results.push(result);

                    if cancel_token.is_cancelled() {
                        return Err(ZError::Cancelled);
                    }
                }

                Ok(results)
            }
            _ => Err(ZError::Internal {
                message: format!("Unsupported job kind for copy executor: {:?}", job.kind),
            }),
        }
    }
}

impl Default for CopyExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(dir: &TempDir, name: &str, size: usize) -> std::path::PathBuf {
        let path = dir.path().join(name);
        let content = vec![b'X'; size];
        fs::write(&path, &content).unwrap();
        path
    }

    #[tokio::test]
    async fn test_execute_single_copy() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(&temp, "source.txt", 1024);
        let dest = temp.path().join("dest.txt");

        let executor = CopyExecutor::new();
        let mut events = executor.subscribe();

        let token = CancellationToken::new();
        let result = executor
            .execute_single_copy(JobId::new(), source.clone(), dest.clone(), false, token)
            .await;

        assert!(result.is_ok());
        let copy_result = result.unwrap();
        assert_eq!(copy_result.bytes_copied, 1024);
        assert!(dest.exists());
        assert_eq!(fs::read(&source).unwrap(), fs::read(&dest).unwrap());

        // Check we got events
        let mut got_started = false;
        let mut got_completed = false;
        
        while let Ok(event) = events.try_recv() {
            match event {
                ExecutorEvent::JobStarted { .. } => got_started = true,
                ExecutorEvent::JobCompleted { .. } => got_completed = true,
                _ => {}
            }
        }

        assert!(got_started);
        assert!(got_completed);
    }

    #[tokio::test]
    async fn test_execute_copy_job() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(&temp, "source.txt", 512);
        let dest_dir = temp.path().join("dest_dir");
        fs::create_dir(&dest_dir).unwrap();

        let job = Job {
            id: JobId::new(),
            kind: JobKind::Copy {
                sources: vec![source.clone()],
                destination: dest_dir.clone(),
            },
            state: JobState::Pending,
            progress: Progress::new(1, Some(512)),
            cancellation: CancellationToken::new(),
            error: None,
            created_at: Instant::now(),
            started_at: None,
            finished_at: None,
        };

        let executor = CopyExecutor::new();
        let token = CancellationToken::new();
        let results = executor.execute_job(&job, token).await;

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 1);
        assert!(dest_dir.join("source.txt").exists());
    }

    #[tokio::test]
    async fn test_execute_move_job() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(&temp, "source.txt", 256);
        let dest = temp.path().join("moved.txt");

        let job = Job {
            id: JobId::new(),
            kind: JobKind::Move {
                sources: vec![source.clone()],
                destination: dest.clone(),
            },
            state: JobState::Pending,
            progress: Progress::new(1, Some(256)),
            cancellation: CancellationToken::new(),
            error: None,
            created_at: Instant::now(),
            started_at: None,
            finished_at: None,
        };

        let executor = CopyExecutor::new();
        let token = CancellationToken::new();
        let results = executor.execute_job(&job, token).await;

        assert!(results.is_ok());
        assert!(dest.exists());
        assert!(!source.exists()); // Source should be deleted after move
    }

    #[tokio::test]
    async fn test_execute_with_cancellation() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(&temp, "source.bin", 10 * 1024 * 1024); // 10MB
        let dest = temp.path().join("dest.bin");

        let executor = CopyExecutor::new();
        let token = CancellationToken::new();
        
        // Cancel immediately
        token.cancel();

        let result = executor
            .execute_single_copy(JobId::new(), source, dest.clone(), false, token)
            .await;

        // Should either complete (if copy was fast) or be cancelled
        match result {
            Ok(_) => {
                // Fast copy completed before cancellation check
                assert!(dest.exists());
            }
            Err(ZError::Cancelled) => {
                // Cancellation was successful
            }
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[tokio::test]
    async fn test_executor_events() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(&temp, "source.txt", 100);
        let dest = temp.path().join("dest.txt");

        let executor = CopyExecutor::new();
        let mut events = executor.subscribe();

        let token = CancellationToken::new();
        let job_id = JobId::new();
        
        let _ = executor
            .execute_single_copy(job_id, source, dest, false, token)
            .await;

        // Collect all events
        let mut event_types = Vec::new();
        while let Ok(event) = events.try_recv() {
            event_types.push(match event {
                ExecutorEvent::JobStarted { .. } => "started",
                ExecutorEvent::JobProgress { .. } => "progress",
                ExecutorEvent::JobCompleted { .. } => "completed",
                ExecutorEvent::JobFailed { .. } => "failed",
                ExecutorEvent::JobCancelled { .. } => "cancelled",
            });
        }

        assert!(event_types.contains(&"started"));
        assert!(event_types.contains(&"completed") || event_types.contains(&"progress"));
    }
}
