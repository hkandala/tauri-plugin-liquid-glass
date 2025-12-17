//! Glass effect operations - create, update, remove

use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::NSRect;
use log::warn;
use objc::runtime::{Class, BOOL};
use objc::{class, msg_send, sel, sel_impl};

use tauri::{AppHandle, Manager, Runtime, WebviewWindow};

use super::backend::get_backend;
use super::registry::{GlassViewRegistry, ViewHandle};
use super::utils::{color_from_hex, run_on_main_sync};
use crate::error::{Error, Result};
use crate::models::LiquidGlassConfig;

// ============================================================================
// Constants
// ============================================================================

/// NSWindowOrderingMode
const NS_WINDOW_BELOW: i64 = -1;

// ============================================================================
// High-Level Operations
// ============================================================================

pub fn create_glass_effect<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    config: &LiquidGlassConfig,
) -> Result<()> {
    let registry = app.state::<GlassViewRegistry>();
    let window_label = window.label().to_string();

    let ns_window = window
        .ns_window()
        .map_err(|_| Error::WindowNotFound(window_label.clone()))?;

    let ns_window_handle = ViewHandle::new(ns_window as id);
    let config = config.clone();

    let (glass_view, tint_overlay) = run_on_main_sync(move || unsafe {
        create_and_attach_glass_view(ns_window_handle, &config)
    })?;

    registry.insert(window_label, glass_view, tint_overlay)?;

    Ok(())
}

pub fn update_glass_effect<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    config: &LiquidGlassConfig,
) -> Result<()> {
    let registry = app.state::<GlassViewRegistry>();
    let window_label = window.label().to_string();

    let (glass_handle, existing_tint) = registry
        .get(&window_label)?
        .ok_or_else(|| Error::WindowNotFound(window_label.clone()))?;

    let config = config.clone();

    let new_tint = run_on_main_sync(move || unsafe {
        apply_glass_config(glass_handle, &config, existing_tint)
    });

    registry.update_tint(&window_label, new_tint)?;

    Ok(())
}

pub fn remove_glass_effect<R: Runtime>(app: &AppHandle<R>, window_label: &str) -> Result<()> {
    let registry = app.state::<GlassViewRegistry>();

    let entry = registry.remove(window_label)?;

    // If no entry exists, that's fine - effect was already disabled
    if let Some((glass_handle, tint_handle)) = entry {
        run_on_main_sync(move || unsafe {
            // Remove tint overlay first (if exists)
            if let Some(tint) = tint_handle {
                let _: () = msg_send![tint.as_id(), removeFromSuperview];
            }
            // Remove glass view
            let _: () = msg_send![glass_handle.as_id(), removeFromSuperview];
        });
    }

    Ok(())
}

// ============================================================================
// Main Thread Operations
// ============================================================================

/// Creates and attaches glass view to window.
///
/// # Safety
/// - Must be called on the main thread
/// - `ns_window_handle` must point to a valid NSWindow
///
/// Returns (glass_view_handle, tint_overlay_handle)
unsafe fn create_and_attach_glass_view(
    ns_window_handle: ViewHandle,
    config: &LiquidGlassConfig,
) -> Result<(ViewHandle, Option<ViewHandle>)> {
    let ns_window = ns_window_handle.as_id();
    let content_view: id = msg_send![ns_window, contentView];

    if content_view == nil {
        return Err(Error::ViewCreationFailed);
    }

    // Check and warn about transparency settings
    check_window_transparency(ns_window);
    check_webview_transparency(content_view);

    let bounds: NSRect = msg_send![content_view, bounds];

    // Create glass view using appropriate backend
    let backend = get_backend();
    let glass_view = backend.create_view(bounds)?;

    // Configure appearance and experimental properties
    let glass_handle = ViewHandle::new(glass_view);
    let tint_overlay = apply_glass_config(glass_handle, config, None);

    // Insert into view hierarchy
    let _: () =
        msg_send![content_view, addSubview: glass_view positioned: NS_WINDOW_BELOW relativeTo: nil];

    Ok((glass_handle, tint_overlay))
}

/// Apply all configuration to glass view
///
/// # Safety
/// - Must be called on the main thread
/// - `glass_handle` must point to a valid glass effect view
///
/// Returns the tint overlay handle if one was created (for NSVisualEffectView fallback)
unsafe fn apply_glass_config(
    glass_handle: ViewHandle,
    config: &LiquidGlassConfig,
    existing_tint_overlay: Option<ViewHandle>,
) -> Option<ViewHandle> {
    let glass = glass_handle.as_id();
    let _: () = msg_send![glass, setWantsLayer: YES];
    let layer: id = msg_send![glass, layer];

    // Apply corner radius
    if layer != nil {
        let _: () = msg_send![layer, setCornerRadius: config.corner_radius];
        let _: () = msg_send![layer, setMasksToBounds: YES];
    }

    let backend = get_backend();

    // Apply or clear tint color
    let tint_overlay = if let Some(ref hex) = config.tint_color {
        if let Some(color) = color_from_hex(hex) {
            backend.apply_tint(glass, layer, color, existing_tint_overlay)
        } else {
            backend.clear_tint(glass, existing_tint_overlay);
            None
        }
    } else {
        backend.clear_tint(glass, existing_tint_overlay);
        None
    };

    // Apply variant
    backend.set_variant(glass, config.variant as i64);

    tint_overlay
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Check if window has transparency configured and warn if not
unsafe fn check_window_transparency(ns_window: id) {
    let is_opaque: BOOL = msg_send![ns_window, isOpaque];
    if is_opaque != NO {
        warn!(
            "Window is opaque. For liquid glass effect to show through, \
             set window transparency in tauri.conf.json or via window builder."
        );
    }
}

/// Check if webview has transparency and warn if not
unsafe fn check_webview_transparency(content_view: id) {
    if let Some(webview) = find_webview(content_view) {
        // Check if webview draws background
        let key: id =
            msg_send![class!(NSString), stringWithUTF8String: c"drawsBackground".as_ptr()];
        let draws_bg: id = msg_send![webview, valueForKey: key];
        if draws_bg != nil {
            let draws: BOOL = msg_send![draws_bg, boolValue];
            if draws != NO {
                warn!(
                    "WebView has background drawing enabled. For liquid glass effect to show through, \
                     set transparent background in your HTML/CSS (e.g., background: transparent)."
                );
            }
        }
    }
}

/// Find WKWebView in view hierarchy
unsafe fn find_webview(view: id) -> Option<id> {
    if view == nil {
        return None;
    }

    if let Some(webview_class) = Class::get("WKWebView") {
        let is_webview: BOOL = msg_send![view, isKindOfClass: webview_class];
        if is_webview != NO {
            return Some(view);
        }
    }

    let subviews: id = msg_send![view, subviews];
    let count: usize = msg_send![subviews, count];
    for i in 0..count {
        let subview: id = msg_send![subviews, objectAtIndex: i];
        if let Some(webview) = find_webview(subview) {
            return Some(webview);
        }
    }

    None
}

