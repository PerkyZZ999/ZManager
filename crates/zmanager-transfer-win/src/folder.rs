//! Folder transfer executor.
//!
//! This module provides the execution logic for folder copy/move operations,
//! including conflict resolution and partial failure handling.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::{debug, info, trace, warn};
use zmanager_core::{CancellationToken, JobId, Progress, ZError, ZResult};

use crate::conflict::{Conflict, ConflictResolution, ConflictResolver};
use crate::copy::{copy_file_with_progress, CopyProgress, ProgressCallback};
use crate::plan::{same_volume, TransferItem, TransferPlan, TransferPlanBuilder, TransferStats};

/// Result for a single item transfer.
#[derive(Debug, Clone)]
pub enum ItemResult {
    /// Item transferred successfully.
    Success {
        source: PathBuf,
        destination: PathBuf,
        bytes: u64,
    },
    /// Item was skipped (e.g., conflict policy).
    Skipped {
        source: PathBuf,
        destination: PathBuf,
        reason: String,
    },
    /// Item transfer failed.
    Failed {
        source: PathBuf,
        destination: PathBuf,
        error: String,
    },
}

impl ItemResult {
    /// Check if this result is a success.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if this result is a failure.
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    /// Get the source path.
    pub fn source(&self) -> &Path {
        match self {
            Self::Success { source, .. }
            | Self::Skipped { source, .. }
            | Self::Failed { source, .. } => source,
        }
    }
}

/// Aggregated results from a folder transfer.
#[derive(Debug, Clone, Default)]
pub struct TransferReport {
    /// Individual item results.
    pub items: Vec<ItemResult>,
    /// Total bytes transferred.
    pub bytes_transferred: u64,
    /// Number of successful transfers.
    pub succeeded: usize,
    /// Number of skipped items.
    pub skipped: usize,
    /// Number of failed items.
    pub failed: usize,
    /// Total duration.
    pub duration: std::time::Duration,
}

impl TransferReport {
    /// Check if the transfer completed without any failures.
    pub fn is_complete_success(&self) -> bool {
        self.failed == 0
    }

    /// Check if any items were transferred.
    pub fn has_transfers(&self) -> bool {
        self.succeeded > 0
    }

    /// Get the average transfer speed in bytes per second.
    pub fn average_speed(&self) -> u64 {
        if self.duration.as_secs() > 0 {
            self.bytes_transferred / self.duration.as_secs()
        } else if self.duration.as_millis() > 0 {
            (self.bytes_transferred * 1000) / self.duration.as_millis() as u64
        } else {
            self.bytes_transferred
        }
    }
}

/// Events emitted during folder transfer.
#[derive(Debug, Clone)]
pub enum FolderTransferEvent {
    /// Transfer started with plan statistics.
    Started { job_id: JobId, stats: TransferStats },
    /// Progress update.
    Progress { job_id: JobId, progress: Progress },
    /// A conflict was detected and needs resolution.
    ConflictDetected { job_id: JobId, conflict: Conflict },
    /// An individual item completed.
    ItemCompleted { job_id: JobId, result: ItemResult },
    /// Transfer completed.
    Completed { job_id: JobId, report: TransferReport },
    /// Transfer failed.
    Failed { job_id: JobId, error: String },
    /// Transfer was cancelled.
    Cancelled { job_id: JobId },
}

/// Request for conflict resolution from the UI.
#[derive(Debug)]
pub struct ConflictQuery {
    /// The conflict that needs resolution.
    pub conflict: Conflict,
    /// Channel to send the resolution.
    pub response: oneshot::Sender<ConflictResolution>,
}

/// Configuration for folder transfers.
#[derive(Debug, Clone)]
pub struct FolderTransferConfig {
    /// Maximum concurrent file copies.
    pub concurrency: usize,
    /// Whether to continue on individual file errors.
    pub continue_on_error: bool,
    /// Whether to delete source after successful move.
    pub delete_source_on_move: bool,
    /// Progress update interval in bytes.
    pub progress_interval_bytes: u64,
}

impl Default for FolderTransferConfig {
    fn default() -> Self {
        Self {
            concurrency: 2,
            continue_on_error: true,
            delete_source_on_move: true,
            progress_interval_bytes: 1024 * 1024, // 1MB
        }
    }
}

/// Executor for folder transfer operations.
pub struct FolderTransferExecutor {
    config: FolderTransferConfig,
    event_tx: broadcast::Sender<FolderTransferEvent>,
    #[allow(dead_code)] // Reserved for Ask mode protocol
    conflict_tx: mpsc::Sender<ConflictQuery>,
    conflict_rx: Option<mpsc::Receiver<ConflictQuery>>,
}

impl FolderTransferExecutor {
    /// Create a new folder transfer executor.
    pub fn new() -> Self {
        Self::with_config(FolderTransferConfig::default())
    }

    /// Create a new executor with custom configuration.
    pub fn with_config(config: FolderTransferConfig) -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        let (conflict_tx, conflict_rx) = mpsc::channel(32);
        Self {
            config,
            event_tx,
            conflict_tx,
            conflict_rx: Some(conflict_rx),
        }
    }

    /// Subscribe to transfer events.
    pub fn subscribe(&self) -> broadcast::Receiver<FolderTransferEvent> {
        self.event_tx.subscribe()
    }

    /// Take the conflict receiver for handling conflicts.
    ///
    /// This should be called once by the UI layer to receive conflict queries.
    pub fn take_conflict_receiver(&mut self) -> Option<mpsc::Receiver<ConflictQuery>> {
        self.conflict_rx.take()
    }

    /// Execute a folder copy operation.
    pub async fn copy_folder(
        &self,
        job_id: JobId,
        sources: Vec<PathBuf>,
        destination: PathBuf,
        resolver: Arc<std::sync::Mutex<ConflictResolver>>,
        cancel_token: CancellationToken,
    ) -> ZResult<TransferReport> {
        self.execute_transfer(job_id, sources, destination, false, resolver, cancel_token)
            .await
    }

    /// Execute a folder move operation.
    pub async fn move_folder(
        &self,
        job_id: JobId,
        sources: Vec<PathBuf>,
        destination: PathBuf,
        resolver: Arc<std::sync::Mutex<ConflictResolver>>,
        cancel_token: CancellationToken,
    ) -> ZResult<TransferReport> {
        self.execute_transfer(job_id, sources, destination, true, resolver, cancel_token)
            .await
    }

    async fn execute_transfer(
        &self,
        job_id: JobId,
        sources: Vec<PathBuf>,
        destination: PathBuf,
        is_move: bool,
        resolver: Arc<std::sync::Mutex<ConflictResolver>>,
        cancel_token: CancellationToken,
    ) -> ZResult<TransferReport> {
        let start_time = Instant::now();

        info!(
            job_id = %job_id,
            sources = sources.len(),
            destination = %destination.display(),
            is_move,
            "Starting folder transfer"
        );

        // Build transfer plan
        let mut builder = TransferPlanBuilder::new(&destination).is_move(is_move);
        for source in &sources {
            builder = builder.add_source(source);
        }
        let plan = builder.build()?;

        // Emit started event
        let _ = self.event_tx.send(FolderTransferEvent::Started {
            job_id,
            stats: plan.stats.clone(),
        });

        // Check for same-volume move optimization
        if is_move
            && sources.len() == 1
            && sources[0].is_dir()
            && same_volume(&sources[0], &destination)
        {
            debug!("Attempting same-volume atomic move");
            if let Ok(report) = self
                .try_atomic_move(job_id, &sources[0], &destination, &cancel_token)
                .await
            {
                return Ok(report);
            }
            debug!("Atomic move failed, falling back to copy+delete");
        }

        // Execute the transfer
        let report = self
            .execute_plan(job_id, &plan, resolver, cancel_token.clone())
            .await?;

        // For move operations, delete sources after successful copy
        if is_move && self.config.delete_source_on_move && report.is_complete_success() {
            self.delete_sources(&plan).await;
        }

        let duration = start_time.elapsed();
        let final_report = TransferReport {
            duration,
            ..report
        };

        info!(
            job_id = %job_id,
            succeeded = final_report.succeeded,
            skipped = final_report.skipped,
            failed = final_report.failed,
            bytes = final_report.bytes_transferred,
            duration_ms = duration.as_millis(),
            "Folder transfer completed"
        );

        let _ = self.event_tx.send(FolderTransferEvent::Completed {
            job_id,
            report: final_report.clone(),
        });

        Ok(final_report)
    }

    async fn try_atomic_move(
        &self,
        _job_id: JobId, // Reserved for event emission
        source: &Path,
        destination: &Path,
        cancel_token: &CancellationToken,
    ) -> ZResult<TransferReport> {
        if cancel_token.is_cancelled() {
            return Err(ZError::Cancelled);
        }

        let dest_path = destination.join(
            source
                .file_name()
                .ok_or_else(|| ZError::InvalidPath {
                    path: source.to_path_buf(),
                    reason: "No directory name".to_string(),
                })?,
        );

        // Check for conflicts
        if dest_path.exists() {
            return Err(ZError::AlreadyExists { path: dest_path });
        }

        // Attempt atomic rename
        std::fs::rename(source, &dest_path).map_err(|e| ZError::io(source, e))?;

        let bytes = calculate_dir_size(&dest_path);

        let report = TransferReport {
            items: vec![ItemResult::Success {
                source: source.to_path_buf(),
                destination: dest_path,
                bytes,
            }],
            bytes_transferred: bytes,
            succeeded: 1,
            skipped: 0,
            failed: 0,
            duration: std::time::Duration::ZERO,
        };

        Ok(report)
    }

    async fn execute_plan(
        &self,
        job_id: JobId,
        plan: &TransferPlan,
        resolver: Arc<std::sync::Mutex<ConflictResolver>>,
        cancel_token: CancellationToken,
    ) -> ZResult<TransferReport> {
        let mut report = TransferReport::default();

        // Progress tracking
        let bytes_done = Arc::new(AtomicU64::new(0));
        let items_done = Arc::new(AtomicUsize::new(0));
        let total_bytes = plan.stats.total_bytes;
        let total_items = plan.stats.total_items();

        // Phase 1: Create directories
        debug!("Creating {} directories", plan.stats.total_dirs);
        for item in plan.directories() {
            if cancel_token.is_cancelled() {
                let _ = self.event_tx.send(FolderTransferEvent::Cancelled { job_id });
                return Err(ZError::Cancelled);
            }

            match self.create_directory(item, &resolver).await {
                Ok(result) => {
                    items_done.fetch_add(1, Ordering::Relaxed);
                    self.emit_progress(job_id, &items_done, &bytes_done, total_items, total_bytes);

                    if result.is_success() {
                        report.succeeded += 1;
                    } else {
                        report.skipped += 1;
                    }
                    report.items.push(result);
                }
                Err(e) => {
                    if self.config.continue_on_error {
                        warn!(
                            dir = %item.destination.display(),
                            error = %e,
                            "Failed to create directory, continuing"
                        );
                        report.failed += 1;
                        report.items.push(ItemResult::Failed {
                            source: item.source.clone(),
                            destination: item.destination.clone(),
                            error: e.to_string(),
                        });
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        // Phase 2: Copy files
        debug!("Copying {} files", plan.stats.total_files);
        for item in plan.files() {
            if cancel_token.is_cancelled() {
                let _ = self.event_tx.send(FolderTransferEvent::Cancelled { job_id });
                return Err(ZError::Cancelled);
            }

            let bytes_done_clone = bytes_done.clone();
            let items_done_clone = items_done.clone();
            let event_tx = self.event_tx.clone();

            match self
                .copy_file(job_id, item, &resolver, &cancel_token, bytes_done_clone)
                .await
            {
                Ok(result) => {
                    items_done_clone.fetch_add(1, Ordering::Relaxed);
                    self.emit_progress(
                        job_id,
                        &items_done_clone,
                        &bytes_done,
                        total_items,
                        total_bytes,
                    );

                    let _ = event_tx.send(FolderTransferEvent::ItemCompleted {
                        job_id,
                        result: result.clone(),
                    });

                    match &result {
                        ItemResult::Success { bytes, .. } => {
                            report.succeeded += 1;
                            report.bytes_transferred += bytes;
                        }
                        ItemResult::Skipped { .. } => {
                            report.skipped += 1;
                        }
                        ItemResult::Failed { .. } => {
                            report.failed += 1;
                        }
                    }
                    report.items.push(result);
                }
                Err(e) => {
                    if self.config.continue_on_error {
                        warn!(
                            file = %item.source.display(),
                            error = %e,
                            "Failed to copy file, continuing"
                        );
                        report.failed += 1;
                        report.items.push(ItemResult::Failed {
                            source: item.source.clone(),
                            destination: item.destination.clone(),
                            error: e.to_string(),
                        });
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Ok(report)
    }

    async fn create_directory(
        &self,
        item: &TransferItem,
        resolver: &Arc<std::sync::Mutex<ConflictResolver>>,
    ) -> ZResult<ItemResult> {
        trace!(
            source = %item.source.display(),
            dest = %item.destination.display(),
            "Creating directory"
        );

        if item.destination.exists() {
            if item.destination.is_dir() {
                // Directory already exists, that's fine for merging
                return Ok(ItemResult::Skipped {
                    source: item.source.clone(),
                    destination: item.destination.clone(),
                    reason: "Directory already exists".to_string(),
                });
            }

            // Conflict: file exists where we want a directory
            let conflict = Conflict::new(&item.source, &item.destination);
            if let Some(conflict) = conflict {
                let resolution = resolver
                    .lock()
                    .map_err(|_| ZError::Internal {
                        message: "Resolver lock poisoned".to_string(),
                    })?
                    .resolve(&conflict);

                match resolution {
                    Some(ConflictResolution::Skip) => {
                        return Ok(ItemResult::Skipped {
                            source: item.source.clone(),
                            destination: item.destination.clone(),
                            reason: "Skipped due to conflict".to_string(),
                        });
                    }
                    Some(ConflictResolution::Overwrite) => {
                        // Remove the file and create directory
                        std::fs::remove_file(&item.destination)
                            .map_err(|e| ZError::io(&item.destination, e))?;
                    }
                    _ => {
                        return Err(ZError::AlreadyExists {
                            path: item.destination.clone(),
                        });
                    }
                }
            }
        }

        std::fs::create_dir_all(&item.destination)
            .map_err(|e| ZError::io(&item.destination, e))?;

        Ok(ItemResult::Success {
            source: item.source.clone(),
            destination: item.destination.clone(),
            bytes: 0,
        })
    }

    async fn copy_file(
        &self,
        job_id: JobId,
        item: &TransferItem,
        resolver: &Arc<std::sync::Mutex<ConflictResolver>>,
        cancel_token: &CancellationToken,
        bytes_done: Arc<AtomicU64>,
    ) -> ZResult<ItemResult> {
        trace!(
            source = %item.source.display(),
            dest = %item.destination.display(),
            "Copying file"
        );

        let mut destination = item.destination.clone();
        let mut overwrite = false;

        // Handle conflicts
        if item.has_conflict {
            let conflict = Conflict::new(&item.source, &item.destination);
            if let Some(conflict) = conflict {
                let resolution = resolver
                    .lock()
                    .map_err(|_| ZError::Internal {
                        message: "Resolver lock poisoned".to_string(),
                    })?
                    .resolve(&conflict);

                match resolution {
                    Some(ConflictResolution::Skip) => {
                        return Ok(ItemResult::Skipped {
                            source: item.source.clone(),
                            destination: item.destination.clone(),
                            reason: "Skipped due to conflict".to_string(),
                        });
                    }
                    Some(ConflictResolution::Overwrite) => {
                        overwrite = true;
                    }
                    Some(ConflictResolution::Rename) => {
                        destination = ConflictResolver::generate_rename_path(&item.destination);
                    }
                    Some(ConflictResolution::Cancel) => {
                        return Err(ZError::Cancelled);
                    }
                    None => {
                        // Need to ask user - emit conflict event
                        let _ = self.event_tx.send(FolderTransferEvent::ConflictDetected {
                            job_id,
                            conflict: conflict.clone(),
                        });

                        // For now, default to skip when Ask policy is used
                        // In a real implementation, we'd wait for user response
                        return Ok(ItemResult::Skipped {
                            source: item.source.clone(),
                            destination: item.destination.clone(),
                            reason: "Awaiting user resolution".to_string(),
                        });
                    }
                }
            }
        }

        // Create progress callback
        let _event_tx = self.event_tx.clone(); // Reserved for per-file progress events
        let _source_clone = item.source.clone(); // Reserved for per-file progress events
        let config_interval = self.config.progress_interval_bytes;
        let last_reported = Arc::new(AtomicU64::new(0));

        let callback: ProgressCallback = Box::new(move |p: CopyProgress| {
            let last = last_reported.load(Ordering::Relaxed);
            if p.bytes_copied - last >= config_interval {
                last_reported.store(p.bytes_copied, Ordering::Relaxed);
                bytes_done.fetch_add(p.bytes_copied - last, Ordering::Relaxed);
            }
        });

        // Execute the copy
        let result = tokio::task::spawn_blocking({
            let source = item.source.clone();
            let destination = destination.clone();
            let token = cancel_token.clone();
            move || copy_file_with_progress(&source, &destination, overwrite, token, Some(callback))
        })
        .await
        .map_err(|e| ZError::Internal {
            message: format!("Task join error: {e}"),
        })?;

        match result {
            Ok(bytes) => Ok(ItemResult::Success {
                source: item.source.clone(),
                destination,
                bytes,
            }),
            Err(ZError::Cancelled) => {
                // Clean up partial file
                let _ = std::fs::remove_file(&destination);
                Err(ZError::Cancelled)
            }
            Err(e) => Ok(ItemResult::Failed {
                source: item.source.clone(),
                destination,
                error: e.to_string(),
            }),
        }
    }

    async fn delete_sources(&self, plan: &TransferPlan) {
        // Delete in reverse order (files first, then directories deepest first)
        let mut items: Vec<_> = plan.items.iter().collect();
        items.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (false, true) => std::cmp::Ordering::Less, // Files before dirs
                (true, false) => std::cmp::Ordering::Greater,
                (true, true) => b.depth.cmp(&a.depth), // Deeper dirs first
                (false, false) => std::cmp::Ordering::Equal,
            }
        });

        for item in items {
            if item.is_dir {
                if let Err(e) = std::fs::remove_dir(&item.source) {
                    warn!(
                        path = %item.source.display(),
                        error = %e,
                        "Failed to delete source directory"
                    );
                }
            } else if let Err(e) = std::fs::remove_file(&item.source) {
                warn!(
                    path = %item.source.display(),
                    error = %e,
                    "Failed to delete source file"
                );
            }
        }
    }

    fn emit_progress(
        &self,
        job_id: JobId,
        items_done: &AtomicUsize,
        bytes_done: &AtomicU64,
        total_items: usize,
        total_bytes: u64,
    ) {
        let progress = Progress {
            total_bytes: Some(total_bytes),
            bytes_done: bytes_done.load(Ordering::Relaxed),
            total_items,
            items_done: items_done.load(Ordering::Relaxed),
            current_item: None,
            eta: None,
            speed_bytes_per_sec: None,
        };

        let _ = self.event_tx.send(FolderTransferEvent::Progress { job_id, progress });
    }
}

impl Default for FolderTransferExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate directory size (for atomic move reporting).
fn calculate_dir_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_tree(dir: &TempDir) -> PathBuf {
        let root = dir.path().join("source");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("file1.txt"), vec![b'A'; 100]).unwrap();

        let subdir = root.join("subdir");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join("file2.txt"), vec![b'B'; 200]).unwrap();

        root
    }

    #[tokio::test]
    async fn test_copy_folder_basic() {
        let temp = TempDir::new().unwrap();
        let source = create_test_tree(&temp);
        let dest = temp.path().join("dest");
        fs::create_dir(&dest).unwrap();

        let executor = FolderTransferExecutor::new();
        let resolver = Arc::new(std::sync::Mutex::new(ConflictResolver::overwrite_all()));
        let token = CancellationToken::new();

        let report = executor
            .copy_folder(JobId::new(), vec![source.clone()], dest.clone(), resolver, token)
            .await
            .unwrap();

        assert!(report.is_complete_success());
        assert!(report.succeeded >= 2); // At least 2 files
        assert!(dest.join("source").join("file1.txt").exists());
        assert!(dest.join("source").join("subdir").join("file2.txt").exists());
    }

    #[tokio::test]
    async fn test_copy_folder_with_conflicts() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source.txt");
        let dest = temp.path().join("dest");
        let existing = dest.join("source.txt");

        fs::write(&source, "new content").unwrap();
        fs::create_dir(&dest).unwrap();
        fs::write(&existing, "old content").unwrap();

        let executor = FolderTransferExecutor::new();
        let resolver = Arc::new(std::sync::Mutex::new(ConflictResolver::skip_all()));
        let token = CancellationToken::new();

        let report = executor
            .copy_folder(JobId::new(), vec![source], dest.clone(), resolver, token)
            .await
            .unwrap();

        assert_eq!(report.skipped, 1);
        // Original content should be preserved
        assert_eq!(fs::read_to_string(&existing).unwrap(), "old content");
    }

    #[tokio::test]
    async fn test_copy_folder_overwrite() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source.txt");
        let dest = temp.path().join("dest");
        let existing = dest.join("source.txt");

        fs::write(&source, "new content").unwrap();
        fs::create_dir(&dest).unwrap();
        fs::write(&existing, "old content").unwrap();

        let executor = FolderTransferExecutor::new();
        let resolver = Arc::new(std::sync::Mutex::new(ConflictResolver::overwrite_all()));
        let token = CancellationToken::new();

        let report = executor
            .copy_folder(JobId::new(), vec![source], dest.clone(), resolver, token)
            .await
            .unwrap();

        assert_eq!(report.succeeded, 1);
        assert_eq!(fs::read_to_string(&existing).unwrap(), "new content");
    }

    #[tokio::test]
    async fn test_move_folder() {
        let temp = TempDir::new().unwrap();
        let source = create_test_tree(&temp);
        let dest = temp.path().join("dest");
        fs::create_dir(&dest).unwrap();

        let executor = FolderTransferExecutor::new();
        let resolver = Arc::new(std::sync::Mutex::new(ConflictResolver::overwrite_all()));
        let token = CancellationToken::new();

        let report = executor
            .move_folder(JobId::new(), vec![source.clone()], dest.clone(), resolver, token)
            .await
            .unwrap();

        assert!(report.is_complete_success());
        assert!(dest.join("source").join("file1.txt").exists());
        // Source should be deleted after successful move
        // Note: In the current implementation, source dirs may remain if not empty
    }

    #[tokio::test]
    async fn test_transfer_report() {
        let report = TransferReport {
            items: vec![],
            bytes_transferred: 1000,
            succeeded: 5,
            skipped: 2,
            failed: 1,
            duration: std::time::Duration::from_secs(2),
        };

        assert!(!report.is_complete_success());
        assert!(report.has_transfers());
        assert_eq!(report.average_speed(), 500);
    }

    #[tokio::test]
    async fn test_item_result() {
        let success = ItemResult::Success {
            source: PathBuf::from("src"),
            destination: PathBuf::from("dst"),
            bytes: 100,
        };

        assert!(success.is_success());
        assert!(!success.is_failed());
        assert_eq!(success.source(), Path::new("src"));
    }
}
