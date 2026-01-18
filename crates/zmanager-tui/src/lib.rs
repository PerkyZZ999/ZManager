//! ZManager TUI - Terminal User Interface Library
//!
//! This crate provides a terminal-based dual-pane file manager
//! built with Ratatui and Crossterm.

pub mod app;
pub mod crash;
pub mod event;
pub mod input;
pub mod terminal;
pub mod ui;

pub use app::App;
pub use crash::{check_for_crash_dumps, clear_crash_dump, install_panic_hook, CrashDump};
pub use event::Event;
pub use terminal::Tui;
