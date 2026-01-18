//! Conflict resolution for file transfer operations.
//!
//! This module provides conflict detection and resolution strategies
//! for when destination files already exist.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

/// A conflict detected during transfer planning or execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// Source file path.
    pub source: PathBuf,
    /// Destination file path (where conflict exists).
    pub destination: PathBuf,
    /// Size of the source file.
    pub source_size: u64,
    /// Size of the existing destination file.
    pub dest_size: u64,
    /// Modification time of source (if available).
    pub source_modified: Option<std::time::SystemTime>,
    /// Modification time of destination (if available).
    pub dest_modified: Option<std::time::SystemTime>,
    /// Whether source is a directory.
    pub is_dir: bool,
}

impl Conflict {
    /// Create a conflict from source and destination paths.
    pub fn new(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> Option<Self> {
        let source = source.as_ref();
        let destination = destination.as_ref();

        if !destination.exists() {
            return None;
        }

        let source_meta = std::fs::metadata(source).ok()?;
        let dest_meta = std::fs::metadata(destination).ok()?;

        Some(Self {
            source: source.to_path_buf(),
            destination: destination.to_path_buf(),
            source_size: source_meta.len(),
            dest_size: dest_meta.len(),
            source_modified: source_meta.modified().ok(),
            dest_modified: dest_meta.modified().ok(),
            is_dir: source_meta.is_dir(),
        })
    }

    /// Check if source is newer than destination.
    pub fn source_is_newer(&self) -> Option<bool> {
        match (self.source_modified, self.dest_modified) {
            (Some(src), Some(dst)) => Some(src > dst),
            _ => None,
        }
    }

    /// Check if files have the same size.
    pub fn same_size(&self) -> bool {
        self.source_size == self.dest_size
    }

    /// Get a human-readable description of the conflict.
    pub fn description(&self) -> String {
        let kind = if self.is_dir { "Directory" } else { "File" };
        format!(
            "{} already exists: {}",
            kind,
            self.destination.display()
        )
    }
}

/// Policy for resolving file conflicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictPolicy {
    /// Always overwrite existing files.
    Overwrite,
    /// Always skip conflicting files.
    Skip,
    /// Rename source file (e.g., "file (1).txt").
    Rename,
    /// Keep newer file (by modification time).
    KeepNewer,
    /// Keep larger file.
    KeepLarger,
    /// Ask user for each conflict.
    #[default]
    Ask,
}

impl ConflictPolicy {
    /// Get a human-readable label for the policy.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Overwrite => "Overwrite",
            Self::Skip => "Skip",
            Self::Rename => "Rename",
            Self::KeepNewer => "Keep Newer",
            Self::KeepLarger => "Keep Larger",
            Self::Ask => "Ask",
        }
    }

    /// Get a description of what the policy does.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Overwrite => "Replace existing files with source files",
            Self::Skip => "Keep existing files, don't copy conflicting sources",
            Self::Rename => "Rename source files to avoid conflicts (e.g., file (1).txt)",
            Self::KeepNewer => "Keep the file with the most recent modification time",
            Self::KeepLarger => "Keep the larger file",
            Self::Ask => "Ask for each conflict",
        }
    }
}

/// Result of conflict resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Overwrite the destination.
    Overwrite,
    /// Skip this file.
    Skip,
    /// Use a renamed destination path.
    Rename,
    /// Cancel the entire operation.
    Cancel,
}

/// Settings for conflict handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictSettings {
    /// Default policy for file conflicts.
    pub file_policy: ConflictPolicy,
    /// Default policy for directory conflicts.
    pub dir_policy: ConflictPolicy,
    /// Whether to apply the policy to all remaining conflicts.
    pub apply_to_all: bool,
}

impl Default for ConflictSettings {
    fn default() -> Self {
        Self {
            file_policy: ConflictPolicy::Ask,
            dir_policy: ConflictPolicy::Overwrite, // Directories usually merge
            apply_to_all: false,
        }
    }
}

/// Resolver for handling conflicts during transfer.
#[derive(Debug, Clone)]
pub struct ConflictResolver {
    settings: ConflictSettings,
    /// Cached resolution for "apply to all".
    cached_resolution: Option<ConflictResolution>,
}

impl ConflictResolver {
    /// Create a new resolver with default settings.
    pub fn new() -> Self {
        Self::with_settings(ConflictSettings::default())
    }

    /// Create a new resolver with custom settings.
    pub fn with_settings(settings: ConflictSettings) -> Self {
        Self {
            settings,
            cached_resolution: None,
        }
    }

    /// Create a resolver that always overwrites.
    pub fn overwrite_all() -> Self {
        Self::with_settings(ConflictSettings {
            file_policy: ConflictPolicy::Overwrite,
            dir_policy: ConflictPolicy::Overwrite,
            apply_to_all: true,
        })
    }

    /// Create a resolver that always skips.
    pub fn skip_all() -> Self {
        Self::with_settings(ConflictSettings {
            file_policy: ConflictPolicy::Skip,
            dir_policy: ConflictPolicy::Skip,
            apply_to_all: true,
        })
    }

    /// Get the current settings.
    pub fn settings(&self) -> &ConflictSettings {
        &self.settings
    }

    /// Update settings.
    pub fn set_settings(&mut self, settings: ConflictSettings) {
        self.settings = settings;
        self.cached_resolution = None;
    }

    /// Set "apply to all" with a specific resolution.
    pub fn apply_to_all(&mut self, resolution: ConflictResolution) {
        self.settings.apply_to_all = true;
        self.cached_resolution = Some(resolution);
    }

    /// Reset "apply to all".
    pub fn reset_apply_to_all(&mut self) {
        self.settings.apply_to_all = false;
        self.cached_resolution = None;
    }

    /// Resolve a conflict using the current policy.
    ///
    /// Returns `None` if the policy is `Ask` and the user needs to decide.
    pub fn resolve(&self, conflict: &Conflict) -> Option<ConflictResolution> {
        // Check for cached "apply to all" resolution
        if self.settings.apply_to_all {
            if let Some(cached) = self.cached_resolution {
                trace!(
                    destination = %conflict.destination.display(),
                    resolution = ?cached,
                    "Using cached resolution"
                );
                return Some(cached);
            }
        }

        let policy = if conflict.is_dir {
            self.settings.dir_policy
        } else {
            self.settings.file_policy
        };

        let resolution = match policy {
            ConflictPolicy::Overwrite => Some(ConflictResolution::Overwrite),
            ConflictPolicy::Skip => Some(ConflictResolution::Skip),
            ConflictPolicy::Rename => Some(ConflictResolution::Rename),
            ConflictPolicy::KeepNewer => {
                match conflict.source_is_newer() {
                    Some(true) => Some(ConflictResolution::Overwrite),
                    Some(false) => Some(ConflictResolution::Skip),
                    None => None, // Can't determine, need to ask
                }
            }
            ConflictPolicy::KeepLarger => {
                if conflict.source_size > conflict.dest_size {
                    Some(ConflictResolution::Overwrite)
                } else {
                    Some(ConflictResolution::Skip)
                }
            }
            ConflictPolicy::Ask => None,
        };

        debug!(
            destination = %conflict.destination.display(),
            policy = ?policy,
            resolution = ?resolution,
            "Resolved conflict"
        );

        resolution
    }

    /// Generate a unique renamed path for a file.
    pub fn generate_rename_path(path: &Path) -> PathBuf {
        let parent = path.parent().unwrap_or(Path::new(""));
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = path.extension().and_then(|e| e.to_str());

        let mut counter = 1;
        loop {
            let new_name = if let Some(ext) = ext {
                format!("{stem} ({counter}).{ext}")
            } else {
                format!("{stem} ({counter})")
            };

            let new_path = parent.join(&new_name);
            if !new_path.exists() {
                return new_path;
            }

            counter += 1;

            // Safety limit
            if counter > 10000 {
                // Fallback to timestamp-based name
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                let new_name = if let Some(ext) = ext {
                    format!("{stem}_{timestamp}.{ext}")
                } else {
                    format!("{stem}_{timestamp}")
                };

                return parent.join(&new_name);
            }
        }
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_conflict_detection() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source.txt");
        let dest = temp.path().join("dest.txt");

        fs::write(&source, "source content").unwrap();
        fs::write(&dest, "dest content").unwrap();

        let conflict = Conflict::new(&source, &dest).unwrap();
        assert_eq!(conflict.source_size, 14);
        assert_eq!(conflict.dest_size, 12);
        assert!(!conflict.is_dir);
    }

    #[test]
    fn test_no_conflict() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source.txt");
        let dest = temp.path().join("nonexistent.txt");

        fs::write(&source, "content").unwrap();

        let conflict = Conflict::new(&source, &dest);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_conflict_policy_overwrite() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source.txt");
        let dest = temp.path().join("dest.txt");

        fs::write(&source, "new").unwrap();
        fs::write(&dest, "old").unwrap();

        let conflict = Conflict::new(&source, &dest).unwrap();
        let resolver = ConflictResolver::overwrite_all();

        assert_eq!(
            resolver.resolve(&conflict),
            Some(ConflictResolution::Overwrite)
        );
    }

    #[test]
    fn test_conflict_policy_skip() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source.txt");
        let dest = temp.path().join("dest.txt");

        fs::write(&source, "new").unwrap();
        fs::write(&dest, "old").unwrap();

        let conflict = Conflict::new(&source, &dest).unwrap();
        let resolver = ConflictResolver::skip_all();

        assert_eq!(resolver.resolve(&conflict), Some(ConflictResolution::Skip));
    }

    #[test]
    fn test_conflict_policy_ask() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source.txt");
        let dest = temp.path().join("dest.txt");

        fs::write(&source, "new").unwrap();
        fs::write(&dest, "old").unwrap();

        let conflict = Conflict::new(&source, &dest).unwrap();
        let resolver = ConflictResolver::new(); // Default is Ask

        assert_eq!(resolver.resolve(&conflict), None);
    }

    #[test]
    fn test_conflict_policy_keep_larger() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source.txt");
        let dest = temp.path().join("dest.txt");

        fs::write(&source, "larger content here").unwrap();
        fs::write(&dest, "small").unwrap();

        let conflict = Conflict::new(&source, &dest).unwrap();
        let resolver = ConflictResolver::with_settings(ConflictSettings {
            file_policy: ConflictPolicy::KeepLarger,
            ..Default::default()
        });

        assert_eq!(
            resolver.resolve(&conflict),
            Some(ConflictResolution::Overwrite)
        );
    }

    #[test]
    fn test_generate_rename_path() {
        let temp = TempDir::new().unwrap();
        let original = temp.path().join("file.txt");
        fs::write(&original, "content").unwrap();

        let renamed = ConflictResolver::generate_rename_path(&original);
        assert_eq!(renamed, temp.path().join("file (1).txt"));

        // Create that file too
        fs::write(&renamed, "content").unwrap();

        let renamed2 = ConflictResolver::generate_rename_path(&original);
        assert_eq!(renamed2, temp.path().join("file (2).txt"));
    }

    #[test]
    fn test_generate_rename_path_no_extension() {
        let temp = TempDir::new().unwrap();
        let original = temp.path().join("myfile");
        fs::write(&original, "content").unwrap();

        let renamed = ConflictResolver::generate_rename_path(&original);
        assert_eq!(renamed, temp.path().join("myfile (1)"));
    }

    #[test]
    fn test_apply_to_all() {
        let temp = TempDir::new().unwrap();
        let source1 = temp.path().join("file1.txt");
        let dest1 = temp.path().join("dest1.txt");
        let source2 = temp.path().join("file2.txt");
        let dest2 = temp.path().join("dest2.txt");

        fs::write(&source1, "s1").unwrap();
        fs::write(&dest1, "d1").unwrap();
        fs::write(&source2, "s2").unwrap();
        fs::write(&dest2, "d2").unwrap();

        let conflict1 = Conflict::new(&source1, &dest1).unwrap();
        let conflict2 = Conflict::new(&source2, &dest2).unwrap();

        let mut resolver = ConflictResolver::new();
        resolver.apply_to_all(ConflictResolution::Skip);

        assert_eq!(resolver.resolve(&conflict1), Some(ConflictResolution::Skip));
        assert_eq!(resolver.resolve(&conflict2), Some(ConflictResolution::Skip));
    }

    #[test]
    fn test_conflict_same_size() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source.txt");
        let dest = temp.path().join("dest.txt");

        fs::write(&source, "same").unwrap();
        fs::write(&dest, "same").unwrap();

        let conflict = Conflict::new(&source, &dest).unwrap();
        assert!(conflict.same_size());
    }

    #[test]
    fn test_policy_labels() {
        assert_eq!(ConflictPolicy::Overwrite.label(), "Overwrite");
        assert_eq!(ConflictPolicy::Skip.label(), "Skip");
        assert_eq!(ConflictPolicy::Rename.label(), "Rename");
        assert!(!ConflictPolicy::KeepNewer.description().is_empty());
    }
}
