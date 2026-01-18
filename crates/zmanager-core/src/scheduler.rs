//! Job scheduler for managing concurrent file operations.
//!
//! The scheduler maintains a queue of jobs and runs them according to
//! concurrency limits, broadcasting progress updates to subscribers.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, error, info, warn};

use crate::job::{Job, JobId, JobInfo, JobKind, JobState, JobStats, Progress};

/// Configuration for the job scheduler.
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Maximum number of jobs that can run concurrently.
    pub max_concurrent_jobs: usize,
    /// Size of the progress broadcast channel.
    pub progress_channel_size: usize,
    /// Maximum number of completed jobs to keep in history.
    pub max_history: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: 2,
            progress_channel_size: 256,
            max_history: 100,
        }
    }
}

/// Events emitted by the scheduler.
#[derive(Debug, Clone)]
pub enum SchedulerEvent {
    /// A new job was added to the queue.
    JobAdded(JobId),
    /// A job started running.
    JobStarted(JobId),
    /// Progress update for a running job.
    JobProgress(JobId, Progress),
    /// A job completed successfully.
    JobCompleted(JobId),
    /// A job failed with an error.
    JobFailed(JobId, String),
    /// A job was cancelled.
    JobCancelled(JobId),
    /// A job was paused.
    JobPaused(JobId),
    /// A job was resumed.
    JobResumed(JobId),
}

/// Commands sent to the scheduler.
#[derive(Debug)]
pub enum SchedulerCommand {
    /// Add a new job to the queue.
    AddJob(Box<Job>),
    /// Cancel a job by ID.
    CancelJob(JobId),
    /// Pause a job by ID.
    PauseJob(JobId),
    /// Resume a paused job.
    ResumeJob(JobId),
    /// Clear completed/failed/cancelled jobs from history.
    ClearHistory,
    /// Shutdown the scheduler.
    Shutdown,
}

/// Handle to interact with the scheduler.
#[derive(Clone)]
pub struct SchedulerHandle {
    command_tx: mpsc::Sender<SchedulerCommand>,
    event_tx: broadcast::Sender<SchedulerEvent>,
    jobs: Arc<RwLock<HashMap<JobId, Job>>>,
}

impl SchedulerHandle {
    /// Submit a new job to the scheduler.
    pub async fn submit(&self, kind: JobKind) -> JobId {
        let job = Job::new(kind);
        let id = job.id;

        // Add to local state first
        self.jobs.write().await.insert(id, job.clone());

        // Send command to scheduler
        if let Err(e) = self
            .command_tx
            .send(SchedulerCommand::AddJob(Box::new(job)))
            .await
        {
            error!("Failed to send AddJob command: {}", e);
        }

        id
    }

    /// Cancel a job.
    pub async fn cancel(&self, id: JobId) -> bool {
        if let Some(job) = self.jobs.read().await.get(&id) {
            if job.state.is_terminal() {
                return false;
            }
        } else {
            return false;
        }

        self.command_tx
            .send(SchedulerCommand::CancelJob(id))
            .await
            .is_ok()
    }

    /// Pause a running job.
    pub async fn pause(&self, id: JobId) -> bool {
        self.command_tx
            .send(SchedulerCommand::PauseJob(id))
            .await
            .is_ok()
    }

    /// Resume a paused job.
    pub async fn resume(&self, id: JobId) -> bool {
        self.command_tx
            .send(SchedulerCommand::ResumeJob(id))
            .await
            .is_ok()
    }

    /// Get information about a specific job.
    pub async fn get_job(&self, id: JobId) -> Option<JobInfo> {
        self.jobs.read().await.get(&id).map(JobInfo::from)
    }

    /// Get information about all jobs.
    pub async fn list_jobs(&self) -> Vec<JobInfo> {
        self.jobs
            .read()
            .await
            .values()
            .map(JobInfo::from)
            .collect()
    }

    /// Get job statistics.
    pub async fn stats(&self) -> JobStats {
        let jobs = self.jobs.read().await;
        let mut stats = JobStats::default();

        for job in jobs.values() {
            match job.state {
                JobState::Pending => stats.pending += 1,
                JobState::Running => stats.running += 1,
                JobState::Paused => stats.paused += 1,
                JobState::Completed => stats.completed += 1,
                JobState::Failed => stats.failed += 1,
                JobState::Cancelled => stats.cancelled += 1,
            }
        }

        stats
    }

    /// Subscribe to scheduler events.
    pub fn subscribe(&self) -> broadcast::Receiver<SchedulerEvent> {
        self.event_tx.subscribe()
    }

    /// Clear completed/failed/cancelled jobs from history.
    pub async fn clear_history(&self) {
        let _ = self.command_tx.send(SchedulerCommand::ClearHistory).await;
    }

    /// Shutdown the scheduler gracefully.
    pub async fn shutdown(&self) {
        let _ = self.command_tx.send(SchedulerCommand::Shutdown).await;
    }
}

/// The main job scheduler.
pub struct Scheduler {
    config: SchedulerConfig,
    jobs: Arc<RwLock<HashMap<JobId, Job>>>,
    command_rx: mpsc::Receiver<SchedulerCommand>,
    #[allow(dead_code)] // Kept for future internal command dispatch
    command_tx: mpsc::Sender<SchedulerCommand>,
    event_tx: broadcast::Sender<SchedulerEvent>,
    running_count: usize,
}

impl Scheduler {
    /// Create a new scheduler with the given configuration.
    pub fn new(config: SchedulerConfig) -> (Self, SchedulerHandle) {
        let (command_tx, command_rx) = mpsc::channel(64);
        let (event_tx, _) = broadcast::channel(config.progress_channel_size);
        let jobs = Arc::new(RwLock::new(HashMap::new()));

        let scheduler = Self {
            config,
            jobs: jobs.clone(),
            command_rx,
            command_tx: command_tx.clone(),
            event_tx: event_tx.clone(),
            running_count: 0,
        };

        let handle = SchedulerHandle {
            command_tx,
            event_tx,
            jobs,
        };

        (scheduler, handle)
    }

    /// Create a scheduler with default configuration.
    pub fn with_defaults() -> (Self, SchedulerHandle) {
        Self::new(SchedulerConfig::default())
    }

    /// Run the scheduler loop.
    ///
    /// This method runs until shutdown is requested.
    pub async fn run(mut self) {
        info!("Job scheduler started");

        while let Some(cmd) = self.command_rx.recv().await {
            match cmd {
                SchedulerCommand::AddJob(job) => {
                    self.handle_add_job(*job).await;
                }
                SchedulerCommand::CancelJob(id) => {
                    self.handle_cancel(id).await;
                }
                SchedulerCommand::PauseJob(id) => {
                    self.handle_pause(id).await;
                }
                SchedulerCommand::ResumeJob(id) => {
                    self.handle_resume(id).await;
                }
                SchedulerCommand::ClearHistory => {
                    self.handle_clear_history().await;
                }
                SchedulerCommand::Shutdown => {
                    info!("Scheduler shutdown requested");
                    break;
                }
            }

            // Try to start pending jobs
            self.try_start_pending().await;
        }

        info!("Job scheduler stopped");
    }

    async fn handle_add_job(&mut self, job: Job) {
        let id = job.id;
        debug!(job_id = %id, "Adding job to queue");

        self.jobs.write().await.insert(id, job);
        let _ = self.event_tx.send(SchedulerEvent::JobAdded(id));
    }

    async fn handle_cancel(&mut self, id: JobId) {
        let mut jobs = self.jobs.write().await;

        if let Some(job) = jobs.get_mut(&id) {
            if !job.state.is_terminal() {
                let was_running = job.state == JobState::Running;
                job.cancel();
                let _ = self.event_tx.send(SchedulerEvent::JobCancelled(id));

                if was_running {
                    self.running_count = self.running_count.saturating_sub(1);
                }

                info!(job_id = %id, "Job cancelled");
            }
        }
    }

    async fn handle_pause(&mut self, id: JobId) {
        let mut jobs = self.jobs.write().await;

        if let Some(job) = jobs.get_mut(&id) {
            if job.state == JobState::Running {
                job.pause();
                self.running_count = self.running_count.saturating_sub(1);
                let _ = self.event_tx.send(SchedulerEvent::JobPaused(id));
                debug!(job_id = %id, "Job paused");
            }
        }
    }

    async fn handle_resume(&mut self, id: JobId) {
        let mut jobs = self.jobs.write().await;

        if let Some(job) = jobs.get_mut(&id) {
            if job.state == JobState::Paused {
                // Check if we can resume immediately or need to wait
                if self.running_count < self.config.max_concurrent_jobs {
                    job.resume();
                    self.running_count += 1;
                    let _ = self.event_tx.send(SchedulerEvent::JobResumed(id));
                    debug!(job_id = %id, "Job resumed");
                } else {
                    // Put back to pending, will start when slot available
                    job.state = JobState::Pending;
                    debug!(job_id = %id, "Job queued for resume");
                }
            }
        }
    }

    async fn handle_clear_history(&mut self) {
        let mut jobs = self.jobs.write().await;
        let initial_count = jobs.len();

        jobs.retain(|_, job| !job.state.is_terminal());

        let removed = initial_count - jobs.len();
        if removed > 0 {
            debug!(removed, "Cleared completed jobs from history");
        }

        // Also trim to max_history if needed
        if jobs.len() > self.config.max_history {
            // Remove oldest completed jobs first
            let mut to_remove: Vec<_> = jobs
                .iter()
                .filter(|(_, j)| j.state.is_terminal())
                .map(|(id, j)| (*id, j.created_at))
                .collect();

            to_remove.sort_by_key(|(_, created)| *created);

            for (id, _) in to_remove.iter().take(jobs.len() - self.config.max_history) {
                jobs.remove(id);
            }
        }
    }

    async fn try_start_pending(&mut self) {
        if self.running_count >= self.config.max_concurrent_jobs {
            return;
        }

        let mut jobs = self.jobs.write().await;

        // Find pending jobs ordered by creation time
        let mut pending: Vec<_> = jobs
            .iter()
            .filter(|(_, j)| j.state == JobState::Pending)
            .map(|(id, j)| (*id, j.created_at))
            .collect();

        pending.sort_by_key(|(_, created)| *created);

        // Start jobs up to the concurrency limit
        for (id, _) in pending {
            if self.running_count >= self.config.max_concurrent_jobs {
                break;
            }

            if let Some(job) = jobs.get_mut(&id) {
                job.start();
                self.running_count += 1;
                let _ = self.event_tx.send(SchedulerEvent::JobStarted(id));
                info!(job_id = %id, "Job started");

                // Note: Actual job execution would be spawned here
                // For now, we just mark it as started
                // The transfer engine (Sprint 5+) will handle actual execution
            }
        }
    }

    /// Update job progress (called by job executors).
    pub async fn update_progress(&self, id: JobId, progress: Progress) {
        let mut jobs = self.jobs.write().await;

        if let Some(job) = jobs.get_mut(&id) {
            job.progress = progress.clone();
            let _ = self.event_tx.send(SchedulerEvent::JobProgress(id, progress));
        }
    }

    /// Mark a job as completed (called by job executors).
    pub async fn complete_job(&mut self, id: JobId) {
        let mut jobs = self.jobs.write().await;

        if let Some(job) = jobs.get_mut(&id) {
            job.complete();
            self.running_count = self.running_count.saturating_sub(1);
            let _ = self.event_tx.send(SchedulerEvent::JobCompleted(id));
            info!(job_id = %id, "Job completed");
        }
    }

    /// Mark a job as failed (called by job executors).
    pub async fn fail_job(&mut self, id: JobId, error: String) {
        let mut jobs = self.jobs.write().await;

        if let Some(job) = jobs.get_mut(&id) {
            job.fail(&error);
            self.running_count = self.running_count.saturating_sub(1);
            let _ = self
                .event_tx
                .send(SchedulerEvent::JobFailed(id, error.clone()));
            warn!(job_id = %id, error = %error, "Job failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let (scheduler, handle) = Scheduler::with_defaults();

        let stats = handle.stats().await;
        assert_eq!(stats.total(), 0);

        // Shutdown immediately
        handle.shutdown().await;

        // Run scheduler briefly
        tokio::time::timeout(Duration::from_millis(100), scheduler.run())
            .await
            .ok();
    }

    #[tokio::test]
    async fn test_submit_job() {
        let (scheduler, handle) = Scheduler::with_defaults();

        // Spawn scheduler
        let scheduler_handle = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_millis(200), scheduler.run())
                .await
                .ok();
        });

        // Submit a job
        let id = handle
            .submit(JobKind::Delete {
                paths: vec![PathBuf::from("test")],
            })
            .await;

        // Give scheduler time to process
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Check job exists
        let job = handle.get_job(id).await;
        assert!(job.is_some());

        let job = job.unwrap();
        assert_eq!(job.id, id);

        handle.shutdown().await;
        let _ = scheduler_handle.await;
    }

    #[tokio::test]
    async fn test_list_jobs() {
        let (scheduler, handle) = Scheduler::with_defaults();

        let scheduler_handle = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_millis(200), scheduler.run())
                .await
                .ok();
        });

        // Submit multiple jobs
        let id1 = handle
            .submit(JobKind::Delete {
                paths: vec![PathBuf::from("a")],
            })
            .await;
        let id2 = handle
            .submit(JobKind::Delete {
                paths: vec![PathBuf::from("b")],
            })
            .await;

        tokio::time::sleep(Duration::from_millis(50)).await;

        let jobs = handle.list_jobs().await;
        assert_eq!(jobs.len(), 2);

        let ids: Vec<_> = jobs.iter().map(|j| j.id).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));

        handle.shutdown().await;
        let _ = scheduler_handle.await;
    }

    #[tokio::test]
    async fn test_cancel_job() {
        let (scheduler, handle) = Scheduler::with_defaults();

        let scheduler_handle = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_millis(200), scheduler.run())
                .await
                .ok();
        });

        let id = handle
            .submit(JobKind::Delete {
                paths: vec![PathBuf::from("test")],
            })
            .await;

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Cancel the job
        let cancelled = handle.cancel(id).await;
        assert!(cancelled);

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Check state
        let job = handle.get_job(id).await.unwrap();
        assert_eq!(job.state, JobState::Cancelled);

        handle.shutdown().await;
        let _ = scheduler_handle.await;
    }

    #[tokio::test]
    async fn test_stats() {
        let (scheduler, handle) = Scheduler::with_defaults();

        let scheduler_handle = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_millis(500), scheduler.run())
                .await
                .ok();
        });

        // Submit jobs with small delays to ensure proper ordering
        handle
            .submit(JobKind::Delete {
                paths: vec![PathBuf::from("a")],
            })
            .await;
        tokio::time::sleep(Duration::from_millis(20)).await;

        handle
            .submit(JobKind::Delete {
                paths: vec![PathBuf::from("b")],
            })
            .await;
        tokio::time::sleep(Duration::from_millis(20)).await;

        handle
            .submit(JobKind::Delete {
                paths: vec![PathBuf::from("c")],
            })
            .await;

        // Give scheduler time to process all commands
        tokio::time::sleep(Duration::from_millis(150)).await;

        let stats = handle.stats().await;
        assert_eq!(stats.total(), 3);

        // Jobs should be distributed between running and pending
        // Exact distribution depends on timing, so just verify total active
        assert!(stats.running + stats.pending == 3);
        // At most max_concurrent (2) should be running
        assert!(stats.running <= 2);

        handle.shutdown().await;
        let _ = scheduler_handle.await;
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let (scheduler, handle) = Scheduler::with_defaults();

        let mut rx = handle.subscribe();

        let scheduler_handle = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_millis(200), scheduler.run())
                .await
                .ok();
        });

        handle
            .submit(JobKind::Delete {
                paths: vec![PathBuf::from("test")],
            })
            .await;

        // Should receive JobAdded and JobStarted events
        let event1 = tokio::time::timeout(Duration::from_millis(100), rx.recv())
            .await
            .ok()
            .and_then(|r| r.ok());
        assert!(matches!(event1, Some(SchedulerEvent::JobAdded(_))));

        let event2 = tokio::time::timeout(Duration::from_millis(100), rx.recv())
            .await
            .ok()
            .and_then(|r| r.ok());
        assert!(matches!(event2, Some(SchedulerEvent::JobStarted(_))));

        handle.shutdown().await;
        let _ = scheduler_handle.await;
    }
}
