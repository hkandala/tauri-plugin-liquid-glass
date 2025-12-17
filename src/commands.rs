//! Tauri commands for the liquid-glass plugin

use tauri::{command, AppHandle, Runtime, WebviewWindow};

use crate::error::Result;
use crate::models::LiquidGlassConfig;
use crate::LiquidGlassExt;

/// Check if liquid glass effect is supported on the current platform
///
/// Returns true if running on macOS 26+ with NSGlassEffectView available.
#[command]
pub fn is_glass_supported<R: Runtime>(app: AppHandle<R>) -> bool {
    app.liquid_glass().is_supported()
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
    app.liquid_glass().set_effect(&window, config)
}
