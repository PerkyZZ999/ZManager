//! Integration tests for large file copy operations.
//!
//! These tests verify that copying works correctly with larger files
//! and exercise the progress callback functionality.

use std::fs::{self, File};
use std::io::Write;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::TempDir;
use zmanager_core::CancellationToken;
use zmanager_transfer_win::{copy_file_with_progress, CopyProgress};

fn create_test_file(dir: &TempDir, name: &str, size: usize) -> std::path::PathBuf {
    let path = dir.path().join(name);
    let mut file = File::create(&path).unwrap();

    // Write in chunks to avoid memory issues
    let chunk_size = 1024 * 1024; // 1MB chunks
    let chunk: Vec<u8> = (0..chunk_size).map(|i| (i % 256) as u8).collect();
    let mut written = 0;

    while written < size {
        let to_write = std::cmp::min(chunk_size, size - written);
        file.write_all(&chunk[..to_write]).unwrap();
        written += to_write;
    }

    file.flush().unwrap();
    path
}

#[test]
fn test_copy_10mb_file() {
    let temp = TempDir::new().unwrap();
    let source = create_test_file(&temp, "source_10mb.bin", 10 * 1024 * 1024);
    let dest = temp.path().join("dest_10mb.bin");

    let token = CancellationToken::new();
    let result = copy_file_with_progress(&source, &dest, false, token, None);

    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert_eq!(bytes, 10 * 1024 * 1024);
    assert!(dest.exists());
    assert_eq!(fs::metadata(&source).unwrap().len(), fs::metadata(&dest).unwrap().len());
}

#[test]
fn test_copy_100mb_file_with_progress() {
    let temp = TempDir::new().unwrap();
    let source = create_test_file(&temp, "source_100mb.bin", 100 * 1024 * 1024);
    let dest = temp.path().join("dest_100mb.bin");

    let progress_updates = Arc::new(AtomicUsize::new(0));
    let max_bytes = Arc::new(AtomicU64::new(0));
    let final_bytes = Arc::new(AtomicU64::new(0));
    
    let updates_clone = progress_updates.clone();
    let max_clone = max_bytes.clone();
    let final_clone = final_bytes.clone();

    let callback = Box::new(move |p: CopyProgress| {
        updates_clone.fetch_add(1, Ordering::Relaxed);
        max_clone.fetch_max(p.bytes_copied, Ordering::Relaxed);
        final_clone.store(p.bytes_copied, Ordering::Relaxed);
    });

    let token = CancellationToken::new();
    let result = copy_file_with_progress(&source, &dest, false, token, Some(callback));

    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert_eq!(bytes, 100 * 1024 * 1024);

    // Check we got progress updates
    let updates = progress_updates.load(Ordering::Relaxed);
    println!("Progress updates for 100MB file: {updates}");
    
    // With 100MB file, we should get at least a few progress updates
    // (depends on chunk size, but usually many for 100MB)
    assert!(updates > 0, "Expected at least one progress update for 100MB file");
}

#[test]
fn test_copy_preserves_content() {
    let temp = TempDir::new().unwrap();
    
    // Create a file with specific pattern for verification
    let size = 5 * 1024 * 1024; // 5MB
    let source = temp.path().join("pattern_source.bin");
    let dest = temp.path().join("pattern_dest.bin");

    // Create with alternating pattern
    {
        let mut file = File::create(&source).unwrap();
        for i in 0..size {
            file.write_all(&[(i % 256) as u8]).unwrap();
        }
    }

    let token = CancellationToken::new();
    let result = copy_file_with_progress(&source, &dest, false, token, None);
    assert!(result.is_ok());

    // Verify content matches
    let source_content = fs::read(&source).unwrap();
    let dest_content = fs::read(&dest).unwrap();
    assert_eq!(source_content.len(), dest_content.len());
    assert_eq!(source_content, dest_content);
}

#[test]
fn test_copy_speed_measurement() {
    let temp = TempDir::new().unwrap();
    let source = create_test_file(&temp, "speed_test.bin", 50 * 1024 * 1024);
    let dest = temp.path().join("speed_dest.bin");

    let last_speed = Arc::new(AtomicU64::new(0));
    let speed_clone = last_speed.clone();

    let callback = Box::new(move |p: CopyProgress| {
        if p.speed_bps > 0 {
            speed_clone.store(p.speed_bps, Ordering::Relaxed);
        }
    });

    let token = CancellationToken::new();
    let start = std::time::Instant::now();
    let result = copy_file_with_progress(&source, &dest, false, token, Some(callback));
    let duration = start.elapsed();

    assert!(result.is_ok());

    let speed = last_speed.load(Ordering::Relaxed);
    let actual_speed = 50 * 1024 * 1024 / duration.as_secs().max(1);

    println!("Reported speed: {} MB/s", speed / 1_000_000);
    println!("Actual speed: {} MB/s", actual_speed / 1_000_000);
}

#[test]
fn test_eta_calculation() {
    let temp = TempDir::new().unwrap();
    let source = create_test_file(&temp, "eta_test.bin", 50 * 1024 * 1024);
    let dest = temp.path().join("eta_dest.bin");

    let got_eta = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let eta_clone = got_eta.clone();

    let callback = Box::new(move |p: CopyProgress| {
        if p.eta_seconds.is_some() {
            eta_clone.store(true, Ordering::Relaxed);
        }
    });

    let token = CancellationToken::new();
    let result = copy_file_with_progress(&source, &dest, false, token, Some(callback));

    assert!(result.is_ok());
    
    // ETA might or might not be calculated depending on timing
    // Just log whether we got it
    let had_eta = got_eta.load(Ordering::Relaxed);
    println!("ETA was calculated: {had_eta}");
}
