//! UI rendering components.
//!
//! This module contains all the widgets and rendering logic
//! for the TUI interface.

pub mod conflict;
pub mod dialog;
pub mod file_list;
pub mod header;
pub mod help;
pub mod layout;
pub mod properties;
pub mod sidebar;
pub mod status_bar;
pub mod styles;
pub mod transfers;

pub use conflict::{ConflictInfo, ConflictModal, ConflictResolution, ConflictResult};
pub use dialog::{Dialog, DialogKind, DialogResult, SortField};
pub use file_list::FileList;
pub use header::Header;
pub use help::{handle_help_key, HelpScreen};
pub use layout::{AppLayout, Pane};
pub use properties::{handle_properties_key, PropertiesPanel};
pub use sidebar::{Sidebar, SidebarSection, SidebarState};
pub use status_bar::StatusBar;
pub use styles::Styles;
pub use transfers::{TransferStatus, TransfersView};
