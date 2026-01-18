//! Sorting specifications for directory listings.

use crate::EntryMeta;
use serde::{Deserialize, Serialize};

/// The field to sort entries by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    /// Sort by file/folder name (case-insensitive).
    #[default]
    Name,
    /// Sort by file size.
    Size,
    /// Sort by modification date.
    Modified,
    /// Sort by creation date.
    Created,
    /// Sort by file extension.
    Extension,
    /// Sort by entry kind (directories first, then files).
    Kind,
}

impl SortField {
    /// Get a human-readable label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Size => "Size",
            Self::Modified => "Date Modified",
            Self::Created => "Date Created",
            Self::Extension => "Type",
            Self::Kind => "Kind",
        }
    }
}

impl std::fmt::Display for SortField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// The order to sort entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    /// Ascending order (A-Z, smallest first, oldest first).
    #[default]
    Ascending,
    /// Descending order (Z-A, largest first, newest first).
    Descending,
}

impl SortOrder {
    /// Toggle the sort order.
    pub fn toggle(&self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }

    /// Get a short label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ascending => "↑",
            Self::Descending => "↓",
        }
    }
}

/// A complete sorting specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortSpec {
    /// The primary sort field.
    pub field: SortField,
    /// The sort order.
    pub order: SortOrder,
    /// Whether to always show directories before files.
    pub directories_first: bool,
}

impl Default for SortSpec {
    fn default() -> Self {
        Self {
            field: SortField::Name,
            order: SortOrder::Ascending,
            directories_first: true,
        }
    }
}

impl SortSpec {
    /// Create a new sort specification.
    pub fn new(field: SortField, order: SortOrder) -> Self {
        Self {
            field,
            order,
            directories_first: true,
        }
    }

    /// Create a sort spec for sorting by name ascending.
    pub fn by_name() -> Self {
        Self::new(SortField::Name, SortOrder::Ascending)
    }

    /// Create a sort spec for sorting by size descending.
    pub fn by_size() -> Self {
        Self::new(SortField::Size, SortOrder::Descending)
    }

    /// Create a sort spec for sorting by modification date descending.
    pub fn by_modified() -> Self {
        Self::new(SortField::Modified, SortOrder::Descending)
    }

    /// Toggle the order if the same field, otherwise set new field with ascending order.
    pub fn toggle_or_set(&mut self, field: SortField) {
        if self.field == field {
            self.order = self.order.toggle();
        } else {
            self.field = field;
            self.order = SortOrder::Ascending;
        }
    }

    /// Sort a slice of entries in place according to this specification.
    pub fn sort(&self, entries: &mut [EntryMeta]) {
        entries.sort_by(|a, b| {
            // Directories first, if enabled
            if self.directories_first {
                match (a.is_directory(), b.is_directory()) {
                    (true, false) => return std::cmp::Ordering::Less,
                    (false, true) => return std::cmp::Ordering::Greater,
                    _ => {}
                }
            }

            // Primary sort field comparison
            let cmp = match self.field {
                SortField::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortField::Size => a.size.cmp(&b.size),
                SortField::Modified => a.modified.cmp(&b.modified),
                SortField::Created => a.created.cmp(&b.created),
                SortField::Extension => {
                    let ext_a = a.extension.as_deref().unwrap_or("");
                    let ext_b = b.extension.as_deref().unwrap_or("");
                    ext_a.cmp(ext_b)
                }
                SortField::Kind => a.kind.label().cmp(b.kind.label()),
            };

            // Apply order
            match self.order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });
    }

    /// Sort a vector of entries and return it (for chaining).
    pub fn sorted(&self, mut entries: Vec<EntryMeta>) -> Vec<EntryMeta> {
        self.sort(&mut entries);
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntryKind;
    use std::path::PathBuf;

    fn make_file(name: &str, size: u64) -> EntryMeta {
        let mut meta = EntryMeta::new(
            name.to_string(),
            PathBuf::from(format!("C:\\test\\{}", name)),
            EntryKind::File,
        );
        meta.size = size;
        meta
    }

    fn make_dir(name: &str) -> EntryMeta {
        EntryMeta::new(
            name.to_string(),
            PathBuf::from(format!("C:\\test\\{}", name)),
            EntryKind::Directory,
        )
    }

    #[test]
    fn test_sort_by_name_ascending() {
        let mut entries = vec![
            make_file("charlie.txt", 100),
            make_file("alpha.txt", 200),
            make_file("bravo.txt", 150),
        ];

        let spec = SortSpec::new(SortField::Name, SortOrder::Ascending);
        spec.sort(&mut entries);

        assert_eq!(entries[0].name, "alpha.txt");
        assert_eq!(entries[1].name, "bravo.txt");
        assert_eq!(entries[2].name, "charlie.txt");
    }

    #[test]
    fn test_sort_by_name_descending() {
        let mut entries = vec![
            make_file("alpha.txt", 100),
            make_file("charlie.txt", 200),
            make_file("bravo.txt", 150),
        ];

        let spec = SortSpec::new(SortField::Name, SortOrder::Descending);
        spec.sort(&mut entries);

        assert_eq!(entries[0].name, "charlie.txt");
        assert_eq!(entries[1].name, "bravo.txt");
        assert_eq!(entries[2].name, "alpha.txt");
    }

    #[test]
    fn test_sort_by_size() {
        let mut entries = vec![
            make_file("medium.txt", 500),
            make_file("small.txt", 100),
            make_file("large.txt", 1000),
        ];

        let spec = SortSpec::new(SortField::Size, SortOrder::Descending);
        spec.sort(&mut entries);

        assert_eq!(entries[0].name, "large.txt");
        assert_eq!(entries[1].name, "medium.txt");
        assert_eq!(entries[2].name, "small.txt");
    }

    #[test]
    fn test_directories_first() {
        let mut entries = vec![
            make_file("zfile.txt", 100),
            make_dir("adir"),
            make_file("afile.txt", 200),
            make_dir("zdir"),
        ];

        let mut spec = SortSpec::new(SortField::Name, SortOrder::Ascending);
        spec.directories_first = true;
        spec.sort(&mut entries);

        // Directories first, sorted by name
        assert!(entries[0].is_directory());
        assert!(entries[1].is_directory());
        assert!(entries[2].is_file());
        assert!(entries[3].is_file());

        assert_eq!(entries[0].name, "adir");
        assert_eq!(entries[1].name, "zdir");
        assert_eq!(entries[2].name, "afile.txt");
        assert_eq!(entries[3].name, "zfile.txt");
    }

    #[test]
    fn test_directories_first_disabled() {
        let mut entries = vec![
            make_file("zfile.txt", 100),
            make_dir("adir"),
            make_file("afile.txt", 200),
        ];

        let mut spec = SortSpec::new(SortField::Name, SortOrder::Ascending);
        spec.directories_first = false;
        spec.sort(&mut entries);

        // Pure alphabetical
        assert_eq!(entries[0].name, "adir");
        assert_eq!(entries[1].name, "afile.txt");
        assert_eq!(entries[2].name, "zfile.txt");
    }

    #[test]
    fn test_case_insensitive_sort() {
        let mut entries = vec![
            make_file("Beta.txt", 100),
            make_file("alpha.txt", 200),
            make_file("GAMMA.txt", 150),
        ];

        let spec = SortSpec::by_name();
        spec.sort(&mut entries);

        assert_eq!(entries[0].name, "alpha.txt");
        assert_eq!(entries[1].name, "Beta.txt");
        assert_eq!(entries[2].name, "GAMMA.txt");
    }

    #[test]
    fn test_toggle_or_set() {
        let mut spec = SortSpec::default();
        assert_eq!(spec.field, SortField::Name);
        assert_eq!(spec.order, SortOrder::Ascending);

        // Toggle same field
        spec.toggle_or_set(SortField::Name);
        assert_eq!(spec.field, SortField::Name);
        assert_eq!(spec.order, SortOrder::Descending);

        // Set different field
        spec.toggle_or_set(SortField::Size);
        assert_eq!(spec.field, SortField::Size);
        assert_eq!(spec.order, SortOrder::Ascending);
    }

    #[test]
    fn test_sort_spec_serialization() {
        let spec = SortSpec::by_modified();
        let json = serde_json::to_string(&spec).expect("serialize");
        let deserialized: SortSpec = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.field, SortField::Modified);
        assert_eq!(deserialized.order, SortOrder::Descending);
    }
}
