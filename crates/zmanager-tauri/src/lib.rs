//! ZManager Tauri Library
//!
//! Backend for the ZManager GUI built with Tauri v2.

mod commands;

use std::sync::Mutex;

/// Configure Tauri with ZManager commands.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_drag::init())
        .manage(Mutex::new(commands::ClipboardState::default()))
        .invoke_handler(tauri::generate_handler![
            // Directory operations
            commands::zmanager_list_dir,
            commands::zmanager_get_drives,
            commands::zmanager_get_parent,
            commands::zmanager_navigate,
            commands::zmanager_delete_entries,
            commands::zmanager_rename_entry,
            commands::zmanager_create_folder,
            commands::zmanager_create_file,
            commands::zmanager_open_file,
            commands::zmanager_get_properties,
            // Favorites (Sprint 16)
            commands::zmanager_get_favorites,
            commands::zmanager_add_favorite,
            commands::zmanager_remove_favorite,
            commands::zmanager_reorder_favorites,
            // Clipboard (Sprint 16)
            commands::zmanager_clipboard_copy,
            commands::zmanager_clipboard_cut,
            commands::zmanager_clipboard_get,
            commands::zmanager_clipboard_paste,
            commands::zmanager_clipboard_clear,
        ])
        .setup(|_app| {
            tracing::info!("ZManager GUI starting...");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
