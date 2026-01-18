//! Event handling for the TUI.
//!
//! This module provides an async event stream that combines
//! terminal events with application events.

use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use tokio::sync::mpsc;
use tracing::debug;

/// Application events.
#[derive(Debug, Clone)]
pub enum Event {
    /// Terminal key event.
    Key(KeyEvent),
    /// Terminal mouse event.
    Mouse(MouseEvent),
    /// Terminal resize event.
    Resize(u16, u16),
    /// Tick event for periodic updates.
    Tick,
    /// Directory contents changed.
    DirectoryChanged(PathBuf),
    /// Job progress update.
    JobProgress {
        job_id: u64,
        percentage: f64,
        bytes_done: u64,
        bytes_total: u64,
    },
    /// Job completed.
    JobCompleted { job_id: u64, success: bool },
    /// Error message to display.
    Error(String),
    /// Request to quit the application.
    Quit,

    // ========== File Operation Events ==========

    /// Execute delete operation on the specified paths.
    ExecuteDelete(Vec<PathBuf>),
    /// Execute rename operation (old path, new path).
    ExecuteRename(PathBuf, PathBuf),
    /// Execute mkdir operation at the specified path.
    ExecuteMkdir(PathBuf),
    /// Execute copy operation (sources, destination).
    ExecuteCopy(Vec<PathBuf>, PathBuf),
    /// Execute move operation (sources, destination).
    ExecuteMove(Vec<PathBuf>, PathBuf),
    /// Refresh all panes.
    RefreshAll,

    // ========== Job Control Events ==========

    /// Pause a job by ID.
    PauseJob(u64),
    /// Resume a job by ID.
    ResumeJob(u64),
    /// Cancel a job by ID.
    CancelJob(u64),
    /// Jobs list updated.
    JobsUpdated(Vec<zmanager_core::JobInfo>),
}

/// Event handler that polls for terminal events.
pub struct EventHandler {
    /// Event sender.
    tx: mpsc::UnboundedSender<Event>,
    /// Event receiver.
    rx: mpsc::UnboundedReceiver<Event>,
    /// Tick rate for periodic updates.
    tick_rate: Duration,
}

impl EventHandler {
    /// Create a new event handler.
    pub fn new(tick_rate_ms: u64) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tx,
            rx,
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }

    /// Get a sender for external events.
    pub fn sender(&self) -> mpsc::UnboundedSender<Event> {
        self.tx.clone()
    }

    /// Start the event loop in a background task.
    pub fn start(&self) {
        let tx = self.tx.clone();
        let tick_rate = self.tick_rate;

        tokio::spawn(async move {
            loop {
                // Poll for events with timeout
                if event::poll(tick_rate).unwrap_or(false) {
                    match event::read() {
                        Ok(CrosstermEvent::Key(key)) => {
                            debug!(?key, "Key event");
                            if tx.send(Event::Key(key)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Mouse(mouse)) => {
                            if tx.send(Event::Mouse(mouse)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Resize(w, h)) => {
                            debug!(w, h, "Resize event");
                            if tx.send(Event::Resize(w, h)).is_err() {
                                break;
                            }
                        }
                        Ok(_) => {}
                        Err(_) => break,
                    }
                } else {
                    // Tick event
                    if tx.send(Event::Tick).is_err() {
                        break;
                    }
                }
            }
        });
    }

    /// Receive the next event.
    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
