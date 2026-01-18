//! # ZManager Transfer Engine (Windows)
//!
//! Windows-specific file transfer engine using CopyFileEx for efficient
//! file copying with progress callbacks.
//!
//! This crate provides:
//! - Single file copy with progress via `CopyFileExW`
//! - Folder copy/move operations with conflict resolution
//! - Transfer planning and enumeration
//! - Transfer reporting with JSON/text export
//! - Windows clipboard integration (CF_HDROP)
//! - Job scheduling and management
//! - Cancellation and pause support

pub mod clipboard;
pub mod conflict;
pub mod copy;
pub mod executor;
pub mod folder;
pub mod job;
pub mod plan;
pub mod report;

// Re-export main types
pub use clipboard::{
    clear_clipboard, clipboard_has_files, read_files_from_clipboard, write_files_to_clipboard,
    Clipboard, ClipboardContent, DropEffect,
};
pub use conflict::{Conflict, ConflictPolicy, ConflictResolution, ConflictResolver};
pub use copy::{copy_file_async, copy_file_with_progress, CopyProgress, CopyResult};
pub use executor::{CopyExecutor, ExecutorConfig, ExecutorEvent};
pub use folder::{
    FolderTransferConfig, FolderTransferEvent, FolderTransferExecutor, ItemResult, TransferReport,
};
pub use job::{JobId, JobKind, JobState, Progress};
pub use plan::{same_volume, TransferItem, TransferPlan, TransferPlanBuilder, TransferStats};
pub use report::{
    DetailedTransferReport, ReportBuilder, ReportStorage, TransferItemResult, TransferOperation,
    TransferStatus, TransferSummary,
};

/// Initialize the transfer engine.
///
/// This sets up tracing and any required Windows resources.
pub fn init() {
    tracing::info!("ZManager Transfer Engine (Windows) initialized");
}
