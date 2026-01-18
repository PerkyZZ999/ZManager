//! File operations: rename, mkdir, open_default
//!
//! This module provides basic file system operations with proper error handling.

use std::path::Path;
use std::process::Command;
use tracing::debug;

use crate::{ZError, ZResult};

/// Rename or move a file/directory from one path to another.
///
/// # Arguments
/// * `from` - Source path
/// * `to` - Destination path
///
/// # Errors
/// * `ZError::NotFound` - Source does not exist
/// * `ZError::AlreadyExists` - Destination already exists
/// * `ZError::PermissionDenied` - Insufficient permissions
/// * `ZError::Io` - Other I/O errors
///
/// # Example
/// ```no_run
/// use zmanager_core::operations::rename;
/// rename("old_name.txt", "new_name.txt").unwrap();
/// ```
pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> ZResult<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    debug!(from = %from.display(), to = %to.display(), "Renaming");

    // Check source exists
    if !from.exists() {
        return Err(ZError::NotFound {
            path: from.to_path_buf(),
        });
    }

    // Check destination doesn't exist (conflict detection)
    if to.exists() {
        return Err(ZError::AlreadyExists {
            path: to.to_path_buf(),
        });
    }

    // Perform the rename
    std::fs::rename(from, to).map_err(|e| ZError::from_io(from, e))?;

    debug!("Rename successful");
    Ok(())
}

/// Create a new directory at the specified path.
///
/// Creates parent directories if they don't exist (like `mkdir -p`).
///
/// # Arguments
/// * `path` - Path where to create the directory
///
/// # Errors
/// * `ZError::AlreadyExists` - Path already exists
/// * `ZError::PermissionDenied` - Insufficient permissions
/// * `ZError::Io` - Other I/O errors
///
/// # Example
/// ```no_run
/// use zmanager_core::operations::mkdir;
/// mkdir("new_folder/subfolder").unwrap();
/// ```
pub fn mkdir(path: impl AsRef<Path>) -> ZResult<()> {
    let path = path.as_ref();

    debug!(path = %path.display(), "Creating directory");

    if path.exists() {
        return Err(ZError::AlreadyExists {
            path: path.to_path_buf(),
        });
    }

    std::fs::create_dir_all(path).map_err(|e| ZError::from_io(path, e))?;

    debug!("Directory created");
    Ok(())
}

/// Open a file or directory with its default application.
///
/// Uses Windows ShellExecute via the `explorer` command.
///
/// # Arguments
/// * `path` - Path to open
///
/// # Errors
/// * `ZError::NotFound` - Path does not exist
/// * `ZError::Io` - Failed to launch process
///
/// # Example
/// ```no_run
/// use zmanager_core::operations::open_default;
/// open_default("document.pdf").unwrap();
/// ```
pub fn open_default(path: impl AsRef<Path>) -> ZResult<()> {
    let path = path.as_ref();

    debug!(path = %path.display(), "Opening with default application");

    if !path.exists() {
        return Err(ZError::NotFound {
            path: path.to_path_buf(),
        });
    }

    // Use 'explorer' on Windows which triggers ShellExecute behavior
    #[cfg(windows)]
    {
        Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| ZError::io(path, e))?;
    }

    #[cfg(not(windows))]
    {
        // Fallback for non-Windows (for testing on other platforms)
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| ZError::io(path, e))?;
    }

    debug!("Opened successfully");
    Ok(())
}

/// Delete a file or directory permanently (bypasses Recycle Bin).
///
/// For Recycle Bin deletion, use `recycle::move_to_recycle_bin()` instead.
///
/// # Arguments
/// * `path` - Path to delete
/// * `recursive` - If true and path is a directory, delete contents recursively
///
/// # Errors
/// * `ZError::NotFound` - Path does not exist
/// * `ZError::PermissionDenied` - Insufficient permissions
/// * `ZError::DirectoryNotEmpty` - Directory has contents and recursive is false
/// * `ZError::Io` - Other I/O errors
pub fn delete_permanent(path: impl AsRef<Path>, recursive: bool) -> ZResult<()> {
    let path = path.as_ref();

    debug!(path = %path.display(), recursive, "Permanently deleting");

    if !path.exists() {
        return Err(ZError::NotFound {
            path: path.to_path_buf(),
        });
    }

    let result = if path.is_dir() {
        if recursive {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_dir(path)
        }
    } else {
        std::fs::remove_file(path)
    };

    result.map_err(|e| {
        // Special case for non-empty directory
        if e.kind() == std::io::ErrorKind::DirectoryNotEmpty
            || (e.kind() == std::io::ErrorKind::Other
                && e.to_string().contains("directory is not empty"))
        {
            ZError::DirectoryNotEmpty {
                path: path.to_path_buf(),
            }
        } else {
            ZError::from_io(path, e)
        }
    })?;

    debug!("Deleted permanently");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_rename_file() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("source.txt");
        let dst = temp.path().join("dest.txt");

        std::fs::write(&src, "content").unwrap();

        rename(&src, &dst).unwrap();

        assert!(!src.exists());
        assert!(dst.exists());
        assert_eq!(std::fs::read_to_string(&dst).unwrap(), "content");
    }

    #[test]
    fn test_rename_directory() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("old_dir");
        let dst = temp.path().join("new_dir");

        std::fs::create_dir(&src).unwrap();
        std::fs::write(src.join("file.txt"), "content").unwrap();

        rename(&src, &dst).unwrap();

        assert!(!src.exists());
        assert!(dst.exists());
        assert!(dst.join("file.txt").exists());
    }

    #[test]
    fn test_rename_not_found() {
        let temp = TempDir::new().unwrap();
        let result = rename(temp.path().join("nonexistent"), temp.path().join("dest"));

        assert!(matches!(result, Err(ZError::NotFound { .. })));
    }

    #[test]
    fn test_rename_conflict() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("source.txt");
        let dst = temp.path().join("existing.txt");

        std::fs::write(&src, "source").unwrap();
        std::fs::write(&dst, "existing").unwrap();

        let result = rename(&src, &dst);

        assert!(matches!(result, Err(ZError::AlreadyExists { .. })));
        // Source should still exist
        assert!(src.exists());
    }

    #[test]
    fn test_mkdir_simple() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("new_folder");

        mkdir(&path).unwrap();

        assert!(path.is_dir());
    }

    #[test]
    fn test_mkdir_nested() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("a/b/c/d");

        mkdir(&path).unwrap();

        assert!(path.is_dir());
    }

    #[test]
    fn test_mkdir_already_exists() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("existing");

        std::fs::create_dir(&path).unwrap();
        let result = mkdir(&path);

        assert!(matches!(result, Err(ZError::AlreadyExists { .. })));
    }

    #[test]
    fn test_delete_permanent_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("file.txt");

        std::fs::write(&path, "content").unwrap();
        delete_permanent(&path, false).unwrap();

        assert!(!path.exists());
    }

    #[test]
    fn test_delete_permanent_empty_dir() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("empty_dir");

        std::fs::create_dir(&path).unwrap();
        delete_permanent(&path, false).unwrap();

        assert!(!path.exists());
    }

    #[test]
    fn test_delete_permanent_recursive() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("parent");

        std::fs::create_dir_all(path.join("child")).unwrap();
        std::fs::write(path.join("file.txt"), "content").unwrap();
        std::fs::write(path.join("child/nested.txt"), "nested").unwrap();

        delete_permanent(&path, true).unwrap();

        assert!(!path.exists());
    }

    #[test]
    fn test_delete_permanent_non_empty_fails() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("non_empty");

        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("file.txt"), "content").unwrap();

        let result = delete_permanent(&path, false);

        assert!(matches!(result, Err(ZError::DirectoryNotEmpty { .. })));
        assert!(path.exists());
    }

    #[test]
    fn test_delete_permanent_not_found() {
        let temp = TempDir::new().unwrap();
        let result = delete_permanent(temp.path().join("nonexistent"), false);

        assert!(matches!(result, Err(ZError::NotFound { .. })));
    }

    // Note: open_default is not tested as it launches external processes
}
