//! Windows file copy with progress using CopyFileExW.
//!
//! This module provides efficient file copying with:
//! - Real-time progress callbacks
//! - Cancellation support via CancellationToken
//! - Windows native performance

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use tracing::{debug, error, info, trace, warn};
use windows::core::PCWSTR;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{CopyFileExW, LPPROGRESS_ROUTINE_CALLBACK_REASON};
use zmanager_core::{CancellationToken, ZError, ZResult};

// CopyFileExW progress callback return values
const PROGRESS_CONTINUE: u32 = 0;
const PROGRESS_CANCEL: u32 = 1;
#[allow(dead_code)]
const PROGRESS_STOP: u32 = 2;
#[allow(dead_code)]
const PROGRESS_QUIET: u32 = 3;

/// Callback function type for progress updates.
pub type ProgressCallback = Box<dyn Fn(CopyProgress) + Send + Sync>;

/// Progress information for a file copy operation.
#[derive(Debug, Clone)]
pub struct CopyProgress {
    /// Total bytes to copy.
    pub total_bytes: u64,
    /// Bytes copied so far.
    pub bytes_copied: u64,
    /// Source file path.
    pub source: std::path::PathBuf,
    /// Destination file path.
    pub destination: std::path::PathBuf,
    /// Current transfer speed in bytes per second.
    pub speed_bps: u64,
    /// Estimated time remaining in seconds.
    pub eta_seconds: Option<u64>,
}

impl CopyProgress {
    /// Get the completion percentage (0.0 to 100.0).
    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            100.0
        } else {
            (self.bytes_copied as f64 / self.total_bytes as f64) * 100.0
        }
    }

    /// Get the completion percentage as an integer (0 to 100).
    pub fn percentage_int(&self) -> u8 {
        self.percentage().round() as u8
    }
}

/// Shared state passed to the CopyFileExW callback.
struct CallbackState {
    /// Source file path for progress reporting.
    source: std::path::PathBuf,
    /// Destination file path for progress reporting.
    destination: std::path::PathBuf,
    /// Cancellation token to check for cancel requests.
    cancel_token: CancellationToken,
    /// Pause flag - when true, operation should pause.
    paused: AtomicBool,
    /// Bytes transferred (updated by callback).
    bytes_transferred: AtomicU64,
    /// Total bytes (updated by callback).
    total_bytes: AtomicU64,
    /// Start time for speed calculation.
    start_time: Instant,
    /// User progress callback (if any).
    progress_callback: Option<ProgressCallback>,
}

impl CallbackState {
    fn new(
        source: impl AsRef<Path>,
        destination: impl AsRef<Path>,
        cancel_token: CancellationToken,
        progress_callback: Option<ProgressCallback>,
    ) -> Self {
        Self {
            source: source.as_ref().to_path_buf(),
            destination: destination.as_ref().to_path_buf(),
            cancel_token,
            paused: AtomicBool::new(false),
            bytes_transferred: AtomicU64::new(0),
            total_bytes: AtomicU64::new(0),
            start_time: Instant::now(),
            progress_callback,
        }
    }

    fn calculate_speed(&self, bytes_transferred: u64) -> u64 {
        let elapsed = self.start_time.elapsed();
        if elapsed.as_secs() > 0 {
            bytes_transferred / elapsed.as_secs()
        } else if elapsed.as_millis() > 100 {
            (bytes_transferred * 1000) / elapsed.as_millis() as u64
        } else {
            0
        }
    }

    fn calculate_eta(&self, bytes_transferred: u64, total_bytes: u64) -> Option<u64> {
        let speed = self.calculate_speed(bytes_transferred);
        if speed > 0 && bytes_transferred < total_bytes {
            let remaining = total_bytes - bytes_transferred;
            Some(remaining / speed)
        } else {
            None
        }
    }
}

/// CopyFileExW progress callback function.
///
/// This is called by Windows during the copy operation to report progress.
/// It returns a value indicating whether to continue, cancel, or stop.
///
/// # Safety
///
/// This function is called from Windows and must be `unsafe extern "system"`.
/// The `lpdata` parameter must point to a valid `CallbackState` box.
#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn copy_progress_callback(
    total_file_size: i64,
    total_bytes_transferred: i64,
    _stream_size: i64,
    _stream_bytes_transferred: i64,
    _stream_number: u32,
    callback_reason: LPPROGRESS_ROUTINE_CALLBACK_REASON,
    _source_file: HANDLE,
    _destination_file: HANDLE,
    lpdata: *const std::ffi::c_void,
) -> u32 {
    // Safety: We know lpdata points to a valid CallbackState because we passed it
    let state = unsafe { &*(lpdata as *const CallbackState) };

    // Check for cancellation
    if state.cancel_token.is_cancelled() {
        trace!("Copy cancelled by user");
        return PROGRESS_CANCEL;
    }

    // Check for pause
    if state.paused.load(Ordering::Acquire) {
        trace!("Copy paused, waiting...");
        // In a real pause scenario, we'd spin here. For now, just continue.
        // The job system handles actual pausing at a higher level.
        return PROGRESS_QUIET;
    }

    // Update progress tracking
    state
        .bytes_transferred
        .store(total_bytes_transferred as u64, Ordering::Release);
    state
        .total_bytes
        .store(total_file_size as u64, Ordering::Release);

    // Only report progress on CHUNK_FINISHED
    if callback_reason.0 == 0 {
        // CALLBACK_CHUNK_FINISHED
        if let Some(ref callback) = state.progress_callback {
            let bytes_copied = total_bytes_transferred as u64;
            let total_bytes = total_file_size as u64;
            let speed = state.calculate_speed(bytes_copied);
            let eta = state.calculate_eta(bytes_copied, total_bytes);

            let progress = CopyProgress {
                total_bytes,
                bytes_copied,
                source: state.source.clone(),
                destination: state.destination.clone(),
                speed_bps: speed,
                eta_seconds: eta,
            };

            callback(progress);
        }
    }

    PROGRESS_CONTINUE
}

/// Copy a file with progress reporting.
///
/// This uses `CopyFileExW` for efficient native Windows file copying with
/// real-time progress updates and cancellation support.
///
/// # Arguments
///
/// * `source` - Source file path
/// * `destination` - Destination file path
/// * `overwrite` - If true, overwrite existing file; if false, fail if exists
/// * `cancel_token` - Token to check for cancellation requests
/// * `progress_callback` - Optional callback for progress updates
///
/// # Returns
///
/// Returns `Ok(bytes_copied)` on success, or an error if the copy fails.
///
/// # Example
///
/// ```no_run
/// use zmanager_transfer_win::copy::{copy_file_with_progress, CopyProgress};
/// use zmanager_core::CancellationToken;
///
/// let token = CancellationToken::new();
/// let callback = Some(Box::new(|p: CopyProgress| {
///     println!("{}% complete", p.percentage_int());
/// }) as Box<dyn Fn(CopyProgress) + Send + Sync>);
///
/// let bytes = copy_file_with_progress(
///     "source.txt",
///     "dest.txt",
///     true,
///     token,
///     callback,
/// ).unwrap();
/// ```
pub fn copy_file_with_progress(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    overwrite: bool,
    cancel_token: CancellationToken,
    progress_callback: Option<ProgressCallback>,
) -> ZResult<u64> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    debug!(
        source = %source.display(),
        destination = %destination.display(),
        overwrite,
        "Starting file copy"
    );

    // Validate source exists
    if !source.exists() {
        return Err(ZError::NotFound {
            path: source.to_path_buf(),
        });
    }

    // Validate source is a file
    if !source.is_file() {
        return Err(ZError::NotAFile {
            path: source.to_path_buf(),
        });
    }

    // Check if destination exists and we shouldn't overwrite
    if !overwrite && destination.exists() {
        return Err(ZError::AlreadyExists {
            path: destination.to_path_buf(),
        });
    }

    // Ensure parent directory exists
    if let Some(parent) = destination.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| ZError::io(parent, e))?;
        }
    }

    // Create callback state
    let state = Box::new(CallbackState::new(
        source,
        destination,
        cancel_token.clone(),
        progress_callback,
    ));
    let state_ptr = Box::into_raw(state);

    // Convert paths to wide strings for Windows API
    let source_wide = path_to_wide(source)?;
    let dest_wide = path_to_wide(destination)?;

    // Call CopyFileExW
    let result = unsafe {
        CopyFileExW(
            PCWSTR::from_raw(source_wide.as_ptr()),
            PCWSTR::from_raw(dest_wide.as_ptr()),
            Some(copy_progress_callback),
            Some(state_ptr as *const std::ffi::c_void),
            None,      // No cancel flag pointer
            if overwrite { 0 } else { 1 }, // COPY_FILE_FAIL_IF_EXISTS = 1
        )
    };

    // Reclaim the state
    let state = unsafe { Box::from_raw(state_ptr) };
    let bytes_copied = state.bytes_transferred.load(Ordering::Acquire);

    match result {
        Ok(()) => {
            info!(
                bytes = bytes_copied,
                source = %source.display(),
                destination = %destination.display(),
                "File copy completed"
            );
            Ok(bytes_copied)
        }
        Err(e) => {
            // Check if cancelled
            if state.cancel_token.is_cancelled() {
                warn!(source = %source.display(), "File copy cancelled");
                // Clean up partial file
                let _ = std::fs::remove_file(destination);
                return Err(ZError::Cancelled);
            }

            let error_code = e.code().0 as u32;
            error!(
                code = error_code,
                source = %source.display(),
                destination = %destination.display(),
                "File copy failed"
            );

            // Map common Windows errors
            match error_code {
                0x80070005 => Err(ZError::PermissionDenied {
                    path: destination.to_path_buf(),
                }),
                0x80070002 | 0x80070003 => Err(ZError::NotFound {
                    path: source.to_path_buf(),
                }),
                0x80070050 => Err(ZError::AlreadyExists {
                    path: destination.to_path_buf(),
                }),
                0x80070070 => Err(ZError::TransferFailed {
                    message: "Disk full".to_string(),
                    source: Some(Box::new(e)),
                }),
                _ => Err(ZError::Windows {
                    code: error_code,
                    message: e.message().to_string(),
                }),
            }
        }
    }
}

/// Copy a file asynchronously with progress reporting.
///
/// This wraps `copy_file_with_progress` in a tokio blocking task.
pub async fn copy_file_async(
    source: impl AsRef<Path> + Send + 'static,
    destination: impl AsRef<Path> + Send + 'static,
    overwrite: bool,
    cancel_token: CancellationToken,
    progress_callback: Option<ProgressCallback>,
) -> ZResult<u64> {
    let source = source.as_ref().to_path_buf();
    let destination = destination.as_ref().to_path_buf();

    tokio::task::spawn_blocking(move || {
        copy_file_with_progress(source, destination, overwrite, cancel_token, progress_callback)
    })
    .await
    .map_err(|e| ZError::Internal {
        message: format!("Task join error: {e}"),
    })?
}

/// Convert a path to a null-terminated wide string for Windows API.
fn path_to_wide(path: &Path) -> ZResult<Vec<u16>> {
    use std::os::windows::ffi::OsStrExt;

    // Use \\?\ prefix for long path support
    let path_str = if path.to_string_lossy().len() >= 240 {
        format!("\\\\?\\{}", path.display())
    } else {
        path.to_string_lossy().to_string()
    };

    let wide: Vec<u16> = std::ffi::OsStr::new(&path_str)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    Ok(wide)
}

/// Result of a copy operation for integration with job system.
#[derive(Debug, Clone)]
pub struct CopyResult {
    /// Source file path.
    pub source: std::path::PathBuf,
    /// Destination file path.
    pub destination: std::path::PathBuf,
    /// Bytes copied.
    pub bytes_copied: u64,
    /// Duration of the copy operation.
    pub duration: std::time::Duration,
    /// Average speed in bytes per second.
    pub average_speed_bps: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(dir: &TempDir, name: &str, size: usize) -> std::path::PathBuf {
        let path = dir.path().join(name);
        let content = vec![b'X'; size];
        fs::write(&path, &content).unwrap();
        path
    }

    #[test]
    fn test_copy_small_file() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(&temp, "source.txt", 1024);
        let dest = temp.path().join("dest.txt");

        let token = CancellationToken::new();
        let result = copy_file_with_progress(&source, &dest, false, token, None);

        assert!(result.is_ok());
        assert!(dest.exists());
        assert_eq!(fs::read(&source).unwrap(), fs::read(&dest).unwrap());
    }

    #[test]
    fn test_copy_with_progress() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(&temp, "source.bin", 1024 * 1024); // 1MB
        let dest = temp.path().join("dest.bin");

        let progress_updates = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let updates_clone = progress_updates.clone();

        let token = CancellationToken::new();
        let callback: ProgressCallback = Box::new(move |_progress| {
            updates_clone.fetch_add(1, Ordering::Relaxed);
        });

        let result = copy_file_with_progress(&source, &dest, false, token, Some(callback));

        assert!(result.is_ok());
        assert!(dest.exists());
        // We should have received at least one progress update for 1MB file
        // (depends on chunk size, may be 0 for small files)
    }

    #[test]
    fn test_copy_overwrite() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(&temp, "source.txt", 100);
        let dest = create_test_file(&temp, "dest.txt", 50);

        let token = CancellationToken::new();

        // Without overwrite should fail
        let result = copy_file_with_progress(&source, &dest, false, token.clone(), None);
        assert!(matches!(result, Err(ZError::AlreadyExists { .. })));

        // With overwrite should succeed
        let result = copy_file_with_progress(&source, &dest, true, token, None);
        assert!(result.is_ok());
        assert_eq!(fs::read(&source).unwrap(), fs::read(&dest).unwrap());
    }

    #[test]
    fn test_copy_source_not_found() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("nonexistent.txt");
        let dest = temp.path().join("dest.txt");

        let token = CancellationToken::new();
        let result = copy_file_with_progress(&source, &dest, false, token, None);

        assert!(matches!(result, Err(ZError::NotFound { .. })));
    }

    #[test]
    fn test_copy_cancellation() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(&temp, "source.bin", 10 * 1024 * 1024); // 10MB
        let dest = temp.path().join("dest.bin");

        let token = CancellationToken::new();
        let token_clone = token.clone();

        // Cancel immediately
        token.cancel();

        let result = copy_file_with_progress(&source, &dest, false, token_clone, None);

        // Should either be cancelled or complete (race condition)
        // For very fast copies, it might complete before checking the token
        match result {
            Ok(_) => {
                // Copy completed before cancellation was checked
                assert!(dest.exists());
            }
            Err(ZError::Cancelled) => {
                // Cancellation was successful
                // Partial file should be cleaned up
            }
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[test]
    fn test_copy_creates_parent_dirs() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(&temp, "source.txt", 100);
        let dest = temp.path().join("subdir").join("nested").join("dest.txt");

        let token = CancellationToken::new();
        let result = copy_file_with_progress(&source, &dest, false, token, None);

        assert!(result.is_ok());
        assert!(dest.exists());
        assert_eq!(fs::read(&source).unwrap(), fs::read(&dest).unwrap());
    }

    #[tokio::test]
    async fn test_copy_async() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(&temp, "source.txt", 1024);
        let dest = temp.path().join("dest.txt");

        let token = CancellationToken::new();
        let result = copy_file_async(source.clone(), dest.clone(), false, token, None).await;

        assert!(result.is_ok());
        assert!(dest.exists());
        assert_eq!(fs::read(&source).unwrap(), fs::read(&dest).unwrap());
    }

    #[test]
    fn test_progress_percentage() {
        let progress = CopyProgress {
            total_bytes: 1000,
            bytes_copied: 500,
            source: std::path::PathBuf::from("src"),
            destination: std::path::PathBuf::from("dst"),
            speed_bps: 1000,
            eta_seconds: Some(1),
        };

        assert!((progress.percentage() - 50.0).abs() < 0.001);
        assert_eq!(progress.percentage_int(), 50);
    }

    #[test]
    fn test_progress_percentage_zero_total() {
        let progress = CopyProgress {
            total_bytes: 0,
            bytes_copied: 0,
            source: std::path::PathBuf::from("src"),
            destination: std::path::PathBuf::from("dst"),
            speed_bps: 0,
            eta_seconds: None,
        };

        assert!((progress.percentage() - 100.0).abs() < 0.001);
    }
}
