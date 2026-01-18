//! Windows Recycle Bin operations using IFileOperation COM interface.
//!
//! This module provides safe wrappers around Windows Shell APIs for moving
//! files to the Recycle Bin, allowing users to recover deleted files.

use std::path::Path;
use tracing::debug;

use crate::{ZError, ZResult};

/// Move a file or directory to the Windows Recycle Bin.
///
/// This is the safe delete operation that allows recovery. For permanent
/// deletion, use `operations::delete_permanent()` instead.
///
/// # Arguments
/// * `path` - Path to move to Recycle Bin
///
/// # Errors
/// * `ZError::NotFound` - Path does not exist
/// * `ZError::PermissionDenied` - Insufficient permissions
/// * `ZError::Windows` - Windows API error
///
/// # Example
/// ```no_run
/// use zmanager_core::recycle::move_to_recycle_bin;
/// move_to_recycle_bin("unwanted_file.txt").unwrap();
/// ```
pub fn move_to_recycle_bin(path: impl AsRef<Path>) -> ZResult<()> {
    let path = path.as_ref();

    debug!(path = %path.display(), "Moving to Recycle Bin");

    if !path.exists() {
        return Err(ZError::NotFound {
            path: path.to_path_buf(),
        });
    }

    #[cfg(windows)]
    {
        move_to_recycle_bin_windows(path)?;
    }

    #[cfg(not(windows))]
    {
        // On non-Windows, we simulate by moving to a trash folder
        // This is just for testing - real usage is Windows-only
        let trash_dir = std::env::temp_dir().join(".zmanager_trash");
        std::fs::create_dir_all(&trash_dir).map_err(|e| ZError::io(&trash_dir, e))?;

        let dest = trash_dir.join(path.file_name().unwrap_or_default());
        std::fs::rename(path, &dest).map_err(|e| ZError::from_io(path, e))?;
    }

    debug!("Moved to Recycle Bin successfully");
    Ok(())
}

#[cfg(windows)]
fn move_to_recycle_bin_windows(path: &Path) -> ZResult<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    // We use SHFileOperationW for simplicity and broad compatibility
    // This is the classic Shell API that works on all Windows versions

    #[repr(C)]
    #[allow(non_snake_case, clippy::upper_case_acronyms)]
    struct SHFILEOPSTRUCTW {
        hwnd: *mut std::ffi::c_void,
        wFunc: u32,
        pFrom: *const u16,
        pTo: *const u16,
        fFlags: u16,
        fAnyOperationsAborted: i32,
        hNameMappings: *mut std::ffi::c_void,
        lpszProgressTitle: *const u16,
    }

    const FO_DELETE: u32 = 0x0003;
    const FOF_ALLOWUNDO: u16 = 0x0040; // Use Recycle Bin
    const FOF_NOCONFIRMATION: u16 = 0x0010; // Don't prompt user
    const FOF_NOERRORUI: u16 = 0x0400; // Don't show error UI
    const FOF_SILENT: u16 = 0x0004; // Don't show progress

    #[link(name = "shell32")]
    unsafe extern "system" {
        fn SHFileOperationW(lpFileOp: *mut SHFILEOPSTRUCTW) -> i32;
    }

    // Path must be double-null terminated for SHFileOperationW
    let wide_path: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .chain(std::iter::once(0))
        .collect();

    let mut file_op = SHFILEOPSTRUCTW {
        hwnd: ptr::null_mut(),
        wFunc: FO_DELETE,
        pFrom: wide_path.as_ptr(),
        pTo: ptr::null(),
        fFlags: FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_NOERRORUI | FOF_SILENT,
        fAnyOperationsAborted: 0,
        hNameMappings: ptr::null_mut(),
        lpszProgressTitle: ptr::null(),
    };

    let result = unsafe { SHFileOperationW(&mut file_op) };

    if result != 0 {
        return Err(ZError::Windows {
            code: result as u32,
            message: format!("SHFileOperationW failed with code {result}"),
        });
    }

    if file_op.fAnyOperationsAborted != 0 {
        return Err(ZError::Cancelled);
    }

    Ok(())
}

/// Move multiple files/directories to the Recycle Bin in a single operation.
///
/// This is more efficient than calling `move_to_recycle_bin()` repeatedly.
///
/// # Arguments
/// * `paths` - Iterator of paths to move to Recycle Bin
///
/// # Returns
/// A vector of results, one for each path (in order)
pub fn move_multiple_to_recycle_bin<I, P>(paths: I) -> Vec<ZResult<()>>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    paths
        .into_iter()
        .map(|p| move_to_recycle_bin(p.as_ref()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_recycle_bin_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("to_delete.txt");

        std::fs::write(&path, "content").unwrap();
        assert!(path.exists());

        move_to_recycle_bin(&path).unwrap();

        // File should no longer exist at original location
        assert!(!path.exists());
    }

    #[test]
    fn test_recycle_bin_directory() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("to_delete_dir");

        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("file.txt"), "content").unwrap();

        move_to_recycle_bin(&dir).unwrap();

        assert!(!dir.exists());
    }

    #[test]
    fn test_recycle_bin_not_found() {
        let temp = TempDir::new().unwrap();
        let result = move_to_recycle_bin(temp.path().join("nonexistent"));

        assert!(matches!(result, Err(ZError::NotFound { .. })));
    }

    #[test]
    fn test_move_multiple() {
        let temp = TempDir::new().unwrap();
        let file1 = temp.path().join("file1.txt");
        let file2 = temp.path().join("file2.txt");
        let nonexistent = temp.path().join("nonexistent");

        std::fs::write(&file1, "1").unwrap();
        std::fs::write(&file2, "2").unwrap();

        let results = move_multiple_to_recycle_bin([&file1, &file2, &nonexistent]);

        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(matches!(results[2], Err(ZError::NotFound { .. })));

        assert!(!file1.exists());
        assert!(!file2.exists());
    }
}
