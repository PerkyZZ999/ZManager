//! Job system types and state machine.
//!
//! This module defines the core types for the job system that manages
//! long-running file operations like copy, move, and delete.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Unique identifier for a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub u64);

impl JobId {
    /// Create a new unique JobId.
    pub fn new() -> Self {
        use std::sync::atomic::AtomicU64;
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Job-{}", self.0)
    }
}

/// The kind of operation a job performs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobKind {
    /// Copy files/directories to a destination
    Copy {
        sources: Vec<PathBuf>,
        destination: PathBuf,
    },
    /// Move files/directories to a destination
    Move {
        sources: Vec<PathBuf>,
        destination: PathBuf,
    },
    /// Delete files/directories (to Recycle Bin)
    Delete { paths: Vec<PathBuf> },
    /// Permanently delete files/directories
    DeletePermanent { paths: Vec<PathBuf> },
    /// Calculate folder size (async operation)
    CalculateSize { path: PathBuf },
}

impl JobKind {
    /// Get a human-readable description of the job kind.
    pub fn description(&self) -> String {
        match self {
            Self::Copy { sources, .. } => {
                let count = sources.len();
                if count == 1 {
                    format!("Copying {}", sources[0].display())
                } else {
                    format!("Copying {count} items")
                }
            }
            Self::Move { sources, .. } => {
                let count = sources.len();
                if count == 1 {
                    format!("Moving {}", sources[0].display())
                } else {
                    format!("Moving {count} items")
                }
            }
            Self::Delete { paths } | Self::DeletePermanent { paths } => {
                let count = paths.len();
                if count == 1 {
                    format!("Deleting {}", paths[0].display())
                } else {
                    format!("Deleting {count} items")
                }
            }
            Self::CalculateSize { path } => {
                format!("Calculating size of {}", path.display())
            }
        }
    }

    /// Get the total number of items this job will process.
    pub fn item_count(&self) -> usize {
        match self {
            Self::Copy { sources, .. } | Self::Move { sources, .. } => sources.len(),
            Self::Delete { paths } | Self::DeletePermanent { paths } => paths.len(),
            Self::CalculateSize { .. } => 1,
        }
    }
}

/// The current state of a job in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    /// Job is queued but not yet started
    Pending,
    /// Job is currently running
    Running,
    /// Job is paused by user
    Paused,
    /// Job completed successfully
    Completed,
    /// Job failed with an error
    Failed,
    /// Job was cancelled by user
    Cancelled,
}

impl JobState {
    /// Check if the job is in a terminal state (cannot transition further).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// Check if the job is currently active (running or paused).
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }
}

impl std::fmt::Display for JobState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Running => write!(f, "Running"),
            Self::Paused => write!(f, "Paused"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Progress information for a running job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    /// Total bytes to process (if known)
    pub total_bytes: Option<u64>,
    /// Bytes processed so far
    pub bytes_done: u64,
    /// Total number of items (files/folders)
    pub total_items: usize,
    /// Items processed so far
    pub items_done: usize,
    /// Current item being processed
    pub current_item: Option<PathBuf>,
    /// Estimated time remaining
    pub eta: Option<Duration>,
    /// Current transfer speed in bytes/sec
    pub speed_bytes_per_sec: Option<u64>,
}

impl Progress {
    /// Create a new Progress with initial values.
    pub fn new(total_items: usize, total_bytes: Option<u64>) -> Self {
        Self {
            total_bytes,
            bytes_done: 0,
            total_items,
            items_done: 0,
            current_item: None,
            eta: None,
            speed_bytes_per_sec: None,
        }
    }

    /// Calculate completion percentage (0.0 to 1.0).
    pub fn percentage(&self) -> f64 {
        if let Some(total) = self.total_bytes {
            if total > 0 {
                return self.bytes_done as f64 / total as f64;
            }
        }
        if self.total_items > 0 {
            return self.items_done as f64 / self.total_items as f64;
        }
        0.0
    }

    /// Calculate completion percentage as integer (0 to 100).
    pub fn percentage_int(&self) -> u8 {
        (self.percentage() * 100.0).round() as u8
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new(0, None)
    }
}

/// A cancellation token for cooperative cancellation of jobs.
///
/// Jobs should check this token periodically and stop gracefully when cancelled.
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Create a new cancellation token.
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Request cancellation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Check if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Reset the token (for reuse).
    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// A complete job with all its metadata.
#[derive(Debug, Clone)]
pub struct Job {
    /// Unique identifier
    pub id: JobId,
    /// What operation to perform
    pub kind: JobKind,
    /// Current state
    pub state: JobState,
    /// Progress information
    pub progress: Progress,
    /// When the job was created
    pub created_at: Instant,
    /// When the job started running (if started)
    pub started_at: Option<Instant>,
    /// When the job finished (if finished)
    pub finished_at: Option<Instant>,
    /// Error message if failed
    pub error: Option<String>,
    /// Cancellation token
    pub cancellation: CancellationToken,
}

impl Job {
    /// Create a new job.
    pub fn new(kind: JobKind) -> Self {
        let total_items = kind.item_count();
        Self {
            id: JobId::new(),
            kind,
            state: JobState::Pending,
            progress: Progress::new(total_items, None),
            created_at: Instant::now(),
            started_at: None,
            finished_at: None,
            error: None,
            cancellation: CancellationToken::new(),
        }
    }

    /// Transition the job to Running state.
    pub fn start(&mut self) {
        if self.state == JobState::Pending {
            self.state = JobState::Running;
            self.started_at = Some(Instant::now());
        }
    }

    /// Pause the job.
    pub fn pause(&mut self) {
        if self.state == JobState::Running {
            self.state = JobState::Paused;
        }
    }

    /// Resume a paused job.
    pub fn resume(&mut self) {
        if self.state == JobState::Paused {
            self.state = JobState::Running;
        }
    }

    /// Mark the job as completed successfully.
    pub fn complete(&mut self) {
        if !self.state.is_terminal() {
            self.state = JobState::Completed;
            self.finished_at = Some(Instant::now());
        }
    }

    /// Mark the job as failed with an error.
    pub fn fail(&mut self, error: impl Into<String>) {
        if !self.state.is_terminal() {
            self.state = JobState::Failed;
            self.error = Some(error.into());
            self.finished_at = Some(Instant::now());
        }
    }

    /// Cancel the job.
    pub fn cancel(&mut self) {
        if !self.state.is_terminal() {
            self.cancellation.cancel();
            self.state = JobState::Cancelled;
            self.finished_at = Some(Instant::now());
        }
    }

    /// Get the elapsed time since job creation.
    pub fn elapsed(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Get the running time (from start to finish or now).
    pub fn running_time(&self) -> Option<Duration> {
        self.started_at.map(|start| {
            self.finished_at
                .map(|end| end.duration_since(start))
                .unwrap_or_else(|| start.elapsed())
        })
    }
}

/// A snapshot of job information for UI display.
///
/// Unlike `Job`, this is serializable and doesn't contain internal state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: JobId,
    pub description: String,
    pub state: JobState,
    pub progress_percent: u8,
    pub items_done: usize,
    pub total_items: usize,
    pub bytes_done: u64,
    pub total_bytes: Option<u64>,
    pub current_item: Option<PathBuf>,
    pub speed_bytes_per_sec: Option<u64>,
    pub eta_secs: Option<u64>,
    pub error: Option<String>,
}

impl From<&Job> for JobInfo {
    fn from(job: &Job) -> Self {
        Self {
            id: job.id,
            description: job.kind.description(),
            state: job.state,
            progress_percent: job.progress.percentage_int(),
            items_done: job.progress.items_done,
            total_items: job.progress.total_items,
            bytes_done: job.progress.bytes_done,
            total_bytes: job.progress.total_bytes,
            current_item: job.progress.current_item.clone(),
            speed_bytes_per_sec: job.progress.speed_bytes_per_sec,
            eta_secs: job.progress.eta.map(|d| d.as_secs()),
            error: job.error.clone(),
        }
    }
}

/// Statistics about all jobs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobStats {
    pub pending: usize,
    pub running: usize,
    pub paused: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
}

impl JobStats {
    /// Total number of jobs.
    pub fn total(&self) -> usize {
        self.pending + self.running + self.paused + self.completed + self.failed + self.cancelled
    }

    /// Number of active (non-terminal) jobs.
    pub fn active(&self) -> usize {
        self.pending + self.running + self.paused
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_id_unique() {
        let id1 = JobId::new();
        let id2 = JobId::new();
        let id3 = JobId::new();

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_job_id_display() {
        let id = JobId(42);
        assert_eq!(id.to_string(), "Job-42");
    }

    #[test]
    fn test_job_kind_description() {
        let copy = JobKind::Copy {
            sources: vec![PathBuf::from("file.txt")],
            destination: PathBuf::from("dest"),
        };
        assert!(copy.description().contains("Copying"));
        assert!(copy.description().contains("file.txt"));

        let delete = JobKind::Delete {
            paths: vec![PathBuf::from("a"), PathBuf::from("b"), PathBuf::from("c")],
        };
        assert!(delete.description().contains("3 items"));
    }

    #[test]
    fn test_job_state_transitions() {
        assert!(!JobState::Pending.is_terminal());
        assert!(!JobState::Running.is_terminal());
        assert!(!JobState::Paused.is_terminal());
        assert!(JobState::Completed.is_terminal());
        assert!(JobState::Failed.is_terminal());
        assert!(JobState::Cancelled.is_terminal());

        assert!(!JobState::Pending.is_active());
        assert!(JobState::Running.is_active());
        assert!(JobState::Paused.is_active());
        assert!(!JobState::Completed.is_active());
    }

    #[test]
    fn test_progress_percentage() {
        let mut progress = Progress::new(10, Some(1000));

        assert_eq!(progress.percentage(), 0.0);
        assert_eq!(progress.percentage_int(), 0);

        progress.bytes_done = 500;
        assert_eq!(progress.percentage(), 0.5);
        assert_eq!(progress.percentage_int(), 50);

        progress.bytes_done = 1000;
        assert_eq!(progress.percentage(), 1.0);
        assert_eq!(progress.percentage_int(), 100);
    }

    #[test]
    fn test_progress_items_fallback() {
        let mut progress = Progress::new(4, None);

        progress.items_done = 2;
        assert_eq!(progress.percentage(), 0.5);
    }

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new();

        assert!(!token.is_cancelled());

        token.cancel();
        assert!(token.is_cancelled());

        token.reset();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn test_cancellation_token_clone() {
        let token1 = CancellationToken::new();
        let token2 = token1.clone();

        assert!(!token1.is_cancelled());
        assert!(!token2.is_cancelled());

        token1.cancel();

        assert!(token1.is_cancelled());
        assert!(token2.is_cancelled()); // Clone shares state
    }

    #[test]
    fn test_job_lifecycle() {
        let mut job = Job::new(JobKind::Delete {
            paths: vec![PathBuf::from("test")],
        });

        assert_eq!(job.state, JobState::Pending);
        assert!(job.started_at.is_none());

        job.start();
        assert_eq!(job.state, JobState::Running);
        assert!(job.started_at.is_some());

        job.pause();
        assert_eq!(job.state, JobState::Paused);

        job.resume();
        assert_eq!(job.state, JobState::Running);

        job.complete();
        assert_eq!(job.state, JobState::Completed);
        assert!(job.finished_at.is_some());
    }

    #[test]
    fn test_job_failure() {
        let mut job = Job::new(JobKind::CalculateSize {
            path: PathBuf::from("test"),
        });

        job.start();
        job.fail("Something went wrong");

        assert_eq!(job.state, JobState::Failed);
        assert_eq!(job.error.as_deref(), Some("Something went wrong"));
    }

    #[test]
    fn test_job_cancel() {
        let mut job = Job::new(JobKind::CalculateSize {
            path: PathBuf::from("test"),
        });

        job.start();
        job.cancel();

        assert_eq!(job.state, JobState::Cancelled);
        assert!(job.cancellation.is_cancelled());
    }

    #[test]
    fn test_job_info_from_job() {
        let mut job = Job::new(JobKind::Copy {
            sources: vec![PathBuf::from("src")],
            destination: PathBuf::from("dst"),
        });

        job.progress.bytes_done = 500;
        job.progress.total_bytes = Some(1000);
        job.start();

        let info = JobInfo::from(&job);

        assert_eq!(info.id, job.id);
        assert!(info.description.contains("Copying"));
        assert_eq!(info.state, JobState::Running);
        assert_eq!(info.progress_percent, 50);
    }

    #[test]
    fn test_job_stats() {
        let stats = JobStats {
            pending: 2,
            running: 1,
            paused: 1,
            completed: 5,
            failed: 1,
            cancelled: 0,
        };

        assert_eq!(stats.total(), 10);
        assert_eq!(stats.active(), 4);
    }
}
