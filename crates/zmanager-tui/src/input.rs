//! Input handling and key mappings.
//!
//! This module defines the key bindings and input actions.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Actions that can be performed in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Quit the application.
    Quit,
    /// Move cursor up.
    Up,
    /// Move cursor down.
    Down,
    /// Move cursor left (Vim h).
    Left,
    /// Move cursor right / enter directory (Vim l).
    Right,
    /// Enter the selected directory.
    Enter,
    /// Go to parent directory.
    GoParent,
    /// Go back in history.
    GoBack,
    /// Go forward in history.
    GoForward,
    /// Toggle selection on current item.
    ToggleSelect,
    /// Select all items.
    SelectAll,
    /// Invert selection.
    InvertSelection,
    /// Clear selection.
    ClearSelection,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Go to first item.
    GoFirst,
    /// Go to last item.
    GoLast,
    /// Toggle hidden files.
    ToggleHidden,
    /// Refresh current directory.
    Refresh,
    /// Switch focus to other pane.
    SwitchPane,
    /// Copy selected items.
    Copy,
    /// Move selected items.
    Move,
    /// Delete selected items.
    Delete,
    /// Rename current item.
    Rename,
    /// Create new directory.
    MakeDir,
    /// Open file with default application.
    Open,
    /// Show file properties.
    Properties,
    /// Open sort menu.
    SortMenu,
    /// Open filter menu.
    FilterMenu,
    /// Open help.
    Help,
    /// Toggle transfers view.
    ToggleTransfers,
    /// Pause selected job.
    PauseJob,
    /// Resume selected job.
    ResumeJob,
    /// Cancel selected job.
    CancelJob,
    /// Toggle sidebar.
    ToggleSidebar,
    /// Add current directory to favorites.
    AddFavorite,
    /// Quick jump to favorite (1-9).
    QuickJump(u8),
    /// No action.
    None,
}

/// Map a key event to an action.
pub fn map_key(key: KeyEvent) -> Action {
    match (key.modifiers, key.code) {
        // Quit
        (KeyModifiers::NONE, KeyCode::Char('q')) => Action::Quit,
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => Action::Quit,
        (KeyModifiers::CONTROL, KeyCode::Char('q')) => Action::Quit,

        // Navigation - Arrow keys
        (KeyModifiers::NONE, KeyCode::Up) => Action::Up,
        (KeyModifiers::NONE, KeyCode::Down) => Action::Down,
        (KeyModifiers::NONE, KeyCode::Left) => Action::GoParent,
        (KeyModifiers::NONE, KeyCode::Right) => Action::Enter,

        // Navigation - Vim keys
        (KeyModifiers::NONE, KeyCode::Char('k')) => Action::Up,
        (KeyModifiers::NONE, KeyCode::Char('j')) => Action::Down,
        (KeyModifiers::NONE, KeyCode::Char('h')) => Action::GoParent,
        (KeyModifiers::NONE, KeyCode::Char('l')) => Action::Enter,

        // Enter directory
        (KeyModifiers::NONE, KeyCode::Enter) => Action::Enter,

        // Parent directory
        (KeyModifiers::NONE, KeyCode::Backspace) => Action::GoParent,
        (KeyModifiers::ALT, KeyCode::Up) => Action::GoParent,

        // History
        (KeyModifiers::ALT, KeyCode::Left) => Action::GoBack,
        (KeyModifiers::ALT, KeyCode::Right) => Action::GoForward,
        (KeyModifiers::NONE, KeyCode::Char('[')) => Action::GoBack,
        (KeyModifiers::NONE, KeyCode::Char(']')) => Action::GoForward,

        // Selection
        (KeyModifiers::NONE, KeyCode::Char(' ')) => Action::ToggleSelect,
        (KeyModifiers::CONTROL, KeyCode::Char('a')) => Action::SelectAll,
        (KeyModifiers::NONE, KeyCode::Char('*')) => Action::InvertSelection,
        (KeyModifiers::NONE, KeyCode::Esc) => Action::ClearSelection,

        // Page navigation
        (KeyModifiers::NONE, KeyCode::PageUp) => Action::PageUp,
        (KeyModifiers::NONE, KeyCode::PageDown) => Action::PageDown,
        (KeyModifiers::CONTROL, KeyCode::Char('u')) => Action::PageUp,
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => Action::PageDown,
        (KeyModifiers::NONE, KeyCode::Home) => Action::GoFirst,
        (KeyModifiers::NONE, KeyCode::End) => Action::GoLast,
        (KeyModifiers::NONE, KeyCode::Char('g')) => Action::GoFirst,
        (KeyModifiers::SHIFT, KeyCode::Char('G')) => Action::GoLast,

        // View toggles
        (KeyModifiers::NONE, KeyCode::Char('.')) => Action::ToggleHidden,
        (KeyModifiers::NONE, KeyCode::F(5)) => Action::Refresh,
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => Action::Refresh,

        // Pane switching
        (KeyModifiers::NONE, KeyCode::Tab) => Action::SwitchPane,

        // File operations
        (KeyModifiers::SHIFT, KeyCode::Char('C')) => Action::Copy,
        (KeyModifiers::SHIFT, KeyCode::Char('M')) => Action::Move,
        (KeyModifiers::NONE, KeyCode::Char('d')) => Action::Delete,
        (KeyModifiers::NONE, KeyCode::Delete) => Action::Delete,
        (KeyModifiers::NONE, KeyCode::Char('r')) => Action::Rename,
        (KeyModifiers::NONE, KeyCode::F(2)) => Action::Rename,
        (KeyModifiers::NONE, KeyCode::Char('n')) => Action::MakeDir,
        (KeyModifiers::NONE, KeyCode::Char('o')) => Action::Open,

        // Info
        (KeyModifiers::NONE, KeyCode::Char('p')) => Action::Properties,
        (KeyModifiers::NONE, KeyCode::Char('i')) => Action::Properties,
        (KeyModifiers::NONE, KeyCode::Char('s')) => Action::SortMenu,
        (KeyModifiers::NONE, KeyCode::Char('f')) => Action::FilterMenu,
        (KeyModifiers::NONE, KeyCode::Char('?')) => Action::Help,
        (KeyModifiers::NONE, KeyCode::F(1)) => Action::Help,

        // Transfers view
        (KeyModifiers::NONE, KeyCode::Char('t')) => Action::ToggleTransfers,
        (KeyModifiers::SHIFT, KeyCode::Char('P')) => Action::PauseJob,
        (KeyModifiers::SHIFT, KeyCode::Char('R')) => Action::ResumeJob,
        (KeyModifiers::SHIFT, KeyCode::Char('X')) => Action::CancelJob,

        // Sidebar / Quick Access
        (KeyModifiers::CONTROL, KeyCode::Char('b')) => Action::ToggleSidebar,
        (KeyModifiers::SHIFT, KeyCode::Char('D')) => Action::AddFavorite,

        // Quick jump to favorites (1-9)
        (KeyModifiers::NONE, KeyCode::Char('1')) => Action::QuickJump(1),
        (KeyModifiers::NONE, KeyCode::Char('2')) => Action::QuickJump(2),
        (KeyModifiers::NONE, KeyCode::Char('3')) => Action::QuickJump(3),
        (KeyModifiers::NONE, KeyCode::Char('4')) => Action::QuickJump(4),
        (KeyModifiers::NONE, KeyCode::Char('5')) => Action::QuickJump(5),
        (KeyModifiers::NONE, KeyCode::Char('6')) => Action::QuickJump(6),
        (KeyModifiers::NONE, KeyCode::Char('7')) => Action::QuickJump(7),
        (KeyModifiers::NONE, KeyCode::Char('8')) => Action::QuickJump(8),
        (KeyModifiers::NONE, KeyCode::Char('9')) => Action::QuickJump(9),

        // Default
        _ => Action::None,
    }
}
