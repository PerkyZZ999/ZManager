//! # ZManager Core
//!
//! Core library providing domain types, error handling, and shared functionality
//! for the ZManager file manager.
//!
//! This crate is platform-agnostic and contains:
//! - Domain types (`EntryKind`, `EntryMeta`, `DirListing`)
//! - Sorting and filtering specifications
//! - Error types and result aliases
//! - File system operations
//! - Navigation state management
//! - Selection model
//! - File operations (rename, delete, mkdir)
//! - Job system for async operations
//! - Configuration management
//! - Drive enumeration
//! - File/folder properties
//! - Directory watching with debouncing
//!
//! Both the TUI and GUI frontends depend on this crate.

pub mod config;
pub mod drives;
pub mod entry;
pub mod error;
pub mod filter;
pub mod fs;
pub mod job;
pub mod navigation;
pub mod operations;
pub mod properties;
pub mod recycle;
pub mod scheduler;
pub mod selection;
pub mod sort;
pub mod watcher;

// Re-export main types for convenience
pub use config::{Config, Favorite, SessionState};
pub use drives::{list_drives, DriveInfo, DriveType};
pub use entry::{DirListing, EntryAttributes, EntryKind, EntryMeta};
pub use error::{ZError, ZResult};
pub use filter::FilterSpec;
pub use fs::{get_entry_meta, list_directory};
pub use job::{CancellationToken, Job, JobId, JobInfo, JobKind, JobState, JobStats, Progress};
pub use navigation::NavigationState;
pub use operations::{delete_permanent, mkdir, open_default, rename};
pub use properties::{calculate_folder_stats, get_properties, FolderStats, Properties};
pub use recycle::{move_multiple_to_recycle_bin, move_to_recycle_bin};
pub use scheduler::{Scheduler, SchedulerConfig, SchedulerEvent, SchedulerHandle};
pub use selection::{ClickModifiers, Selection};
pub use sort::{SortField, SortOrder, SortSpec};
pub use watcher::{DirectoryWatcher, WatcherConfig, WatchEvent, WatchEventKind};
