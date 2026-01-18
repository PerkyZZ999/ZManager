//! Drive enumeration for Windows.
//!
//! This module provides functionality to list available drives
//! with their labels, types, and free space.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::ZResult;

/// Type of drive/volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriveType {
    /// Unknown drive type.
    Unknown,
    /// Drive has no root directory (not mounted).
    NoRootDir,
    /// Removable drive (USB, SD card, etc.).
    Removable,
    /// Fixed drive (internal HDD/SSD).
    Fixed,
    /// Network drive.
    Network,
    /// CD/DVD drive.
    CdRom,
    /// RAM disk.
    RamDisk,
}

impl DriveType {
    /// Get a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::NoRootDir => "Not Mounted",
            Self::Removable => "Removable",
            Self::Fixed => "Local Disk",
            Self::Network => "Network Drive",
            Self::CdRom => "CD/DVD Drive",
            Self::RamDisk => "RAM Disk",
        }
    }

    /// Get an icon name for the drive type.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Unknown => "drive",
            Self::NoRootDir => "drive",
            Self::Removable => "usb",
            Self::Fixed => "hard-drive",
            Self::Network => "network",
            Self::CdRom => "disc",
            Self::RamDisk => "memory",
        }
    }
}

/// Information about a drive/volume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    /// The drive letter or mount point (e.g., "C:\\").
    pub path: PathBuf,
    /// Volume label (e.g., "Windows", "Data").
    pub label: String,
    /// Type of drive.
    pub drive_type: DriveType,
    /// File system (e.g., "NTFS", "FAT32").
    pub file_system: Option<String>,
    /// Total capacity in bytes.
    pub total_bytes: Option<u64>,
    /// Free space in bytes.
    pub free_bytes: Option<u64>,
    /// Whether the drive is ready/accessible.
    pub is_ready: bool,
}

impl DriveInfo {
    /// Get the display name for the drive.
    pub fn display_name(&self) -> String {
        let letter = self
            .path
            .to_str()
            .unwrap_or("")
            .trim_end_matches('\\')
            .to_string();

        if self.label.is_empty() {
            format!("{} ({})", self.drive_type.description(), letter)
        } else {
            format!("{} ({})", self.label, letter)
        }
    }

    /// Get used space in bytes.
    pub fn used_bytes(&self) -> Option<u64> {
        match (self.total_bytes, self.free_bytes) {
            (Some(total), Some(free)) => Some(total.saturating_sub(free)),
            _ => None,
        }
    }

    /// Get usage percentage (0.0 to 1.0).
    pub fn usage_percent(&self) -> Option<f64> {
        match (self.total_bytes, self.free_bytes) {
            (Some(total), Some(free)) if total > 0 => {
                Some((total.saturating_sub(free)) as f64 / total as f64)
            }
            _ => None,
        }
    }

    /// Format free space as human-readable string.
    pub fn free_space_display(&self) -> String {
        self.free_bytes
            .map(format_bytes)
            .unwrap_or_else(|| "N/A".to_string())
    }

    /// Format total space as human-readable string.
    pub fn total_space_display(&self) -> String {
        self.total_bytes
            .map(format_bytes)
            .unwrap_or_else(|| "N/A".to_string())
    }
}

/// Format bytes as human-readable string.
fn format_bytes(bytes: u64) -> String {
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

/// List all available drives on the system.
#[cfg(windows)]
pub fn list_drives() -> ZResult<Vec<DriveInfo>> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    debug!("Enumerating drives");

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetLogicalDrives() -> u32;
        fn GetDriveTypeW(lpRootPathName: *const u16) -> u32;
        fn GetVolumeInformationW(
            lpRootPathName: *const u16,
            lpVolumeNameBuffer: *mut u16,
            nVolumeNameSize: u32,
            lpVolumeSerialNumber: *mut u32,
            lpMaximumComponentLength: *mut u32,
            lpFileSystemFlags: *mut u32,
            lpFileSystemNameBuffer: *mut u16,
            nFileSystemNameSize: u32,
        ) -> i32;
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }

    const DRIVE_UNKNOWN: u32 = 0;
    const DRIVE_NO_ROOT_DIR: u32 = 1;
    const DRIVE_REMOVABLE: u32 = 2;
    const DRIVE_FIXED: u32 = 3;
    const DRIVE_REMOTE: u32 = 4;
    const DRIVE_CDROM: u32 = 5;
    const DRIVE_RAMDISK: u32 = 6;

    let bitmask = unsafe { GetLogicalDrives() };

    let mut drives = Vec::new();

    for i in 0..26u8 {
        if (bitmask & (1 << i)) != 0 {
            let letter = (b'A' + i) as char;
            let root = format!("{}:\\", letter);
            let root_wide: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();

            // Get drive type
            let drive_type_raw = unsafe { GetDriveTypeW(root_wide.as_ptr()) };
            let drive_type = match drive_type_raw {
                DRIVE_UNKNOWN => DriveType::Unknown,
                DRIVE_NO_ROOT_DIR => DriveType::NoRootDir,
                DRIVE_REMOVABLE => DriveType::Removable,
                DRIVE_FIXED => DriveType::Fixed,
                DRIVE_REMOTE => DriveType::Network,
                DRIVE_CDROM => DriveType::CdRom,
                DRIVE_RAMDISK => DriveType::RamDisk,
                _ => DriveType::Unknown,
            };

            // Skip unmounted drives
            if drive_type == DriveType::NoRootDir {
                continue;
            }

            // Get volume info
            let mut label_buf = [0u16; 256];
            let mut fs_buf = [0u16; 256];
            let mut serial = 0u32;
            let mut max_component = 0u32;
            let mut fs_flags = 0u32;

            let volume_result = unsafe {
                GetVolumeInformationW(
                    root_wide.as_ptr(),
                    label_buf.as_mut_ptr(),
                    label_buf.len() as u32,
                    &mut serial,
                    &mut max_component,
                    &mut fs_flags,
                    fs_buf.as_mut_ptr(),
                    fs_buf.len() as u32,
                )
            };

            let (label, file_system, is_ready) = if volume_result != 0 {
                let label_end = label_buf.iter().position(|&c| c == 0).unwrap_or(0);
                let fs_end = fs_buf.iter().position(|&c| c == 0).unwrap_or(0);

                let label = OsString::from_wide(&label_buf[..label_end])
                    .to_string_lossy()
                    .to_string();
                let file_system = OsString::from_wide(&fs_buf[..fs_end])
                    .to_string_lossy()
                    .to_string();

                (label, Some(file_system), true)
            } else {
                (String::new(), None, false)
            };

            // Get disk space
            let (total_bytes, free_bytes) = if is_ready {
                let mut free_caller = 0u64;
                let mut total = 0u64;
                let mut free = 0u64;

                let space_result = unsafe {
                    GetDiskFreeSpaceExW(
                        root_wide.as_ptr(),
                        &mut free_caller,
                        &mut total,
                        &mut free,
                    )
                };

                if space_result != 0 {
                    (Some(total), Some(free))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            drives.push(DriveInfo {
                path: PathBuf::from(&root),
                label,
                drive_type,
                file_system,
                total_bytes,
                free_bytes,
                is_ready,
            });
        }
    }

    debug!(count = drives.len(), "Drives enumerated");
    Ok(drives)
}

/// List all available drives (non-Windows fallback).
#[cfg(not(windows))]
pub fn list_drives() -> ZResult<Vec<DriveInfo>> {
    use std::fs;

    debug!("Enumerating mount points (non-Windows)");

    // On non-Windows, we'll list common mount points
    let mount_points = ["/", "/home", "/tmp", "/mnt", "/media"];

    let drives = mount_points
        .iter()
        .filter(|p| fs::metadata(p).is_ok())
        .map(|p| DriveInfo {
            path: PathBuf::from(p),
            label: p.to_string(),
            drive_type: DriveType::Fixed,
            file_system: None,
            total_bytes: None,
            free_bytes: None,
            is_ready: true,
        })
        .collect();

    Ok(drives)
}

/// Get information about a specific drive.
pub fn get_drive_info(path: impl AsRef<std::path::Path>) -> ZResult<Option<DriveInfo>> {
    let drives = list_drives()?;
    let path = path.as_ref();

    // Find the drive that contains this path
    Ok(drives.into_iter().find(|d| path.starts_with(&d.path)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 1024), "1.00 TB");
    }

    #[test]
    fn test_drive_type_descriptions() {
        assert_eq!(DriveType::Fixed.description(), "Local Disk");
        assert_eq!(DriveType::Removable.description(), "Removable");
        assert_eq!(DriveType::Network.description(), "Network Drive");
    }

    #[test]
    fn test_drive_info_display_name() {
        let drive = DriveInfo {
            path: PathBuf::from("C:\\"),
            label: "Windows".to_string(),
            drive_type: DriveType::Fixed,
            file_system: Some("NTFS".to_string()),
            total_bytes: Some(500 * 1024 * 1024 * 1024),
            free_bytes: Some(100 * 1024 * 1024 * 1024),
            is_ready: true,
        };

        assert_eq!(drive.display_name(), "Windows (C:)");

        let unlabeled = DriveInfo {
            path: PathBuf::from("D:\\"),
            label: String::new(),
            drive_type: DriveType::Fixed,
            file_system: None,
            total_bytes: None,
            free_bytes: None,
            is_ready: false,
        };

        assert_eq!(unlabeled.display_name(), "Local Disk (D:)");
    }

    #[test]
    fn test_drive_info_usage() {
        let drive = DriveInfo {
            path: PathBuf::from("C:\\"),
            label: "Test".to_string(),
            drive_type: DriveType::Fixed,
            file_system: Some("NTFS".to_string()),
            total_bytes: Some(1000),
            free_bytes: Some(400),
            is_ready: true,
        };

        assert_eq!(drive.used_bytes(), Some(600));
        assert!((drive.usage_percent().unwrap() - 0.6).abs() < 0.001);
    }

    #[test]
    #[cfg(windows)]
    fn test_list_drives_windows() {
        use std::path::Path;

        let drives = list_drives().unwrap();

        // Should have at least one drive (C:)
        assert!(!drives.is_empty());

        // C: drive should exist and be fixed
        let c_drive = drives.iter().find(|d| d.path == Path::new("C:\\"));
        assert!(c_drive.is_some());

        let c = c_drive.unwrap();
        assert_eq!(c.drive_type, DriveType::Fixed);
        assert!(c.is_ready);
        assert!(c.total_bytes.is_some());
        assert!(c.free_bytes.is_some());
    }
}
