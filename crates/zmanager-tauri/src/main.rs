//! ZManager Tauri Application Entry Point
//!
//! This is the binary entry point for the ZManager GUI application.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Initialize tracing for debug builds
    #[cfg(debug_assertions)]
    {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "zmanager_tauri_lib=debug,zmanager_core=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    zmanager_tauri_lib::run();
}
