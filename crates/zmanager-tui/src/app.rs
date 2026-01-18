//! Application state management.

use std::path::PathBuf;

use ratatui::widgets::ListState;
use tokio::sync::mpsc;
use zmanager_core::{
    Config, DriveInfo, EntryMeta, Favorite, FilterSpec, JobInfo, NavigationState, Properties,
    Selection, SortField as CoreSortField, SortSpec, ZResult,
};

use crate::{
    event::Event,
    input::Action,
    ui::{layout::Pane, ConflictModal, Dialog, SidebarState, SortField},
};

/// Pending operation after dialog confirmation.
#[derive(Debug, Clone)]
pub enum PendingOperation {
    /// Delete the specified files.
    Delete(Vec<PathBuf>),
    /// Rename a file (from, to).
    Rename(PathBuf),
    /// Create a new directory.
    MakeDir,
    /// Copy files to the other pane.
    Copy(Vec<PathBuf>, PathBuf),
    /// Move files to the other pane.
    Move(Vec<PathBuf>, PathBuf),
}

/// View mode for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    /// Normal dual-pane file browser.
    #[default]
    Browser,
    /// Transfers/jobs view.
    Transfers,
}

/// Application state for the TUI.
pub struct App {
    /// Whether the app should quit.
    pub should_quit: bool,

    /// Left pane state.
    pub left: PaneState,

    /// Right pane state.
    pub right: PaneState,

    /// Which pane is active.
    pub active_pane: Pane,

    /// Current sort specification.
    pub sort: SortSpec,

    /// Current filter specification.
    pub filter: FilterSpec,

    /// Whether to show hidden files.
    pub show_hidden: bool,

    /// Active dialog (if any).
    pub dialog: Option<Dialog>,

    /// Pending operation waiting for dialog result.
    pub pending_operation: Option<PendingOperation>,

    /// Current view mode.
    pub view_mode: ViewMode,

    /// Jobs list for transfers view.
    pub jobs: Vec<JobInfo>,

    /// Selected job index in transfers view.
    pub jobs_list_state: ListState,

    /// Active conflict modal (if any).
    pub conflict_modal: Option<ConflictModal>,

    /// Status message to display (with optional timeout).
    pub status_message: Option<(String, bool)>, // (message, is_error)

    /// Whether the sidebar is visible.
    pub sidebar_visible: bool,

    /// Sidebar state.
    pub sidebar_state: SidebarState,

    /// Favorites list.
    pub favorites: Vec<Favorite>,

    /// Available drives.
    pub drives: Vec<DriveInfo>,

    /// Whether help screen is visible.
    pub show_help: bool,

    /// Properties to display (if showing properties panel).
    pub properties: Option<Properties>,

    /// Application config.
    pub config: Config,

    /// Event sender for async operations.
    event_tx: mpsc::UnboundedSender<Event>,
}

/// State for a single pane.
pub struct PaneState {
    /// Navigation state (current path, history).
    pub nav: NavigationState,

    /// Directory entries.
    pub entries: Vec<EntryMeta>,

    /// Selection state.
    pub selection: Selection,

    /// List widget state (for scrolling).
    pub list_state: ListState,
}

impl PaneState {
    /// Create a new pane state at the given path.
    pub fn new(path: PathBuf) -> Self {
        Self {
            nav: NavigationState::new(path),
            entries: Vec::new(),
            selection: Selection::new(),
            list_state: ListState::default(),
        }
    }

    /// Get the current cursor position.
    pub fn cursor(&self) -> usize {
        self.selection.cursor()
    }

    /// Set the cursor position.
    pub fn set_cursor(&mut self, pos: usize) {
        self.selection.set_cursor(pos);
        self.list_state.select(Some(pos.min(self.entries.len().saturating_sub(1))));
    }

    /// Move cursor up.
    pub fn move_up(&mut self) {
        self.selection.move_up();
        self.sync_list_state();
    }

    /// Move cursor down.
    pub fn move_down(&mut self) {
        self.selection.move_down();
        self.sync_list_state();
    }

    /// Page up.
    pub fn page_up(&mut self, page_size: usize) {
        self.selection.page_up(page_size);
        self.sync_list_state();
    }

    /// Page down.
    pub fn page_down(&mut self, page_size: usize) {
        self.selection.page_down(page_size);
        self.sync_list_state();
    }

    /// Go to first item.
    pub fn go_first(&mut self) {
        self.selection.move_to_first();
        self.sync_list_state();
    }

    /// Go to last item.
    pub fn go_last(&mut self) {
        self.selection.move_to_last();
        self.sync_list_state();
    }

    /// Sync list_state with selection cursor.
    fn sync_list_state(&mut self) {
        self.list_state.select(Some(self.selection.cursor()));
    }

    /// Get the entry at the cursor.
    pub fn current_entry(&self) -> Option<&EntryMeta> {
        self.entries.get(self.selection.cursor())
    }

    /// Get indices of selected entries.
    pub fn selected_indices(&self) -> Vec<usize> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| self.selection.is_selected(&entry.path))
            .map(|(i, _)| i)
            .collect()
    }

    /// Calculate total size of selected entries.
    pub fn selected_size(&self) -> u64 {
        self.entries
            .iter()
            .filter(|entry| self.selection.is_selected(&entry.path))
            .map(|e| e.size)
            .sum()
    }

    /// Toggle selection at cursor.
    pub fn toggle_select(&mut self) {
        self.selection.toggle_at_cursor(&self.entries);
    }

    /// Select all entries.
    pub fn select_all(&mut self) {
        self.selection.select_all(&self.entries);
    }

    /// Invert selection.
    pub fn invert_selection(&mut self) {
        self.selection.invert(&self.entries);
    }

    /// Clear selection.
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Update entries and sync selection.
    pub fn set_entries(&mut self, entries: Vec<EntryMeta>) {
        self.entries = entries;
        self.selection.set_entry_count(self.entries.len());
        // Ensure cursor is within bounds
        if self.selection.cursor() >= self.entries.len() && !self.entries.is_empty() {
            self.selection.set_cursor(self.entries.len() - 1);
        }
        self.sync_list_state();
    }
}

impl App {
    /// Create a new application with the given starting paths.
    pub fn new(left_path: PathBuf, right_path: PathBuf, event_tx: mpsc::UnboundedSender<Event>) -> Self {
        // Load config or use defaults
        let config = Config::load().unwrap_or_default();
        let favorites = config.favorites.clone();

        // Load drives
        let drives = zmanager_core::list_drives().unwrap_or_default();

        Self {
            should_quit: false,
            left: PaneState::new(left_path),
            right: PaneState::new(right_path),
            active_pane: Pane::default(),
            sort: SortSpec::default(),
            filter: FilterSpec::default(),
            show_hidden: false,
            dialog: None,
            pending_operation: None,
            view_mode: ViewMode::default(),
            jobs: Vec::new(),
            jobs_list_state: ListState::default(),
            conflict_modal: None,
            status_message: None,
            sidebar_visible: false,
            sidebar_state: SidebarState::new(),
            favorites,
            drives,
            show_help: false,
            properties: None,
            config,
            event_tx,
        }
    }

    /// Get the active pane state.
    pub fn active(&self) -> &PaneState {
        match self.active_pane {
            Pane::Left => &self.left,
            Pane::Right => &self.right,
        }
    }

    /// Get the active pane state mutably.
    pub fn active_mut(&mut self) -> &mut PaneState {
        match self.active_pane {
            Pane::Left => &mut self.left,
            Pane::Right => &mut self.right,
        }
    }

    /// Get the inactive pane state.
    #[allow(dead_code)]
    pub fn inactive(&self) -> &PaneState {
        match self.active_pane {
            Pane::Left => &self.right,
            Pane::Right => &self.left,
        }
    }

    /// Switch to the other pane.
    pub fn switch_pane(&mut self) {
        self.active_pane = self.active_pane.toggle();
    }

    /// Handle an action.
    pub fn handle_action(&mut self, action: Action) -> ZResult<()> {
        match action {
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Up => {
                self.active_mut().move_up();
            }
            Action::Down => {
                self.active_mut().move_down();
            }
            Action::PageUp => {
                self.active_mut().page_up(10);
            }
            Action::PageDown => {
                self.active_mut().page_down(10);
            }
            Action::GoFirst => {
                self.active_mut().go_first();
            }
            Action::GoLast => {
                self.active_mut().go_last();
            }
            Action::Enter | Action::Right => {
                self.enter_directory()?;
            }
            Action::GoParent | Action::Left => {
                self.go_parent()?;
            }
            Action::GoBack => {
                self.go_back()?;
            }
            Action::GoForward => {
                self.go_forward()?;
            }
            Action::SwitchPane => {
                self.switch_pane();
            }
            Action::ToggleSelect => {
                self.active_mut().toggle_select();
                self.active_mut().move_down();
            }
            Action::SelectAll => {
                self.active_mut().select_all();
            }
            Action::InvertSelection => {
                self.active_mut().invert_selection();
            }
            Action::ClearSelection => {
                self.active_mut().clear_selection();
            }
            Action::Refresh => {
                self.refresh_active()?;
            }
            Action::Delete => {
                self.initiate_delete();
            }
            Action::Rename => {
                self.initiate_rename();
            }
            Action::MakeDir => {
                self.initiate_mkdir();
            }
            Action::Copy => {
                self.initiate_copy();
            }
            Action::Move => {
                self.initiate_move();
            }
            Action::ToggleHidden => {
                self.toggle_hidden();
            }
            Action::SortMenu => {
                self.show_sort_menu();
            }
            Action::Open => {
                self.open_current()?;
            }
            Action::ToggleTransfers => {
                self.toggle_transfers_view();
            }
            Action::PauseJob => {
                self.pause_selected_job();
            }
            Action::ResumeJob => {
                self.resume_selected_job();
            }
            Action::CancelJob => {
                self.cancel_selected_job();
            }
            Action::ToggleSidebar => {
                self.toggle_sidebar();
            }
            Action::AddFavorite => {
                self.add_current_to_favorites();
            }
            Action::QuickJump(num) => {
                self.quick_jump_to_favorite(num);
            }
            Action::Properties => {
                self.show_properties();
            }
            Action::Help => {
                self.show_help = true;
            }
            // Not implemented yet
            Action::FilterMenu
            | Action::None => {}
        }
        Ok(())
    }

    /// Enter the directory at cursor.
    fn enter_directory(&mut self) -> ZResult<()> {
        let pane = self.active_mut();
        if let Some(entry) = pane.current_entry().cloned() {
            if entry.kind.is_directory() {
                pane.nav.navigate_to(&entry.path);
                pane.selection.clear();
                pane.set_cursor(0);
                // Request directory refresh
                let _ = self.event_tx.send(Event::DirectoryChanged(entry.path));
            }
        }
        Ok(())
    }

    /// Go to parent directory.
    fn go_parent(&mut self) -> ZResult<()> {
        let pane = self.active_mut();
        if let Some(parent) = pane.nav.current_path().parent() {
            let parent = parent.to_path_buf();
            pane.nav.navigate_to(&parent);
            pane.selection.clear();
            pane.set_cursor(0);
            let _ = self.event_tx.send(Event::DirectoryChanged(parent));
        }
        Ok(())
    }

    /// Go back in history.
    fn go_back(&mut self) -> ZResult<()> {
        let pane = self.active_mut();
        if pane.nav.go_back().is_some() {
            pane.selection.clear();
            pane.set_cursor(0);
        }
        // Send event after releasing mutable borrow
        let path = self.active().nav.current_path().to_path_buf();
        let _ = self.event_tx.send(Event::DirectoryChanged(path));
        Ok(())
    }

    /// Go forward in history.
    fn go_forward(&mut self) -> ZResult<()> {
        let pane = self.active_mut();
        if pane.nav.go_forward().is_some() {
            pane.selection.clear();
            pane.set_cursor(0);
        }
        // Send event after releasing mutable borrow
        let path = self.active().nav.current_path().to_path_buf();
        let _ = self.event_tx.send(Event::DirectoryChanged(path));
        Ok(())
    }

    /// Refresh the active pane.
    fn refresh_active(&mut self) -> ZResult<()> {
        let path = self.active().nav.current_path().to_path_buf();
        let _ = self.event_tx.send(Event::DirectoryChanged(path));
        Ok(())
    }

    /// Update entries for a pane.
    pub fn update_entries(&mut self, pane: Pane, entries: Vec<EntryMeta>) {
        let pane_state = match pane {
            Pane::Left => &mut self.left,
            Pane::Right => &mut self.right,
        };
        pane_state.set_entries(entries);
    }

    // ========== File Operations ==========

    /// Initiate delete operation (shows confirmation dialog).
    fn initiate_delete(&mut self) {
        let files = self.get_operation_targets();
        if files.is_empty() {
            return;
        }

        let count = files.len();
        let message = if count == 1 {
            format!("Delete '{}'?", files[0].file_name().unwrap_or_default().to_string_lossy())
        } else {
            format!("Delete {} items?", count)
        };

        self.pending_operation = Some(PendingOperation::Delete(files));
        self.dialog = Some(Dialog::confirm("Confirm Delete", message));
    }

    /// Initiate rename operation (shows input dialog).
    fn initiate_rename(&mut self) {
        let pane = self.active();
        if let Some(entry) = pane.current_entry() {
            let current_name = entry.path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            self.pending_operation = Some(PendingOperation::Rename(entry.path.clone()));
            self.dialog = Some(Dialog::input("Rename", "New name:", current_name));
        }
    }

    /// Initiate mkdir operation (shows input dialog).
    fn initiate_mkdir(&mut self) {
        self.pending_operation = Some(PendingOperation::MakeDir);
        self.dialog = Some(Dialog::input("New Folder", "Folder name:", ""));
    }

    /// Initiate copy operation.
    fn initiate_copy(&mut self) {
        let files = self.get_operation_targets();
        if files.is_empty() {
            return;
        }

        let destination = self.inactive().nav.current_path().to_path_buf();
        let count = files.len();
        let message = if count == 1 {
            format!("Copy '{}' to other pane?", files[0].file_name().unwrap_or_default().to_string_lossy())
        } else {
            format!("Copy {} items to other pane?", count)
        };

        self.pending_operation = Some(PendingOperation::Copy(files, destination));
        self.dialog = Some(Dialog::confirm("Confirm Copy", message));
    }

    /// Initiate move operation.
    fn initiate_move(&mut self) {
        let files = self.get_operation_targets();
        if files.is_empty() {
            return;
        }

        let destination = self.inactive().nav.current_path().to_path_buf();
        let count = files.len();
        let message = if count == 1 {
            format!("Move '{}' to other pane?", files[0].file_name().unwrap_or_default().to_string_lossy())
        } else {
            format!("Move {} items to other pane?", count)
        };

        self.pending_operation = Some(PendingOperation::Move(files, destination));
        self.dialog = Some(Dialog::confirm("Confirm Move", message));
    }

    /// Get the files to operate on (selection or current).
    fn get_operation_targets(&self) -> Vec<PathBuf> {
        let pane = self.active();
        let selected: Vec<PathBuf> = pane.entries
            .iter()
            .filter(|e| pane.selection.is_selected(&e.path))
            .map(|e| e.path.clone())
            .collect();

        if selected.is_empty() {
            // Use the current entry if no selection
            pane.current_entry()
                .map(|e| vec![e.path.clone()])
                .unwrap_or_default()
        } else {
            selected
        }
    }

    /// Toggle hidden files visibility.
    fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        // Refresh both panes to apply the filter
        let left_path = self.left.nav.current_path().to_path_buf();
        let right_path = self.right.nav.current_path().to_path_buf();
        let _ = self.event_tx.send(Event::DirectoryChanged(left_path));
        let _ = self.event_tx.send(Event::DirectoryChanged(right_path));
    }

    /// Show the sort menu.
    fn show_sort_menu(&mut self) {
        let current = match self.sort.field {
            CoreSortField::Name => SortField::Name,
            CoreSortField::Size => SortField::Size,
            CoreSortField::Modified => SortField::Modified,
            CoreSortField::Extension => SortField::Extension,
            CoreSortField::Kind => SortField::Kind,
            _ => SortField::Name,
        };
        self.dialog = Some(Dialog::sort_menu(current));
    }

    /// Open the current file/directory.
    fn open_current(&mut self) -> ZResult<()> {
        let pane = self.active();
        if let Some(entry) = pane.current_entry() {
            if entry.kind.is_directory() {
                // Navigate into directory
                self.enter_directory()?;
            } else {
                // Open file with default application
                #[cfg(windows)]
                {
                    use std::process::Command;
                    let _ = Command::new("cmd")
                        .args(["/C", "start", "", entry.path.to_string_lossy().as_ref()])
                        .spawn();
                }
            }
        }
        Ok(())
    }

    /// Apply the sort field selection from the menu.
    pub fn apply_sort(&mut self, field: SortField) {
        self.sort.field = match field {
            SortField::Name => CoreSortField::Name,
            SortField::Size => CoreSortField::Size,
            SortField::Modified => CoreSortField::Modified,
            SortField::Extension => CoreSortField::Extension,
            SortField::Kind => CoreSortField::Kind,
        };
        // Refresh to re-sort
        let left_path = self.left.nav.current_path().to_path_buf();
        let right_path = self.right.nav.current_path().to_path_buf();
        let _ = self.event_tx.send(Event::DirectoryChanged(left_path));
        let _ = self.event_tx.send(Event::DirectoryChanged(right_path));
    }

    /// Execute pending delete operation.
    pub fn execute_delete(&mut self, files: Vec<PathBuf>) {
        // Send delete event to be handled asynchronously
        let _ = self.event_tx.send(Event::ExecuteDelete(files));
    }

    /// Execute pending rename operation.
    pub fn execute_rename(&mut self, old_path: PathBuf, new_name: String) {
        let new_path = old_path.parent()
            .map(|p| p.join(&new_name))
            .unwrap_or_else(|| PathBuf::from(&new_name));
        let _ = self.event_tx.send(Event::ExecuteRename(old_path, new_path));
    }

    /// Execute pending mkdir operation.
    pub fn execute_mkdir(&mut self, name: String) {
        let parent = self.active().nav.current_path().to_path_buf();
        let new_path = parent.join(&name);
        let _ = self.event_tx.send(Event::ExecuteMkdir(new_path));
    }

    /// Execute pending copy operation.
    pub fn execute_copy(&mut self, sources: Vec<PathBuf>, destination: PathBuf) {
        let _ = self.event_tx.send(Event::ExecuteCopy(sources, destination));
    }

    /// Execute pending move operation.
    pub fn execute_move(&mut self, sources: Vec<PathBuf>, destination: PathBuf) {
        let _ = self.event_tx.send(Event::ExecuteMove(sources, destination));
    }

    /// Show an error message dialog.
    pub fn show_error(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.dialog = Some(Dialog::error(title, message));
    }

    /// Show an info message dialog.
    pub fn show_message(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.dialog = Some(Dialog::message(title, message));
    }

    /// Check if there's an active dialog.
    pub fn has_dialog(&self) -> bool {
        self.dialog.is_some()
    }

    /// Close the current dialog.
    pub fn close_dialog(&mut self) {
        self.dialog = None;
        self.pending_operation = None;
    }

    // ========== Transfers View ==========

    /// Toggle transfers view.
    pub fn toggle_transfers_view(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Browser => ViewMode::Transfers,
            ViewMode::Transfers => ViewMode::Browser,
        };
        // Reset list selection when entering transfers view
        if self.view_mode == ViewMode::Transfers {
            self.jobs_list_state.select(Some(0));
        }
    }

    /// Get the currently selected job ID (in transfers view).
    pub fn selected_job(&self) -> Option<&JobInfo> {
        self.jobs_list_state.selected()
            .and_then(|i| self.jobs.get(i))
    }

    /// Pause the selected job.
    pub fn pause_selected_job(&mut self) {
        if self.view_mode != ViewMode::Transfers {
            return;
        }
        if let Some(job) = self.selected_job() {
            let _ = self.event_tx.send(Event::PauseJob(job.id.0));
        }
    }

    /// Resume the selected job.
    pub fn resume_selected_job(&mut self) {
        if self.view_mode != ViewMode::Transfers {
            return;
        }
        if let Some(job) = self.selected_job() {
            let _ = self.event_tx.send(Event::ResumeJob(job.id.0));
        }
    }

    /// Cancel the selected job.
    pub fn cancel_selected_job(&mut self) {
        if self.view_mode != ViewMode::Transfers {
            return;
        }
        if let Some(job) = self.selected_job() {
            let _ = self.event_tx.send(Event::CancelJob(job.id.0));
        }
    }

    /// Update the jobs list.
    pub fn update_jobs(&mut self, jobs: Vec<JobInfo>) {
        self.jobs = jobs;
        // Ensure selection is valid
        if let Some(selected) = self.jobs_list_state.selected() {
            if selected >= self.jobs.len() && !self.jobs.is_empty() {
                self.jobs_list_state.select(Some(self.jobs.len() - 1));
            }
        }
    }

    /// Move selection up in transfers view.
    pub fn jobs_up(&mut self) {
        if let Some(selected) = self.jobs_list_state.selected() {
            if selected > 0 {
                self.jobs_list_state.select(Some(selected - 1));
            }
        } else if !self.jobs.is_empty() {
            self.jobs_list_state.select(Some(0));
        }
    }

    /// Move selection down in transfers view.
    pub fn jobs_down(&mut self) {
        if let Some(selected) = self.jobs_list_state.selected() {
            if selected < self.jobs.len().saturating_sub(1) {
                self.jobs_list_state.select(Some(selected + 1));
            }
        } else if !self.jobs.is_empty() {
            self.jobs_list_state.select(Some(0));
        }
    }

    /// Set a status message (will be shown in status bar).
    pub fn set_status(&mut self, message: impl Into<String>, is_error: bool) {
        self.status_message = Some((message.into(), is_error));
    }

    /// Clear the status message.
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Check if in transfers view.
    pub fn is_transfers_view(&self) -> bool {
        self.view_mode == ViewMode::Transfers
    }

    /// Check if there's an active conflict modal.
    pub fn has_conflict(&self) -> bool {
        self.conflict_modal.is_some()
    }

    /// Close the conflict modal.
    pub fn close_conflict(&mut self) {
        self.conflict_modal = None;
    }

    // ========== Sidebar / Quick Access ==========

    /// Toggle sidebar visibility.
    pub fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
        if self.sidebar_visible {
            // Refresh drives when showing sidebar
            self.drives = zmanager_core::list_drives().unwrap_or_default();
        }
    }

    /// Add current directory to favorites.
    pub fn add_current_to_favorites(&mut self) {
        let path = self.active().nav.current_path().to_path_buf();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Favorite")
            .to_string();

        let favorite = Favorite::new(name.clone(), path);
        self.config.add_favorite(favorite);
        self.favorites = self.config.favorites.clone();

        // Try to save config
        if let Err(e) = self.config.save() {
            self.set_status(format!("Failed to save config: {}", e), true);
        } else {
            self.set_status(format!("Added '{}' to favorites", name), false);
        }
    }

    /// Quick jump to a favorite by number (1-9).
    pub fn quick_jump_to_favorite(&mut self, num: u8) {
        // Only works when sidebar is visible
        if !self.sidebar_visible {
            return;
        }

        let idx = (num as usize).saturating_sub(1);
        if idx < self.favorites.len() {
            self.sidebar_state.select_by_number(num as usize, self.favorites.len());
            // Navigate to the favorite
            if let Some(fav) = self.favorites.get(idx) {
                if fav.is_valid() {
                    self.navigate_to_path(fav.path.clone());
                } else {
                    self.set_status(format!("Favorite '{}' is broken", fav.name), true);
                }
            }
        }
    }

    /// Navigate to a specific path.
    pub fn navigate_to_path(&mut self, path: PathBuf) {
        let pane = self.active_mut();
        pane.nav.navigate_to(&path);
        pane.selection.clear();
        pane.set_cursor(0);
        let _ = self.event_tx.send(Event::DirectoryChanged(path));
    }

    /// Navigate to the selected sidebar item.
    pub fn navigate_to_sidebar_selection(&mut self) {
        match self.sidebar_state.section {
            crate::ui::SidebarSection::Favorites => {
                if let Some(idx) = self.sidebar_state.selected_favorite() {
                    if let Some(fav) = self.favorites.get(idx) {
                        if fav.is_valid() {
                            self.navigate_to_path(fav.path.clone());
                        } else {
                            self.set_status(format!("Favorite '{}' is broken", fav.name), true);
                        }
                    }
                }
            }
            crate::ui::SidebarSection::Drives => {
                if let Some(idx) = self.sidebar_state.selected_drive() {
                    if let Some(drive) = self.drives.get(idx) {
                        if drive.is_ready {
                            self.navigate_to_path(drive.path.clone());
                        } else {
                            self.set_status("Drive is not ready", true);
                        }
                    }
                }
            }
        }
    }

    /// Move sidebar selection up.
    pub fn sidebar_up(&mut self) {
        self.sidebar_state.up(self.favorites.len(), self.drives.len());
    }

    /// Move sidebar selection down.
    pub fn sidebar_down(&mut self) {
        self.sidebar_state.down(self.favorites.len(), self.drives.len());
    }

    /// Toggle sidebar section.
    pub fn sidebar_toggle_section(&mut self) {
        self.sidebar_state.toggle_section();
    }

    /// Remove selected favorite from sidebar.
    pub fn remove_selected_favorite(&mut self) {
        if let Some(idx) = self.sidebar_state.selected_favorite() {
            if let Some(fav) = self.favorites.get(idx) {
                let id = fav.id.clone();
                let name = fav.name.clone();
                self.config.remove_favorite(&id);
                self.favorites = self.config.favorites.clone();

                if let Err(e) = self.config.save() {
                    self.set_status(format!("Failed to save config: {}", e), true);
                } else {
                    self.set_status(format!("Removed '{}' from favorites", name), false);
                }
            }
        }
    }

    // ========== Properties ==========

    /// Show properties for the current entry.
    pub fn show_properties(&mut self) {
        if let Some(entry) = self.active().current_entry() {
            match zmanager_core::get_properties(&entry.path) {
                Ok(props) => {
                    self.properties = Some(props);
                }
                Err(e) => {
                    self.set_status(format!("Failed to get properties: {}", e), true);
                }
            }
        }
    }

    /// Close the properties panel.
    pub fn close_properties(&mut self) {
        self.properties = None;
    }

    /// Check if properties panel is visible.
    pub fn has_properties(&self) -> bool {
        self.properties.is_some()
    }

    /// Close the help screen.
    pub fn close_help(&mut self) {
        self.show_help = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_app() -> App {
        let (tx, _rx) = mpsc::unbounded_channel();
        App::new(PathBuf::from("C:\\"), PathBuf::from("D:\\"), tx)
    }

    #[test]
    fn app_starts_with_left_pane_active() {
        let app = create_test_app();
        assert_eq!(app.active_pane, Pane::Left);
    }

    #[test]
    fn switch_pane_toggles() {
        let mut app = create_test_app();
        app.switch_pane();
        assert_eq!(app.active_pane, Pane::Right);
        app.switch_pane();
        assert_eq!(app.active_pane, Pane::Left);
    }

    #[test]
    fn quit_action_sets_flag() {
        let mut app = create_test_app();
        app.handle_action(Action::Quit).unwrap();
        assert!(app.should_quit);
    }
}
