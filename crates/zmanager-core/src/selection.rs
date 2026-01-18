//! Selection model for file entries.

use crate::EntryMeta;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Selection state for a directory listing.
#[derive(Debug, Clone, Default)]
pub struct Selection {
    /// Set of selected entry paths.
    selected: HashSet<PathBuf>,

    /// The cursor position (focused item index).
    cursor: usize,

    /// Anchor for range selection (Shift+click/arrow).
    anchor: Option<usize>,

    /// Total number of entries (for bounds checking).
    entry_count: usize,
}

impl Selection {
    /// Create a new empty selection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a selection with a given entry count.
    pub fn with_count(entry_count: usize) -> Self {
        Self {
            entry_count,
            ..Self::default()
        }
    }

    /// Update the entry count (after listing refresh).
    pub fn set_entry_count(&mut self, count: usize) {
        self.entry_count = count;
        // Clamp cursor to valid range
        if self.cursor >= count && count > 0 {
            self.cursor = count - 1;
        }
        // Clear anchor if out of range
        if let Some(anchor) = self.anchor {
            if anchor >= count {
                self.anchor = None;
            }
        }
        // Remove selections for entries that no longer exist
        // Note: We can't do this perfectly without knowing which paths were removed
    }

    /// Get the current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Check if an index is the cursor position.
    pub fn is_cursor(&self, index: usize) -> bool {
        self.cursor == index
    }

    /// Check if a path is selected.
    pub fn is_selected(&self, path: &Path) -> bool {
        self.selected.contains(path)
    }

    /// Check if an entry is selected by index (requires entry list).
    pub fn is_index_selected(&self, index: usize, entries: &[EntryMeta]) -> bool {
        entries
            .get(index)
            .map(|e| self.is_selected(&e.path))
            .unwrap_or(false)
    }

    /// Get the number of selected items.
    pub fn count(&self) -> usize {
        self.selected.len()
    }

    /// Check if the selection is empty.
    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    /// Get all selected paths.
    pub fn selected_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.selected.iter()
    }

    /// Get selected entries from a list.
    pub fn selected_entries<'a>(&self, entries: &'a [EntryMeta]) -> Vec<&'a EntryMeta> {
        entries
            .iter()
            .filter(|e| self.is_selected(&e.path))
            .collect()
    }

    /// Move cursor up.
    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        self.anchor = None;
    }

    /// Move cursor down.
    pub fn move_down(&mut self) {
        if self.cursor + 1 < self.entry_count {
            self.cursor += 1;
        }
        self.anchor = None;
    }

    /// Move cursor up with selection extension (Shift+Up).
    pub fn move_up_extend(&mut self, entries: &[EntryMeta]) {
        if self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }

        if self.cursor > 0 {
            self.cursor -= 1;
            self.update_range_selection(entries);
        }
    }

    /// Move cursor down with selection extension (Shift+Down).
    pub fn move_down_extend(&mut self, entries: &[EntryMeta]) {
        if self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }

        if self.cursor + 1 < self.entry_count {
            self.cursor += 1;
            self.update_range_selection(entries);
        }
    }

    /// Move cursor to the first entry.
    pub fn move_to_first(&mut self) {
        self.cursor = 0;
        self.anchor = None;
    }

    /// Move cursor to the last entry.
    pub fn move_to_last(&mut self) {
        if self.entry_count > 0 {
            self.cursor = self.entry_count - 1;
        }
        self.anchor = None;
    }

    /// Move cursor up by a page (for Page Up).
    pub fn page_up(&mut self, page_size: usize) {
        self.cursor = self.cursor.saturating_sub(page_size);
        self.anchor = None;
    }

    /// Move cursor down by a page (for Page Down).
    pub fn page_down(&mut self, page_size: usize) {
        self.cursor = (self.cursor + page_size).min(self.entry_count.saturating_sub(1));
        self.anchor = None;
    }

    /// Set cursor to a specific position.
    pub fn set_cursor(&mut self, index: usize) {
        if index < self.entry_count {
            self.cursor = index;
            self.anchor = None;
        }
    }

    /// Toggle selection of the entry at cursor.
    pub fn toggle_at_cursor(&mut self, entries: &[EntryMeta]) {
        if let Some(entry) = entries.get(self.cursor) {
            self.toggle(&entry.path);
        }
    }

    /// Toggle selection of a path.
    pub fn toggle(&mut self, path: &Path) {
        if self.selected.contains(path) {
            self.selected.remove(path);
        } else {
            self.selected.insert(path.to_path_buf());
        }
    }

    /// Select a single path (clear others first).
    pub fn select_single(&mut self, path: &Path) {
        self.selected.clear();
        self.selected.insert(path.to_path_buf());
    }

    /// Select the entry at cursor (single selection).
    pub fn select_at_cursor(&mut self, entries: &[EntryMeta]) {
        if let Some(entry) = entries.get(self.cursor) {
            self.select_single(&entry.path);
        }
    }

    /// Add a path to the selection.
    pub fn add(&mut self, path: &Path) {
        self.selected.insert(path.to_path_buf());
    }

    /// Remove a path from the selection.
    pub fn remove(&mut self, path: &Path) {
        self.selected.remove(path);
    }

    /// Select all entries.
    pub fn select_all(&mut self, entries: &[EntryMeta]) {
        self.selected.clear();
        for entry in entries {
            self.selected.insert(entry.path.clone());
        }
    }

    /// Clear the selection.
    pub fn clear(&mut self) {
        self.selected.clear();
        self.anchor = None;
    }

    /// Invert the selection.
    pub fn invert(&mut self, entries: &[EntryMeta]) {
        let mut new_selection = HashSet::new();
        for entry in entries {
            if !self.selected.contains(&entry.path) {
                new_selection.insert(entry.path.clone());
            }
        }
        self.selected = new_selection;
    }

    /// Select range from anchor to cursor.
    pub fn select_range(&mut self, entries: &[EntryMeta], from: usize, to: usize) {
        let (start, end) = if from <= to { (from, to) } else { (to, from) };

        for entry in entries.iter().skip(start).take(end - start + 1) {
            self.selected.insert(entry.path.clone());
        }
    }

    /// Update range selection based on current anchor and cursor.
    fn update_range_selection(&mut self, entries: &[EntryMeta]) {
        if let Some(anchor) = self.anchor {
            // Clear and reselect the range
            self.selected.clear();
            self.select_range(entries, anchor, self.cursor);
        }
    }

    /// Click on an entry (for mouse interaction).
    pub fn click(&mut self, index: usize, entries: &[EntryMeta], modifiers: ClickModifiers) {
        if index >= entries.len() {
            return;
        }

        let path = &entries[index].path;

        match (modifiers.ctrl, modifiers.shift) {
            (true, false) => {
                // Ctrl+Click: Toggle selection
                self.toggle(path);
                self.cursor = index;
            }
            (false, true) => {
                // Shift+Click: Range selection from anchor/cursor
                let anchor = self.anchor.unwrap_or(self.cursor);
                self.selected.clear();
                self.select_range(entries, anchor, index);
                self.cursor = index;
            }
            (true, true) => {
                // Ctrl+Shift+Click: Add range to selection
                let anchor = self.anchor.unwrap_or(self.cursor);
                self.select_range(entries, anchor, index);
                self.cursor = index;
            }
            (false, false) => {
                // Plain click: Single selection
                self.selected.clear();
                self.selected.insert(path.clone());
                self.cursor = index;
                self.anchor = Some(index);
            }
        }
    }

    /// Get the entry at cursor, if any.
    pub fn cursor_entry<'a>(&self, entries: &'a [EntryMeta]) -> Option<&'a EntryMeta> {
        entries.get(self.cursor)
    }

    /// Get paths to operate on: selected items, or cursor item if nothing selected.
    pub fn operation_targets<'a>(&self, entries: &'a [EntryMeta]) -> Vec<&'a EntryMeta> {
        if self.selected.is_empty() {
            // Use cursor item
            entries.get(self.cursor).into_iter().collect()
        } else {
            // Use selected items
            self.selected_entries(entries)
        }
    }
}

/// Modifier keys for click operations.
#[derive(Debug, Clone, Copy, Default)]
pub struct ClickModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl ClickModifiers {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Self::default()
        }
    }

    pub fn shift() -> Self {
        Self {
            shift: true,
            ..Self::default()
        }
    }

    pub fn ctrl_shift() -> Self {
        Self {
            ctrl: true,
            shift: true,
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntryKind;

    fn make_entries(names: &[&str]) -> Vec<EntryMeta> {
        names
            .iter()
            .map(|name| {
                EntryMeta::new(
                    name.to_string(),
                    PathBuf::from(format!("/test/{}", name)),
                    EntryKind::File,
                )
            })
            .collect()
    }

    #[test]
    fn test_empty_selection() {
        let sel = Selection::new();
        assert!(sel.is_empty());
        assert_eq!(sel.count(), 0);
        assert_eq!(sel.cursor(), 0);
    }

    #[test]
    fn test_cursor_movement() {
        let mut sel = Selection::with_count(5);

        assert_eq!(sel.cursor(), 0);

        sel.move_down();
        assert_eq!(sel.cursor(), 1);

        sel.move_down();
        sel.move_down();
        assert_eq!(sel.cursor(), 3);

        sel.move_up();
        assert_eq!(sel.cursor(), 2);

        sel.move_to_first();
        assert_eq!(sel.cursor(), 0);

        sel.move_to_last();
        assert_eq!(sel.cursor(), 4);
    }

    #[test]
    fn test_cursor_bounds() {
        let mut sel = Selection::with_count(3);

        // Can't go below 0
        sel.move_up();
        assert_eq!(sel.cursor(), 0);

        // Can't go above count - 1
        sel.move_to_last();
        sel.move_down();
        assert_eq!(sel.cursor(), 2);
    }

    #[test]
    fn test_toggle_selection() {
        let entries = make_entries(&["a.txt", "b.txt", "c.txt"]);
        let mut sel = Selection::with_count(entries.len());

        // Toggle first entry
        sel.toggle_at_cursor(&entries);
        assert!(sel.is_selected(&entries[0].path));
        assert_eq!(sel.count(), 1);

        // Toggle again to deselect
        sel.toggle_at_cursor(&entries);
        assert!(!sel.is_selected(&entries[0].path));
        assert_eq!(sel.count(), 0);
    }

    #[test]
    fn test_select_all() {
        let entries = make_entries(&["a.txt", "b.txt", "c.txt"]);
        let mut sel = Selection::with_count(entries.len());

        sel.select_all(&entries);
        assert_eq!(sel.count(), 3);
        assert!(entries.iter().all(|e| sel.is_selected(&e.path)));
    }

    #[test]
    fn test_clear_selection() {
        let entries = make_entries(&["a.txt", "b.txt", "c.txt"]);
        let mut sel = Selection::with_count(entries.len());

        sel.select_all(&entries);
        assert_eq!(sel.count(), 3);

        sel.clear();
        assert!(sel.is_empty());
    }

    #[test]
    fn test_invert_selection() {
        let entries = make_entries(&["a.txt", "b.txt", "c.txt"]);
        let mut sel = Selection::with_count(entries.len());

        sel.add(&entries[0].path);
        assert_eq!(sel.count(), 1);

        sel.invert(&entries);
        assert_eq!(sel.count(), 2);
        assert!(!sel.is_selected(&entries[0].path));
        assert!(sel.is_selected(&entries[1].path));
        assert!(sel.is_selected(&entries[2].path));
    }

    #[test]
    fn test_range_selection() {
        let entries = make_entries(&["a.txt", "b.txt", "c.txt", "d.txt", "e.txt"]);
        let mut sel = Selection::with_count(entries.len());

        sel.select_range(&entries, 1, 3);
        assert_eq!(sel.count(), 3);
        assert!(!sel.is_selected(&entries[0].path));
        assert!(sel.is_selected(&entries[1].path));
        assert!(sel.is_selected(&entries[2].path));
        assert!(sel.is_selected(&entries[3].path));
        assert!(!sel.is_selected(&entries[4].path));
    }

    #[test]
    fn test_click_plain() {
        let entries = make_entries(&["a.txt", "b.txt", "c.txt"]);
        let mut sel = Selection::with_count(entries.len());

        sel.click(1, &entries, ClickModifiers::none());
        assert_eq!(sel.cursor(), 1);
        assert_eq!(sel.count(), 1);
        assert!(sel.is_selected(&entries[1].path));
    }

    #[test]
    fn test_click_ctrl() {
        let entries = make_entries(&["a.txt", "b.txt", "c.txt"]);
        let mut sel = Selection::with_count(entries.len());

        sel.click(0, &entries, ClickModifiers::none());
        sel.click(2, &entries, ClickModifiers::ctrl());

        assert_eq!(sel.count(), 2);
        assert!(sel.is_selected(&entries[0].path));
        assert!(sel.is_selected(&entries[2].path));
    }

    #[test]
    fn test_click_shift() {
        let entries = make_entries(&["a.txt", "b.txt", "c.txt", "d.txt"]);
        let mut sel = Selection::with_count(entries.len());

        sel.click(1, &entries, ClickModifiers::none());
        sel.click(3, &entries, ClickModifiers::shift());

        assert_eq!(sel.count(), 3); // b, c, d
        assert!(!sel.is_selected(&entries[0].path));
        assert!(sel.is_selected(&entries[1].path));
        assert!(sel.is_selected(&entries[2].path));
        assert!(sel.is_selected(&entries[3].path));
    }

    #[test]
    fn test_operation_targets() {
        let entries = make_entries(&["a.txt", "b.txt", "c.txt"]);
        let mut sel = Selection::with_count(entries.len());

        // No selection: use cursor
        sel.set_cursor(1);
        let targets = sel.operation_targets(&entries);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].name, "b.txt");

        // With selection: use selected
        sel.add(&entries[0].path);
        sel.add(&entries[2].path);
        let targets = sel.operation_targets(&entries);
        assert_eq!(targets.len(), 2);
    }

    #[test]
    fn test_page_navigation() {
        let mut sel = Selection::with_count(100);

        sel.page_down(20);
        assert_eq!(sel.cursor(), 20);

        sel.page_down(20);
        assert_eq!(sel.cursor(), 40);

        sel.page_up(30);
        assert_eq!(sel.cursor(), 10);

        sel.page_up(30);
        assert_eq!(sel.cursor(), 0);
    }

    #[test]
    fn test_set_entry_count() {
        let mut sel = Selection::with_count(10);
        sel.set_cursor(8);
        assert_eq!(sel.cursor(), 8);

        // Shrink entry count
        sel.set_entry_count(5);
        assert_eq!(sel.cursor(), 4); // Clamped
    }
}
