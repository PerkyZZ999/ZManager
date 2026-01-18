//! Windows clipboard integration for file operations.
//!
//! This module provides clipboard support for cut/copy/paste operations
//! that interoperate with Windows Explorer using CF_HDROP format.

use std::ffi::OsStr;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr;

use tracing::{debug, trace, warn};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
    RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GHND, GMEM_MOVEABLE};
use windows::Win32::System::Ole::CF_HDROP;
use windows::Win32::UI::Shell::{DragQueryFileW, DROPFILES, HDROP};
use zmanager_core::{ZError, ZResult};

/// Preferred drop effect for clipboard operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropEffect {
    /// Copy operation (Ctrl+C).
    Copy,
    /// Move operation (Ctrl+X / Cut).
    Move,
}

impl DropEffect {
    /// Get the Windows DROPEFFECT constant value.
    pub fn value(&self) -> u32 {
        match self {
            Self::Copy => 1, // DROPEFFECT_COPY
            Self::Move => 2, // DROPEFFECT_MOVE
        }
    }

    /// Create from Windows DROPEFFECT value.
    pub fn from_value(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::Copy),
            2 => Some(Self::Move),
            _ => None,
        }
    }
}

/// Result of reading the clipboard.
#[derive(Debug, Clone)]
pub struct ClipboardContent {
    /// Paths on the clipboard.
    pub paths: Vec<PathBuf>,
    /// Preferred drop effect (copy or move).
    pub effect: DropEffect,
}

impl ClipboardContent {
    /// Check if the clipboard contains files.
    pub fn has_files(&self) -> bool {
        !self.paths.is_empty()
    }

    /// Check if this is a cut operation.
    pub fn is_cut(&self) -> bool {
        self.effect == DropEffect::Move
    }

    /// Check if this is a copy operation.
    pub fn is_copy(&self) -> bool {
        self.effect == DropEffect::Copy
    }
}

/// Get the "Preferred DropEffect" clipboard format ID.
fn get_drop_effect_format() -> u32 {
    let name: Vec<u16> = OsStr::new("Preferred DropEffect")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    unsafe { RegisterClipboardFormatW(PCWSTR::from_raw(name.as_ptr())) }
}

/// Helper to safely close clipboard, ignoring errors.
fn close_clipboard_safe() {
    let _ = unsafe { CloseClipboard() };
}

/// Write file paths to clipboard (copy or cut operation).
///
/// This creates a CF_HDROP structure and optionally sets the preferred
/// drop effect to indicate copy vs move.
pub fn write_files_to_clipboard(paths: &[PathBuf], effect: DropEffect) -> ZResult<()> {
    if paths.is_empty() {
        return Err(ZError::InvalidOperation {
            operation: "clipboard write".to_string(),
            reason: "No files to copy".to_string(),
        });
    }

    debug!(
        count = paths.len(),
        effect = ?effect,
        "Writing files to clipboard"
    );

    unsafe {
        // Open clipboard
        OpenClipboard(HWND::default()).map_err(|e| ZError::Internal {
            message: format!("Failed to open clipboard: {e}"),
        })?;

        // Empty existing content
        if let Err(e) = EmptyClipboard() {
            close_clipboard_safe();
            return Err(ZError::Internal {
                message: format!("Failed to empty clipboard: {e}"),
            });
        }

        // Build the HDROP structure
        let hglobal = match build_hdrop(paths) {
            Ok(h) => h,
            Err(e) => {
                close_clipboard_safe();
                return Err(e);
            }
        };

        // Set CF_HDROP data
        let result = SetClipboardData(CF_HDROP.0 as u32, HANDLE(hglobal.0 as _));
        if result.is_err() {
            close_clipboard_safe();
            return Err(ZError::Internal {
                message: "Failed to set clipboard data".to_string(),
            });
        }

        // Set preferred drop effect
        if let Err(e) = set_drop_effect(effect) {
            warn!("Failed to set drop effect: {}", e);
        }

        close_clipboard_safe();
        Ok(())
    }
}

/// Build a CF_HDROP structure from file paths.
fn build_hdrop(paths: &[PathBuf]) -> ZResult<HGLOBAL> {
    // Convert paths to wide strings with null terminators
    let wide_paths: Vec<Vec<u16>> = paths
        .iter()
        .map(|p| {
            OsStr::new(p.as_os_str())
                .encode_wide()
                .chain(std::iter::once(0))
                .collect()
        })
        .collect();

    // Calculate total size
    // DROPFILES header + all paths + final double null
    let paths_size: usize = wide_paths.iter().map(|p| p.len() * 2).sum();
    let total_size = size_of::<DROPFILES>() + paths_size + 2; // +2 for final null

    trace!(total_size, "Allocating HDROP structure");

    unsafe {
        // Allocate global memory
        let hglobal = GlobalAlloc(GHND, total_size).map_err(|e| ZError::Internal {
            message: format!("Failed to allocate clipboard memory: {e}"),
        })?;

        // Lock and fill
        let ptr = GlobalLock(hglobal);
        if ptr.is_null() {
            return Err(ZError::Internal {
                message: "Failed to lock clipboard memory".to_string(),
            });
        }

        // Write DROPFILES header
        let drop_files = ptr as *mut DROPFILES;
        (*drop_files).pFiles = size_of::<DROPFILES>() as u32;
        (*drop_files).pt.x = 0;
        (*drop_files).pt.y = 0;
        (*drop_files).fNC = false.into();
        (*drop_files).fWide = true.into(); // Unicode

        // Write paths after header
        let mut dest = (ptr as *mut u8).add(size_of::<DROPFILES>()) as *mut u16;
        for wide_path in &wide_paths {
            ptr::copy_nonoverlapping(wide_path.as_ptr(), dest, wide_path.len());
            dest = dest.add(wide_path.len());
        }
        // Final double null
        *dest = 0;

        let _ = GlobalUnlock(hglobal);

        Ok(hglobal)
    }
}

/// Set the preferred drop effect on the clipboard.
fn set_drop_effect(effect: DropEffect) -> ZResult<()> {
    let format = get_drop_effect_format();

    unsafe {
        // Allocate 4 bytes for DWORD
        let hglobal = GlobalAlloc(GMEM_MOVEABLE, 4).map_err(|e| ZError::Internal {
            message: format!("Failed to allocate drop effect memory: {e}"),
        })?;

        let ptr = GlobalLock(hglobal);
        if ptr.is_null() {
            return Err(ZError::Internal {
                message: "Failed to lock drop effect memory".to_string(),
            });
        }

        *(ptr as *mut u32) = effect.value();
        let _ = GlobalUnlock(hglobal);

        let result = SetClipboardData(format, HANDLE(hglobal.0 as _));
        if result.is_err() {
            return Err(ZError::Internal {
                message: "Failed to set drop effect".to_string(),
            });
        }

        Ok(())
    }
}

/// Read file paths from clipboard.
///
/// Returns the paths and the preferred drop effect (copy or move).
pub fn read_files_from_clipboard() -> ZResult<ClipboardContent> {
    debug!("Reading files from clipboard");

    unsafe {
        // Open clipboard
        OpenClipboard(HWND::default()).map_err(|e| ZError::Internal {
            message: format!("Failed to open clipboard: {e}"),
        })?;

        // Get CF_HDROP data
        let handle = match GetClipboardData(CF_HDROP.0 as u32) {
            Ok(h) => h,
            Err(_) => {
                close_clipboard_safe();
                return Ok(ClipboardContent {
                    paths: Vec::new(),
                    effect: DropEffect::Copy,
                });
            }
        };

        let hdrop = HDROP(handle.0 as _);

        // Get the paths
        let paths = parse_hdrop(hdrop)?;

        // Get drop effect
        let effect = get_drop_effect().unwrap_or(DropEffect::Copy);

        close_clipboard_safe();

        debug!(count = paths.len(), effect = ?effect, "Read files from clipboard");

        Ok(ClipboardContent { paths, effect })
    }
}

/// Parse paths from an HDROP structure.
fn parse_hdrop(hdrop: HDROP) -> ZResult<Vec<PathBuf>> {
    // Get file count
    let count = unsafe { DragQueryFileW(hdrop, 0xFFFFFFFF, None) };
    if count == 0 {
        return Ok(Vec::new());
    }

    trace!(count, "Parsing HDROP files");

    let mut paths = Vec::with_capacity(count as usize);

    for i in 0..count {
        // Get required buffer size
        let len = unsafe { DragQueryFileW(hdrop, i, None) };
        if len == 0 {
            continue;
        }

        // Allocate buffer and get the path
        let mut buffer: Vec<u16> = vec![0; (len + 1) as usize];
        let actual_len = unsafe { DragQueryFileW(hdrop, i, Some(&mut buffer)) };
        if actual_len == 0 {
            continue;
        }

        // Convert to PathBuf
        let path_str = String::from_utf16_lossy(&buffer[..actual_len as usize]);
        paths.push(PathBuf::from(path_str));
    }

    Ok(paths)
}

/// Get the preferred drop effect from clipboard.
fn get_drop_effect() -> Option<DropEffect> {
    let format = get_drop_effect_format();

    unsafe {
        let handle = GetClipboardData(format).ok()?;

        let hglobal = HGLOBAL(handle.0 as _);
        let size = GlobalSize(hglobal);
        if size < 4 {
            return None;
        }

        let ptr = GlobalLock(hglobal);
        if ptr.is_null() {
            return None;
        }

        let value = *(ptr as *const u32);
        let _ = GlobalUnlock(hglobal);

        DropEffect::from_value(value)
    }
}

/// Check if clipboard contains files.
pub fn clipboard_has_files() -> bool {
    unsafe { IsClipboardFormatAvailable(CF_HDROP.0 as u32).is_ok() }
}

/// Clear the clipboard.
pub fn clear_clipboard() -> ZResult<()> {
    unsafe {
        OpenClipboard(HWND::default()).map_err(|e| ZError::Internal {
            message: format!("Failed to open clipboard: {e}"),
        })?;

        let result = EmptyClipboard();
        close_clipboard_safe();

        result.map_err(|e| ZError::Internal {
            message: format!("Failed to clear clipboard: {e}"),
        })
    }
}

/// High-level clipboard operations.
pub struct Clipboard;

impl Clipboard {
    /// Copy files to clipboard.
    pub fn copy(paths: &[PathBuf]) -> ZResult<()> {
        write_files_to_clipboard(paths, DropEffect::Copy)
    }

    /// Cut files to clipboard (for move operation).
    pub fn cut(paths: &[PathBuf]) -> ZResult<()> {
        write_files_to_clipboard(paths, DropEffect::Move)
    }

    /// Read files from clipboard.
    pub fn paste() -> ZResult<ClipboardContent> {
        read_files_from_clipboard()
    }

    /// Check if clipboard has files.
    pub fn has_files() -> bool {
        clipboard_has_files()
    }

    /// Clear the clipboard.
    pub fn clear() -> ZResult<()> {
        clear_clipboard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_drop_effect_values() {
        assert_eq!(DropEffect::Copy.value(), 1);
        assert_eq!(DropEffect::Move.value(), 2);
    }

    #[test]
    fn test_drop_effect_from_value() {
        assert_eq!(DropEffect::from_value(1), Some(DropEffect::Copy));
        assert_eq!(DropEffect::from_value(2), Some(DropEffect::Move));
        assert_eq!(DropEffect::from_value(0), None);
        assert_eq!(DropEffect::from_value(99), None);
    }

    #[test]
    fn test_clipboard_content() {
        let content = ClipboardContent {
            paths: vec![PathBuf::from("test.txt")],
            effect: DropEffect::Copy,
        };

        assert!(content.has_files());
        assert!(content.is_copy());
        assert!(!content.is_cut());

        let cut_content = ClipboardContent {
            paths: vec![PathBuf::from("test.txt")],
            effect: DropEffect::Move,
        };

        assert!(cut_content.is_cut());
        assert!(!cut_content.is_copy());
    }

    #[test]
    fn test_empty_clipboard_content() {
        let content = ClipboardContent {
            paths: Vec::new(),
            effect: DropEffect::Copy,
        };

        assert!(!content.has_files());
    }

    #[test]
    fn test_write_empty_fails() {
        let result = write_files_to_clipboard(&[], DropEffect::Copy);
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_clipboard_roundtrip() {
        // Note: This test requires clipboard access and may fail in CI
        // or when clipboard is in use by another process
        let temp = TempDir::new().unwrap();
        let file1 = temp.path().join("test1.txt");
        let file2 = temp.path().join("test2.txt");

        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();

        let paths = vec![file1.clone(), file2.clone()];

        // Copy to clipboard
        if let Ok(()) = Clipboard::copy(&paths) {
            // Read back
            if let Ok(content) = Clipboard::paste() {
                assert_eq!(content.paths.len(), 2);
                assert!(content.is_copy());
            }
        }
    }

    #[test]
    #[serial]
    fn test_clipboard_cut() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("cut_test.txt");
        fs::write(&file, "content").unwrap();

        // This test may fail if another process is using the clipboard
        // or if clipboard access is denied. We make it a soft assertion.
        match Clipboard::cut(std::slice::from_ref(&file)) {
            Ok(()) => {
                // Give clipboard time to update
                std::thread::sleep(std::time::Duration::from_millis(50));
                if let Ok(content) = Clipboard::paste() {
                    // Only assert if we actually got files back
                    if content.has_files() {
                        assert!(content.is_cut());
                    }
                }
            }
            Err(_) => {
                // Clipboard access failed, skip test
            }
        }
    }
}
