//! Desktop implementation of the Liquid Glass plugin
//!
//! This module provides the `LiquidGlass` struct that exposes the plugin's Rust API.

use tauri::{AppHandle, Runtime, WebviewWindow};

use crate::error::Result;
use crate::models::LiquidGlassConfig;

#[cfg(target_os = "macos")]
use crate::glass_effect;

/// Liquid Glass plugin API
///
/// Access this struct through the [`LiquidGlassExt`](crate::LiquidGlassExt) trait:
///
/// ```rust,no_run
/// use tauri_plugin_liquid_glass::LiquidGlassExt;
///
/// // In a Tauri command or setup hook:
/// fn example(app: tauri::AppHandle, window: tauri::WebviewWindow) {
///     let supported = app.liquid_glass().is_supported();
///     app.liquid_glass().set_effect(&window, Default::default()).unwrap();
/// }
/// ```
pub struct LiquidGlass<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
}

impl<R: Runtime> LiquidGlass<R> {
    pub(crate) fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }

    /// Check if liquid glass effect is supported on the current platform
    ///
    /// Returns true if running on macOS 26+ with NSGlassEffectView available.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_plugin_liquid_glass::LiquidGlassExt;
    ///
    /// fn check_support(app: tauri::AppHandle) {
    ///     let supported = app.liquid_glass().is_supported();
    ///     println!("Liquid Glass supported: {}", supported);
    /// }
    /// ```
    pub fn is_supported(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            glass_effect::is_glass_supported()
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    /// Set liquid glass effect on a window
    ///
    /// - If `config.enabled` is true: creates or updates the glass effect with the given configuration
    /// - If `config.enabled` is false: removes the glass effect if present
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_plugin_liquid_glass::{LiquidGlassExt, LiquidGlassConfig, GlassMaterialVariant};
    ///
    /// fn apply_glass(app: tauri::AppHandle, window: tauri::WebviewWindow) {
    ///     // Enable with default settings
    ///     app.liquid_glass().set_effect(&window, Default::default()).unwrap();
    ///
    ///     // Enable with custom settings
    ///     app.liquid_glass().set_effect(&window, LiquidGlassConfig {
    ///         corner_radius: 24.0,
    ///         tint_color: Some("#ffffff20".into()),
    ///         variant: GlassMaterialVariant::Sidebar,
    ///         ..Default::default()
    ///     }).unwrap();
    ///
    ///     // Disable
    ///     app.liquid_glass().set_effect(&window, LiquidGlassConfig {
    ///         enabled: false,
    ///         ..Default::default()
    ///     }).unwrap();
    /// }
    /// ```
    pub fn set_effect(&self, window: &WebviewWindow<R>, config: LiquidGlassConfig) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            glass_effect::set_liquid_glass_effect(&self.app, window, config)
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (window, config);
            Ok(()) // No-op on non-macOS
        }
    }
}
