//! Configuration management for ZManager.
//!
//! This module handles loading, saving, and validating the TOML configuration file.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::{ZError, ZResult};

/// The main configuration for ZManager.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// General application settings.
    pub general: GeneralConfig,
    /// Appearance and display settings.
    pub appearance: AppearanceConfig,
    /// File operation settings.
    pub operations: OperationsConfig,
    /// Favorites/Quick Access entries.
    pub favorites: Vec<Favorite>,
    /// Session state (last directories, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<SessionState>,
}

impl Config {
    /// Load configuration from the default location.
    ///
    /// Creates a default config file if one doesn't exist.
    pub fn load() -> ZResult<Self> {
        let path = Self::default_path()?;
        Self::load_from(&path)
    }

    /// Load configuration from a specific path.
    pub fn load_from(path: &Path) -> ZResult<Self> {
        debug!(path = %path.display(), "Loading configuration");

        if !path.exists() {
            info!("Config file not found, creating default with initial favorites");
            let mut config = Self::default();
            config.add_default_favorites();
            config.save_to(path)?;
            return Ok(config);
        }

        let content = std::fs::read_to_string(path).map_err(|e| ZError::io(path, e))?;

        let mut config: Self = toml::from_str(&content).map_err(|e| ZError::Config {
            message: format!("Failed to parse config: {e}"),
        })?;

        // Deduplicate favorites to fix any corrupted config
        let old_count = config.favorites.len();
        config.deduplicate_favorites();
        let new_count = config.favorites.len();
        if old_count != new_count {
            info!("Removed {} duplicate favorites", old_count - new_count);
            // Save the cleaned config
            config.save_to(path)?;
        }

        config.validate()?;

        info!("Configuration loaded successfully");
        Ok(config)
    }

    /// Save configuration to the default location.
    pub fn save(&self) -> ZResult<()> {
        let path = Self::default_path()?;
        self.save_to(&path)
    }

    /// Save configuration to a specific path.
    pub fn save_to(&self, path: &Path) -> ZResult<()> {
        debug!(path = %path.display(), "Saving configuration");

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ZError::io(parent, e))?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| ZError::Config {
            message: format!("Failed to serialize config: {e}"),
        })?;

        std::fs::write(path, content).map_err(|e| ZError::io(path, e))?;

        info!("Configuration saved");
        Ok(())
    }

    /// Get the default configuration file path.
    ///
    /// On Windows: `%APPDATA%\ZManager\config.toml`
    pub fn default_path() -> ZResult<PathBuf> {
        let config_dir = dirs::config_dir().ok_or_else(|| ZError::Config {
            message: "Could not determine config directory".to_string(),
        })?;

        Ok(config_dir.join("ZManager").join("config.toml"))
    }

    /// Validate the configuration.
    pub fn validate(&self) -> ZResult<()> {
        // Validate history limits
        if self.general.max_history == 0 {
            return Err(ZError::Config {
                message: "max_history must be greater than 0".to_string(),
            });
        }

        // Validate favorites
        for fav in &self.favorites {
            if fav.name.is_empty() {
                warn!("Favorite with empty name found");
            }
        }

        Ok(())
    }

    /// Add a favorite.
    /// Skips if a favorite with the same path already exists.
    pub fn add_favorite(&mut self, favorite: Favorite) {
        // Check for duplicate paths (case-insensitive on Windows)
        let path_normalized = favorite.path.to_string_lossy().to_lowercase();
        let path_exists = self.favorites.iter().any(|f| {
            f.path.to_string_lossy().to_lowercase() == path_normalized
        });
        
        if path_exists {
            debug!("Favorite with path {:?} already exists, skipping", favorite.path);
            return;
        }
        
        // Also check for duplicate IDs (shouldn't happen with new hash-based IDs, but safety first)
        let id_exists = self.favorites.iter().any(|f| f.id == favorite.id);
        if id_exists {
            debug!("Favorite with id {:?} already exists, skipping", favorite.id);
            return;
        }
        
        // Set order to end of list if not specified
        let mut fav = favorite;
        if fav.order == 0 {
            fav.order = self.favorites.len() as u32 + 1;
        }
        self.favorites.push(fav);
    }

    /// Remove a favorite by ID.
    pub fn remove_favorite(&mut self, id: &str) -> bool {
        let initial_len = self.favorites.len();
        self.favorites.retain(|f| f.id != id);
        self.favorites.len() < initial_len
    }

    /// Get a favorite by ID.
    pub fn get_favorite(&self, id: &str) -> Option<&Favorite> {
        self.favorites.iter().find(|f| f.id == id)
    }

    /// Update favorite order.
    pub fn reorder_favorites(&mut self, ids: &[String]) {
        for (i, id) in ids.iter().enumerate() {
            if let Some(fav) = self.favorites.iter_mut().find(|f| f.id == *id) {
                fav.order = i as u32;
            }
        }
        self.favorites.sort_by_key(|f| f.order);
    }

    /// Deduplicate favorites by both ID and path (case-insensitive).
    /// Keeps the first occurrence of each unique ID and path.
    pub fn deduplicate_favorites(&mut self) {
        use std::collections::HashSet;
        
        let mut seen_ids: HashSet<String> = HashSet::new();
        let mut seen_paths: HashSet<String> = HashSet::new();
        
        self.favorites.retain(|f| {
            let id_normalized = f.id.to_lowercase();
            let path_normalized = f.path.to_string_lossy().to_lowercase();
            
            // Keep only if both ID and path are unique
            let id_is_new = seen_ids.insert(id_normalized);
            let path_is_new = seen_paths.insert(path_normalized);
            
            // Keep this entry only if BOTH are new (first occurrence)
            id_is_new && path_is_new
        });
        
        // Renumber orders
        for (i, fav) in self.favorites.iter_mut().enumerate() {
            fav.order = i as u32;
        }
    }

    /// Add default favorites for a fresh installation.
    /// Uses Windows standard folders based on user profile.
    pub fn add_default_favorites(&mut self) {
        // Get user directories using the dirs crate
        if let Some(home) = dirs::home_dir() {
            let mut fav = Favorite::new("Home", &home);
            fav.icon = Some("home".to_string());
            self.add_favorite(fav);
        }
        
        if let Some(desktop) = dirs::desktop_dir() {
            let mut fav = Favorite::new("Desktop", &desktop);
            fav.icon = Some("desktop".to_string());
            self.add_favorite(fav);
        }
        
        if let Some(downloads) = dirs::download_dir() {
            let mut fav = Favorite::new("Downloads", &downloads);
            fav.icon = Some("arrow_download".to_string());
            self.add_favorite(fav);
        }
        
        if let Some(documents) = dirs::document_dir() {
            let mut fav = Favorite::new("Documents", &documents);
            fav.icon = Some("document".to_string());
            self.add_favorite(fav);
        }
    }
}

/// General application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Whether to show hidden files by default.
    pub show_hidden: bool,
    /// Whether to show system files.
    pub show_system: bool,
    /// Whether to confirm before deleting to Recycle Bin.
    pub confirm_delete: bool,
    /// Whether to confirm before permanent delete.
    pub confirm_permanent_delete: bool,
    /// Maximum number of history entries (back/forward).
    pub max_history: usize,
    /// Default sort field.
    pub default_sort_field: String,
    /// Default sort order (asc/desc).
    pub default_sort_ascending: bool,
    /// Starting directory (empty = last used or home).
    pub start_directory: Option<PathBuf>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            show_hidden: false,
            show_system: false,
            confirm_delete: true,
            confirm_permanent_delete: true,
            max_history: 100,
            default_sort_field: "name".to_string(),
            default_sort_ascending: true,
            start_directory: None,
        }
    }
}

/// Appearance and display settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    /// Theme name.
    pub theme: String,
    /// Whether to use icons.
    pub show_icons: bool,
    /// Date format string.
    pub date_format: String,
    /// Whether to show file extensions.
    pub show_extensions: bool,
    /// Whether to use human-readable file sizes (KB, MB, etc.).
    pub human_readable_sizes: bool,
    /// Column widths (for TUI/GUI).
    pub column_widths: ColumnWidths,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            show_icons: true,
            date_format: "%Y-%m-%d %H:%M".to_string(),
            show_extensions: true,
            human_readable_sizes: true,
            column_widths: ColumnWidths::default(),
        }
    }
}

/// Column width settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ColumnWidths {
    pub name: u16,
    pub size: u16,
    pub date: u16,
    pub kind: u16,
}

impl Default for ColumnWidths {
    fn default() -> Self {
        Self {
            name: 40,
            size: 10,
            date: 20,
            kind: 10,
        }
    }
}

/// File operation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OperationsConfig {
    /// Maximum concurrent file operations.
    pub max_concurrent_jobs: usize,
    /// Buffer size for copy operations (in KB).
    pub copy_buffer_size_kb: usize,
    /// Whether to use fast move (rename) when on same volume.
    pub fast_move_same_volume: bool,
    /// Whether to preserve timestamps when copying.
    pub preserve_timestamps: bool,
    /// Whether to follow symlinks when copying.
    pub follow_symlinks: bool,
}

impl Default for OperationsConfig {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: 2,
            copy_buffer_size_kb: 64, // 64 KB buffer
            fast_move_same_volume: true,
            preserve_timestamps: true,
            follow_symlinks: false,
        }
    }
}

/// A favorite/quick access entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    /// Unique identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Path to the favorite location.
    pub path: PathBuf,
    /// Sort order (lower = higher in list).
    pub order: u32,
    /// Optional icon name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

impl Favorite {
    /// Create a new favorite.
    /// ID is generated from name + path hash to ensure uniqueness.
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        let name = name.into();
        let path = path.into();

        // Generate ID from name + short hash of path for uniqueness
        let name_part: String = name
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect();
        
        // Create a simple hash from the path to ensure unique IDs
        let path_str = path.to_string_lossy().to_lowercase();
        let path_hash: u32 = path_str.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32).wrapping_mul(31));
        let id = format!("{}-{:x}", name_part, path_hash & 0xFFFF);

        Self {
            id,
            name,
            path,
            order: 0,
            icon: None,
        }
    }

    /// Check if the favorite path exists and is accessible.
    pub fn is_valid(&self) -> bool {
        self.path.exists()
    }

    /// Check if the favorite is broken (path doesn't exist).
    pub fn is_broken(&self) -> bool {
        !self.is_valid()
    }
}

/// Session state that can be saved/restored between runs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionState {
    /// Last active directory for the left pane.
    pub last_left_dir: Option<PathBuf>,
    /// Last active directory for the right pane (dual-pane mode).
    pub last_right_dir: Option<PathBuf>,
    /// Window size/position (for GUI).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_state: Option<WindowState>,
    /// Last used sort settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sort: Option<SortSettings>,
}

/// Window state for GUI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

/// Sort settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortSettings {
    pub field: String,
    pub ascending: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert!(!config.general.show_hidden);
        assert!(config.general.confirm_delete);
        assert_eq!(config.favorites.len(), 0);
    }

    #[test]
    fn test_config_save_load() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.toml");

        let mut config = Config::default();
        config.general.show_hidden = true;
        config.add_favorite(Favorite::new("Home", "/home/user"));

        config.save_to(&path).unwrap();
        assert!(path.exists());

        let loaded = Config::load_from(&path).unwrap();
        assert!(loaded.general.show_hidden);
        assert_eq!(loaded.favorites.len(), 1);
        assert_eq!(loaded.favorites[0].name, "Home");
    }

    #[test]
    fn test_config_creates_default() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("new_config.toml");

        assert!(!path.exists());

        let config = Config::load_from(&path).unwrap();
        assert!(path.exists());
        assert!(!config.general.show_hidden); // Default value
    }

    #[test]
    fn test_favorite_operations() {
        let mut config = Config::default();

        let fav1 = Favorite::new("Downloads", "/home/user/Downloads");
        let fav2 = Favorite::new("Documents", "/home/user/Documents");

        config.add_favorite(fav1);
        config.add_favorite(fav2);

        assert_eq!(config.favorites.len(), 2);
        assert_eq!(config.favorites[0].order, 1);
        assert_eq!(config.favorites[1].order, 2);

        // Get by ID
        assert!(config.get_favorite("downloads").is_some());
        assert!(config.get_favorite("nonexistent").is_none());

        // Remove
        assert!(config.remove_favorite("downloads"));
        assert_eq!(config.favorites.len(), 1);
        assert!(!config.remove_favorite("downloads")); // Already removed
    }

    #[test]
    fn test_favorite_reorder() {
        let mut config = Config::default();

        config.add_favorite(Favorite::new("A", "/a"));
        config.add_favorite(Favorite::new("B", "/b"));
        config.add_favorite(Favorite::new("C", "/c"));

        config.reorder_favorites(&["c".to_string(), "a".to_string(), "b".to_string()]);

        assert_eq!(config.favorites[0].id, "c");
        assert_eq!(config.favorites[1].id, "a");
        assert_eq!(config.favorites[2].id, "b");
    }

    #[test]
    fn test_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        config.general.max_history = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_toml_format() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();

        assert!(toml_str.contains("[general]"));
        assert!(toml_str.contains("show_hidden"));
        assert!(toml_str.contains("[appearance]"));
        assert!(toml_str.contains("[operations]"));
    }

    #[test]
    fn test_favorite_validation() {
        let temp = TempDir::new().unwrap();
        let existing = temp.path().join("existing");
        std::fs::create_dir(&existing).unwrap();

        let valid = Favorite::new("Existing", &existing);
        assert!(valid.is_valid());
        assert!(!valid.is_broken());

        let invalid = Favorite::new("Missing", "/nonexistent/path");
        assert!(!invalid.is_valid());
        assert!(invalid.is_broken());
    }
}
