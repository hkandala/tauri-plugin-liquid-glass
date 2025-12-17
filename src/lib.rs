//! Tauri plugin for macOS 26+ Liquid Glass effect support
//!
//! This plugin provides native macOS Liquid Glass effects for Tauri applications.
//! On macOS 26 (Tahoe) and later, it uses the private NSGlassEffectView API.
//! On older macOS versions, it falls back to NSVisualEffectView.
//!
//! # Example
//!
//! ```rust,no_run
//! use tauri_liquid_glass::LiquidGlassExt;
//!
//! tauri::Builder::default()
//!     .plugin(tauri_liquid_glass::init())
//!     .setup(|app| {
//!         // Access the plugin API via extension trait
//!         let supported = app.liquid_glass().is_supported();
//!         println!("Liquid Glass supported: {}", supported);
//!         Ok(())
//!     })
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
mod desktop;
mod error;
mod models;

#[cfg(target_os = "macos")]
mod glass_effect;

pub use desktop::LiquidGlass;
pub use error::{Error, Result};
pub use models::*;

// ============================================================================
// Extension Trait
// ============================================================================

/// Extension trait for accessing the Liquid Glass plugin API
///
/// This trait is implemented for all types that implement [`Manager`],
/// including [`AppHandle`](tauri::AppHandle), [`App`](tauri::App), and [`WebviewWindow`](tauri::WebviewWindow).
///
/// # Example
///
/// ```rust,no_run
/// use tauri_liquid_glass::LiquidGlassExt;
///
/// fn example(app: tauri::AppHandle, window: tauri::WebviewWindow) {
///     // Check if supported
///     let supported = app.liquid_glass().is_supported();
///
///     // Apply effect
///     app.liquid_glass().set_effect(&window, Default::default()).unwrap();
/// }
/// ```
pub trait LiquidGlassExt<R: Runtime> {
    /// Returns a handle to the Liquid Glass plugin API
    fn liquid_glass(&self) -> &LiquidGlass<R>;
}

impl<R: Runtime, T: Manager<R>> LiquidGlassExt<R> for T {
    fn liquid_glass(&self) -> &LiquidGlass<R> {
        self.state::<LiquidGlass<R>>().inner()
    }
}

// ============================================================================
// Plugin Initialization
// ============================================================================

/// Initialize the liquid-glass plugin
///
/// # Example
///
/// ```rust,no_run
/// use tauri_liquid_glass::LiquidGlassExt;
///
/// tauri::Builder::default()
///     .plugin(tauri_liquid_glass::init())
///     .setup(|app| {
///         // Use the extension trait to access the API
///         let supported = app.liquid_glass().is_supported();
///         Ok(())
///     })
///     .run(tauri::generate_context!())
///     .expect("error while running tauri application");
/// ```
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("liquid-glass")
        .invoke_handler(tauri::generate_handler![
            commands::is_glass_supported,
            commands::set_liquid_glass_effect,
        ])
        .setup(|app, _api| {
            // Manage the LiquidGlass struct for the extension trait
            app.manage(LiquidGlass::new(app.clone()));

            #[cfg(target_os = "macos")]
            {
                app.manage(glass_effect::GlassViewRegistry::default());
            }
            Ok(())
        })
        .build()
}
