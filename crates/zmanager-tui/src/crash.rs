//! Crash reporting and panic handling.
//!
//! Installs a panic hook that writes crash dumps to disk for debugging.

use std::fs::{self, File};
use std::io::Write;
use std::panic::{self, PanicHookInfo};
use std::path::PathBuf;

use tracing::{error, info, warn};

/// Crash dump directory relative to user's app data.
const CRASH_DIR: &str = "ZManager\\crashes";

/// Install the crash reporting panic hook.
pub fn install_panic_hook() {
    let default_hook = panic::take_hook();
    
    panic::set_hook(Box::new(move |info| {
        // Try to write crash dump first
        if let Err(e) = write_crash_dump(info) {
            eprintln!("Failed to write crash dump: {}", e);
        }
        
        // Then call the default hook for normal panic behavior
        default_hook(info);
    }));
    
    info!("Crash reporting initialized");
}

/// Check for crash dumps from previous runs.
///
/// Returns the most recent crash dump if one exists.
pub fn check_for_crash_dumps() -> Option<CrashDump> {
    let crash_dir = get_crash_dir()?;
    
    if !crash_dir.exists() {
        return None;
    }
    
    // Find the most recent crash dump
    let mut latest: Option<(PathBuf, std::time::SystemTime)> = None;
    
    if let Ok(entries) = fs::read_dir(&crash_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "crash").unwrap_or(false) {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(modified) = meta.modified() {
                        match &latest {
                            None => latest = Some((path, modified)),
                            Some((_, last_mod)) if modified > *last_mod => {
                                latest = Some((path, modified));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    
    latest.and_then(|(path, _)| CrashDump::load(&path).ok())
}

/// Clear a crash dump file after it has been handled.
pub fn clear_crash_dump(dump: &CrashDump) {
    if let Err(e) = fs::remove_file(&dump.path) {
        warn!("Failed to remove crash dump: {}", e);
    } else {
        info!("Cleared crash dump: {:?}", dump.path);
    }
}

/// A crash dump from a previous panic.
#[derive(Debug)]
pub struct CrashDump {
    /// Path to the crash dump file.
    pub path: PathBuf,
    /// Timestamp when the crash occurred.
    pub timestamp: String,
    /// Panic message.
    pub message: String,
    /// Backtrace if available.
    pub backtrace: String,
    /// Location where the panic occurred.
    pub location: String,
}

impl CrashDump {
    /// Load a crash dump from a file.
    fn load(path: &PathBuf) -> Result<Self, std::io::Error> {
        let content = fs::read_to_string(path)?;
        
        let mut timestamp = String::new();
        let mut message = String::new();
        let mut location = String::new();
        let mut backtrace = String::new();
        let mut in_backtrace = false;
        
        for line in content.lines() {
            if line.starts_with("Timestamp: ") {
                timestamp = line.strip_prefix("Timestamp: ").unwrap_or("").to_string();
            } else if line.starts_with("Message: ") {
                message = line.strip_prefix("Message: ").unwrap_or("").to_string();
            } else if line.starts_with("Location: ") {
                location = line.strip_prefix("Location: ").unwrap_or("").to_string();
            } else if line.starts_with("Backtrace:") {
                in_backtrace = true;
            } else if in_backtrace {
                if !backtrace.is_empty() {
                    backtrace.push('\n');
                }
                backtrace.push_str(line);
            }
        }
        
        Ok(Self {
            path: path.clone(),
            timestamp,
            message,
            location,
            backtrace,
        })
    }
    
    /// Get a short summary of the crash.
    pub fn summary(&self) -> String {
        format!(
            "Crash at {}: {} ({})",
            self.timestamp,
            self.message,
            self.location
        )
    }
}

/// Write a crash dump to disk.
fn write_crash_dump(info: &PanicHookInfo) -> Result<(), std::io::Error> {
    let crash_dir = get_crash_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Could not find crash directory")
    })?;
    
    // Create crash directory if it doesn't exist
    fs::create_dir_all(&crash_dir)?;
    
    // Generate filename with timestamp
    let now = chrono::Local::now();
    let filename = format!("crash_{}.crash", now.format("%Y%m%d_%H%M%S"));
    let path = crash_dir.join(&filename);
    
    // Extract panic info
    let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic".to_string()
    };
    
    let location = info
        .location()
        .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
        .unwrap_or_else(|| "unknown".to_string());
    
    // Capture backtrace
    let backtrace = std::backtrace::Backtrace::capture();
    
    // Write crash dump
    let mut file = File::create(&path)?;
    writeln!(file, "ZManager Crash Report")?;
    writeln!(file, "=====================")?;
    writeln!(file)?;
    writeln!(file, "Timestamp: {}", now.format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(file, "Version: {}", env!("CARGO_PKG_VERSION"))?;
    writeln!(file)?;
    writeln!(file, "Message: {}", message)?;
    writeln!(file, "Location: {}", location)?;
    writeln!(file)?;
    writeln!(file, "Backtrace:")?;
    writeln!(file, "{}", backtrace)?;
    
    error!(
        "Crash dump written to {:?}: {} at {}",
        path, message, location
    );
    
    Ok(())
}

/// Get the crash dump directory path.
fn get_crash_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join(CRASH_DIR))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_crash_dir() {
        let dir = get_crash_dir();
        assert!(dir.is_some());
        let dir = dir.unwrap();
        assert!(dir.ends_with("ZManager\\crashes") || dir.ends_with("ZManager/crashes"));
    }
    
    #[test]
    fn test_crash_dump_summary() {
        let dump = CrashDump {
            path: PathBuf::from("/tmp/crash.crash"),
            timestamp: "2024-01-15 10:30:00".to_string(),
            message: "assertion failed".to_string(),
            location: "main.rs:42:1".to_string(),
            backtrace: String::new(),
        };
        
        let summary = dump.summary();
        assert!(summary.contains("2024-01-15"));
        assert!(summary.contains("assertion failed"));
        assert!(summary.contains("main.rs:42:1"));
    }
}
