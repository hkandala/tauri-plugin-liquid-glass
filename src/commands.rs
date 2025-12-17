//! Tauri commands for the liquid-glass plugin

use tauri::{command, AppHandle, Runtime, WebviewWindow};

use crate::error::Result;
use crate::models::LiquidGlassConfig;

#[cfg(target_os = "macos")]
use crate::macos;

/// Check if liquid glass effect is supported on the current platform
///
/// Returns true if running on macOS 26+ with NSGlassEffectView available.
#[command]
pub fn is_glass_supported<R: Runtime>(_app: AppHandle<R>) -> bool {
    #[cfg(target_os = "macos")]
    {
        macos::is_glass_supported()
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
/// All configuration options have sensible defaults, so you can pass an empty object
/// to enable the effect with default settings.
#[command]
pub fn set_liquid_glass_effect<R: Runtime>(
    app: AppHandle<R>,
    window: WebviewWindow<R>,
    config: LiquidGlassConfig,
) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::set_liquid_glass_effect(&app, &window, config)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, window, config);
        Ok(()) // No-op on non-macOS
    }
}
