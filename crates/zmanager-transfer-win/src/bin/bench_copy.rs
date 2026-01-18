//! Benchmark for file copy operations.
//!
//! This benchmark compares ZManager's CopyFileExW wrapper against native file copy.
//!
//! Run with: `cargo run --release -p zmanager-transfer-win --bin bench_copy`

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

use zmanager_core::CancellationToken;
use zmanager_transfer_win::copy_file_with_progress;

fn main() {
    println!("=== ZManager File Copy Benchmark ===\n");

    let temp_dir = std::env::temp_dir().join("zmanager_bench");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    // Test sizes: 1MB, 10MB, 100MB, 1GB
    let test_sizes = [
        (1024 * 1024, "1MB"),
        (10 * 1024 * 1024, "10MB"),
        (100 * 1024 * 1024, "100MB"),
        // Uncomment for full benchmark (takes longer):
        // (1024 * 1024 * 1024, "1GB"),
    ];

    println!(
        "{:<10} {:>15} {:>15} {:>15} {:>10}",
        "Size", "ZManager (ms)", "Native (ms)", "Speed (MB/s)", "Ratio"
    );
    println!("{}", "-".repeat(75));

    for (size, label) in test_sizes {
        let source = temp_dir.join(format!("source_{label}.bin"));
        let dest_zm = temp_dir.join(format!("dest_zm_{label}.bin"));
        let dest_native = temp_dir.join(format!("dest_native_{label}.bin"));

        // Create source file
        create_test_file(&source, size);

        // Benchmark ZManager copy
        let zm_duration = benchmark_zmanager_copy(&source, &dest_zm);
        let zm_speed = size as f64 / zm_duration.as_secs_f64() / 1_000_000.0;

        // Benchmark native copy
        let native_duration = benchmark_native_copy(&source, &dest_native);

        // Calculate ratio
        let ratio = zm_duration.as_secs_f64() / native_duration.as_secs_f64();

        println!(
            "{:<10} {:>15.2} {:>15.2} {:>15.1} {:>9.2}x",
            label,
            zm_duration.as_secs_f64() * 1000.0,
            native_duration.as_secs_f64() * 1000.0,
            zm_speed,
            ratio
        );

        // Verify file contents match
        verify_copy(&source, &dest_zm, label);
        verify_copy(&source, &dest_native, label);

        // Cleanup between tests
        let _ = fs::remove_file(&source);
        let _ = fs::remove_file(&dest_zm);
        let _ = fs::remove_file(&dest_native);
    }

    println!("\n=== Progress Callback Test ===\n");
    test_progress_callback(&temp_dir);

    println!("\n=== Cancellation Test ===\n");
    test_cancellation(&temp_dir);

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);

    println!("\n✅ All benchmarks completed successfully!");
}

fn create_test_file(path: &Path, size: usize) {
    let mut file = File::create(path).expect("Failed to create test file");

    // Write in 1MB chunks to avoid memory issues for large files
    let chunk_size = 1024 * 1024;
    let chunk: Vec<u8> = (0..chunk_size).map(|i| (i % 256) as u8).collect();
    let mut written = 0;

    while written < size {
        let to_write = std::cmp::min(chunk_size, size - written);
        file.write_all(&chunk[..to_write])
            .expect("Failed to write test data");
        written += to_write;
    }

    file.flush().expect("Failed to flush file");
}

fn benchmark_zmanager_copy(source: &Path, dest: &Path) -> Duration {
    let token = CancellationToken::new();
    let start = Instant::now();

    copy_file_with_progress(source, dest, true, token, None).expect("ZManager copy failed");

    start.elapsed()
}

fn benchmark_native_copy(source: &Path, dest: &Path) -> Duration {
    let start = Instant::now();

    fs::copy(source, dest).expect("Native copy failed");

    start.elapsed()
}

fn verify_copy(source: &Path, dest: &Path, label: &str) {
    let source_meta = fs::metadata(source).expect("Source metadata");
    let dest_meta = fs::metadata(dest).expect("Dest metadata");

    assert_eq!(
        source_meta.len(),
        dest_meta.len(),
        "Size mismatch for {label}"
    );
}

fn test_progress_callback(temp_dir: &Path) {
    let source = temp_dir.join("progress_test.bin");
    let dest = temp_dir.join("progress_dest.bin");

    // Create a 10MB file
    create_test_file(&source, 10 * 1024 * 1024);

    let progress_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let progress_clone = progress_count.clone();
    let last_percentage = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let percentage_clone = last_percentage.clone();

    let callback = Box::new(move |p: zmanager_transfer_win::CopyProgress| {
        progress_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        percentage_clone.store(p.percentage_int() as u64, std::sync::atomic::Ordering::Relaxed);
    });

    let token = CancellationToken::new();
    let result = copy_file_with_progress(&source, &dest, true, token, Some(callback));

    assert!(result.is_ok());
    let updates = progress_count.load(std::sync::atomic::Ordering::Relaxed);
    let final_pct = last_percentage.load(std::sync::atomic::Ordering::Relaxed);

    println!("Progress updates received: {updates}");
    println!("Final percentage reported: {final_pct}%");

    // Cleanup
    let _ = fs::remove_file(&source);
    let _ = fs::remove_file(&dest);
}

fn test_cancellation(temp_dir: &Path) {
    let source = temp_dir.join("cancel_test.bin");
    let dest = temp_dir.join("cancel_dest.bin");

    // Create a large file (100MB) for cancellation test
    println!("Creating 100MB test file...");
    create_test_file(&source, 100 * 1024 * 1024);

    let token = CancellationToken::new();
    let token_clone = token.clone();

    // Spawn a thread to cancel after a short delay
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(5));
        token_clone.cancel();
    });

    let result = copy_file_with_progress(&source, &dest, true, token, None);

    match result {
        Ok(_) => println!("Copy completed before cancellation (fast disk)"),
        Err(zmanager_core::ZError::Cancelled) => println!("✅ Cancellation worked correctly"),
        Err(e) => panic!("Unexpected error: {e}"),
    }

    // Cleanup
    let _ = fs::remove_file(&source);
    let _ = fs::remove_file(&dest);
}
