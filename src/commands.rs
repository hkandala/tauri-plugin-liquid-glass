//! Tauri commands for the liquid-glass plugin

use tauri::{command, AppHandle, Runtime, WebviewWindow};

use crate::error::Result;
use crate::models::{GlassMaterialVariant, GlassOptions};

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

/// Add a glass effect to a window
///
/// Returns a view ID that can be used to configure or remove the effect.
/// Returns -1 on non-macOS platforms.
#[command]
pub fn add_glass_effect<R: Runtime>(
    app: AppHandle<R>,
    window: WebviewWindow<R>,
    options: Option<GlassOptions>,
) -> Result<i32> {
    #[cfg(target_os = "macos")]
    {
        macos::add_glass_effect(&app, &window, options.unwrap_or_default())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, window, options);
        Ok(-1) // No-op on non-macOS
    }
}

/// Configure an existing glass view
///
/// Allows updating corner radius and tint color after creation.
#[command]
pub fn configure_glass<R: Runtime>(
    app: AppHandle<R>,
    view_id: i32,
    corner_radius: Option<f64>,
    tint_color: Option<String>,
) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::configure_glass(&app, view_id, corner_radius, tint_color)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, view_id, corner_radius, tint_color);
        Ok(())
    }
}

/// Set the glass material variant (experimental)
///
/// This uses private Apple APIs and may change in future macOS versions.
#[command]
pub fn set_variant<R: Runtime>(
    app: AppHandle<R>,
    view_id: i32,
    variant: GlassMaterialVariant,
) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::set_variant(&app, view_id, variant)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, view_id, variant);
        Ok(())
    }
}

/// Enable or disable scrim overlay (experimental)
///
/// This uses private Apple APIs and may change in future macOS versions.
#[command]
pub fn set_scrim<R: Runtime>(app: AppHandle<R>, view_id: i32, enabled: bool) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::set_scrim(&app, view_id, enabled)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, view_id, enabled);
        Ok(())
    }
}

/// Enable or disable subdued state (experimental)
///
/// This uses private Apple APIs and may change in future macOS versions.
#[command]
pub fn set_subdued<R: Runtime>(app: AppHandle<R>, view_id: i32, enabled: bool) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::set_subdued(&app, view_id, enabled)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, view_id, enabled);
        Ok(())
    }
}

/// Remove a glass effect from a window
#[command]
pub fn remove_glass_effect<R: Runtime>(app: AppHandle<R>, view_id: i32) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::remove_glass_effect(&app, view_id)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, view_id);
        Ok(())
    }
}
