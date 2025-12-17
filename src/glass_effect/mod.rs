//! Liquid Glass effect implementation for macOS
//!
//! This module provides native macOS Liquid Glass effects for Tauri applications.
//! On macOS 26 (Tahoe) and later, it uses the private NSGlassEffectView API.
//! On older macOS versions, it falls back to NSVisualEffectView.

mod backend;
mod operations;
mod registry;
mod utils;

use tauri::{AppHandle, Manager, Runtime, WebviewWindow};

use crate::error::Result;
use crate::models::LiquidGlassConfig;

// Re-export public types
pub use registry::GlassViewRegistry;

// ============================================================================
// Public API
// ============================================================================

/// Check if liquid glass (NSGlassEffectView) is supported
pub fn is_glass_supported() -> bool {
    utils::run_on_main_sync(utils::glass_class_available)
}

/// Set liquid glass effect on a window
///
/// - If `config.enabled` is true: creates or updates the glass effect
/// - If `config.enabled` is false: removes the glass effect if present
pub fn set_liquid_glass_effect<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    config: LiquidGlassConfig,
) -> Result<()> {
    let registry = app.state::<GlassViewRegistry>();
    let window_label = window.label().to_string();

    if config.enabled {
        let existing = registry.contains(&window_label)?;

        if existing {
            operations::update_glass_effect(app, window, &config)
        } else {
            operations::create_glass_effect(app, window, &config)
        }
    } else {
        operations::remove_glass_effect(app, &window_label)
    }
}
