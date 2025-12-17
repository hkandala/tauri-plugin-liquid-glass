//! Tauri plugin for macOS 26+ Liquid Glass effect support
//!
//! This plugin provides native macOS Liquid Glass effects for Tauri applications.
//! On macOS 26 (Tahoe) and later, it uses the private NSGlassEffectView API.
//! On older macOS versions, it falls back to NSVisualEffectView.
//!
//! # Example
//!
//! ```rust,no_run
//! tauri::Builder::default()
//!     .plugin(tauri_plugin_liquid_glass::init())
//!     .run(tauri::generate_context!())
//!     .expect("error while running tauri application");
//! ```

// The cocoa/objc crates are deprecated in favor of objc2, but objc2 requires
// significant architectural changes (MainThreadMarker, strict Send/Sync) without
// functional benefit. These crates remain fully functional for our use case.
#![allow(deprecated)]

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod commands;
mod error;
mod models;

#[cfg(target_os = "macos")]
mod macos;

pub use error::Error;
pub use models::*;

/// Initialize the liquid-glass plugin
///
/// # Example
///
/// ```rust,no_run
/// tauri::Builder::default()
///     .plugin(tauri_plugin_liquid_glass::init())
///     .run(tauri::generate_context!())
///     .expect("error while running tauri application");
/// ```
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("liquid-glass")
        .invoke_handler(tauri::generate_handler![
            commands::is_glass_supported,
            commands::add_glass_effect,
            commands::configure_glass,
            commands::set_variant,
            commands::set_scrim,
            commands::set_subdued,
            commands::remove_glass_effect,
        ])
        .setup(|app, _api| {
            #[cfg(target_os = "macos")]
            {
                app.manage(macos::GlassViewRegistry::default());
            }
            Ok(())
        })
        .build()
}
