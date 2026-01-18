//! Job system types for transfer operations.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A unique identifier for a transfer job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub u64);

impl JobId {
    /// Create a new job ID from a counter value.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Job#{}", self.0)
    }
}

/// The kind of transfer operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    /// Copy files/folders to destination.
    Copy,
    /// Move files/folders to destination (copy + delete source on success).
    Move,
    /// Delete files/folders.
    Delete,
}

impl JobKind {
    /// Get a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Copy => "Copying",
            Self::Move => "Moving",
            Self::Delete => "Deleting",
        }
    }

    /// Get a past-tense label.
    pub fn completed_label(&self) -> &'static str {
        match self {
            Self::Copy => "Copied",
            Self::Move => "Moved",
            Self::Delete => "Deleted",
        }
    }
}

/// The state of a transfer job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    /// Job is queued but not yet started.
    Pending,
    /// Job is actively running.
    Running,
    /// Job is paused by user.
    Paused,
    /// Job completed successfully.
    Completed,
    /// Job failed with an error.
    Failed,
    /// Job was cancelled by user.
    Cancelled,
    /// Job is waiting for user input (e.g., conflict resolution).
    WaitingForInput,
}

impl JobState {
    /// Returns `true` if the job is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// Returns `true` if the job is actively working.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Returns `true` if the job can be paused.
    pub fn can_pause(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Returns `true` if the job can be resumed.
    pub fn can_resume(&self) -> bool {
        matches!(self, Self::Paused)
    }

    /// Returns `true` if the job can be cancelled.
    pub fn can_cancel(&self) -> bool {
        !self.is_terminal()
    }

    /// Get a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Running => "Running",
            Self::Paused => "Paused",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
            Self::WaitingForInput => "Waiting",
        }
    }
}

/// Progress information for a transfer job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    /// The job this progress belongs to.
    pub job_id: JobId,

    /// Current job state.
    pub state: JobState,

    /// Total bytes to transfer.
    pub total_bytes: u64,

    /// Bytes transferred so far.
    pub transferred_bytes: u64,

    /// Total number of items (files + folders).
    pub total_items: usize,

    /// Items processed so far.
    pub processed_items: usize,

    /// Current file being processed (if any).
    pub current_file: Option<PathBuf>,

    /// Current transfer speed in bytes per second.
    pub speed_bps: u64,

    /// Estimated time remaining in seconds.
    pub eta_seconds: Option<u64>,

    /// Number of items that failed.
    pub failed_items: usize,

    /// Number of items that were skipped.
    pub skipped_items: usize,
}

impl Progress {
    /// Create a new progress tracker for a job.
    pub fn new(job_id: JobId, total_bytes: u64, total_items: usize) -> Self {
        Self {
            job_id,
            state: JobState::Pending,
            total_bytes,
            transferred_bytes: 0,
            total_items,
            processed_items: 0,
            current_file: None,
            speed_bps: 0,
            eta_seconds: None,
            failed_items: 0,
            skipped_items: 0,
        }
    }

    /// Get the completion percentage (0.0 to 100.0).
    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            if self.total_items == 0 {
                100.0
            } else {
                (self.processed_items as f64 / self.total_items as f64) * 100.0
            }
        } else {
            (self.transferred_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }

    /// Get a human-readable speed string (e.g., "15.5 MB/s").
    pub fn speed_display(&self) -> String {
        format_speed(self.speed_bps)
    }

    /// Get a human-readable ETA string (e.g., "2m 30s").
    pub fn eta_display(&self) -> String {
        match self.eta_seconds {
            Some(secs) => format_duration(secs),
            None => "Calculating...".to_string(),
        }
    }

    /// Check if the job has completed (successfully or not).
    pub fn is_finished(&self) -> bool {
        self.state.is_terminal()
    }
}

/// Format bytes per second as a human-readable speed.
fn format_speed(bps: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bps >= GB {
        format!("{:.2} GB/s", bps as f64 / GB as f64)
    } else if bps >= MB {
        format!("{:.2} MB/s", bps as f64 / MB as f64)
    } else if bps >= KB {
        format!("{:.2} KB/s", bps as f64 / KB as f64)
    } else {
        format!("{} B/s", bps)
    }
}

/// Format seconds as a human-readable duration.
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        let mins = secs / 60;
        let remaining = secs % 60;
        if remaining > 0 {
            format!("{}m {}s", mins, remaining)
        } else {
            format!("{}m", mins)
        }
    } else {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        if mins > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}h", hours)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_id_display() {
        let id = JobId::new(42);
        assert_eq!(id.to_string(), "Job#42");
    }

    #[test]
    fn test_job_kind_labels() {
        assert_eq!(JobKind::Copy.label(), "Copying");
        assert_eq!(JobKind::Move.label(), "Moving");
        assert_eq!(JobKind::Delete.label(), "Deleting");

        assert_eq!(JobKind::Copy.completed_label(), "Copied");
    }

    #[test]
    fn test_job_state_terminal() {
        assert!(!JobState::Pending.is_terminal());
        assert!(!JobState::Running.is_terminal());
        assert!(!JobState::Paused.is_terminal());
        assert!(JobState::Completed.is_terminal());
        assert!(JobState::Failed.is_terminal());
        assert!(JobState::Cancelled.is_terminal());
    }

    #[test]
    fn test_job_state_can_pause() {
        assert!(JobState::Running.can_pause());
        assert!(!JobState::Paused.can_pause());
        assert!(!JobState::Pending.can_pause());
    }

    #[test]
    fn test_job_state_can_resume() {
        assert!(JobState::Paused.can_resume());
        assert!(!JobState::Running.can_resume());
    }

    #[test]
    fn test_progress_percentage_bytes() {
        let mut progress = Progress::new(JobId::new(1), 1000, 10);
        progress.transferred_bytes = 500;

        assert!((progress.percentage() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_progress_percentage_items_only() {
        let mut progress = Progress::new(JobId::new(1), 0, 10);
        progress.processed_items = 5;

        assert!((progress.percentage() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_progress_percentage_empty() {
        let progress = Progress::new(JobId::new(1), 0, 0);
        assert!((progress.percentage() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(500), "500 B/s");
        assert_eq!(format_speed(1024), "1.00 KB/s");
        assert_eq!(format_speed(1048576), "1.00 MB/s");
        assert_eq!(format_speed(1073741824), "1.00 GB/s");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3600), "1h");
        assert_eq!(format_duration(3660), "1h 1m");
    }
}
