//! ZManager TUI - Terminal User Interface
//!
//! A dual-pane file manager for the terminal.

use std::path::PathBuf;

use anyhow::Result;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use zmanager_core::{list_directory, DirectoryWatcher};
use zmanager_tui::{
    app::{App, PendingOperation, ViewMode},
    check_for_crash_dumps, clear_crash_dump,
    event::{Event, EventHandler},
    input::{map_key, Action},
    install_panic_hook,
    terminal::Tui,
    ui::{
        file_list::FileList,
        handle_help_key, handle_properties_key,
        header::Header,
        layout::{AppLayout, Pane},
        status_bar::StatusBar,
        DialogResult, HelpScreen, PropertiesPanel, Sidebar, TransfersView,
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    // Install panic hook for crash reporting (must be done before anything else)
    install_panic_hook();
    
    // Initialize tracing to file (not stdout, since we're using the terminal)
    let file_appender = tracing_appender::rolling::daily("logs", "zmanager.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("ZManager TUI starting...");
    
    // Check for crash dumps from previous runs
    if let Some(dump) = check_for_crash_dumps() {
        warn!("Previous crash detected: {}", dump.summary());
        // Clear the crash dump after logging
        clear_crash_dump(&dump);
    }

    // Get starting paths
    let left_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("C:\\"));
    let right_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("C:\\"));

    // Run the application
    let result = run(left_path, right_path).await;

    if let Err(ref e) = result {
        error!("Application error: {}", e);
    }

    info!("ZManager TUI exiting.");
    result
}

async fn run(left_path: PathBuf, right_path: PathBuf) -> Result<()> {
    // Create event handler (200ms tick rate)
    let mut event_handler = EventHandler::new(200);
    let event_tx = event_handler.sender();

    // Create application state
    let mut app = App::new(left_path.clone(), right_path.clone(), event_tx.clone());

    // Initialize terminal
    let mut tui = Tui::new()?;
    tui.enter()?;

    // Start event handler
    event_handler.start();

    // Set up directory watcher for auto-refresh
    let watcher = DirectoryWatcher::new()?;
    
    // Watch both pane directories
    watcher.watch(&left_path)?;
    watcher.watch(&right_path)?;
    
    // Subscribe to watcher events
    let mut watch_rx = watcher.subscribe();

    // Load initial directory contents
    load_directory(&mut app, Pane::Left, &left_path)?;
    load_directory(&mut app, Pane::Right, &right_path)?;

    // Main event loop
    loop {
        // Render
        tui.draw(|frame| {
            render(&app, frame);
        })?;

        // Handle events from multiple sources using tokio::select
        tokio::select! {
            // Handle TUI events
            event = event_handler.next() => {
                match event {
                    Some(Event::Key(key)) => {
                        // Check for modal overlays first (in order of priority)
                        if app.show_help {
                            if handle_help_key(key) {
                                app.close_help();
                            }
                        } else if app.has_properties() {
                            if handle_properties_key(key) {
                                app.close_properties();
                            }
                        } else if app.has_conflict() {
                            handle_conflict_key(&mut app, key);
                        } else if app.has_dialog() {
                            handle_dialog_key(&mut app, key);
                        } else if app.view_mode == ViewMode::Transfers {
                            handle_transfers_key(&mut app, key);
                        } else if app.sidebar_visible {
                            handle_sidebar_key(&mut app, key)?;
                        } else {
                            let action = map_key(key);
                            debug!("Key: {:?} -> Action: {:?}", key, action);
                            app.handle_action(action)?;
                        }
                    }
                    Some(Event::Tick) => {
                        // Clear old status messages after 3 seconds
                        // (Would need timestamp tracking for proper implementation)
                    }
                    Some(Event::Resize(_, _)) => {
                        // Terminal resized, will re-render on next loop
                    }
                    Some(Event::DirectoryChanged(path)) => {
                        // Reload directory contents
                        let pane = app.active_pane;
                        if let Err(e) = load_directory(&mut app, pane, &path) {
                            error!("Failed to load directory: {}", e);
                        }
                    }
                    Some(Event::ExecuteDelete(files)) => {
                        execute_delete(&mut app, files);
                    }
                    Some(Event::ExecuteRename(old_path, new_path)) => {
                        execute_rename(&mut app, old_path, new_path);
                    }
                    Some(Event::ExecuteMkdir(path)) => {
                        execute_mkdir(&mut app, path);
                    }
                    Some(Event::ExecuteCopy(sources, dest)) => {
                        execute_copy(&mut app, sources, dest);
                    }
                    Some(Event::ExecuteMove(sources, dest)) => {
                        execute_move(&mut app, sources, dest);
                    }
                    Some(Event::PauseJob(job_id)) => {
                        debug!("Pausing job {}", job_id);
                        app.set_status(format!("Paused job {}", job_id), false);
                    }
                    Some(Event::ResumeJob(job_id)) => {
                        debug!("Resuming job {}", job_id);
                        app.set_status(format!("Resumed job {}", job_id), false);
                    }
                    Some(Event::CancelJob(job_id)) => {
                        debug!("Cancelling job {}", job_id);
                        app.set_status(format!("Cancelled job {}", job_id), false);
                    }
                    Some(Event::JobsUpdated(jobs)) => {
                        app.update_jobs(jobs);
                    }
                    Some(Event::RefreshAll) => {
                        let left = app.left.nav.current_path().to_path_buf();
                        let right = app.right.nav.current_path().to_path_buf();
                        let _ = load_directory(&mut app, Pane::Left, &left);
                        let _ = load_directory(&mut app, Pane::Right, &right);
                    }
                    Some(Event::Quit) => {
                        app.should_quit = true;
                    }
                    Some(Event::Error(msg)) => {
                        error!("Event error: {}", msg);
                        app.show_error("Error", msg);
                    }
                    Some(_) => {}
                    None => {
                        // Channel closed
                        break;
                    }
                }
            }
            
            // Handle file watcher events
            watch_event = watch_rx.recv() => {
                if let Ok(event) = watch_event {
                    debug!("File watcher event: {:?}", event);
                    // Auto-refresh the appropriate pane
                    let left_dir = app.left.nav.current_path().to_path_buf();
                    let right_dir = app.right.nav.current_path().to_path_buf();
                    
                    if event.directory == left_dir {
                        if let Err(e) = load_directory(&mut app, Pane::Left, &left_dir) {
                            warn!("Auto-refresh failed for left pane: {}", e);
                        }
                    }
                    if event.directory == right_dir {
                        if let Err(e) = load_directory(&mut app, Pane::Right, &right_dir) {
                            warn!("Auto-refresh failed for right pane: {}", e);
                        }
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Cleanup
    tui.exit()?;
    Ok(())
}

fn load_directory(app: &mut App, pane: Pane, path: &PathBuf) -> Result<()> {
    let sort = Some(&app.sort);
    
    // Apply hidden filter if needed
    let mut filter = app.filter.clone();
    if !app.show_hidden {
        // Filter will exclude hidden files (handled in list_directory)
        filter.show_hidden = false;
    } else {
        filter.show_hidden = true;
    }
    
    let filter_ref = if filter.is_default() && app.show_hidden { None } else { Some(&filter) };
    let listing = list_directory(path, sort, filter_ref)?;
    app.update_entries(pane, listing.entries);
    debug!("Loaded {} entries from {:?}", app.active().entries.len(), path);
    Ok(())
}

fn render(app: &App, frame: &mut ratatui::Frame) {
    use ratatui::layout::{Constraint, Direction, Layout};
    
    let layout = AppLayout::new(frame);
    let (base_left_area, right_area) = layout.dual_panes();

    // Check if we're in transfers view mode
    if app.view_mode == ViewMode::Transfers {
        render_transfers_view(app, frame, &layout);
        return;
    }

    // Determine if sidebar is visible and split the left area
    let (sidebar_area, left_area) = if app.sidebar_visible {
        // Split the left pane horizontally: sidebar on the left (25%), file list on the right (75%)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(base_left_area);
        (Some(chunks[0]), chunks[1])
    } else {
        (None, base_left_area)
    };

    // Render sidebar if visible
    if let Some(sidebar_rect) = sidebar_area {
        let sidebar = Sidebar::new(&app.favorites, &app.drives, app.sidebar_state.section);
        let mut sidebar_state = app.sidebar_state.clone();
        frame.render_stateful_widget(sidebar, sidebar_rect, &mut sidebar_state);
    }

    // Render left pane header
    let left_header = Header::new(app.left.nav.current_path(), app.active_pane == Pane::Left);
    frame.render_widget(left_header, layout.left_header);

    // Render right pane header
    let right_header = Header::new(app.right.nav.current_path(), app.active_pane == Pane::Right);
    frame.render_widget(right_header, layout.right_header);

    // Render left file list
    let left_selected = app.left.selected_indices();
    let left_list = FileList::new(&app.left.entries, &left_selected, app.active_pane == Pane::Left);
    let mut left_state = app.left.list_state.clone();
    frame.render_stateful_widget(left_list, left_area, &mut left_state);

    // Render right file list
    let right_selected = app.right.selected_indices();
    let right_list = FileList::new(&app.right.entries, &right_selected, app.active_pane == Pane::Right);
    let mut right_state = app.right.list_state.clone();
    frame.render_stateful_widget(right_list, right_area, &mut right_state);

    // Render status bar (may include status message)
    render_status_bar(app, frame, &layout);

    // Render conflict modal on top if present
    if let Some(ref modal) = app.conflict_modal {
        modal.render(frame.area(), frame.buffer_mut());
    }

    // Render dialog on top if present
    if let Some(dialog) = &app.dialog {
        dialog.render(frame.area(), frame.buffer_mut());
    }

    // Render help screen on top if shown
    if app.show_help {
        let help = HelpScreen;
        frame.render_widget(help, frame.area());
    }

    // Render properties panel on top if shown
    if let Some(ref props) = app.properties {
        let panel = PropertiesPanel::new(props);
        frame.render_widget(panel, frame.area());
    }
}

fn render_transfers_view(app: &App, frame: &mut ratatui::Frame, layout: &AppLayout) {
    // Use the full dual-pane area for transfers
    let (left_area, right_area) = layout.dual_panes();
    let full_area = ratatui::layout::Rect {
        x: left_area.x,
        y: layout.left_header.y,
        width: left_area.width + right_area.width + 1, // +1 for the divider
        height: left_area.height + layout.left_header.height,
    };
    
    // Create transfers view
    let transfers = TransfersView::new(&app.jobs, true);
    let mut list_state = app.jobs_list_state.clone();
    frame.render_stateful_widget(transfers, full_area, &mut list_state);
    
    // Render status bar
    render_status_bar(app, frame, layout);
}

fn render_status_bar(app: &App, frame: &mut ratatui::Frame, layout: &AppLayout) {
    use ratatui::style::{Color, Style};
    use ratatui::text::Span;
    use ratatui::widgets::Paragraph;
    
    // Check for status message first
    if let Some((ref message, is_error)) = app.status_message {
        let style = if is_error {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        let status = Paragraph::new(Span::styled(message.as_str(), style));
        frame.render_widget(status, layout.status);
    } else if app.view_mode == ViewMode::Transfers {
        // Show transfers-specific status bar
        let job_count = app.jobs.len();
        let active_count = app.jobs.iter().filter(|j| j.state == zmanager_core::JobState::Running).count();
        let status_text = format!(
            " {} job(s) | {} active | [P]ause [R]esume [X]Cancel [t]Back to browser",
            job_count, active_count
        );
        let status = Paragraph::new(status_text);
        frame.render_widget(status, layout.status);
    } else {
        // Normal status bar
        let active = app.active();
        let status = StatusBar::new(
            active.entries.len(),
            active.selected_indices().len(),
            active.selected_size(),
        );
        frame.render_widget(status, layout.status);
    }
}

// ========== Dialog Handling ==========

fn handle_transfers_key(app: &mut App, key: crossterm::event::KeyEvent) {
    let action = map_key(key);
    
    match action {
        Action::Up => app.jobs_up(),
        Action::Down => app.jobs_down(),
        Action::ToggleTransfers => app.toggle_transfers_view(),
        Action::PauseJob => app.pause_selected_job(),
        Action::ResumeJob => app.resume_selected_job(),
        Action::CancelJob => app.cancel_selected_job(),
        Action::Quit => app.should_quit = true,
        _ => {}
    }
}

fn handle_sidebar_key(app: &mut App, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
    use crossterm::event::KeyCode;
    
    let action = map_key(key);
    
    match action {
        Action::Up => app.sidebar_up(),
        Action::Down => app.sidebar_down(),
        Action::Enter => app.navigate_to_sidebar_selection(),
        Action::ToggleSidebar => app.toggle_sidebar(),
        Action::Delete => app.remove_selected_favorite(),
        Action::Quit => app.should_quit = true,
        // QuickJump still works when sidebar is visible
        Action::QuickJump(n) => app.quick_jump_to_favorite(n),
        // Let other actions through to normal handling (like Properties, Help)
        Action::Properties => app.show_properties(),
        Action::Help => app.show_help = true,
        // Fallback for tab key to toggle section (not in Action enum)
        _ => {
            // Handle Tab key for section switching
            if key.code == KeyCode::Tab {
                app.sidebar_toggle_section();
            }
        }
    }
    Ok(())
}

fn handle_conflict_key(app: &mut App, key: crossterm::event::KeyEvent) {
    use zmanager_tui::ui::{ConflictResolution, ConflictResult};
    
    if let Some(ref mut modal) = app.conflict_modal {
        let result = modal.handle_key(key);
        match result {
            ConflictResult::Open => {
                // Modal is still open, nothing to do
            }
            ConflictResult::Resolved(resolution, apply_to_all) => {
                if resolution == ConflictResolution::Cancel {
                    app.close_conflict();
                } else {
                    debug!("Conflict resolved: {:?}, apply_to_all: {}", resolution, apply_to_all);
                    // TODO: Apply resolution to transfer engine
                    app.set_status(format!("Conflict resolution: {:?}", resolution), false);
                    app.close_conflict();
                }
            }
        }
    }
}

fn handle_dialog_key(app: &mut App, key: crossterm::event::KeyEvent) {
    let result = if let Some(ref mut dialog) = app.dialog {
        dialog.handle_key(key)
    } else {
        return;
    };

    match result {
        DialogResult::Open => {
            // Dialog still active, nothing to do
        }
        DialogResult::Cancelled => {
            app.close_dialog();
        }
        DialogResult::Confirmed(value) => {
            // Handle based on pending operation
            if let Some(op) = app.pending_operation.take() {
                match op {
                    PendingOperation::Delete(files) => {
                        app.execute_delete(files);
                    }
                    PendingOperation::Rename(old_path) => {
                        app.execute_rename(old_path, value);
                    }
                    PendingOperation::MakeDir => {
                        if !value.is_empty() {
                            app.execute_mkdir(value);
                        }
                    }
                    PendingOperation::Copy(sources, dest) => {
                        app.execute_copy(sources, dest);
                    }
                    PendingOperation::Move(sources, dest) => {
                        app.execute_move(sources, dest);
                    }
                }
            }
            app.close_dialog();
        }
        DialogResult::SortSelected(field) => {
            app.apply_sort(field);
            app.close_dialog();
        }
    }
}

// ========== File Operation Execution ==========

fn execute_delete(app: &mut App, files: Vec<PathBuf>) {
    for file in &files {
        debug!("Deleting: {:?}", file);
        if let Err(e) = std::fs::remove_file(file) {
            // Try as directory
            if let Err(e2) = std::fs::remove_dir_all(file) {
                error!("Failed to delete {:?}: {} / {}", file, e, e2);
                app.show_error("Delete Failed", format!("Could not delete: {}", e2));
                return;
            }
        }
    }
    
    // Refresh the active pane
    let path = app.active().nav.current_path().to_path_buf();
    let _ = load_directory(app, app.active_pane, &path);
    
    app.show_message("Deleted", format!("{} item(s) deleted", files.len()));
}

fn execute_rename(app: &mut App, old_path: PathBuf, new_path: PathBuf) {
    debug!("Renaming {:?} to {:?}", old_path, new_path);
    
    if let Err(e) = std::fs::rename(&old_path, &new_path) {
        error!("Failed to rename: {}", e);
        app.show_error("Rename Failed", format!("{}", e));
        return;
    }
    
    // Refresh the active pane
    let path = app.active().nav.current_path().to_path_buf();
    let _ = load_directory(app, app.active_pane, &path);
}

fn execute_mkdir(app: &mut App, path: PathBuf) {
    debug!("Creating directory: {:?}", path);
    
    if let Err(e) = std::fs::create_dir(&path) {
        error!("Failed to create directory: {}", e);
        app.show_error("Create Folder Failed", format!("{}", e));
        return;
    }
    
    // Refresh the active pane
    let parent = app.active().nav.current_path().to_path_buf();
    let _ = load_directory(app, app.active_pane, &parent);
}

fn execute_copy(app: &mut App, sources: Vec<PathBuf>, destination: PathBuf) {
    debug!("Copying {} files to {:?}", sources.len(), destination);
    
    let mut success_count = 0;
    for source in &sources {
        let file_name = source.file_name().unwrap_or_default();
        let dest_path = destination.join(file_name);
        
        if source.is_dir() {
            // Use recursive copy for directories
            if let Err(e) = copy_dir_recursive(source, &dest_path) {
                error!("Failed to copy directory {:?}: {}", source, e);
                app.show_error("Copy Failed", format!("Could not copy {}: {}", file_name.to_string_lossy(), e));
                continue;
            }
        } else if let Err(e) = std::fs::copy(source, &dest_path) {
            error!("Failed to copy {:?}: {}", source, e);
            app.show_error("Copy Failed", format!("Could not copy {}: {}", file_name.to_string_lossy(), e));
            continue;
        }
        success_count += 1;
    }
    
    // Refresh both panes
    let left = app.left.nav.current_path().to_path_buf();
    let right = app.right.nav.current_path().to_path_buf();
    let _ = load_directory(app, Pane::Left, &left);
    let _ = load_directory(app, Pane::Right, &right);
    
    if success_count > 0 {
        app.show_message("Copied", format!("{} item(s) copied", success_count));
    }
}

fn execute_move(app: &mut App, sources: Vec<PathBuf>, destination: PathBuf) {
    debug!("Moving {} files to {:?}", sources.len(), destination);
    
    let mut success_count = 0;
    for source in &sources {
        let file_name = source.file_name().unwrap_or_default();
        let dest_path = destination.join(file_name);
        
        // Try rename first (works if same filesystem)
        if std::fs::rename(source, &dest_path).is_err() {
            // Fall back to copy + delete
            if source.is_dir() {
                if let Err(e) = copy_dir_recursive(source, &dest_path) {
                    error!("Failed to move directory {:?}: {}", source, e);
                    app.show_error("Move Failed", format!("Could not move {}: {}", file_name.to_string_lossy(), e));
                    continue;
                }
                if let Err(e) = std::fs::remove_dir_all(source) {
                    error!("Failed to remove source directory: {}", e);
                }
            } else {
                if let Err(e) = std::fs::copy(source, &dest_path) {
                    error!("Failed to move {:?}: {}", source, e);
                    app.show_error("Move Failed", format!("Could not move {}: {}", file_name.to_string_lossy(), e));
                    continue;
                }
                if let Err(e) = std::fs::remove_file(source) {
                    error!("Failed to remove source file: {}", e);
                }
            }
        }
        success_count += 1;
    }
    
    // Refresh both panes
    let left = app.left.nav.current_path().to_path_buf();
    let right = app.right.nav.current_path().to_path_buf();
    let _ = load_directory(app, Pane::Left, &left);
    let _ = load_directory(app, Pane::Right, &right);
    
    if success_count > 0 {
        app.show_message("Moved", format!("{} item(s) moved", success_count));
    }
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
