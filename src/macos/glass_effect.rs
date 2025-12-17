//! Core implementation for NSGlassEffectView and fallback NSVisualEffectView

use std::collections::HashMap;
use std::sync::Mutex;

use cocoa::appkit::{
    NSViewHeightSizable, NSViewWidthSizable, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState,
};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::NSRect;
use log::warn;
use objc::runtime::{Class, Object, Sel, BOOL};
use objc::{class, msg_send, sel, sel_impl};

use tauri::{AppHandle, Manager, Runtime, WebviewWindow};

use super::utils::{color_from_hex, glass_class_available, is_macos_26_or_later, run_on_main_sync};
use crate::error::{Error, Result};
use crate::models::LiquidGlassConfig;

// ============================================================================
// Constants
// ============================================================================

/// NSWindowOrderingMode
const NS_WINDOW_BELOW: i64 = -1;

/// NSAutoresizingMaskOptions (combined for convenience)
fn autoresize_mask() -> u64 {
    NSViewWidthSizable | NSViewHeightSizable
}

// ============================================================================
// Glass View Registry
// ============================================================================

/// Entry for tracking a glass view.
/// Stores raw pointer address (usize) for thread-safe storage.
struct GlassViewEntry {
    glass_view: usize,
    /// Tint overlay view for NSVisualEffectView fallback (NSGlassEffectView has native tint support)
    tint_overlay: Option<usize>,
}

// SAFETY: GlassViewEntry stores usize values (raw pointer addresses).
// All actual view operations are performed on the main thread via run_on_main_sync.
unsafe impl Send for GlassViewEntry {}
unsafe impl Sync for GlassViewEntry {}

/// Registry for tracking created glass views by window label
pub struct GlassViewRegistry {
    views: Mutex<HashMap<String, GlassViewEntry>>,
}

impl Default for GlassViewRegistry {
    fn default() -> Self {
        Self {
            views: Mutex::new(HashMap::new()),
        }
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Check if liquid glass (NSGlassEffectView) is supported
pub fn is_glass_supported() -> bool {
    run_on_main_sync(|| is_macos_26_or_later() && glass_class_available())
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
        // Check if window already has a glass effect
        let existing = registry.views.lock().unwrap().contains_key(&window_label);

        if existing {
            // Update existing glass effect
            update_glass_effect(app, window, &config)
        } else {
            // Create new glass effect
            create_glass_effect(app, window, &config)
        }
    } else {
        // Remove glass effect if present
        remove_glass_effect_internal(app, &window_label)
    }
}

// ============================================================================
// Internal - Create Glass Effect
// ============================================================================

fn create_glass_effect<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    config: &LiquidGlassConfig,
) -> Result<()> {
    let registry = app.state::<GlassViewRegistry>();
    let window_label = window.label().to_string();

    let ns_window = window
        .ns_window()
        .map_err(|_| Error::WindowNotFound(window_label.clone()))?;

    let ns_window_addr = ns_window as usize;
    let config = config.clone();

    let (glass_view_addr, tint_overlay) =
        run_on_main_sync(move || unsafe { create_and_attach_glass_view(ns_window_addr, &config) })?;

    registry.views.lock().unwrap().insert(
        window_label,
        GlassViewEntry {
            glass_view: glass_view_addr,
            tint_overlay,
        },
    );

    Ok(())
}

// ============================================================================
// Internal - Update Glass Effect
// ============================================================================

fn update_glass_effect<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    config: &LiquidGlassConfig,
) -> Result<()> {
    let registry = app.state::<GlassViewRegistry>();
    let window_label = window.label().to_string();

    let (glass_addr, existing_tint) = {
        let views = registry.views.lock().unwrap();
        let entry = views
            .get(&window_label)
            .ok_or_else(|| Error::WindowNotFound(window_label.clone()))?;
        (entry.glass_view, entry.tint_overlay)
    };

    let config = config.clone();

    let new_tint =
        run_on_main_sync(move || unsafe { apply_glass_config(glass_addr, &config, existing_tint) });

    // Update tint overlay in registry
    if let Ok(mut views) = registry.views.lock() {
        if let Some(entry) = views.get_mut(&window_label) {
            entry.tint_overlay = new_tint;
        }
    }

    Ok(())
}

// ============================================================================
// Internal - Remove Glass Effect
// ============================================================================

fn remove_glass_effect_internal<R: Runtime>(app: &AppHandle<R>, window_label: &str) -> Result<()> {
    let registry = app.state::<GlassViewRegistry>();

    let entry = registry.views.lock().unwrap().remove(window_label);

    // If no entry exists, that's fine - effect was already disabled
    if let Some(entry) = entry {
        let glass_addr = entry.glass_view;
        let tint_addr = entry.tint_overlay;

        run_on_main_sync(move || unsafe {
            // Remove tint overlay first (if exists)
            if let Some(addr) = tint_addr {
                let tint: id = addr as id;
                let _: () = msg_send![tint, removeFromSuperview];
            }
            // Remove glass view
            let glass: id = glass_addr as id;
            let _: () = msg_send![glass, removeFromSuperview];
        });
    }

    Ok(())
}

// ============================================================================
// Internal Implementation - Main Thread Operations
// ============================================================================

/// Creates and attaches glass view to window. Must be called on main thread.
/// Returns (glass_view_addr, tint_overlay_addr)
unsafe fn create_and_attach_glass_view(
    ns_window_addr: usize,
    config: &LiquidGlassConfig,
) -> Result<(usize, Option<usize>)> {
    let ns_window: id = ns_window_addr as id;
    let content_view: id = msg_send![ns_window, contentView];

    if content_view == nil {
        return Err(Error::ViewCreationFailed);
    }

    // Check and warn about transparency settings
    check_window_transparency(ns_window);
    check_webview_transparency(content_view);

    let bounds: NSRect = msg_send![content_view, bounds];

    // Create glass view based on OS version
    let glass_view = if glass_class_available() {
        create_glass_effect_view(bounds)?
    } else {
        create_visual_effect_view(bounds)
    };

    // Configure appearance and experimental properties
    let tint_overlay = apply_glass_config(glass_view as usize, config, None);

    // Insert into view hierarchy
    let _: () =
        msg_send![content_view, addSubview: glass_view positioned: NS_WINDOW_BELOW relativeTo: nil];

    Ok((glass_view as usize, tint_overlay))
}

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

// ============================================================================
// View Creation
// ============================================================================

/// Create NSGlassEffectView (macOS 26+ private API)
unsafe fn create_glass_effect_view(bounds: NSRect) -> Result<id> {
    let glass_class = Class::get("NSGlassEffectView").ok_or(Error::ViewCreationFailed)?;

    let glass: id = msg_send![glass_class, alloc];
    let glass: id = msg_send![glass, initWithFrame: bounds];
    let _: () = msg_send![glass, setAutoresizingMask: autoresize_mask()];

    Ok(glass)
}

/// Create NSVisualEffectView (fallback for older macOS)
unsafe fn create_visual_effect_view(bounds: NSRect) -> id {
    let visual: id = msg_send![class!(NSVisualEffectView), alloc];
    let visual: id = msg_send![visual, initWithFrame: bounds];

    let _: () = msg_send![visual, setAutoresizingMask: autoresize_mask()];
    let _: () = msg_send![visual, setBlendingMode: NSVisualEffectBlendingMode::BehindWindow];
    let _: () = msg_send![visual, setMaterial: NSVisualEffectMaterial::UnderWindowBackground];
    let _: () = msg_send![visual, setState: NSVisualEffectState::Active];

    visual
}

// ============================================================================
// View Configuration
// ============================================================================

/// Apply all configuration to glass view
/// Returns the tint overlay address if one was created (for NSVisualEffectView fallback)
unsafe fn apply_glass_config(
    glass_addr: usize,
    config: &LiquidGlassConfig,
    existing_tint_overlay: Option<usize>,
) -> Option<usize> {
    let glass: id = glass_addr as id;
    let _: () = msg_send![glass, setWantsLayer: YES];
    let layer: id = msg_send![glass, layer];

    // Apply corner radius
    if layer != nil {
        let _: () = msg_send![layer, setCornerRadius: config.corner_radius];
        let _: () = msg_send![layer, setMasksToBounds: YES];
    }

    // Apply or clear tint color
    let tint_overlay = if let Some(ref hex) = config.tint_color {
        if let Some(color) = color_from_hex(hex) {
            apply_tint_color(glass, layer, color, existing_tint_overlay)
        } else {
            clear_tint_color(glass, layer, existing_tint_overlay);
            None
        }
    } else {
        // Clear tint color when None
        clear_tint_color(glass, layer, existing_tint_overlay);
        None
    };

    // Apply variant (only for NSGlassEffectView)
    if glass_class_available() {
        set_view_property(glass_addr, "variant", config.variant as i64);
    }

    tint_overlay
}

/// Apply tint color to glass view
/// For NSGlassEffectView: uses native setTintColor:
/// For NSVisualEffectView: creates/updates a colored overlay subview
/// Returns the tint overlay address if one was created/updated
unsafe fn apply_tint_color(
    glass: id,
    layer: id,
    color: id,
    existing_overlay: Option<usize>,
) -> Option<usize> {
    let sel = sel!(setTintColor:);
    let responds: BOOL = msg_send![glass, respondsToSelector: sel];
    if responds != NO {
        // NSGlassEffectView - use native tint
        let _: () = msg_send![glass, setTintColor: color];
        None
    } else {
        // NSVisualEffectView fallback - use overlay subview
        let overlay: id = if let Some(addr) = existing_overlay {
            // Reuse existing overlay
            addr as id
        } else {
            // Create new overlay view
            let bounds: NSRect = msg_send![glass, bounds];
            let overlay: id = msg_send![class!(NSView), alloc];
            let overlay: id = msg_send![overlay, initWithFrame: bounds];
            let _: () = msg_send![overlay, setAutoresizingMask: autoresize_mask()];
            let _: () = msg_send![overlay, setWantsLayer: YES];
            let _: () = msg_send![glass, addSubview: overlay];
            overlay
        };

        // Apply color to overlay layer (CGColor preserves alpha for transparency)
        let overlay_layer: id = msg_send![overlay, layer];
        if overlay_layer != nil {
            let cg_color: id = msg_send![color, CGColor];
            let _: () = msg_send![overlay_layer, setBackgroundColor: cg_color];

            // Apply same corner radius as parent
            if layer != nil {
                let radius: f64 = msg_send![layer, cornerRadius];
                let _: () = msg_send![overlay_layer, setCornerRadius: radius];
                let _: () = msg_send![overlay_layer, setMasksToBounds: YES];
            }
        }

        Some(overlay as usize)
    }
}

/// Clear tint color from glass view
unsafe fn clear_tint_color(glass: id, _layer: id, existing_overlay: Option<usize>) {
    let sel = sel!(setTintColor:);
    let responds: BOOL = msg_send![glass, respondsToSelector: sel];
    if responds != NO {
        // NSGlassEffectView - clear native tint
        let _: () = msg_send![glass, setTintColor: nil];
    } else if let Some(addr) = existing_overlay {
        // NSVisualEffectView fallback - remove overlay subview
        let overlay: id = addr as id;
        let _: () = msg_send![overlay, removeFromSuperview];
    }
}

// ============================================================================
// Dynamic Property Setting (Experimental APIs)
// ============================================================================

/// Set property on view using selector lookup
unsafe fn set_view_property(view_addr: usize, key: &str, value: i64) {
    let view: *mut Object = view_addr as *mut Object;

    // Try private setter: set_<key>:
    let private_sel = Sel::register(&format!("set_{}:", key));
    if try_send_i64(view, private_sel, value) {
        return;
    }

    // Try public setter: setKey:
    let public_sel = Sel::register(&format!(
        "set{}{}:",
        key.chars().next().unwrap().to_uppercase(),
        &key[1..]
    ));
    try_send_i64(view, public_sel, value);
}

/// Try to send an i64 message to an object
unsafe fn try_send_i64(obj: *mut Object, sel: Sel, value: i64) -> bool {
    let responds: BOOL = msg_send![obj, respondsToSelector: sel];
    if responds != NO {
        let _: () = objc::__send_message(&*obj, sel, (value,)).unwrap_or(());
        true
    } else {
        false
    }
}
