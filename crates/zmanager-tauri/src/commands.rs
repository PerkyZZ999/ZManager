//! Tauri commands for the ZManager GUI.
//!
//! These commands are exposed to the frontend via `invoke()`.
//! All commands follow the `zmanager_*` naming convention.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use zmanager_core::{
    list_directory, list_drives as core_list_drives, DirListing, DriveInfo as CoreDriveInfo,
    DriveType, FilterSpec, SortSpec, Config, Favorite,
};

/// Response wrapper for IPC commands.
/// Follows { ok: bool, data?, error? } pattern per IPC_Contract.md.
#[derive(Debug, Clone, Serialize)]
pub struct IpcResponse<T> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> IpcResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(error.into()),
        }
    }
}

/// List directory contents with optional sorting and filtering.
#[tauri::command]
pub async fn zmanager_list_dir(
    path: String,
    sort: Option<SortSpec>,
    filter: Option<FilterSpec>,
) -> IpcResponse<DirListing> {
    tracing::debug!("list_dir called for: {}", path);

    // Use zmanager-core's list_directory function
    match list_directory(&path, sort.as_ref(), filter.as_ref()) {
        Ok(listing) => IpcResponse::success(listing),
        Err(e) => {
            tracing::error!("Failed to list directory {}: {}", path, e);
            IpcResponse::failure(e.to_string())
        }
    }
}

/// Drive information for the frontend.
/// Serialized version of zmanager-core's DriveInfo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfoDto {
    pub path: String,
    pub label: String,
    pub total_bytes: Option<u64>,
    pub free_bytes: Option<u64>,
    pub drive_type: String,
    pub file_system: Option<String>,
    pub is_ready: bool,
}

impl From<CoreDriveInfo> for DriveInfoDto {
    fn from(info: CoreDriveInfo) -> Self {
        let drive_type = match info.drive_type {
            DriveType::Fixed => "Fixed",
            DriveType::Removable => "Removable",
            DriveType::Network => "Network",
            DriveType::CdRom => "CdRom",
            DriveType::RamDisk => "RamDisk",
            DriveType::Unknown => "Unknown",
            DriveType::NoRootDir => "NoRootDir",
        };

        Self {
            path: info.path.to_string_lossy().to_string(),
            label: info.label,
            total_bytes: info.total_bytes,
            free_bytes: info.free_bytes,
            drive_type: drive_type.to_string(),
            file_system: info.file_system,
            is_ready: info.is_ready,
        }
    }
}

/// Get available drives on the system.
#[tauri::command]
pub async fn zmanager_get_drives() -> IpcResponse<Vec<DriveInfoDto>> {
    tracing::debug!("get_drives called");

    match core_list_drives() {
        Ok(drives) => {
            let dtos: Vec<DriveInfoDto> = drives.into_iter().map(DriveInfoDto::from).collect();
            IpcResponse::success(dtos)
        }
        Err(e) => {
            tracing::error!("Failed to list drives: {}", e);
            IpcResponse::failure(e.to_string())
        }
    }
}

/// Get parent directory path.
#[tauri::command]
pub async fn zmanager_get_parent(path: String) -> IpcResponse<Option<String>> {
    let path_buf = PathBuf::from(&path);
    let parent = path_buf.parent().map(|p| p.to_string_lossy().to_string());
    IpcResponse::success(parent)
}

/// Navigate to a directory and get its contents.
#[tauri::command]
pub async fn zmanager_navigate(
    path: String,
    sort: Option<SortSpec>,
    filter: Option<FilterSpec>,
) -> IpcResponse<DirListing> {
    tracing::debug!("navigate called for: {}", path);

    // Validate path exists
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return IpcResponse::failure(format!("Path does not exist: {}", path));
    }

    if !path_buf.is_dir() {
        return IpcResponse::failure(format!("Not a directory: {}", path));
    }

    // List the directory
    zmanager_list_dir(path, sort, filter).await
}

// ============================================================================
// File Operations - Sprint 14
// ============================================================================

/// Delete response with success count and any errors
#[derive(Debug, Clone, Serialize)]
pub struct DeleteResult {
    pub deleted: u32,
    pub failed: u32,
    pub errors: Vec<String>,
}

/// Delete files/folders to the Recycle Bin.
/// Uses shell operation for safe deletion.
#[tauri::command]
pub async fn zmanager_delete_entries(paths: Vec<String>) -> IpcResponse<DeleteResult> {
    tracing::debug!("delete_entries called for {} items", paths.len());

    if paths.is_empty() {
        return IpcResponse::failure("No paths provided");
    }

    // Use zmanager-core's move_multiple_to_recycle_bin
    let results = zmanager_core::move_multiple_to_recycle_bin(&paths);
    
    let mut deleted = 0u32;
    let mut failed = 0u32;
    let mut errors = Vec::new();
    
    for (idx, result) in results.into_iter().enumerate() {
        match result {
            Ok(()) => deleted += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("{}: {}", paths[idx], e));
            }
        }
    }
    
    tracing::info!("Deleted {} items, {} failed", deleted, failed);
    IpcResponse::success(DeleteResult { deleted, failed, errors })
}

/// Rename a file or folder.
#[tauri::command]
pub async fn zmanager_rename_entry(path: String, new_name: String) -> IpcResponse<String> {
    tracing::debug!("rename_entry: {} -> {}", path, new_name);

    // Validate new name doesn't contain path separators
    if new_name.contains('/') || new_name.contains('\\') {
        return IpcResponse::failure("New name cannot contain path separators");
    }

    if new_name.is_empty() {
        return IpcResponse::failure("New name cannot be empty");
    }

    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return IpcResponse::failure(format!("Path does not exist: {}", path));
    }

    // Get parent directory and construct new path
    let parent = match path_buf.parent() {
        Some(p) => p,
        None => return IpcResponse::failure("Cannot rename root path"),
    };

    let new_path = parent.join(&new_name);

    if new_path.exists() {
        return IpcResponse::failure(format!("A file or folder named '{}' already exists", new_name));
    }

    // Perform rename
    match std::fs::rename(&path_buf, &new_path) {
        Ok(()) => {
            let new_path_str = new_path.to_string_lossy().to_string();
            tracing::info!("Renamed {} -> {}", path, new_path_str);
            IpcResponse::success(new_path_str)
        }
        Err(e) => {
            tracing::error!("Failed to rename {}: {}", path, e);
            IpcResponse::failure(e.to_string())
        }
    }
}

/// Create a new folder.
#[tauri::command]
pub async fn zmanager_create_folder(parent: String, name: String) -> IpcResponse<String> {
    tracing::debug!("create_folder: {} in {}", name, parent);

    // Validate name
    if name.contains('/') || name.contains('\\') {
        return IpcResponse::failure("Folder name cannot contain path separators");
    }

    if name.is_empty() {
        return IpcResponse::failure("Folder name cannot be empty");
    }

    let parent_path = PathBuf::from(&parent);
    if !parent_path.exists() {
        return IpcResponse::failure(format!("Parent directory does not exist: {}", parent));
    }

    if !parent_path.is_dir() {
        return IpcResponse::failure(format!("Parent is not a directory: {}", parent));
    }

    let new_path = parent_path.join(&name);

    if new_path.exists() {
        return IpcResponse::failure(format!("A file or folder named '{}' already exists", name));
    }

    // Create the folder
    match std::fs::create_dir(&new_path) {
        Ok(()) => {
            let new_path_str = new_path.to_string_lossy().to_string();
            tracing::info!("Created folder: {}", new_path_str);
            IpcResponse::success(new_path_str)
        }
        Err(e) => {
            tracing::error!("Failed to create folder {}: {}", new_path.display(), e);
            IpcResponse::failure(e.to_string())
        }
    }
}

/// Open a file or folder with the default application.
#[tauri::command]
pub async fn zmanager_open_file(path: String) -> IpcResponse<()> {
    tracing::debug!("open_file: {}", path);

    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return IpcResponse::failure(format!("Path does not exist: {}", path));
    }

    // Use the open crate or shell execute
    match open::that(&path) {
        Ok(()) => {
            tracing::info!("Opened: {}", path);
            IpcResponse::success(())
        }
        Err(e) => {
            tracing::error!("Failed to open {}: {}", path, e);
            IpcResponse::failure(e.to_string())
        }
    }
}

/// File properties response
#[derive(Debug, Clone, Serialize)]
pub struct FileProperties {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub is_readonly: bool,
    pub is_hidden: bool,
    pub is_system: bool,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub accessed: Option<String>,
}

/// Get properties of a file or folder.
#[tauri::command]
pub async fn zmanager_get_properties(path: String) -> IpcResponse<FileProperties> {
    tracing::debug!("get_properties: {}", path);

    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return IpcResponse::failure(format!("Path does not exist: {}", path));
    }

    let metadata = match std::fs::metadata(&path_buf) {
        Ok(m) => m,
        Err(e) => return IpcResponse::failure(format!("Failed to get metadata: {}", e)),
    };

    let name = path_buf
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());

    // Get timestamps
    let created = metadata.created().ok().map(|t| {
        chrono::DateTime::<chrono::Utc>::from(t)
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string()
    });

    let modified = metadata.modified().ok().map(|t| {
        chrono::DateTime::<chrono::Utc>::from(t)
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string()
    });

    let accessed = metadata.accessed().ok().map(|t| {
        chrono::DateTime::<chrono::Utc>::from(t)
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string()
    });

    // Get Windows-specific attributes
    #[cfg(windows)]
    let (is_readonly, is_hidden, is_system) = {
        use std::os::windows::fs::MetadataExt;
        let attrs = metadata.file_attributes();
        const FILE_ATTRIBUTE_READONLY: u32 = 0x1;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
        (
            attrs & FILE_ATTRIBUTE_READONLY != 0,
            attrs & FILE_ATTRIBUTE_HIDDEN != 0,
            attrs & FILE_ATTRIBUTE_SYSTEM != 0,
        )
    };

    #[cfg(not(windows))]
    let (is_readonly, is_hidden, is_system) = {
        (metadata.permissions().readonly(), name.starts_with('.'), false)
    };

    IpcResponse::success(FileProperties {
        path,
        name,
        size: metadata.len(),
        is_dir: metadata.is_dir(),
        is_readonly,
        is_hidden,
        is_system,
        created,
        modified,
        accessed,
    })
}

// ============================================================================
// Favorites Management - Sprint 16
// ============================================================================

/// Favorite DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteDto {
    pub id: String,
    pub name: String,
    pub path: String,
    pub order: u32,
    pub icon: Option<String>,
    pub is_valid: bool,
}

impl From<&Favorite> for FavoriteDto {
    fn from(fav: &Favorite) -> Self {
        Self {
            id: fav.id.clone(),
            name: fav.name.clone(),
            path: fav.path.to_string_lossy().to_string(),
            order: fav.order,
            icon: fav.icon.clone(),
            is_valid: fav.is_valid(),
        }
    }
}

/// Get all favorites
#[tauri::command]
pub async fn zmanager_get_favorites() -> IpcResponse<Vec<FavoriteDto>> {
    tracing::debug!("get_favorites called");
    
    match Config::load() {
        Ok(config) => {
            let favorites: Vec<FavoriteDto> = config.favorites.iter().map(FavoriteDto::from).collect();
            IpcResponse::success(favorites)
        }
        Err(e) => {
            tracing::error!("Failed to load config: {}", e);
            IpcResponse::failure(e.to_string())
        }
    }
}

/// Add a new favorite
#[tauri::command]
pub async fn zmanager_add_favorite(name: String, path: String, icon: Option<String>) -> IpcResponse<FavoriteDto> {
    tracing::debug!("add_favorite: {} at {}", name, path);
    
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return IpcResponse::failure(format!("Path does not exist: {}", path));
    }
    
    let mut favorite = Favorite::new(&name, &path_buf);
    favorite.icon = icon;
    
    match Config::load() {
        Ok(mut config) => {
            config.add_favorite(favorite.clone());
            if let Err(e) = config.save() {
                return IpcResponse::failure(format!("Failed to save config: {}", e));
            }
            IpcResponse::success(FavoriteDto::from(&favorite))
        }
        Err(e) => IpcResponse::failure(e.to_string()),
    }
}

/// Remove a favorite by ID
#[tauri::command]
pub async fn zmanager_remove_favorite(id: String) -> IpcResponse<bool> {
    tracing::debug!("remove_favorite: {}", id);
    
    match Config::load() {
        Ok(mut config) => {
            let removed = config.remove_favorite(&id);
            if removed {
                if let Err(e) = config.save() {
                    return IpcResponse::failure(format!("Failed to save config: {}", e));
                }
            }
            IpcResponse::success(removed)
        }
        Err(e) => IpcResponse::failure(e.to_string()),
    }
}

/// Reorder favorites
#[tauri::command]
pub async fn zmanager_reorder_favorites(ids: Vec<String>) -> IpcResponse<()> {
    tracing::debug!("reorder_favorites: {:?}", ids);
    
    match Config::load() {
        Ok(mut config) => {
            // Update order based on position in ids array
            for (idx, id) in ids.iter().enumerate() {
                if let Some(fav) = config.favorites.iter_mut().find(|f| &f.id == id) {
                    fav.order = idx as u32;
                }
            }
            // Sort favorites by order
            config.favorites.sort_by_key(|f| f.order);
            
            if let Err(e) = config.save() {
                return IpcResponse::failure(format!("Failed to save config: {}", e));
            }
            IpcResponse::success(())
        }
        Err(e) => IpcResponse::failure(e.to_string()),
    }
}

// ============================================================================
// Clipboard Operations - Sprint 16
// ============================================================================

/// Clipboard operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardOperation {
    Copy,
    Cut,
}

/// Clipboard state stored in app state
#[derive(Debug, Clone, Default)]
pub struct ClipboardState {
    pub paths: Vec<PathBuf>,
    pub operation: Option<ClipboardOperation>,
}

/// Clipboard DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardDto {
    pub paths: Vec<String>,
    pub operation: Option<ClipboardOperation>,
}

/// Copy files to clipboard
#[tauri::command]
pub fn zmanager_clipboard_copy(
    paths: Vec<String>,
    state: tauri::State<'_, std::sync::Mutex<ClipboardState>>,
) -> Result<(), String> {
    tracing::debug!("clipboard_copy: {} items", paths.len());
    
    let path_bufs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    
    let mut clipboard = state.lock().map_err(|e| e.to_string())?;
    clipboard.paths = path_bufs;
    clipboard.operation = Some(ClipboardOperation::Copy);
    
    Ok(())
}

/// Cut files to clipboard
#[tauri::command]
pub fn zmanager_clipboard_cut(
    paths: Vec<String>,
    state: tauri::State<'_, std::sync::Mutex<ClipboardState>>,
) -> Result<(), String> {
    tracing::debug!("clipboard_cut: {} items", paths.len());
    
    let path_bufs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    
    let mut clipboard = state.lock().map_err(|e| e.to_string())?;
    clipboard.paths = path_bufs;
    clipboard.operation = Some(ClipboardOperation::Cut);
    
    Ok(())
}

/// Get clipboard contents
#[tauri::command]
pub fn zmanager_clipboard_get(
    state: tauri::State<'_, std::sync::Mutex<ClipboardState>>,
) -> Result<ClipboardDto, String> {
    let clipboard = state.lock().map_err(|e| e.to_string())?;
    let paths: Vec<String> = clipboard.paths.iter().map(|p| p.to_string_lossy().to_string()).collect();
    Ok(ClipboardDto {
        paths,
        operation: clipboard.operation.clone(),
    })
}

/// Paste files from clipboard to destination
#[tauri::command]
pub fn zmanager_clipboard_paste(
    destination: String,
    state: tauri::State<'_, std::sync::Mutex<ClipboardState>>,
) -> Result<u32, String> {
    tracing::debug!("clipboard_paste to: {}", destination);
    
    let dest_path = PathBuf::from(&destination);
    if !dest_path.is_dir() {
        return Err(format!("Destination is not a directory: {}", destination));
    }
    
    let (paths, operation) = {
        let clipboard = state.lock().map_err(|e| e.to_string())?;
        (clipboard.paths.clone(), clipboard.operation.clone())
    };
    
    if paths.is_empty() {
        return Err("Clipboard is empty".to_string());
    }
    
    let operation = match operation {
        Some(op) => op,
        None => return Err("No clipboard operation set".to_string()),
    };
    
    let mut success_count = 0u32;
    
    for src_path in &paths {
        let file_name = match src_path.file_name() {
            Some(n) => n,
            None => continue,
        };
        
        let dest_file = dest_path.join(file_name);
        
        // Skip if source and destination are the same
        if src_path == &dest_file {
            continue;
        }
        
        let result = match operation {
            ClipboardOperation::Copy => {
                if src_path.is_dir() {
                    copy_dir_recursive(src_path, &dest_file)
                } else {
                    std::fs::copy(src_path, &dest_file).map(|_| ())
                }
            }
            ClipboardOperation::Cut => {
                std::fs::rename(src_path, &dest_file)
            }
        };
        
        match result {
            Ok(()) => success_count += 1,
            Err(e) => tracing::error!("Failed to paste {}: {}", src_path.display(), e),
        }
    }
    
    // Clear clipboard after cut operation
    if matches!(operation, ClipboardOperation::Cut) {
        let mut clipboard = state.lock().map_err(|e| e.to_string())?;
        clipboard.paths.clear();
        clipboard.operation = None;
    }
    
    tracing::info!("Pasted {} items", success_count);
    Ok(success_count)
}

/// Clear the clipboard
#[tauri::command]
pub fn zmanager_clipboard_clear(
    state: tauri::State<'_, std::sync::Mutex<ClipboardState>>,
) -> Result<(), String> {
    let mut clipboard = state.lock().map_err(|e| e.to_string())?;
    clipboard.paths.clear();
    clipboard.operation = None;
    Ok(())
}

/// Create a new empty file.
#[tauri::command]
pub async fn zmanager_create_file(parent: String, name: String) -> IpcResponse<String> {
    tracing::debug!("create_file: {} in {}", name, parent);

    // Validate name
    if name.contains('/') || name.contains('\\') {
        return IpcResponse::failure("File name cannot contain path separators");
    }

    if name.is_empty() {
        return IpcResponse::failure("File name cannot be empty");
    }

    let parent_path = PathBuf::from(&parent);
    if !parent_path.exists() {
        return IpcResponse::failure(format!("Parent directory does not exist: {}", parent));
    }

    if !parent_path.is_dir() {
        return IpcResponse::failure(format!("Parent is not a directory: {}", parent));
    }

    let new_path = parent_path.join(&name);

    if new_path.exists() {
        return IpcResponse::failure(format!("A file named '{}' already exists", name));
    }

    // Create the file
    match std::fs::File::create(&new_path) {
        Ok(_) => {
            let new_path_str = new_path.to_string_lossy().to_string();
            tracing::info!("Created file: {}", new_path_str);
            IpcResponse::success(new_path_str)
        }
        Err(e) => {
            tracing::error!("Failed to create file {}: {}", new_path.display(), e);
            IpcResponse::failure(e.to_string())
        }
    }
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &PathBuf, dest: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(dest)?;
    
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }
    
    Ok(())
}
