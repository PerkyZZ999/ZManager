//! Directory listing and file system operations.

use crate::{DirListing, EntryAttributes, EntryKind, EntryMeta, FilterSpec, SortSpec, ZError, ZResult};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, instrument, warn};

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

/// Windows file attribute constants.
#[cfg(windows)]
mod win_attrs {
    pub const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    pub const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
    pub const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
    pub const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x20;
    pub const FILE_ATTRIBUTE_READONLY: u32 = 0x1;
    pub const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
}

/// Prefix a path with `\\?\` for long path support on Windows.
/// Returns the original path if already prefixed or on non-Windows.
pub fn to_long_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let path_str = path.to_string_lossy();
        if path_str.starts_with(r"\\?\") {
            path.to_path_buf()
        } else if path.is_absolute() {
            PathBuf::from(format!(r"\\?\{}", path_str))
        } else {
            path.to_path_buf()
        }
    }
    #[cfg(not(windows))]
    {
        path.to_path_buf()
    }
}

/// Check if a path is "long" (>= 240 characters, leaving room for filenames).
pub fn is_long_path(path: &Path) -> bool {
    path.to_string_lossy().len() >= 240
}

/// List the contents of a directory.
///
/// # Arguments
/// * `path` - The directory path to list
/// * `sort` - Optional sorting specification
/// * `filter` - Optional filtering specification
///
/// # Returns
/// A `DirListing` containing all matching entries, sorted as specified.
#[instrument(skip(path, sort, filter))]
pub fn list_directory(
    path: impl AsRef<Path>,
    sort: Option<&SortSpec>,
    filter: Option<&FilterSpec>,
) -> ZResult<DirListing> {
    let path = path.as_ref();
    let read_path = if is_long_path(path) {
        to_long_path(path)
    } else {
        path.to_path_buf()
    };

    debug!(path = %path.display(), "Listing directory");

    // Verify path exists and is a directory
    let metadata = fs::metadata(&read_path).map_err(|e| ZError::from_io(path, e))?;
    if !metadata.is_dir() {
        return Err(ZError::NotADirectory {
            path: path.to_path_buf(),
        });
    }

    // Read directory entries
    let read_dir = fs::read_dir(&read_path).map_err(|e| ZError::from_io(path, e))?;

    let mut entries = Vec::new();

    for entry_result in read_dir {
        match entry_result {
            Ok(entry) => {
                match read_entry_meta(&entry) {
                    Ok(meta) => {
                        // Apply filter if provided
                        let include = filter.is_none_or(|f| f.matches(&meta));
                        if include {
                            entries.push(meta);
                        }
                    }
                    Err(e) => {
                        // Log but continue on individual entry errors
                        warn!("Failed to read entry {:?}: {}", entry.path(), e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read directory entry: {}", e);
            }
        }
    }

    // Apply sorting if provided
    if let Some(sort_spec) = sort {
        sort_spec.sort(&mut entries);
    } else {
        // Default sort: directories first, then by name
        SortSpec::default().sort(&mut entries);
    }

    Ok(DirListing::new(path.to_path_buf(), entries))
}

/// Read metadata for a single directory entry.
fn read_entry_meta(entry: &fs::DirEntry) -> ZResult<EntryMeta> {
    let path = entry.path();
    let name = entry
        .file_name()
        .to_string_lossy()
        .into_owned();

    // Get metadata (don't follow symlinks)
    let metadata = entry.metadata().map_err(|e| ZError::from_io(&path, e))?;

    // Determine entry kind and attributes
    let (kind, attributes, link_target, is_broken_link) = analyze_entry(&path, &metadata)?;

    // Extract timestamps
    let created = metadata
        .created()
        .ok()
        .and_then(|t| chrono::DateTime::from_timestamp(
            t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64,
            0,
        ));

    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| chrono::DateTime::from_timestamp(
            t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64,
            0,
        ));

    let accessed = metadata
        .accessed()
        .ok()
        .and_then(|t| chrono::DateTime::from_timestamp(
            t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64,
            0,
        ));

    // Get size (0 for directories)
    let size = if kind.is_file() {
        metadata.len()
    } else {
        0
    };

    // Extract extension
    let extension = if kind.is_file() {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
    } else {
        None
    };

    Ok(EntryMeta {
        name,
        path,
        kind,
        size,
        created,
        modified,
        accessed,
        attributes,
        link_target,
        is_broken_link,
        extension,
    })
}

/// Analyze an entry to determine its kind, attributes, and link target.
#[cfg(windows)]
fn analyze_entry(
    path: &Path,
    metadata: &fs::Metadata,
) -> ZResult<(EntryKind, EntryAttributes, Option<PathBuf>, bool)> {
    use win_attrs::*;

    let attrs = metadata.file_attributes();

    // Extract Windows attributes
    let attributes = EntryAttributes {
        hidden: (attrs & FILE_ATTRIBUTE_HIDDEN) != 0,
        system: (attrs & FILE_ATTRIBUTE_SYSTEM) != 0,
        readonly: (attrs & FILE_ATTRIBUTE_READONLY) != 0,
        archive: (attrs & FILE_ATTRIBUTE_ARCHIVE) != 0,
    };

    // Check if it's a reparse point (symlink or junction)
    let is_reparse = (attrs & FILE_ATTRIBUTE_REPARSE_POINT) != 0;
    let is_dir = (attrs & FILE_ATTRIBUTE_DIRECTORY) != 0;

    if is_reparse {
        // It's a symlink or junction
        let (kind, link_target, is_broken) = analyze_reparse_point(path, is_dir)?;
        Ok((kind, attributes, link_target, is_broken))
    } else if is_dir {
        Ok((EntryKind::Directory, attributes, None, false))
    } else {
        Ok((EntryKind::File, attributes, None, false))
    }
}

#[cfg(not(windows))]
fn analyze_entry(
    path: &Path,
    metadata: &fs::Metadata,
) -> ZResult<(EntryKind, EntryAttributes, Option<PathBuf>, bool)> {
    let attributes = EntryAttributes::default();

    if metadata.is_symlink() {
        // Read symlink target
        match fs::read_link(path) {
            Ok(target) => {
                let is_broken = !target.exists();
                Ok((EntryKind::Symlink, attributes, Some(target), is_broken))
            }
            Err(_) => Ok((EntryKind::Symlink, attributes, None, true)),
        }
    } else if metadata.is_dir() {
        Ok((EntryKind::Directory, attributes, None, false))
    } else {
        Ok((EntryKind::File, attributes, None, false))
    }
}

/// Analyze a Windows reparse point to determine if it's a symlink or junction.
#[cfg(windows)]
fn analyze_reparse_point(
    path: &Path,
    is_dir: bool,
) -> ZResult<(EntryKind, Option<PathBuf>, bool)> {
    // Try to read the link target
    match fs::read_link(path) {
        Ok(target) => {
            // Check if target exists
            let is_broken = !target.exists() && !path.join(&target).exists();

            // Determine if it's a symlink or junction
            // Junctions are directory-only and typically have absolute paths
            let kind = if is_dir && target.is_absolute() {
                // Heuristic: junctions usually have absolute paths
                // A more accurate check would use DeviceIoControl with FSCTL_GET_REPARSE_POINT
                EntryKind::Junction
            } else {
                EntryKind::Symlink
            };

            Ok((kind, Some(target), is_broken))
        }
        Err(_) => {
            // Couldn't read target, assume broken
            let kind = if is_dir {
                EntryKind::Junction
            } else {
                EntryKind::Symlink
            };
            Ok((kind, None, true))
        }
    }
}

/// Get metadata for a single path.
#[instrument(skip(path))]
pub fn get_entry_meta(path: impl AsRef<Path>) -> ZResult<EntryMeta> {
    let path = path.as_ref();
    debug!(path = %path.display(), "Getting entry metadata");
    let read_path = if is_long_path(path) {
        to_long_path(path)
    } else {
        path.to_path_buf()
    };

    let metadata = fs::symlink_metadata(&read_path).map_err(|e| ZError::from_io(path, e))?;

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    let (kind, attributes, link_target, is_broken_link) = analyze_entry(path, &metadata)?;

    let created = metadata
        .created()
        .ok()
        .and_then(|t| chrono::DateTime::from_timestamp(
            t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64,
            0,
        ));

    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| chrono::DateTime::from_timestamp(
            t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64,
            0,
        ));

    let accessed = metadata
        .accessed()
        .ok()
        .and_then(|t| chrono::DateTime::from_timestamp(
            t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64,
            0,
        ));

    let size = if kind.is_file() {
        metadata.len()
    } else {
        0
    };

    let extension = if kind.is_file() {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
    } else {
        None
    };

    Ok(EntryMeta {
        name,
        path: path.to_path_buf(),
        kind,
        size,
        created,
        modified,
        accessed,
        attributes,
        link_target,
        is_broken_link,
        extension,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();

        // Create some files
        File::create(dir.path().join("file1.txt"))
            .unwrap()
            .write_all(b"hello")
            .unwrap();
        File::create(dir.path().join("file2.md"))
            .unwrap()
            .write_all(b"world")
            .unwrap();
        File::create(dir.path().join("large.bin"))
            .unwrap()
            .write_all(&[0u8; 10000])
            .unwrap();

        // Create subdirectory
        fs::create_dir(dir.path().join("subdir")).unwrap();

        // Create hidden file (on Windows, we'd need to set attributes)
        File::create(dir.path().join(".hidden")).unwrap();

        dir
    }

    #[test]
    fn test_list_directory_basic() {
        let dir = setup_test_dir();
        let listing = list_directory(dir.path(), None, None).unwrap();

        assert_eq!(listing.path, dir.path());
        assert!(!listing.is_empty());
        // 4 files + 1 directory
        assert_eq!(listing.len(), 5);
    }

    #[test]
    fn test_list_directory_with_filter() {
        let dir = setup_test_dir();
        let filter = FilterSpec::new().with_pattern("file");
        let listing = list_directory(dir.path(), None, Some(&filter)).unwrap();

        // Should match file1.txt and file2.md
        assert_eq!(listing.len(), 2);
        assert!(listing.entries.iter().all(|e| e.name.contains("file")));
    }

    #[test]
    fn test_list_directory_with_sort() {
        let dir = setup_test_dir();
        let sort = SortSpec::by_name();
        let listing = list_directory(dir.path(), Some(&sort), None).unwrap();

        // Check directories come first (default behavior)
        let first_file_idx = listing
            .entries
            .iter()
            .position(|e| e.is_file())
            .unwrap_or(0);
        let first_dir_idx = listing
            .entries
            .iter()
            .position(|e| e.is_directory())
            .unwrap_or(usize::MAX);

        assert!(first_dir_idx < first_file_idx);
    }

    #[test]
    fn test_list_directory_not_found() {
        let result = list_directory("/nonexistent/path/12345", None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_not_found());
    }

    #[test]
    fn test_list_directory_not_a_directory() {
        let dir = setup_test_dir();
        let file_path = dir.path().join("file1.txt");
        let result = list_directory(&file_path, None, None);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ZError::NotADirectory { .. }));
    }

    #[test]
    fn test_entry_metadata() {
        let dir = setup_test_dir();
        let listing = list_directory(dir.path(), None, None).unwrap();

        // Find the large.bin file
        let large_file = listing.entries.iter().find(|e| e.name == "large.bin");
        assert!(large_file.is_some());
        let large_file = large_file.unwrap();

        assert!(large_file.is_file());
        assert_eq!(large_file.size, 10000);
        assert_eq!(large_file.extension, Some("bin".to_string()));
    }

    #[test]
    fn test_directory_stats() {
        let dir = setup_test_dir();
        let listing = list_directory(dir.path(), None, None).unwrap();

        assert_eq!(listing.dir_count, 1); // subdir
        assert_eq!(listing.file_count, 4); // file1.txt, file2.md, large.bin, .hidden
        assert!(listing.total_size > 0);
    }

    #[test]
    fn test_get_entry_meta() {
        let dir = setup_test_dir();
        let file_path = dir.path().join("file1.txt");

        let meta = get_entry_meta(&file_path).unwrap();

        assert_eq!(meta.name, "file1.txt");
        assert!(meta.is_file());
        assert_eq!(meta.size, 5); // "hello"
        assert_eq!(meta.extension, Some("txt".to_string()));
    }

    #[test]
    fn test_long_path_conversion() {
        let short = Path::new(r"C:\Users\test");
        assert!(!is_long_path(short));

        let long_path = to_long_path(short);
        #[cfg(windows)]
        assert!(long_path.to_string_lossy().starts_with(r"\\?\"));
    }

    #[test]
    fn test_extension_filter_integration() {
        let dir = setup_test_dir();
        let filter = FilterSpec::new().with_extension("txt");
        let listing = list_directory(dir.path(), None, Some(&filter)).unwrap();

        // Should only match file1.txt (and include the directory since ext filter doesn't affect dirs)
        let txt_files: Vec<_> = listing.entries.iter().filter(|e| e.is_file()).collect();
        assert_eq!(txt_files.len(), 1);
        assert_eq!(txt_files[0].name, "file1.txt");
    }
}
