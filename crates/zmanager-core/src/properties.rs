//! File and folder properties collection.
//!
//! This module provides functionality to gather detailed properties
//! about files and folders, including async size calculation for folders.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::entry::EntryKind;
use crate::{ZError, ZResult};

/// Detailed properties for a file or folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Properties {
    /// The path to the item.
    pub path: PathBuf,
    /// Display name.
    pub name: String,
    /// Type of entry.
    pub kind: EntryKind,
    /// Size in bytes (for files) or calculated size (for folders).
    pub size: Option<u64>,
    /// Number of files (for folders).
    pub file_count: Option<usize>,
    /// Number of subdirectories (for folders).
    pub folder_count: Option<usize>,
    /// Creation time.
    pub created: Option<SystemTime>,
    /// Last modification time.
    pub modified: Option<SystemTime>,
    /// Last access time.
    pub accessed: Option<SystemTime>,
    /// Whether the item is read-only.
    pub readonly: bool,
    /// Whether the item is hidden.
    pub hidden: bool,
    /// Whether the item is a system file.
    pub system: bool,
    /// Whether the item is an archive (ready for backup).
    pub archive: bool,
    /// Link target (for symlinks/junctions).
    pub link_target: Option<PathBuf>,
    /// File extension (for files).
    pub extension: Option<String>,
    /// MIME type (if determinable).
    pub mime_type: Option<String>,
}

impl Properties {
    /// Get a human-readable size string.
    pub fn size_display(&self) -> String {
        self.size.map(format_size).unwrap_or_else(|| "-".into())
    }

    /// Get a summary string (e.g., "10 files, 3 folders").
    pub fn contents_summary(&self) -> Option<String> {
        match (self.file_count, self.folder_count) {
            (Some(files), Some(folders)) => Some(format!("{files} files, {folders} folders")),
            (Some(files), None) => Some(format!("{files} files")),
            (None, Some(folders)) => Some(format!("{folders} folders")),
            (None, None) => None,
        }
    }

    /// Format the modification time for display.
    pub fn modified_display(&self) -> Option<String> {
        self.modified.map(|t| {
            let datetime: chrono::DateTime<chrono::Local> = t.into();
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        })
    }

    /// Format the creation time for display.
    pub fn created_display(&self) -> Option<String> {
        self.created.map(|t| {
            let datetime: chrono::DateTime<chrono::Local> = t.into();
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        })
    }

    /// Get attribute flags as a string (like "RHSA").
    pub fn attributes_display(&self) -> String {
        let mut attrs = String::with_capacity(4);
        if self.readonly {
            attrs.push('R');
        }
        if self.hidden {
            attrs.push('H');
        }
        if self.system {
            attrs.push('S');
        }
        if self.archive {
            attrs.push('A');
        }
        if attrs.is_empty() {
            "-".to_string()
        } else {
            attrs
        }
    }
}

/// Format size as human-readable string.
fn format_size(bytes: u64) -> String {
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
        format!("{} bytes", bytes)
    }
}

/// Get basic properties for a path (without folder size calculation).
pub fn get_properties(path: impl AsRef<Path>) -> ZResult<Properties> {
    let path = path.as_ref();

    debug!(path = %path.display(), "Getting properties");

    let metadata = std::fs::metadata(path).map_err(|e| ZError::from_io(path, e))?;

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    let kind = if metadata.is_dir() {
        // Check if it's a symlink to a directory
        if std::fs::symlink_metadata(path)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
        {
            EntryKind::Symlink
        } else {
            EntryKind::Directory
        }
    } else if metadata.is_file() {
        EntryKind::File
    } else if metadata.file_type().is_symlink() {
        EntryKind::Symlink
    } else {
        EntryKind::File // Default fallback
    };

    let size = if metadata.is_file() {
        Some(metadata.len())
    } else {
        None // Folder size requires async calculation
    };

    let extension = if metadata.is_file() {
        path.extension().map(|e| e.to_string_lossy().into_owned())
    } else {
        None
    };

    // Get link target if symlink
    let link_target = if kind == EntryKind::Symlink {
        std::fs::read_link(path).ok()
    } else {
        None
    };

    // Get timestamps
    let created = metadata.created().ok();
    let modified = metadata.modified().ok();
    let accessed = metadata.accessed().ok();

    // Get Windows attributes
    #[cfg(windows)]
    let (readonly, hidden, system, archive) = {
        use std::os::windows::fs::MetadataExt;

        const FILE_ATTRIBUTE_READONLY: u32 = 0x1;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
        const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x20;

        let attrs = metadata.file_attributes();
        (
            attrs & FILE_ATTRIBUTE_READONLY != 0,
            attrs & FILE_ATTRIBUTE_HIDDEN != 0,
            attrs & FILE_ATTRIBUTE_SYSTEM != 0,
            attrs & FILE_ATTRIBUTE_ARCHIVE != 0,
        )
    };

    #[cfg(not(windows))]
    let (readonly, hidden, system, archive) = {
        use std::os::unix::fs::PermissionsExt;

        let readonly = metadata.permissions().readonly();
        let hidden = name.starts_with('.');
        (readonly, hidden, false, false)
    };

    // Simple MIME type detection based on extension
    let mime_type = extension.as_ref().and_then(|ext| guess_mime_type(ext));

    Ok(Properties {
        path: path.to_path_buf(),
        name,
        kind,
        size,
        file_count: None,
        folder_count: None,
        created,
        modified,
        accessed,
        readonly,
        hidden,
        system,
        archive,
        link_target,
        extension,
        mime_type,
    })
}

/// Calculate folder size and item counts.
///
/// This can be slow for large directories, so it should be run async.
pub fn calculate_folder_stats(path: impl AsRef<Path>) -> ZResult<FolderStats> {
    let path = path.as_ref();

    debug!(path = %path.display(), "Calculating folder stats");

    if !path.is_dir() {
        return Err(ZError::NotADirectory {
            path: path.to_path_buf(),
        });
    }

    let mut stats = FolderStats::default();
    calculate_folder_stats_recursive(path, &mut stats)?;

    debug!(
        path = %path.display(),
        size = stats.total_size,
        files = stats.file_count,
        folders = stats.folder_count,
        "Folder stats calculated"
    );

    Ok(stats)
}

fn calculate_folder_stats_recursive(path: &Path, stats: &mut FolderStats) -> ZResult<()> {
    let entries = std::fs::read_dir(path).map_err(|e| ZError::from_io(path, e))?;

    for entry in entries.flatten() {
        let entry_path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue, // Skip inaccessible entries
        };

        if metadata.is_dir() {
            stats.folder_count += 1;
            // Recursively process subdirectory
            let _ = calculate_folder_stats_recursive(&entry_path, stats);
        } else if metadata.is_file() {
            stats.file_count += 1;
            stats.total_size += metadata.len();
        }
    }

    Ok(())
}

/// Statistics about a folder's contents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FolderStats {
    /// Total size of all files in bytes.
    pub total_size: u64,
    /// Number of files.
    pub file_count: usize,
    /// Number of subdirectories.
    pub folder_count: usize,
}

impl FolderStats {
    /// Get human-readable size.
    pub fn size_display(&self) -> String {
        format_size(self.total_size)
    }

    /// Get summary string.
    pub fn summary(&self) -> String {
        format!(
            "{}, {} files, {} folders",
            self.size_display(),
            self.file_count,
            self.folder_count
        )
    }
}

/// Get properties for multiple paths.
pub fn get_multiple_properties(paths: &[PathBuf]) -> Vec<ZResult<Properties>> {
    paths.iter().map(get_properties).collect()
}

/// Guess MIME type from file extension.
fn guess_mime_type(extension: &str) -> Option<String> {
    let ext = extension.to_lowercase();
    let mime = match ext.as_str() {
        // Text
        "txt" => "text/plain",
        "md" | "markdown" => "text/markdown",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "csv" => "text/csv",

        // Images
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",

        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "m4a" => "audio/mp4",

        // Video
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",

        // Archives
        "zip" => "application/zip",
        "rar" => "application/vnd.rar",
        "7z" => "application/x-7z-compressed",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",

        // Documents
        "pdf" => "application/pdf",
        "doc" | "docx" => "application/msword",
        "xls" | "xlsx" => "application/vnd.ms-excel",
        "ppt" | "pptx" => "application/vnd.ms-powerpoint",

        // Programming
        "rs" => "text/x-rust",
        "py" => "text/x-python",
        "ts" => "text/typescript",
        "tsx" => "text/typescript-jsx",
        "jsx" => "text/javascript-jsx",
        "go" => "text/x-go",
        "c" | "h" => "text/x-c",
        "cpp" | "hpp" => "text/x-c++",
        "java" => "text/x-java",

        // Executables
        "exe" => "application/x-msdownload",
        "dll" => "application/x-msdownload",
        "msi" => "application/x-msi",

        _ => return None,
    };
    Some(mime.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 bytes");
        assert_eq!(format_size(100), "100 bytes");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_get_file_properties() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");
        std::fs::write(&file, "Hello, World!").unwrap();

        let props = get_properties(&file).unwrap();

        assert_eq!(props.name, "test.txt");
        assert_eq!(props.kind, EntryKind::File);
        assert_eq!(props.size, Some(13));
        assert_eq!(props.extension.as_deref(), Some("txt"));
        assert_eq!(props.mime_type.as_deref(), Some("text/plain"));
        assert!(props.modified.is_some());
    }

    #[test]
    fn test_get_directory_properties() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("subdir");
        std::fs::create_dir(&dir).unwrap();

        let props = get_properties(&dir).unwrap();

        assert_eq!(props.kind, EntryKind::Directory);
        assert!(props.size.is_none()); // No size for dirs initially
        assert!(props.extension.is_none());
    }

    #[test]
    fn test_calculate_folder_stats() {
        let temp = TempDir::new().unwrap();

        // Create structure:
        // - file1.txt (10 bytes)
        // - file2.txt (20 bytes)
        // - subdir/
        //   - file3.txt (30 bytes)
        std::fs::write(temp.path().join("file1.txt"), "0123456789").unwrap();
        std::fs::write(temp.path().join("file2.txt"), "01234567890123456789").unwrap();

        let subdir = temp.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        std::fs::write(subdir.join("file3.txt"), "012345678901234567890123456789").unwrap();

        let stats = calculate_folder_stats(temp.path()).unwrap();

        assert_eq!(stats.file_count, 3);
        assert_eq!(stats.folder_count, 1);
        assert_eq!(stats.total_size, 60);
    }

    #[test]
    fn test_folder_stats_not_dir() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("file.txt");
        std::fs::write(&file, "content").unwrap();

        let result = calculate_folder_stats(&file);
        assert!(matches!(result, Err(ZError::NotADirectory { .. })));
    }

    #[test]
    fn test_properties_display() {
        let props = Properties {
            path: PathBuf::from("test.txt"),
            name: "test.txt".to_string(),
            kind: EntryKind::File,
            size: Some(1024 * 1024 * 5), // 5 MB
            file_count: None,
            folder_count: None,
            created: None,
            modified: None,
            accessed: None,
            readonly: true,
            hidden: true,
            system: false,
            archive: true,
            link_target: None,
            extension: Some("txt".to_string()),
            mime_type: Some("text/plain".to_string()),
        };

        assert_eq!(props.size_display(), "5.00 MB");
        assert_eq!(props.attributes_display(), "RHA");
    }

    #[test]
    fn test_guess_mime_type() {
        assert_eq!(
            guess_mime_type("txt").as_deref(),
            Some("text/plain")
        );
        assert_eq!(
            guess_mime_type("PNG").as_deref(),
            Some("image/png")
        );
        assert_eq!(
            guess_mime_type("rs").as_deref(),
            Some("text/x-rust")
        );
        assert!(guess_mime_type("xyz").is_none());
    }
}
