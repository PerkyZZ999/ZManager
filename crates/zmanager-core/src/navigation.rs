//! Navigation state and history management.

use crate::{DirListing, FilterSpec, SortSpec, ZResult};
use std::path::{Path, PathBuf};
use tracing::{debug, instrument};

/// Maximum history size to prevent unbounded memory growth.
const MAX_HISTORY_SIZE: usize = 100;

/// Navigation state for a single pane.
#[derive(Debug, Clone)]
pub struct NavigationState {
    /// Current directory path.
    current_path: PathBuf,

    /// Back history stack (most recent at the end).
    back_stack: Vec<PathBuf>,

    /// Forward history stack (most recent at the end).
    forward_stack: Vec<PathBuf>,

    /// Current sort specification.
    pub sort: SortSpec,

    /// Current filter specification.
    pub filter: FilterSpec,

    /// Cached directory listing (invalidated on navigation or refresh).
    cached_listing: Option<DirListing>,
}

impl NavigationState {
    /// Create a new navigation state starting at the given path.
    pub fn new(start_path: impl Into<PathBuf>) -> Self {
        Self {
            current_path: start_path.into(),
            back_stack: Vec::new(),
            forward_stack: Vec::new(),
            sort: SortSpec::default(),
            filter: FilterSpec::default(),
            cached_listing: None,
        }
    }

    /// Create navigation state starting at the user's home directory.
    pub fn at_home() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self::new(home)
    }

    /// Create navigation state starting at a default location.
    #[cfg(windows)]
    pub fn default_start() -> Self {
        // Start at C:\ on Windows
        Self::new(r"C:\")
    }

    #[cfg(not(windows))]
    pub fn default_start() -> Self {
        Self::at_home()
    }

    /// Get the current directory path.
    pub fn current_path(&self) -> &Path {
        &self.current_path
    }

    /// Check if we can go back.
    pub fn can_go_back(&self) -> bool {
        !self.back_stack.is_empty()
    }

    /// Check if we can go forward.
    pub fn can_go_forward(&self) -> bool {
        !self.forward_stack.is_empty()
    }

    /// Check if we can go to the parent directory.
    pub fn can_go_up(&self) -> bool {
        self.current_path.parent().is_some()
    }

    /// Navigate to a new directory, pushing the current path to history.
    #[instrument(skip(self, path))]
    pub fn navigate_to(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();

        // Don't navigate to the same path
        if path == self.current_path {
            return;
        }

        debug!(from = %self.current_path.display(), to = %path.display(), "Navigating to new directory");

        // Push current path to back stack
        self.back_stack.push(self.current_path.clone());

        // Trim back stack if too large
        if self.back_stack.len() > MAX_HISTORY_SIZE {
            self.back_stack.remove(0);
        }

        // Clear forward stack (new navigation branch)
        self.forward_stack.clear();

        // Update current path
        self.current_path = path;

        // Invalidate cache
        self.cached_listing = None;
    }

    /// Go back to the previous directory.
    /// Returns the new current path, or None if back stack is empty.
    #[instrument(skip(self))]
    pub fn go_back(&mut self) -> Option<&Path> {
        if let Some(prev_path) = self.back_stack.pop() {
            debug!(to = %prev_path.display(), "Going back");

            // Push current to forward stack
            self.forward_stack.push(self.current_path.clone());

            // Trim forward stack if too large
            if self.forward_stack.len() > MAX_HISTORY_SIZE {
                self.forward_stack.remove(0);
            }

            self.current_path = prev_path;
            self.cached_listing = None;

            Some(&self.current_path)
        } else {
            None
        }
    }

    /// Go forward to the next directory (after going back).
    /// Returns the new current path, or None if forward stack is empty.
    #[instrument(skip(self))]
    pub fn go_forward(&mut self) -> Option<&Path> {
        if let Some(next_path) = self.forward_stack.pop() {
            debug!(to = %next_path.display(), "Going forward");

            // Push current to back stack
            self.back_stack.push(self.current_path.clone());

            // Trim back stack if too large
            if self.back_stack.len() > MAX_HISTORY_SIZE {
                self.back_stack.remove(0);
            }

            self.current_path = next_path;
            self.cached_listing = None;

            Some(&self.current_path)
        } else {
            None
        }
    }

    /// Go up to the parent directory.
    /// Returns the new current path, or None if already at root.
    #[instrument(skip(self))]
    pub fn go_up(&mut self) -> Option<&Path> {
        if let Some(parent) = self.current_path.parent() {
            let parent = parent.to_path_buf();
            debug!(to = %parent.display(), "Going up to parent");
            self.navigate_to(parent);
            Some(&self.current_path)
        } else {
            None
        }
    }

    /// Refresh the current directory listing.
    pub fn refresh(&mut self) -> ZResult<&DirListing> {
        self.cached_listing = None;
        self.get_listing()
    }

    /// Get the current directory listing, using cache if available.
    pub fn get_listing(&mut self) -> ZResult<&DirListing> {
        if self.cached_listing.is_none() {
            let listing = crate::fs::list_directory(
                &self.current_path,
                Some(&self.sort),
                Some(&self.filter),
            )?;
            self.cached_listing = Some(listing);
        }

        Ok(self.cached_listing.as_ref().unwrap())
    }

    /// Invalidate the cached listing (e.g., after file changes).
    pub fn invalidate_cache(&mut self) {
        self.cached_listing = None;
    }

    /// Update the sort specification and invalidate cache.
    pub fn set_sort(&mut self, sort: SortSpec) {
        self.sort = sort;
        self.cached_listing = None;
    }

    /// Update the filter specification and invalidate cache.
    pub fn set_filter(&mut self, filter: FilterSpec) {
        self.filter = filter;
        self.cached_listing = None;
    }

    /// Toggle sort field (cycle through or toggle order).
    pub fn toggle_sort(&mut self, field: crate::SortField) {
        self.sort.toggle_or_set(field);
        self.cached_listing = None;
    }

    /// Toggle hidden files visibility.
    pub fn toggle_hidden(&mut self) {
        self.filter.toggle_hidden();
        self.cached_listing = None;
    }

    /// Set a search/filter pattern.
    pub fn set_pattern(&mut self, pattern: Option<String>) {
        self.filter.pattern = pattern;
        self.cached_listing = None;
    }

    /// Get the back history (for display).
    pub fn back_history(&self) -> &[PathBuf] {
        &self.back_stack
    }

    /// Get the forward history (for display).
    pub fn forward_history(&self) -> &[PathBuf] {
        &self.forward_stack
    }

    /// Clear all history.
    pub fn clear_history(&mut self) {
        self.back_stack.clear();
        self.forward_stack.clear();
    }
}

impl Default for NavigationState {
    fn default() -> Self {
        Self::default_start()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_nav_dirs() -> (TempDir, PathBuf, PathBuf, PathBuf) {
        let root = TempDir::new().unwrap();
        let dir_a = root.path().join("dir_a");
        let dir_b = root.path().join("dir_b");
        let dir_c = root.path().join("dir_c");

        fs::create_dir_all(&dir_a).unwrap();
        fs::create_dir_all(&dir_b).unwrap();
        fs::create_dir_all(&dir_c).unwrap();

        (root, dir_a, dir_b, dir_c)
    }

    #[test]
    fn test_navigation_basic() {
        let (root, dir_a, dir_b, _) = setup_nav_dirs();
        let mut nav = NavigationState::new(root.path());

        assert_eq!(nav.current_path(), root.path());
        assert!(!nav.can_go_back());
        assert!(!nav.can_go_forward());

        // Navigate to dir_a
        nav.navigate_to(&dir_a);
        assert_eq!(nav.current_path(), dir_a);
        assert!(nav.can_go_back());
        assert!(!nav.can_go_forward());

        // Navigate to dir_b
        nav.navigate_to(&dir_b);
        assert_eq!(nav.current_path(), dir_b);
        assert!(nav.can_go_back());
    }

    #[test]
    fn test_go_back_forward() {
        let (root, dir_a, dir_b, _) = setup_nav_dirs();
        let mut nav = NavigationState::new(root.path());

        nav.navigate_to(&dir_a);
        nav.navigate_to(&dir_b);

        // Go back to dir_a
        let path = nav.go_back().map(|p| p.to_path_buf());
        assert_eq!(path.as_ref(), Some(&dir_a));
        assert!(nav.can_go_forward());

        // Go back to root
        let path = nav.go_back().map(|p| p.to_path_buf());
        assert_eq!(path.as_ref(), Some(&root.path().to_path_buf()));
        assert!(nav.can_go_forward());
        assert!(!nav.can_go_back());

        // Go forward to dir_a
        let path = nav.go_forward().map(|p| p.to_path_buf());
        assert_eq!(path.as_ref(), Some(&dir_a));

        // Go forward to dir_b
        let path = nav.go_forward().map(|p| p.to_path_buf());
        assert_eq!(path.as_ref(), Some(&dir_b));
        assert!(!nav.can_go_forward());
    }

    #[test]
    fn test_navigate_clears_forward() {
        let (root, dir_a, dir_b, dir_c) = setup_nav_dirs();
        let mut nav = NavigationState::new(root.path());

        nav.navigate_to(&dir_a);
        nav.navigate_to(&dir_b);
        nav.go_back(); // Now at dir_a, can go forward to dir_b

        assert!(nav.can_go_forward());

        // Navigate to dir_c - should clear forward history
        nav.navigate_to(&dir_c);
        assert!(!nav.can_go_forward());
    }

    #[test]
    fn test_go_up() {
        let (root, dir_a, _, _) = setup_nav_dirs();
        let mut nav = NavigationState::new(&dir_a);

        assert!(nav.can_go_up());

        let parent = nav.go_up().map(|p| p.to_path_buf());
        assert_eq!(parent.as_ref(), Some(&root.path().to_path_buf()));
        assert!(nav.can_go_back()); // Should have history
    }

    #[test]
    fn test_navigate_to_same_path() {
        let (_root, dir_a, _, _) = setup_nav_dirs();
        let mut nav = NavigationState::new(&dir_a);

        // Navigate to same path should be no-op
        nav.navigate_to(&dir_a);
        assert!(!nav.can_go_back()); // No history added
    }

    #[test]
    fn test_history_size_limit() {
        let root = TempDir::new().unwrap();
        let mut nav = NavigationState::new(root.path());

        // Create many directories and navigate through them
        for i in 0..150 {
            let dir = root.path().join(format!("dir_{}", i));
            fs::create_dir_all(&dir).unwrap();
            nav.navigate_to(&dir);
        }

        // History should be limited
        assert!(nav.back_stack.len() <= MAX_HISTORY_SIZE);
    }

    #[test]
    fn test_toggle_hidden() {
        let (root, _, _, _) = setup_nav_dirs();
        let mut nav = NavigationState::new(root.path());

        assert!(!nav.filter.show_hidden);
        nav.toggle_hidden();
        assert!(nav.filter.show_hidden);
        nav.toggle_hidden();
        assert!(!nav.filter.show_hidden);
    }
}
