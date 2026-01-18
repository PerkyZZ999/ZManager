//! Transfer report model with persistence and export.
//!
//! This module provides detailed transfer reports with per-item results,
//! error tracking, and export capabilities (JSON + text).

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use zmanager_core::{JobId, ZError, ZResult};

/// Status of an individual transfer item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferStatus {
    /// Item transferred successfully.
    Success,
    /// Item was skipped (e.g., conflict policy).
    Skipped,
    /// Item transfer failed.
    Failed,
}

impl TransferStatus {
    /// Get a display label for the status.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::Skipped => "Skipped",
            Self::Failed => "Failed",
        }
    }

    /// Get a symbol for compact display.
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Success => "✓",
            Self::Skipped => "○",
            Self::Failed => "✗",
        }
    }
}

/// Detailed result for a single transferred item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferItemResult {
    /// Source path.
    pub source: PathBuf,
    /// Destination path.
    pub destination: PathBuf,
    /// Whether this was a directory.
    pub is_directory: bool,
    /// Size in bytes (0 for directories).
    pub size_bytes: u64,
    /// Transfer status.
    pub status: TransferStatus,
    /// Reason for skip or failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Duration for this item (if tracked).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

impl TransferItemResult {
    /// Create a success result.
    pub fn success(source: PathBuf, destination: PathBuf, size_bytes: u64) -> Self {
        Self {
            source,
            destination,
            is_directory: false,
            size_bytes,
            status: TransferStatus::Success,
            reason: None,
            duration_ms: None,
        }
    }

    /// Create a directory success result.
    pub fn success_dir(source: PathBuf, destination: PathBuf) -> Self {
        Self {
            source,
            destination,
            is_directory: true,
            size_bytes: 0,
            status: TransferStatus::Success,
            reason: None,
            duration_ms: None,
        }
    }

    /// Create a skipped result.
    pub fn skipped(source: PathBuf, destination: PathBuf, reason: impl Into<String>) -> Self {
        Self {
            source,
            destination,
            is_directory: false,
            size_bytes: 0,
            status: TransferStatus::Skipped,
            reason: Some(reason.into()),
            duration_ms: None,
        }
    }

    /// Create a failed result.
    pub fn failed(source: PathBuf, destination: PathBuf, error: impl Into<String>) -> Self {
        Self {
            source,
            destination,
            is_directory: false,
            size_bytes: 0,
            status: TransferStatus::Failed,
            reason: Some(error.into()),
            duration_ms: None,
        }
    }

    /// Set the duration for this item.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration_ms = Some(duration.as_millis() as u64);
        self
    }

    /// Check if the transfer succeeded.
    pub fn is_success(&self) -> bool {
        self.status == TransferStatus::Success
    }

    /// Check if the transfer failed.
    pub fn is_failed(&self) -> bool {
        self.status == TransferStatus::Failed
    }
}

/// Summary statistics for a transfer operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransferSummary {
    /// Total number of items processed.
    pub total_items: usize,
    /// Number of items that succeeded.
    pub succeeded: usize,
    /// Number of items that were skipped.
    pub skipped: usize,
    /// Number of items that failed.
    pub failed: usize,
    /// Total bytes transferred.
    pub bytes_transferred: u64,
    /// Total duration in milliseconds.
    pub duration_ms: u64,
    /// Number of directories created.
    pub directories_created: usize,
    /// Number of files copied.
    pub files_copied: usize,
}

impl TransferSummary {
    /// Calculate percentage of successful items.
    pub fn success_percentage(&self) -> f64 {
        if self.total_items == 0 {
            100.0
        } else {
            (self.succeeded as f64 / self.total_items as f64) * 100.0
        }
    }

    /// Check if all items succeeded.
    pub fn is_complete_success(&self) -> bool {
        self.failed == 0
    }

    /// Get average transfer speed in bytes per second.
    pub fn average_speed(&self) -> u64 {
        if self.duration_ms == 0 {
            self.bytes_transferred
        } else {
            (self.bytes_transferred * 1000) / self.duration_ms
        }
    }

    /// Get human-readable duration.
    pub fn duration_display(&self) -> String {
        let secs = self.duration_ms / 1000;
        let ms = self.duration_ms % 1000;

        if secs >= 3600 {
            format!("{}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60)
        } else if secs >= 60 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else if secs > 0 {
            format!("{}.{:03}s", secs, ms)
        } else {
            format!("{}ms", self.duration_ms)
        }
    }
}

/// Complete transfer report with all details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedTransferReport {
    /// Unique identifier for the job.
    pub job_id: JobId,
    /// Type of operation (copy/move).
    pub operation: TransferOperation,
    /// When the transfer started.
    #[serde(with = "system_time_serde")]
    pub started_at: SystemTime,
    /// When the transfer completed.
    #[serde(with = "system_time_serde")]
    pub completed_at: SystemTime,
    /// Summary statistics.
    pub summary: TransferSummary,
    /// Individual item results.
    pub items: Vec<TransferItemResult>,
    /// Whether the operation was cancelled.
    pub was_cancelled: bool,
}

/// Type of transfer operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferOperation {
    Copy,
    Move,
}

impl TransferOperation {
    /// Get a display label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Copy => "Copy",
            Self::Move => "Move",
        }
    }
}

impl DetailedTransferReport {
    /// Create a new report builder.
    pub fn builder(job_id: JobId, operation: TransferOperation) -> ReportBuilder {
        ReportBuilder::new(job_id, operation)
    }

    /// Get all failed items.
    pub fn failed_items(&self) -> impl Iterator<Item = &TransferItemResult> {
        self.items.iter().filter(|i| i.status == TransferStatus::Failed)
    }

    /// Get all successful items.
    pub fn successful_items(&self) -> impl Iterator<Item = &TransferItemResult> {
        self.items.iter().filter(|i| i.status == TransferStatus::Success)
    }

    /// Get all skipped items.
    pub fn skipped_items(&self) -> impl Iterator<Item = &TransferItemResult> {
        self.items.iter().filter(|i| i.status == TransferStatus::Skipped)
    }

    /// Export the report to JSON.
    pub fn to_json(&self) -> ZResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| ZError::Internal {
            message: format!("Failed to serialize report: {e}"),
        })
    }

    /// Export the report to a JSON file.
    pub fn save_json(&self, path: &Path) -> ZResult<()> {
        let file = File::create(path).map_err(|e| ZError::io(path, e))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self).map_err(|e| ZError::Internal {
            message: format!("Failed to write report: {e}"),
        })
    }

    /// Export the report to plain text.
    pub fn to_text(&self) -> String {
        let mut out = String::new();

        // Header
        out.push_str(&format!(
            "=== {} Report ===\n",
            self.operation.label()
        ));
        out.push_str(&format!("Job ID: {}\n", self.job_id));
        out.push_str(&format!(
            "Duration: {}\n",
            self.summary.duration_display()
        ));
        out.push('\n');

        // Summary
        out.push_str("--- Summary ---\n");
        out.push_str(&format!("Total items: {}\n", self.summary.total_items));
        out.push_str(&format!(
            "Succeeded: {} ({:.1}%)\n",
            self.summary.succeeded,
            self.summary.success_percentage()
        ));
        out.push_str(&format!("Skipped: {}\n", self.summary.skipped));
        out.push_str(&format!("Failed: {}\n", self.summary.failed));
        out.push_str(&format!(
            "Bytes transferred: {}\n",
            format_bytes(self.summary.bytes_transferred)
        ));
        out.push_str(&format!(
            "Average speed: {}/s\n",
            format_bytes(self.summary.average_speed())
        ));

        if self.was_cancelled {
            out.push_str("\n*** OPERATION WAS CANCELLED ***\n");
        }

        // Failed items
        let failed: Vec<_> = self.failed_items().collect();
        if !failed.is_empty() {
            out.push_str("\n--- Failed Items ---\n");
            for item in failed {
                out.push_str(&format!(
                    "✗ {} → {}\n  Error: {}\n",
                    item.source.display(),
                    item.destination.display(),
                    item.reason.as_deref().unwrap_or("Unknown error")
                ));
            }
        }

        // Skipped items
        let skipped: Vec<_> = self.skipped_items().collect();
        if !skipped.is_empty() {
            out.push_str("\n--- Skipped Items ---\n");
            for item in skipped {
                out.push_str(&format!(
                    "○ {} → {}\n  Reason: {}\n",
                    item.source.display(),
                    item.destination.display(),
                    item.reason.as_deref().unwrap_or("Unknown reason")
                ));
            }
        }

        out
    }

    /// Export the report to a text file.
    pub fn save_text(&self, path: &Path) -> ZResult<()> {
        let mut file = File::create(path).map_err(|e| ZError::io(path, e))?;
        file.write_all(self.to_text().as_bytes())
            .map_err(|e| ZError::io(path, e))
    }
}

/// Builder for constructing transfer reports.
pub struct ReportBuilder {
    job_id: JobId,
    operation: TransferOperation,
    started_at: SystemTime,
    items: Vec<TransferItemResult>,
    was_cancelled: bool,
}

impl ReportBuilder {
    /// Create a new report builder.
    pub fn new(job_id: JobId, operation: TransferOperation) -> Self {
        Self {
            job_id,
            operation,
            started_at: SystemTime::now(),
            items: Vec::new(),
            was_cancelled: false,
        }
    }

    /// Add an item result.
    pub fn add_item(&mut self, item: TransferItemResult) {
        self.items.push(item);
    }

    /// Mark the operation as cancelled.
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.was_cancelled = cancelled;
    }

    /// Build the final report.
    pub fn build(self) -> DetailedTransferReport {
        let completed_at = SystemTime::now();
        let duration = completed_at
            .duration_since(self.started_at)
            .unwrap_or_default();

        let mut summary = TransferSummary {
            total_items: self.items.len(),
            duration_ms: duration.as_millis() as u64,
            ..Default::default()
        };

        for item in &self.items {
            match item.status {
                TransferStatus::Success => {
                    summary.succeeded += 1;
                    summary.bytes_transferred += item.size_bytes;
                    if item.is_directory {
                        summary.directories_created += 1;
                    } else {
                        summary.files_copied += 1;
                    }
                }
                TransferStatus::Skipped => summary.skipped += 1,
                TransferStatus::Failed => summary.failed += 1,
            }
        }

        DetailedTransferReport {
            job_id: self.job_id,
            operation: self.operation,
            started_at: self.started_at,
            completed_at,
            summary,
            items: self.items,
            was_cancelled: self.was_cancelled,
        }
    }
}

/// Format bytes to human-readable string.
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Serde helper for SystemTime.
mod system_time_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u128::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_millis(millis as u64))
    }
}

/// Report storage for persisting reports during and after transfer.
pub struct ReportStorage {
    reports_dir: PathBuf,
}

impl ReportStorage {
    /// Create a new report storage.
    pub fn new(reports_dir: PathBuf) -> Self {
        Self { reports_dir }
    }

    /// Get the default reports directory.
    pub fn default_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ZManager")
            .join("reports")
    }

    /// Ensure the reports directory exists.
    pub fn ensure_dir(&self) -> ZResult<()> {
        std::fs::create_dir_all(&self.reports_dir)
            .map_err(|e| ZError::io(&self.reports_dir, e))
    }

    /// Save a report to storage.
    pub fn save(&self, report: &DetailedTransferReport) -> ZResult<PathBuf> {
        self.ensure_dir()?;

        let duration = report
            .started_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();

        // Use milliseconds for unique filenames
        let timestamp_ms = duration.as_millis();

        let filename = format!(
            "{}_{}.json",
            report.operation.label().to_lowercase(),
            timestamp_ms
        );
        let path = self.reports_dir.join(&filename);

        report.save_json(&path)?;
        Ok(path)
    }

    /// Load a report from storage.
    pub fn load(&self, filename: &str) -> ZResult<DetailedTransferReport> {
        let path = self.reports_dir.join(filename);
        let content = std::fs::read_to_string(&path).map_err(|e| ZError::io(&path, e))?;
        serde_json::from_str(&content).map_err(|e| ZError::Internal {
            message: format!("Failed to parse report: {e}"),
        })
    }

    /// List all stored reports.
    pub fn list(&self) -> ZResult<Vec<String>> {
        if !self.reports_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = std::fs::read_dir(&self.reports_dir)
            .map_err(|e| ZError::io(&self.reports_dir, e))?;

        let mut reports = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Some(name) = path.file_name() {
                    reports.push(name.to_string_lossy().to_string());
                }
            }
        }

        reports.sort();
        reports.reverse(); // Most recent first
        Ok(reports)
    }

    /// Delete old reports, keeping only the most recent N.
    pub fn cleanup(&self, keep_count: usize) -> ZResult<usize> {
        let reports = self.list()?;
        let to_delete = reports.into_iter().skip(keep_count);

        let mut deleted = 0;
        for filename in to_delete {
            let path = self.reports_dir.join(&filename);
            if std::fs::remove_file(&path).is_ok() {
                deleted += 1;
            }
        }

        Ok(deleted)
    }
}

impl Default for ReportStorage {
    fn default() -> Self {
        Self::new(Self::default_dir())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_transfer_status_labels() {
        assert_eq!(TransferStatus::Success.label(), "Success");
        assert_eq!(TransferStatus::Skipped.label(), "Skipped");
        assert_eq!(TransferStatus::Failed.label(), "Failed");
    }

    #[test]
    fn test_transfer_status_symbols() {
        assert_eq!(TransferStatus::Success.symbol(), "✓");
        assert_eq!(TransferStatus::Skipped.symbol(), "○");
        assert_eq!(TransferStatus::Failed.symbol(), "✗");
    }

    #[test]
    fn test_item_result_success() {
        let result = TransferItemResult::success(
            PathBuf::from("src.txt"),
            PathBuf::from("dst.txt"),
            1000,
        );

        assert!(result.is_success());
        assert!(!result.is_failed());
        assert!(!result.is_directory);
        assert_eq!(result.size_bytes, 1000);
    }

    #[test]
    fn test_item_result_dir() {
        let result = TransferItemResult::success_dir(
            PathBuf::from("src_dir"),
            PathBuf::from("dst_dir"),
        );

        assert!(result.is_success());
        assert!(result.is_directory);
        assert_eq!(result.size_bytes, 0);
    }

    #[test]
    fn test_item_result_failed() {
        let result = TransferItemResult::failed(
            PathBuf::from("src.txt"),
            PathBuf::from("dst.txt"),
            "Access denied",
        );

        assert!(result.is_failed());
        assert_eq!(result.reason.as_deref(), Some("Access denied"));
    }

    #[test]
    fn test_summary_percentage() {
        let summary = TransferSummary {
            total_items: 10,
            succeeded: 8,
            skipped: 1,
            failed: 1,
            ..Default::default()
        };

        assert!((summary.success_percentage() - 80.0).abs() < 0.01);
        assert!(!summary.is_complete_success());
    }

    #[test]
    fn test_summary_complete_success() {
        let summary = TransferSummary {
            total_items: 10,
            succeeded: 10,
            skipped: 0,
            failed: 0,
            ..Default::default()
        };

        assert!(summary.is_complete_success());
    }

    #[test]
    fn test_summary_speed() {
        let summary = TransferSummary {
            bytes_transferred: 1_000_000,
            duration_ms: 1000,
            ..Default::default()
        };

        assert_eq!(summary.average_speed(), 1_000_000);
    }

    #[test]
    fn test_summary_duration_display() {
        assert_eq!(
            TransferSummary { duration_ms: 500, ..Default::default() }.duration_display(),
            "500ms"
        );
        assert_eq!(
            TransferSummary { duration_ms: 2500, ..Default::default() }.duration_display(),
            "2.500s"
        );
        assert_eq!(
            TransferSummary { duration_ms: 65000, ..Default::default() }.duration_display(),
            "1m 5s"
        );
        assert_eq!(
            TransferSummary { duration_ms: 3725000, ..Default::default() }.duration_display(),
            "1h 2m 5s"
        );
    }

    #[test]
    fn test_report_builder() {
        let mut builder = ReportBuilder::new(JobId::new(), TransferOperation::Copy);
        builder.add_item(TransferItemResult::success(
            PathBuf::from("a.txt"),
            PathBuf::from("b.txt"),
            100,
        ));
        builder.add_item(TransferItemResult::failed(
            PathBuf::from("c.txt"),
            PathBuf::from("d.txt"),
            "Error",
        ));

        let report = builder.build();

        assert_eq!(report.summary.total_items, 2);
        assert_eq!(report.summary.succeeded, 1);
        assert_eq!(report.summary.failed, 1);
        assert_eq!(report.summary.bytes_transferred, 100);
    }

    #[test]
    fn test_report_json_export() {
        let mut builder = ReportBuilder::new(JobId::new(), TransferOperation::Move);
        builder.add_item(TransferItemResult::success(
            PathBuf::from("src"),
            PathBuf::from("dst"),
            50,
        ));
        let report = builder.build();

        let json = report.to_json().unwrap();
        assert!(json.contains("\"operation\": \"move\""));
        assert!(json.contains("\"succeeded\": 1"));
    }

    #[test]
    fn test_report_text_export() {
        let mut builder = ReportBuilder::new(JobId::new(), TransferOperation::Copy);
        builder.add_item(TransferItemResult::success(
            PathBuf::from("src"),
            PathBuf::from("dst"),
            1000,
        ));
        builder.add_item(TransferItemResult::failed(
            PathBuf::from("bad"),
            PathBuf::from("fail"),
            "Permission denied",
        ));
        let report = builder.build();

        let text = report.to_text();
        assert!(text.contains("=== Copy Report ==="));
        assert!(text.contains("Succeeded: 1"));
        assert!(text.contains("Failed: 1"));
        assert!(text.contains("Permission denied"));
    }

    #[test]
    fn test_report_storage() {
        let temp = TempDir::new().unwrap();
        let storage = ReportStorage::new(temp.path().to_path_buf());

        let mut builder = ReportBuilder::new(JobId::new(), TransferOperation::Copy);
        builder.add_item(TransferItemResult::success(
            PathBuf::from("a"),
            PathBuf::from("b"),
            100,
        ));
        let report = builder.build();

        // Save
        let saved_path = storage.save(&report).unwrap();
        assert!(saved_path.exists());

        // List
        let list = storage.list().unwrap();
        assert_eq!(list.len(), 1);

        // Load
        let loaded = storage.load(&list[0]).unwrap();
        assert_eq!(loaded.summary.succeeded, 1);
    }

    #[test]
    fn test_report_storage_cleanup() {
        let temp = TempDir::new().unwrap();
        let storage = ReportStorage::new(temp.path().to_path_buf());

        // Create several reports with longer delays to ensure different timestamps
        for _i in 0..5 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            let builder = ReportBuilder::new(JobId::new(), TransferOperation::Copy);
            let report = builder.build();
            storage.save(&report).unwrap();
        }

        let initial_count = storage.list().unwrap().len();
        assert_eq!(initial_count, 5);

        // Cleanup, keeping 2
        let deleted = storage.cleanup(2).unwrap();
        assert_eq!(deleted, 3);

        let final_count = storage.list().unwrap().len();
        assert_eq!(final_count, 2);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1500), "1.46 KB");
        assert_eq!(format_bytes(1_500_000), "1.43 MB");
        assert_eq!(format_bytes(1_500_000_000), "1.40 GB");
        assert_eq!(format_bytes(1_500_000_000_000), "1.36 TB");
    }

    #[test]
    fn test_operation_labels() {
        assert_eq!(TransferOperation::Copy.label(), "Copy");
        assert_eq!(TransferOperation::Move.label(), "Move");
    }
}
