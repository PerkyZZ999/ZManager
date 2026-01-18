//! Benchmarks for directory listing operations.
//!
//! Run with: cargo bench --package zmanager-core

use std::fs::{self, File};
use std::io::Write;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use zmanager_core::{list_directory, FilterSpec, SortSpec};

/// Create a test directory with a specified number of files.
fn create_test_dir(file_count: usize) -> TempDir {
    let dir = TempDir::new().unwrap();

    for i in 0..file_count {
        let filename = format!("file_{:06}.txt", i);
        let path = dir.path().join(&filename);
        let mut file = File::create(&path).unwrap();
        // Write some content so files have size
        writeln!(file, "File content for {}", filename).unwrap();
    }

    // Add some subdirectories
    let subdir_count = file_count / 100;
    for i in 0..subdir_count.max(1) {
        fs::create_dir(dir.path().join(format!("subdir_{:04}", i))).unwrap();
    }

    dir
}

/// Measure the time to execute a function multiple times and return stats.
fn benchmark<F>(name: &str, iterations: usize, mut f: F) -> BenchResult
where
    F: FnMut(),
{
    // Warmup
    for _ in 0..3 {
        f();
    }

    // Actual measurements
    let mut times = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        f();
        times.push(start.elapsed());
    }

    times.sort();

    let total: Duration = times.iter().sum();
    let avg = total / iterations as u32;
    let min = times[0];
    let max = times[times.len() - 1];
    let median = times[times.len() / 2];
    let p95 = times[(times.len() as f64 * 0.95) as usize];

    BenchResult {
        name: name.to_string(),
        iterations,
        avg,
        min,
        max,
        median,
        p95,
    }
}

#[derive(Debug)]
struct BenchResult {
    name: String,
    iterations: usize,
    avg: Duration,
    min: Duration,
    max: Duration,
    median: Duration,
    p95: Duration,
}

impl BenchResult {
    fn print(&self) {
        println!("\n{}", self.name);
        println!("  Iterations: {}", self.iterations);
        println!("  Average:    {:?}", self.avg);
        println!("  Median:     {:?}", self.median);
        println!("  Min:        {:?}", self.min);
        println!("  Max:        {:?}", self.max);
        println!("  P95:        {:?}", self.p95);
    }
}

fn main() {
    println!("ZManager Core Benchmarks");
    println!("========================\n");

    // Small directory (100 files)
    println!("Setting up 100 files...");
    let small_dir = create_test_dir(100);
    let small_result = benchmark("list_directory (100 files)", 100, || {
        list_directory(small_dir.path(), None, None).unwrap();
    });
    small_result.print();

    // Medium directory (1,000 files)
    println!("\nSetting up 1,000 files...");
    let medium_dir = create_test_dir(1_000);
    let medium_result = benchmark("list_directory (1,000 files)", 50, || {
        list_directory(medium_dir.path(), None, None).unwrap();
    });
    medium_result.print();

    // Large directory (10,000 files)
    println!("\nSetting up 10,000 files...");
    let large_dir = create_test_dir(10_000);
    let large_result = benchmark("list_directory (10,000 files)", 20, || {
        list_directory(large_dir.path(), None, None).unwrap();
    });
    large_result.print();

    // With sorting
    let sort = SortSpec::by_modified();
    let sort_result = benchmark("list_directory (10,000 files, sorted by date)", 20, || {
        list_directory(large_dir.path(), Some(&sort), None).unwrap();
    });
    sort_result.print();

    // With filtering
    let filter = FilterSpec::new().with_pattern("file_001");
    let filter_result = benchmark("list_directory (10,000 files, filtered)", 20, || {
        list_directory(large_dir.path(), None, Some(&filter)).unwrap();
    });
    filter_result.print();

    // Very large directory (50,000 files) - only if system can handle it
    println!("\nSetting up 50,000 files (this may take a moment)...");
    let xlarge_dir = create_test_dir(50_000);
    let xlarge_result = benchmark("list_directory (50,000 files)", 10, || {
        list_directory(xlarge_dir.path(), None, None).unwrap();
    });
    xlarge_result.print();

    println!("\n\nBenchmark Summary");
    println!("-----------------");
    println!(
        "100 files:    {:?} avg",
        small_result.avg
    );
    println!(
        "1,000 files:  {:?} avg",
        medium_result.avg
    );
    println!(
        "10,000 files: {:?} avg",
        large_result.avg
    );
    println!(
        "50,000 files: {:?} avg",
        xlarge_result.avg
    );

    // Calculate throughput
    let throughput_50k = 50_000.0 / xlarge_result.avg.as_secs_f64();
    println!("\nThroughput: {:.0} files/sec", throughput_50k);
}
