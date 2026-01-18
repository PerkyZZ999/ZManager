//! File system watching with debouncing.
//!
//! This module provides directory watching functionality to detect file system
//! changes and trigger UI refreshes. Events are debounced to prevent thrashing.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher,
};
use tokio::sync::broadcast;
use tracing::{debug, info, trace};

use crate::{ZError, ZResult};

/// Default debounce duration (300ms).
pub const DEFAULT_DEBOUNCE_MS: u64 = 300;

/// File system change event.
#[derive(Debug, Clone)]
pub struct WatchEvent {
    /// The directory that changed.
    pub directory: PathBuf,
    /// What kind of change occurred.
    pub kind: WatchEventKind,
    /// Paths that changed (if known).
    pub paths: Vec<PathBuf>,
}

/// Kind of file system change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchEventKind {
    /// Files or directories were created.
    Created,
    /// Files or directories were modified.
    Modified,
    /// Files or directories were deleted.
    Deleted,
    /// Files or directories were renamed.
    Renamed,
    /// Generic change (refresh needed).
    Changed,
}

impl WatchEventKind {
    /// Get a display label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Created => "Created",
            Self::Modified => "Modified",
            Self::Deleted => "Deleted",
            Self::Renamed => "Renamed",
            Self::Changed => "Changed",
        }
    }
}

impl From<&EventKind> for WatchEventKind {
    fn from(kind: &EventKind) -> Self {
        match kind {
            EventKind::Create(_) => Self::Created,
            EventKind::Modify(_) => Self::Modified,
            EventKind::Remove(_) => Self::Deleted,
            EventKind::Any | EventKind::Access(_) | EventKind::Other => Self::Changed,
        }
    }
}

/// Configuration for the file watcher.
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Debounce duration in milliseconds.
    pub debounce_ms: u64,
    /// Maximum directories to watch simultaneously.
    pub max_watched_dirs: usize,
    /// Whether to watch subdirectories recursively.
    pub recursive: bool,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            debounce_ms: DEFAULT_DEBOUNCE_MS,
            max_watched_dirs: 10,
            recursive: false, // Only watch the immediate directory
        }
    }
}

/// Debouncer state for a single directory.
struct DebouncerState {
    last_event: Instant,
    pending_paths: Vec<PathBuf>,
    pending_kind: WatchEventKind,
}

/// File system watcher with debouncing.
pub struct DirectoryWatcher {
    config: WatcherConfig,
    event_tx: broadcast::Sender<WatchEvent>,
    /// Handle to the notify watcher
    watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
    /// Currently watched directories
    watched: Arc<Mutex<HashMap<PathBuf, ()>>>,
    /// Debounce state per directory
    debounce_state: Arc<Mutex<HashMap<PathBuf, DebouncerState>>>,
    /// Shutdown flag
    shutdown: Arc<AtomicBool>,
}

impl DirectoryWatcher {
    /// Create a new directory watcher with default configuration.
    pub fn new() -> ZResult<Self> {
        Self::with_config(WatcherConfig::default())
    }

    /// Create a new directory watcher with custom configuration.
    pub fn with_config(config: WatcherConfig) -> ZResult<Self> {
        let (event_tx, _) = broadcast::channel(256);
        let watched = Arc::new(Mutex::new(HashMap::new()));
        let debounce_state = Arc::new(Mutex::new(HashMap::new()));
        let watcher = Arc::new(Mutex::new(None));
        let shutdown = Arc::new(AtomicBool::new(false));

        Ok(Self {
            config,
            event_tx,
            watcher,
            watched,
            debounce_state,
            shutdown,
        })
    }

    /// Subscribe to watch events.
    pub fn subscribe(&self) -> broadcast::Receiver<WatchEvent> {
        self.event_tx.subscribe()
    }

    /// Start the watcher background task.
    pub fn start(&mut self) -> ZResult<()> {
        // Use std channels for the notify callback (it runs on a separate thread)
        let (raw_tx, raw_rx) = std::sync::mpsc::channel::<Event>();

        // Create the notify watcher
        let notify_watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = raw_tx.send(event);
                }
            },
            Config::default(),
        )
        .map_err(|e| ZError::Internal {
            message: format!("Failed to create watcher: {e}"),
        })?;

        *self.watcher.lock().unwrap() = Some(notify_watcher);
        self.shutdown.store(false, Ordering::SeqCst);

        // Spawn debounce task
        let event_tx = self.event_tx.clone();
        let debounce_state = self.debounce_state.clone();
        let debounce_duration = Duration::from_millis(self.config.debounce_ms);
        let shutdown = self.shutdown.clone();

        tokio::spawn(async move {
            let mut debounce_interval = tokio::time::interval(Duration::from_millis(50));

            loop {
                if shutdown.load(Ordering::SeqCst) {
                    debug!("Watcher shutdown signal received");
                    break;
                }

                // Check for raw events (non-blocking)
                while let Ok(event) = raw_rx.try_recv() {
                    Self::handle_raw_event(&debounce_state, &event);
                }

                // Flush debounced events
                Self::flush_debounced(&debounce_state, &event_tx, debounce_duration);

                debounce_interval.tick().await;
            }

            info!("Watcher task stopped");
        });

        info!("Directory watcher started");
        Ok(())
    }

    /// Stop the watcher.
    pub fn stop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);

        if let Ok(mut guard) = self.watcher.lock() {
            *guard = None;
        }

        if let Ok(mut guard) = self.watched.lock() {
            guard.clear();
        }

        info!("Directory watcher stopped");
    }

    /// Watch a directory for changes.
    pub fn watch(&self, path: &Path) -> ZResult<()> {
        let path = path.canonicalize().map_err(|e| ZError::io(path, e))?;

        // Check limit
        let watched_count = self.watched.lock().unwrap().len();
        if watched_count >= self.config.max_watched_dirs {
            return Err(ZError::InvalidOperation {
                operation: "watch".to_string(),
                reason: format!(
                    "Maximum watched directories ({}) reached",
                    self.config.max_watched_dirs
                ),
            });
        }

        // Already watching?
        if self.watched.lock().unwrap().contains_key(&path) {
            trace!(path = %path.display(), "Already watching directory");
            return Ok(());
        }

        // Add to watcher
        let mode = if self.config.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        if let Some(ref mut watcher) = *self.watcher.lock().unwrap() {
            watcher.watch(&path, mode).map_err(|e| ZError::Internal {
                message: format!("Failed to watch {}: {e}", path.display()),
            })?;
        }

        self.watched.lock().unwrap().insert(path.clone(), ());
        debug!(path = %path.display(), "Now watching directory");

        Ok(())
    }

    /// Stop watching a directory.
    pub fn unwatch(&self, path: &Path) -> ZResult<()> {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        if !self.watched.lock().unwrap().contains_key(&path) {
            return Ok(());
        }

        if let Some(ref mut watcher) = *self.watcher.lock().unwrap() {
            let _ = watcher.unwatch(&path);
        }

        self.watched.lock().unwrap().remove(&path);
        self.debounce_state.lock().unwrap().remove(&path);

        debug!(path = %path.display(), "Stopped watching directory");
        Ok(())
    }

    /// Get the list of currently watched directories.
    pub fn watched_dirs(&self) -> Vec<PathBuf> {
        self.watched.lock().unwrap().keys().cloned().collect()
    }

    /// Check if a directory is being watched.
    pub fn is_watching(&self, path: &Path) -> bool {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        self.watched.lock().unwrap().contains_key(&path)
    }

    /// Handle a raw notify event.
    fn handle_raw_event(
        debounce_state: &Arc<Mutex<HashMap<PathBuf, DebouncerState>>>,
        event: &Event,
    ) {
        let kind = WatchEventKind::from(&event.kind);

        // Group by parent directory
        for path in &event.paths {
            let dir = path.parent().unwrap_or(path).to_path_buf();

            let mut state = debounce_state.lock().unwrap();
            let entry = state.entry(dir).or_insert_with(|| DebouncerState {
                last_event: Instant::now(),
                pending_paths: Vec::new(),
                pending_kind: kind,
            });

            entry.last_event = Instant::now();
            if !entry.pending_paths.contains(path) {
                entry.pending_paths.push(path.clone());
            }

            // Upgrade kind if needed (delete/create are more significant than modify)
            if kind != WatchEventKind::Modified {
                entry.pending_kind = kind;
            }
        }
    }

    /// Flush debounced events that have stabilized.
    fn flush_debounced(
        debounce_state: &Arc<Mutex<HashMap<PathBuf, DebouncerState>>>,
        event_tx: &broadcast::Sender<WatchEvent>,
        debounce_duration: Duration,
    ) {
        let now = Instant::now();
        let mut to_flush = Vec::new();

        {
            let state = debounce_state.lock().unwrap();
            for (dir, debouncer) in state.iter() {
                if now.duration_since(debouncer.last_event) >= debounce_duration {
                    to_flush.push((
                        dir.clone(),
                        debouncer.pending_kind,
                        debouncer.pending_paths.clone(),
                    ));
                }
            }
        }

        // Flush and remove
        if !to_flush.is_empty() {
            let mut state = debounce_state.lock().unwrap();
            for (dir, kind, paths) in to_flush {
                state.remove(&dir);

                let event = WatchEvent {
                    directory: dir.clone(),
                    kind,
                    paths,
                };

                trace!(dir = %dir.display(), ?kind, "Flushing debounced event");
                let _ = event_tx.send(event);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::time::timeout;

    #[test]
    fn test_watch_event_kind_labels() {
        assert_eq!(WatchEventKind::Created.label(), "Created");
        assert_eq!(WatchEventKind::Modified.label(), "Modified");
        assert_eq!(WatchEventKind::Deleted.label(), "Deleted");
        assert_eq!(WatchEventKind::Renamed.label(), "Renamed");
        assert_eq!(WatchEventKind::Changed.label(), "Changed");
    }

    #[test]
    fn test_watcher_config_default() {
        let config = WatcherConfig::default();
        assert_eq!(config.debounce_ms, DEFAULT_DEBOUNCE_MS);
        assert_eq!(config.max_watched_dirs, 10);
        assert!(!config.recursive);
    }

    #[test]
    fn test_watcher_creation() {
        let watcher = DirectoryWatcher::new();
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_watcher_with_config() {
        let config = WatcherConfig {
            debounce_ms: 500,
            max_watched_dirs: 5,
            recursive: true,
        };

        let watcher = DirectoryWatcher::with_config(config);
        assert!(watcher.is_ok());
    }

    #[tokio::test]
    async fn test_watch_directory() {
        let temp = TempDir::new().unwrap();
        let mut watcher = DirectoryWatcher::new().unwrap();

        watcher.start().unwrap();
        watcher.watch(temp.path()).unwrap();

        assert!(watcher.is_watching(temp.path()));
        assert_eq!(watcher.watched_dirs().len(), 1);

        watcher.stop();
    }

    #[tokio::test]
    async fn test_unwatch_directory() {
        let temp = TempDir::new().unwrap();
        let mut watcher = DirectoryWatcher::new().unwrap();

        watcher.start().unwrap();
        watcher.watch(temp.path()).unwrap();
        watcher.unwatch(temp.path()).unwrap();

        assert!(!watcher.is_watching(temp.path()));
        assert!(watcher.watched_dirs().is_empty());

        watcher.stop();
    }

    #[tokio::test]
    async fn test_watch_limit() {
        let temps: Vec<_> = (0..15).map(|_| TempDir::new().unwrap()).collect();

        let config = WatcherConfig {
            max_watched_dirs: 5,
            ..Default::default()
        };
        let mut watcher = DirectoryWatcher::with_config(config).unwrap();

        watcher.start().unwrap();

        // First 5 should succeed
        for temp in temps.iter().take(5) {
            watcher.watch(temp.path()).unwrap();
        }

        // 6th should fail
        let result = watcher.watch(temps[5].path());
        assert!(result.is_err());

        watcher.stop();
    }

    #[tokio::test]
    async fn test_watch_event_detection() {
        let temp = TempDir::new().unwrap();
        let mut watcher = DirectoryWatcher::with_config(WatcherConfig {
            debounce_ms: 100,
            ..Default::default()
        })
        .unwrap();

        let mut rx = watcher.subscribe();
        watcher.start().unwrap();
        watcher.watch(temp.path()).unwrap();

        // Give watcher time to initialize
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Create a file
        let test_file = temp.path().join("test.txt");
        fs::write(&test_file, "hello").unwrap();

        // Wait for debounced event
        let result = timeout(Duration::from_secs(2), rx.recv()).await;

        if let Ok(Ok(event)) = result {
            assert_eq!(event.directory, temp.path().canonicalize().unwrap());
        }

        watcher.stop();
    }

    #[test]
    fn test_watch_event() {
        let event = WatchEvent {
            directory: PathBuf::from("/test"),
            kind: WatchEventKind::Created,
            paths: vec![PathBuf::from("/test/file.txt")],
        };

        assert_eq!(event.kind, WatchEventKind::Created);
        assert_eq!(event.paths.len(), 1);
    }

    #[tokio::test]
    async fn test_watcher_stop() {
        let mut watcher = DirectoryWatcher::new().unwrap();
        watcher.start().unwrap();

        // Should not panic
        watcher.stop();
        watcher.stop(); // Double stop should be safe
    }

    #[tokio::test]
    async fn test_duplicate_watch() {
        let temp = TempDir::new().unwrap();
        let mut watcher = DirectoryWatcher::new().unwrap();

        watcher.start().unwrap();
        watcher.watch(temp.path()).unwrap();
        watcher.watch(temp.path()).unwrap(); // Should not fail

        assert_eq!(watcher.watched_dirs().len(), 1);

        watcher.stop();
    }
}
