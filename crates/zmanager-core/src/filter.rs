//! Filtering specifications for directory listings.

use crate::EntryMeta;
use serde::{Deserialize, Serialize};

/// A specification for filtering directory entries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterSpec {
    /// Text pattern to match against entry names (case-insensitive).
    /// If `None`, all entries match.
    pub pattern: Option<String>,

    /// Whether to show hidden files.
    pub show_hidden: bool,

    /// Whether to show system files.
    pub show_system: bool,

    /// Filter by file extensions (lowercase, without dots).
    /// If empty, all extensions match.
    pub extensions: Vec<String>,

    /// Minimum file size in bytes. `None` means no minimum.
    pub min_size: Option<u64>,

    /// Maximum file size in bytes. `None` means no maximum.
    pub max_size: Option<u64>,
}

impl FilterSpec {
    /// Create a new filter with default settings (show everything except hidden/system).
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a filter that shows all files including hidden.
    pub fn show_all() -> Self {
        Self {
            show_hidden: true,
            show_system: true,
            ..Self::default()
        }
    }

    /// Set the text pattern to filter by.
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Clear the text pattern.
    pub fn clear_pattern(&mut self) {
        self.pattern = None;
    }

    /// Set whether to show hidden files.
    pub fn with_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        self
    }

    /// Toggle visibility of hidden files.
    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
    }

    /// Add an extension to filter by (lowercase, without dot).
    pub fn with_extension(mut self, ext: impl Into<String>) -> Self {
        self.extensions.push(ext.into().to_lowercase());
        self
    }

    /// Set size range filter.
    pub fn with_size_range(mut self, min: Option<u64>, max: Option<u64>) -> Self {
        self.min_size = min;
        self.max_size = max;
        self
    }

    /// Check if an entry matches this filter.
    pub fn matches(&self, entry: &EntryMeta) -> bool {
        // Hidden file check
        if !self.show_hidden && entry.is_hidden() {
            return false;
        }

        // System file check
        if !self.show_system && entry.attributes.system {
            return false;
        }

        // Pattern check (case-insensitive)
        if let Some(ref pattern) = self.pattern {
            let pattern_lower = pattern.to_lowercase();
            let name_lower = entry.name.to_lowercase();
            if !name_lower.contains(&pattern_lower) {
                return false;
            }
        }

        // Extension check (only for files)
        if !self.extensions.is_empty() && entry.is_file() {
            match &entry.extension {
                Some(ext) if self.extensions.contains(ext) => {}
                _ => return false,
            }
        }

        // Size range check (only for files)
        if entry.is_file() {
            if let Some(min) = self.min_size {
                if entry.size < min {
                    return false;
                }
            }
            if let Some(max) = self.max_size {
                if entry.size > max {
                    return false;
                }
            }
        }

        true
    }

    /// Filter a slice of entries, returning only those that match.
    pub fn filter<'a>(&self, entries: &'a [EntryMeta]) -> Vec<&'a EntryMeta> {
        entries.iter().filter(|e| self.matches(e)).collect()
    }

    /// Filter and clone matching entries.
    pub fn filter_owned(&self, entries: &[EntryMeta]) -> Vec<EntryMeta> {
        entries
            .iter()
            .filter(|e| self.matches(e))
            .cloned()
            .collect()
    }

    /// Returns `true` if this filter is "empty" (matches everything visible).
    pub fn is_default(&self) -> bool {
        self.pattern.is_none()
            && !self.show_hidden
            && !self.show_system
            && self.extensions.is_empty()
            && self.min_size.is_none()
            && self.max_size.is_none()
    }

    /// Returns a description of active filters for display.
    pub fn active_filters_description(&self) -> Option<String> {
        let mut parts = Vec::new();

        if let Some(ref pattern) = self.pattern {
            parts.push(format!("\"{}\"", pattern));
        }

        if self.show_hidden {
            parts.push("hidden".to_string());
        }

        if !self.extensions.is_empty() {
            parts.push(format!(".{}", self.extensions.join(", .")));
        }

        if self.min_size.is_some() || self.max_size.is_some() {
            let min_str = self.min_size.map(crate::entry::format_size);
            let max_str = self.max_size.map(crate::entry::format_size);
            match (min_str, max_str) {
                (Some(min), Some(max)) => parts.push(format!("{} - {}", min, max)),
                (Some(min), None) => parts.push(format!("> {}", min)),
                (None, Some(max)) => parts.push(format!("< {}", max)),
                _ => {}
            }
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntryKind;
    use std::path::PathBuf;

    fn make_file(name: &str, size: u64, hidden: bool) -> EntryMeta {
        let mut meta = EntryMeta::new(
            name.to_string(),
            PathBuf::from(format!("C:\\test\\{}", name)),
            EntryKind::File,
        );
        meta.size = size;
        meta.attributes.hidden = hidden;
        meta
    }

    fn make_dir(name: &str, hidden: bool) -> EntryMeta {
        let mut meta = EntryMeta::new(
            name.to_string(),
            PathBuf::from(format!("C:\\test\\{}", name)),
            EntryKind::Directory,
        );
        meta.attributes.hidden = hidden;
        meta
    }

    #[test]
    fn test_default_filter_hides_hidden() {
        let filter = FilterSpec::new();
        let visible = make_file("visible.txt", 100, false);
        let hidden = make_file("hidden.txt", 100, true);

        assert!(filter.matches(&visible));
        assert!(!filter.matches(&hidden));
    }

    #[test]
    fn test_show_hidden() {
        let filter = FilterSpec::new().with_hidden(true);
        let hidden = make_file("hidden.txt", 100, true);

        assert!(filter.matches(&hidden));
    }

    #[test]
    fn test_pattern_filter() {
        let filter = FilterSpec::new().with_pattern("doc");

        let matches = make_file("document.txt", 100, false);
        let no_match = make_file("image.png", 100, false);

        assert!(filter.matches(&matches));
        assert!(!filter.matches(&no_match));
    }

    #[test]
    fn test_pattern_case_insensitive() {
        let filter = FilterSpec::new().with_pattern("DOC");

        let matches = make_file("document.txt", 100, false);
        assert!(filter.matches(&matches));
    }

    #[test]
    fn test_extension_filter() {
        let filter = FilterSpec::new().with_extension("txt").with_extension("md");

        let txt = make_file("readme.txt", 100, false);
        let md = make_file("notes.md", 100, false);
        let png = make_file("image.png", 100, false);
        let dir = make_dir("folder", false);

        assert!(filter.matches(&txt));
        assert!(filter.matches(&md));
        assert!(!filter.matches(&png));
        // Directories are not filtered by extension
        assert!(filter.matches(&dir));
    }

    #[test]
    fn test_size_range_filter() {
        let filter = FilterSpec::new().with_size_range(Some(100), Some(1000));

        let small = make_file("small.txt", 50, false);
        let medium = make_file("medium.txt", 500, false);
        let large = make_file("large.txt", 2000, false);
        let dir = make_dir("folder", false);

        assert!(!filter.matches(&small));
        assert!(filter.matches(&medium));
        assert!(!filter.matches(&large));
        // Directories are not filtered by size
        assert!(filter.matches(&dir));
    }

    #[test]
    fn test_combined_filters() {
        let filter = FilterSpec::new()
            .with_pattern("report")
            .with_extension("pdf")
            .with_size_range(Some(1000), None);

        let matches = {
            let mut e = make_file("annual-report.pdf", 5000, false);
            e.extension = Some("pdf".to_string());
            e
        };
        let wrong_ext = make_file("report.txt", 5000, false);
        let wrong_size = {
            let mut e = make_file("report.pdf", 500, false);
            e.extension = Some("pdf".to_string());
            e
        };
        let wrong_pattern = {
            let mut e = make_file("invoice.pdf", 5000, false);
            e.extension = Some("pdf".to_string());
            e
        };

        assert!(filter.matches(&matches));
        assert!(!filter.matches(&wrong_ext));
        assert!(!filter.matches(&wrong_size));
        assert!(!filter.matches(&wrong_pattern));
    }

    #[test]
    fn test_filter_owned() {
        let entries = vec![
            make_file("visible.txt", 100, false),
            make_file("hidden.txt", 100, true),
            make_file("another.txt", 200, false),
        ];

        let filter = FilterSpec::new();
        let filtered = filter.filter_owned(&entries);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|e| !e.is_hidden()));
    }

    #[test]
    fn test_active_filters_description() {
        let filter = FilterSpec::new();
        assert!(filter.active_filters_description().is_none());

        let filter = FilterSpec::new().with_pattern("test");
        assert_eq!(
            filter.active_filters_description(),
            Some("\"test\"".to_string())
        );

        let filter = FilterSpec::new()
            .with_hidden(true)
            .with_extension("txt")
            .with_extension("md");
        let desc = filter.active_filters_description().unwrap();
        assert!(desc.contains("hidden"));
        assert!(desc.contains(".txt"));
    }

    #[test]
    fn test_toggle_hidden() {
        let mut filter = FilterSpec::new();
        assert!(!filter.show_hidden);

        filter.toggle_hidden();
        assert!(filter.show_hidden);

        filter.toggle_hidden();
        assert!(!filter.show_hidden);
    }

    #[test]
    fn test_filter_serialization() {
        let filter = FilterSpec::new()
            .with_pattern("test")
            .with_hidden(true)
            .with_extension("rs");

        let json = serde_json::to_string(&filter).expect("serialize");
        let deserialized: FilterSpec = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.pattern, Some("test".to_string()));
        assert!(deserialized.show_hidden);
        assert_eq!(deserialized.extensions, vec!["rs"]);
    }
}
