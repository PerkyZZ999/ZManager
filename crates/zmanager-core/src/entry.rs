//! Core domain types for file system entries.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The kind of a file system entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
    /// A regular file.
    File,
    /// A directory (folder).
    Directory,
    /// A symbolic link (reparse point on Windows).
    Symlink,
    /// A junction point (Windows-specific directory link).
    Junction,
}

impl EntryKind {
    /// Returns `true` if this is a file.
    pub fn is_file(&self) -> bool {
        matches!(self, Self::File)
    }

    /// Returns `true` if this is a directory.
    pub fn is_directory(&self) -> bool {
        matches!(self, Self::Directory)
    }

    /// Returns `true` if this is a symlink or junction.
    pub fn is_link(&self) -> bool {
        matches!(self, Self::Symlink | Self::Junction)
    }

    /// Returns a human-readable label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Directory => "Folder",
            Self::Symlink => "Symlink",
            Self::Junction => "Junction",
        }
    }
}

impl std::fmt::Display for EntryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// File attributes (Windows-centric, with cross-platform compatibility).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryAttributes {
    /// The file/folder is hidden.
    pub hidden: bool,
    /// The file/folder is a system file.
    pub system: bool,
    /// The file/folder is read-only.
    pub readonly: bool,
    /// The file/folder is archived (backup flag).
    pub archive: bool,
}

impl EntryAttributes {
    /// Create attributes with all flags set to false.
    pub fn none() -> Self {
        Self::default()
    }

    /// Returns `true` if any special attribute is set.
    pub fn has_any(&self) -> bool {
        self.hidden || self.system || self.readonly
    }
}

/// Metadata for a single file system entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryMeta {
    /// The file/folder name (not the full path).
    pub name: String,

    /// The absolute path to this entry.
    pub path: PathBuf,

    /// The kind of entry (file, directory, symlink, junction).
    pub kind: EntryKind,

    /// Size in bytes. For directories, this is typically 0.
    /// Use `calculate_folder_size` for actual folder sizes.
    pub size: u64,

    /// Creation time (Windows: birth time).
    pub created: Option<DateTime<Utc>>,

    /// Last modification time.
    pub modified: Option<DateTime<Utc>>,

    /// Last access time.
    pub accessed: Option<DateTime<Utc>>,

    /// File attributes.
    pub attributes: EntryAttributes,

    /// For symlinks/junctions: the resolved target path.
    /// `None` if not a link, or if resolution failed.
    pub link_target: Option<PathBuf>,

    /// `true` if this is a broken symlink (target doesn't exist).
    pub is_broken_link: bool,

    /// The file extension (lowercase, without the dot).
    /// `None` for directories or files without extensions.
    pub extension: Option<String>,
}

impl EntryMeta {
    /// Create a new `EntryMeta` with minimal required fields.
    pub fn new(name: String, path: PathBuf, kind: EntryKind) -> Self {
        let extension = if kind.is_file() {
            path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
        } else {
            None
        };

        Self {
            name,
            path,
            kind,
            size: 0,
            created: None,
            modified: None,
            accessed: None,
            attributes: EntryAttributes::default(),
            link_target: None,
            is_broken_link: false,
            extension,
        }
    }

    /// Returns `true` if this entry should be hidden by default.
    pub fn is_hidden(&self) -> bool {
        self.attributes.hidden || self.name.starts_with('.')
    }

    /// Returns `true` if this is a file.
    pub fn is_file(&self) -> bool {
        self.kind.is_file()
    }

    /// Returns `true` if this is a directory.
    pub fn is_directory(&self) -> bool {
        self.kind.is_directory()
    }

    /// Returns `true` if this is any kind of link.
    pub fn is_link(&self) -> bool {
        self.kind.is_link()
    }

    /// Get a human-readable size string (e.g., "1.5 MB").
    pub fn size_display(&self) -> String {
        format_size(self.size)
    }
}

/// A listing of directory contents with summary statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirListing {
    /// The directory path that was listed.
    pub path: PathBuf,

    /// The entries in this directory.
    pub entries: Vec<EntryMeta>,

    /// Number of files in this directory (direct children only).
    pub file_count: usize,

    /// Number of directories in this directory (direct children only).
    pub dir_count: usize,

    /// Total size of all files (direct children only).
    pub total_size: u64,
}

impl DirListing {
    /// Create a new directory listing.
    pub fn new(path: PathBuf, entries: Vec<EntryMeta>) -> Self {
        let file_count = entries.iter().filter(|e| e.is_file()).count();
        let dir_count = entries.iter().filter(|e| e.is_directory()).count();
        let total_size = entries.iter().filter(|e| e.is_file()).map(|e| e.size).sum();

        Self {
            path,
            entries,
            file_count,
            dir_count,
            total_size,
        }
    }

    /// Returns `true` if the directory is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Get an entry by name.
    pub fn get(&self, name: &str) -> Option<&EntryMeta> {
        self.entries.iter().find(|e| e.name == name)
    }
}

/// Format a byte size as a human-readable string.
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_kind_is_methods() {
        assert!(EntryKind::File.is_file());
        assert!(!EntryKind::File.is_directory());
        assert!(!EntryKind::File.is_link());

        assert!(EntryKind::Directory.is_directory());
        assert!(!EntryKind::Directory.is_file());

        assert!(EntryKind::Symlink.is_link());
        assert!(EntryKind::Junction.is_link());
    }

    #[test]
    fn test_entry_kind_labels() {
        assert_eq!(EntryKind::File.label(), "File");
        assert_eq!(EntryKind::Directory.label(), "Folder");
        assert_eq!(EntryKind::Symlink.label(), "Symlink");
        assert_eq!(EntryKind::Junction.label(), "Junction");
    }

    #[test]
    fn test_entry_meta_new() {
        let meta = EntryMeta::new(
            "test.txt".to_string(),
            PathBuf::from("C:\\test\\test.txt"),
            EntryKind::File,
        );

        assert_eq!(meta.name, "test.txt");
        assert!(meta.is_file());
        assert_eq!(meta.extension, Some("txt".to_string()));
    }

    #[test]
    fn test_entry_meta_directory_no_extension() {
        let meta = EntryMeta::new(
            "folder".to_string(),
            PathBuf::from("C:\\test\\folder"),
            EntryKind::Directory,
        );

        assert!(meta.is_directory());
        assert_eq!(meta.extension, None);
    }

    #[test]
    fn test_entry_meta_hidden() {
        let mut meta = EntryMeta::new(
            "normal.txt".to_string(),
            PathBuf::from("C:\\normal.txt"),
            EntryKind::File,
        );
        assert!(!meta.is_hidden());

        meta.attributes.hidden = true;
        assert!(meta.is_hidden());

        // Unix-style hidden (starts with dot)
        let dot_meta = EntryMeta::new(
            ".hidden".to_string(),
            PathBuf::from("C:\\.hidden"),
            EntryKind::File,
        );
        assert!(dot_meta.is_hidden());
    }

    #[test]
    fn test_dir_listing_stats() {
        let entries = vec![
            {
                let mut e = EntryMeta::new(
                    "file1.txt".to_string(),
                    PathBuf::from("C:\\test\\file1.txt"),
                    EntryKind::File,
                );
                e.size = 1000;
                e
            },
            {
                let mut e = EntryMeta::new(
                    "file2.txt".to_string(),
                    PathBuf::from("C:\\test\\file2.txt"),
                    EntryKind::File,
                );
                e.size = 2000;
                e
            },
            EntryMeta::new(
                "subdir".to_string(),
                PathBuf::from("C:\\test\\subdir"),
                EntryKind::Directory,
            ),
        ];

        let listing = DirListing::new(PathBuf::from("C:\\test"), entries);

        assert_eq!(listing.len(), 3);
        assert_eq!(listing.file_count, 2);
        assert_eq!(listing.dir_count, 1);
        assert_eq!(listing.total_size, 3000);
        assert!(!listing.is_empty());
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(1073741824), "1.00 GB");
        assert_eq!(format_size(1099511627776), "1.00 TB");
    }

    #[test]
    fn test_entry_meta_serialization() {
        let meta = EntryMeta::new(
            "test.txt".to_string(),
            PathBuf::from("C:\\test\\test.txt"),
            EntryKind::File,
        );

        let json = serde_json::to_string(&meta).expect("serialize");
        let deserialized: EntryMeta = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.name, meta.name);
        assert_eq!(deserialized.kind, meta.kind);
    }
}
