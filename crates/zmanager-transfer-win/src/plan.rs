//! Transfer plan builder for folder operations.
//!
//! This module builds a complete transfer plan by enumerating source trees
//! and generating destination paths with conflict detection.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{debug, trace};
use walkdir::WalkDir;
use zmanager_core::{ZError, ZResult};

/// An individual item in a transfer plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferItem {
    /// Source path (file or directory).
    pub source: PathBuf,
    /// Destination path.
    pub destination: PathBuf,
    /// Whether this is a directory.
    pub is_dir: bool,
    /// Size in bytes (0 for directories).
    pub size: u64,
    /// Depth relative to the source root (0 for root items).
    pub depth: usize,
    /// Whether a conflict exists at the destination.
    pub has_conflict: bool,
}

impl TransferItem {
    /// Create a new transfer item.
    pub fn new(
        source: PathBuf,
        destination: PathBuf,
        is_dir: bool,
        size: u64,
        depth: usize,
    ) -> Self {
        let has_conflict = destination.exists();
        Self {
            source,
            destination,
            is_dir,
            size,
            depth,
            has_conflict,
        }
    }
}

/// Statistics for a transfer plan.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransferStats {
    /// Total number of files.
    pub total_files: usize,
    /// Total number of directories.
    pub total_dirs: usize,
    /// Total bytes to transfer.
    pub total_bytes: u64,
    /// Number of items with conflicts.
    pub conflicts: usize,
    /// Number of items that would be skipped.
    pub skipped: usize,
}

impl TransferStats {
    /// Total number of items (files + directories).
    pub fn total_items(&self) -> usize {
        self.total_files + self.total_dirs
    }
}

/// A complete transfer plan for a folder operation.
#[derive(Debug, Clone)]
pub struct TransferPlan {
    /// All items to transfer, in order (directories first, then files).
    pub items: Vec<TransferItem>,
    /// Statistics about the transfer.
    pub stats: TransferStats,
    /// Whether this is a move operation.
    pub is_move: bool,
    /// Source roots (original source paths).
    pub source_roots: Vec<PathBuf>,
    /// Destination root.
    pub destination_root: PathBuf,
}

impl TransferPlan {
    /// Get all directories in depth-first order (for creation).
    pub fn directories(&self) -> impl Iterator<Item = &TransferItem> {
        self.items.iter().filter(|item| item.is_dir)
    }

    /// Get all files.
    pub fn files(&self) -> impl Iterator<Item = &TransferItem> {
        self.items.iter().filter(|item| !item.is_dir)
    }

    /// Get items with conflicts.
    pub fn conflicts(&self) -> impl Iterator<Item = &TransferItem> {
        self.items.iter().filter(|item| item.has_conflict)
    }

    /// Check if there are any conflicts.
    pub fn has_conflicts(&self) -> bool {
        self.stats.conflicts > 0
    }
}

/// Builder for creating transfer plans.
#[derive(Debug)]
pub struct TransferPlanBuilder {
    sources: Vec<PathBuf>,
    destination: PathBuf,
    is_move: bool,
    follow_symlinks: bool,
    max_depth: Option<usize>,
}

impl TransferPlanBuilder {
    /// Create a new transfer plan builder.
    pub fn new(destination: impl AsRef<Path>) -> Self {
        Self {
            sources: Vec::new(),
            destination: destination.as_ref().to_path_buf(),
            is_move: false,
            follow_symlinks: false,
            max_depth: None,
        }
    }

    /// Add a source path.
    pub fn add_source(mut self, source: impl AsRef<Path>) -> Self {
        self.sources.push(source.as_ref().to_path_buf());
        self
    }

    /// Add multiple source paths.
    pub fn add_sources(mut self, sources: impl IntoIterator<Item = impl AsRef<Path>>) -> Self {
        for source in sources {
            self.sources.push(source.as_ref().to_path_buf());
        }
        self
    }

    /// Set whether this is a move operation.
    pub fn is_move(mut self, is_move: bool) -> Self {
        self.is_move = is_move;
        self
    }

    /// Set whether to follow symbolic links.
    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    /// Set maximum depth for directory traversal.
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Build the transfer plan.
    pub fn build(self) -> ZResult<TransferPlan> {
        if self.sources.is_empty() {
            return Err(ZError::Internal {
                message: "No sources provided for transfer plan".to_string(),
            });
        }

        debug!(
            sources = self.sources.len(),
            destination = %self.destination.display(),
            is_move = self.is_move,
            "Building transfer plan"
        );

        let mut items = Vec::new();
        let mut stats = TransferStats::default();

        // Ensure destination directory exists or will be created
        let dest_is_dir = self.destination.is_dir()
            || self.sources.len() > 1
            || self.sources.first().map(|s| s.is_dir()).unwrap_or(false);

        for source in &self.sources {
            if !source.exists() {
                return Err(ZError::NotFound {
                    path: source.clone(),
                });
            }

            if source.is_file() {
                // Single file transfer
                let dest_path = if dest_is_dir {
                    let file_name = source.file_name().ok_or_else(|| ZError::InvalidPath {
                        path: source.clone(),
                        reason: "No file name".to_string(),
                    })?;
                    self.destination.join(file_name)
                } else {
                    self.destination.clone()
                };

                let size = std::fs::metadata(source)
                    .map(|m| m.len())
                    .unwrap_or(0);

                let item = TransferItem::new(source.clone(), dest_path, false, size, 0);

                if item.has_conflict {
                    stats.conflicts += 1;
                }
                stats.total_files += 1;
                stats.total_bytes += size;

                items.push(item);
            } else if source.is_dir() {
                // Directory transfer - enumerate contents
                self.enumerate_directory(source, &self.destination, &mut items, &mut stats)?;
            }
        }

        // Sort items: directories first (by depth), then files
        items.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (true, true) => a.depth.cmp(&b.depth), // Shallower dirs first
                (false, false) => a.source.cmp(&b.source),
            }
        });

        let plan = TransferPlan {
            items,
            stats,
            is_move: self.is_move,
            source_roots: self.sources,
            destination_root: self.destination,
        };

        debug!(
            files = plan.stats.total_files,
            dirs = plan.stats.total_dirs,
            bytes = plan.stats.total_bytes,
            conflicts = plan.stats.conflicts,
            "Transfer plan built"
        );

        Ok(plan)
    }

    fn enumerate_directory(
        &self,
        source_root: &Path,
        dest_root: &Path,
        items: &mut Vec<TransferItem>,
        stats: &mut TransferStats,
    ) -> ZResult<()> {
        let source_parent = source_root.parent().unwrap_or(source_root);
        let source_name = source_root
            .file_name()
            .ok_or_else(|| ZError::InvalidPath {
                path: source_root.to_path_buf(),
                reason: "No directory name".to_string(),
            })?;

        // The destination for this source directory
        let _dest_for_source = dest_root.join(source_name);

        let mut walker = WalkDir::new(source_root);

        if !self.follow_symlinks {
            walker = walker.follow_links(false);
        }

        if let Some(depth) = self.max_depth {
            walker = walker.max_depth(depth);
        }

        for entry in walker {
            let entry = entry.map_err(|e| {
                let path = e.path().map(|p| p.to_path_buf()).unwrap_or_default();
                ZError::Io {
                    path: path.clone(),
                    message: e.to_string(),
                    source: e
                        .into_io_error()
                        .unwrap_or_else(|| std::io::Error::other("walkdir error")),
                }
            })?;

            let source_path = entry.path();
            let relative_path = source_path.strip_prefix(source_parent).map_err(|_| {
                ZError::InvalidPath {
                    path: source_path.to_path_buf(),
                    reason: "Failed to compute relative path".to_string(),
                }
            })?;

            let dest_path = dest_root.join(relative_path);
            let depth = entry.depth();
            let is_dir = entry.file_type().is_dir();

            let size = if is_dir {
                0
            } else {
                entry.metadata().map(|m| m.len()).unwrap_or(0)
            };

            trace!(
                source = %source_path.display(),
                dest = %dest_path.display(),
                is_dir,
                size,
                depth,
                "Enumerated item"
            );

            let item = TransferItem::new(source_path.to_path_buf(), dest_path, is_dir, size, depth);

            if item.has_conflict {
                stats.conflicts += 1;
            }

            if is_dir {
                stats.total_dirs += 1;
            } else {
                stats.total_files += 1;
                stats.total_bytes += size;
            }

            items.push(item);
        }

        Ok(())
    }
}

/// Check if two paths are on the same volume (for move optimization).
#[cfg(windows)]
pub fn same_volume(path1: &Path, path2: &Path) -> bool {
    // Get the root of each path
    let root1 = get_volume_root(path1);
    let root2 = get_volume_root(path2);

    match (root1, root2) {
        (Some(r1), Some(r2)) => r1.eq_ignore_ascii_case(&r2),
        _ => false,
    }
}

#[cfg(windows)]
fn get_volume_root(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();

    // Handle UNC paths: \\server\share
    if path_str.starts_with("\\\\") {
        let parts: Vec<&str> = path_str.trim_start_matches("\\\\").splitn(3, '\\').collect();
        if parts.len() >= 2 {
            return Some(format!("\\\\{}\\{}", parts[0], parts[1]));
        }
    }

    // Handle drive letters: C:\
    if path_str.len() >= 2 {
        let chars: Vec<char> = path_str.chars().take(2).collect();
        if chars[1] == ':' {
            return Some(format!("{}:", chars[0].to_ascii_uppercase()));
        }
    }

    None
}

#[cfg(not(windows))]
pub fn same_volume(path1: &Path, path2: &Path) -> bool {
    // On non-Windows, compare mount points using stat
    use std::os::unix::fs::MetadataExt;
    
    let meta1 = std::fs::metadata(path1).ok();
    let meta2 = std::fs::metadata(path2).ok();
    
    match (meta1, meta2) {
        (Some(m1), Some(m2)) => m1.dev() == m2.dev(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_tree(dir: &TempDir) -> PathBuf {
        let root = dir.path().join("source");
        fs::create_dir_all(&root).unwrap();

        // Create structure:
        // source/
        //   file1.txt (100 bytes)
        //   subdir/
        //     file2.txt (200 bytes)
        //     nested/
        //       file3.txt (300 bytes)

        fs::write(root.join("file1.txt"), vec![b'A'; 100]).unwrap();

        let subdir = root.join("subdir");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join("file2.txt"), vec![b'B'; 200]).unwrap();

        let nested = subdir.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("file3.txt"), vec![b'C'; 300]).unwrap();

        root
    }

    #[test]
    fn test_build_plan_single_file() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source.txt");
        let dest_dir = temp.path().join("dest");

        fs::write(&source, "hello").unwrap();
        fs::create_dir(&dest_dir).unwrap();

        let plan = TransferPlanBuilder::new(&dest_dir)
            .add_source(&source)
            .build()
            .unwrap();

        assert_eq!(plan.stats.total_files, 1);
        assert_eq!(plan.stats.total_dirs, 0);
        assert_eq!(plan.stats.total_bytes, 5);
        assert_eq!(plan.items.len(), 1);
        assert!(!plan.items[0].is_dir);
        assert_eq!(plan.items[0].destination, dest_dir.join("source.txt"));
    }

    #[test]
    fn test_build_plan_directory() {
        let temp = TempDir::new().unwrap();
        let source = create_test_tree(&temp);
        let dest_dir = temp.path().join("dest");
        fs::create_dir(&dest_dir).unwrap();

        let plan = TransferPlanBuilder::new(&dest_dir)
            .add_source(&source)
            .build()
            .unwrap();

        assert_eq!(plan.stats.total_files, 3);
        assert_eq!(plan.stats.total_dirs, 3); // source, subdir, nested
        assert_eq!(plan.stats.total_bytes, 600);
        assert_eq!(plan.stats.total_items(), 6);

        // Directories should come first
        let first_item = &plan.items[0];
        assert!(first_item.is_dir);
    }

    #[test]
    fn test_build_plan_with_conflicts() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source.txt");
        let dest_dir = temp.path().join("dest");
        let existing = dest_dir.join("source.txt");

        fs::write(&source, "new content").unwrap();
        fs::create_dir(&dest_dir).unwrap();
        fs::write(&existing, "old content").unwrap();

        let plan = TransferPlanBuilder::new(&dest_dir)
            .add_source(&source)
            .build()
            .unwrap();

        assert_eq!(plan.stats.conflicts, 1);
        assert!(plan.has_conflicts());
        assert!(plan.items[0].has_conflict);
    }

    #[test]
    fn test_build_plan_multiple_sources() {
        let temp = TempDir::new().unwrap();
        let source1 = temp.path().join("file1.txt");
        let source2 = temp.path().join("file2.txt");
        let dest_dir = temp.path().join("dest");

        fs::write(&source1, "content1").unwrap();
        fs::write(&source2, "content2").unwrap();
        fs::create_dir(&dest_dir).unwrap();

        let plan = TransferPlanBuilder::new(&dest_dir)
            .add_sources([&source1, &source2])
            .build()
            .unwrap();

        assert_eq!(plan.stats.total_files, 2);
        assert_eq!(plan.items.len(), 2);
    }

    #[test]
    fn test_build_plan_source_not_found() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("nonexistent.txt");
        let dest = temp.path().join("dest");

        let result = TransferPlanBuilder::new(&dest)
            .add_source(&source)
            .build();

        assert!(matches!(result, Err(ZError::NotFound { .. })));
    }

    #[test]
    fn test_plan_iterators() {
        let temp = TempDir::new().unwrap();
        let source = create_test_tree(&temp);
        let dest_dir = temp.path().join("dest");
        fs::create_dir(&dest_dir).unwrap();

        let plan = TransferPlanBuilder::new(&dest_dir)
            .add_source(&source)
            .build()
            .unwrap();

        let dirs: Vec<_> = plan.directories().collect();
        let files: Vec<_> = plan.files().collect();

        assert_eq!(dirs.len(), 3);
        assert_eq!(files.len(), 3);
        assert!(dirs.iter().all(|d| d.is_dir));
        assert!(files.iter().all(|f| !f.is_dir));
    }

    #[test]
    fn test_same_volume() {
        let temp = TempDir::new().unwrap();
        let path1 = temp.path().join("file1.txt");
        let path2 = temp.path().join("file2.txt");

        // Same temp directory should be same volume
        assert!(same_volume(&path1, &path2));
    }

    #[test]
    fn test_transfer_stats() {
        let stats = TransferStats {
            total_files: 10,
            total_dirs: 5,
            total_bytes: 1000,
            conflicts: 2,
            skipped: 0,
        };

        assert_eq!(stats.total_items(), 15);
    }
}
