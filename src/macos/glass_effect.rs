//! Core implementation for NSGlassEffectView and fallback NSVisualEffectView

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Mutex,
};

use cocoa::appkit::{
    NSViewHeightSizable, NSViewWidthSizable, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState,
};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::NSRect;
use objc::runtime::{Class, Object, Sel, BOOL};
use objc::{class, msg_send, sel, sel_impl};

use tauri::{AppHandle, Manager, Runtime, WebviewWindow};

use super::utils::{color_from_hex, glass_class_available, is_macos_26_or_later, run_on_main_sync};
use crate::error::{Error, Result};
use crate::models::{GlassMaterialVariant, GlassOptions};

// ============================================================================
// Constants
// ============================================================================

/// NSWindowOrderingMode
const NS_WINDOW_ABOVE: i64 = 1;
const NS_WINDOW_BELOW: i64 = -1;

/// NSBox type constants
const NS_BOX_CUSTOM: u64 = 4;
const NS_NO_BORDER: u64 = 0;

/// NSAutoresizingMaskOptions (combined for convenience)
fn autoresize_mask() -> u64 {
    NSViewWidthSizable | NSViewHeightSizable
}

// ============================================================================
// Glass View Registry
// ============================================================================

/// Entry for tracking a glass view and its associated views.
/// Stores raw pointer addresses (usize) for thread-safe storage.
struct GlassViewEntry {
    glass_view: usize,
    background_view: Option<usize>,
}

// SAFETY: GlassViewEntry stores usize values (raw pointer addresses).
// All actual view operations are performed on the main thread via run_on_main_sync.
unsafe impl Send for GlassViewEntry {}
unsafe impl Sync for GlassViewEntry {}

/// Registry for tracking created glass views by ID
pub struct GlassViewRegistry {
    views: Mutex<HashMap<i32, GlassViewEntry>>,
    next_id: AtomicI32,
}

impl Default for GlassViewRegistry {
    fn default() -> Self {
        Self {
            views: Mutex::new(HashMap::new()),
            next_id: AtomicI32::new(0),
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

/// Add a glass effect to a window
pub fn add_glass_effect<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    options: GlassOptions,
) -> Result<i32> {
    let registry = app.state::<GlassViewRegistry>();
    let view_id = registry.next_id.fetch_add(1, Ordering::SeqCst);

    let ns_window = window
        .ns_window()
        .map_err(|_| Error::WindowNotFound(window.label().to_string()))?;

    let ns_window_addr = ns_window as usize;

    let result = run_on_main_sync(move || unsafe {
        create_and_attach_glass_view(ns_window_addr, &options)
    })?;

    registry.views.lock().unwrap().insert(
        view_id,
        GlassViewEntry {
            glass_view: result.0,
            background_view: result.1,
        },
    );

    Ok(view_id)
}

/// Configure an existing glass view
pub fn configure_glass<R: Runtime>(
    app: &AppHandle<R>,
    view_id: i32,
    corner_radius: Option<f64>,
    tint_color: Option<String>,
) -> Result<()> {
    let registry = app.state::<GlassViewRegistry>();

    let views = registry.views.lock().unwrap();
    let entry = views.get(&view_id).ok_or(Error::ViewNotFound(view_id))?;
    let glass_addr = entry.glass_view;
    let bg_addr = entry.background_view;
    drop(views);

    run_on_main_sync(move || unsafe {
        update_glass_config(glass_addr, bg_addr, corner_radius, tint_color);
    });

    Ok(())
}

/// Set glass material variant (experimental)
pub fn set_variant<R: Runtime>(
    app: &AppHandle<R>,
    view_id: i32,
    variant: GlassMaterialVariant,
) -> Result<()> {
    set_glass_property(app, view_id, "variant", variant as i64)
}

/// Set scrim state (experimental)
pub fn set_scrim<R: Runtime>(app: &AppHandle<R>, view_id: i32, enabled: bool) -> Result<()> {
    set_glass_property(app, view_id, "scrim", i64::from(enabled))
}

/// Set subdued state (experimental)
pub fn set_subdued<R: Runtime>(app: &AppHandle<R>, view_id: i32, enabled: bool) -> Result<()> {
    set_glass_property(app, view_id, "subdued", i64::from(enabled))
}

/// Remove a glass effect from a window
pub fn remove_glass_effect<R: Runtime>(app: &AppHandle<R>, view_id: i32) -> Result<()> {
    let registry = app.state::<GlassViewRegistry>();

    let entry = registry
        .views
        .lock()
        .unwrap()
        .remove(&view_id)
        .ok_or(Error::ViewNotFound(view_id))?;

    let glass_addr = entry.glass_view;
    let bg_addr = entry.background_view;

    run_on_main_sync(move || unsafe {
        let glass: id = glass_addr as id;
        let _: () = msg_send![glass, removeFromSuperview];

        if let Some(addr) = bg_addr {
            let bg: id = addr as id;
            let _: () = msg_send![bg, removeFromSuperview];
        }
    });

    Ok(())
}

// ============================================================================
// Internal Implementation - Main Thread Operations
// ============================================================================

/// Creates and attaches glass view to window. Must be called on main thread.
unsafe fn create_and_attach_glass_view(
    ns_window_addr: usize,
    options: &GlassOptions,
) -> Result<(usize, Option<usize>)> {
    let ns_window: id = ns_window_addr as id;
    let content_view: id = msg_send![ns_window, contentView];

    if content_view == nil {
        return Err(Error::ViewCreationFailed);
    }

    // Configure window for transparency
    configure_window_transparency(ns_window);

    // Make webview transparent so glass shows through
    make_webview_transparent(content_view);

    let bounds: NSRect = msg_send![content_view, bounds];

    // Create glass view based on OS version
    let (glass_view, background_view) = if glass_class_available() {
        create_glass_effect_view(bounds, options)?
    } else {
        create_visual_effect_view(bounds)
    };

    // Configure appearance
    configure_view_appearance(glass_view, background_view, options);

    // Insert into view hierarchy
    insert_glass_into_hierarchy(content_view, glass_view, background_view);

    Ok((glass_view as usize, background_view.map(|v| v as usize)))
}

/// Configure NSWindow for full transparency
unsafe fn configure_window_transparency(ns_window: id) {
    let _: () = msg_send![ns_window, setOpaque: NO];

    let clear_color: id = msg_send![class!(NSColor), clearColor];
    let _: () = msg_send![ns_window, setBackgroundColor: clear_color];
    let _: () = msg_send![ns_window, setHasShadow: YES];

    let content_view: id = msg_send![ns_window, contentView];
    if content_view != nil {
        let _: () = msg_send![content_view, setWantsLayer: YES];
        let layer: id = msg_send![content_view, layer];
        if layer != nil {
            let cg_color: id = msg_send![clear_color, CGColor];
            let _: () = msg_send![layer, setBackgroundColor: cg_color];
        }
    }
}

/// Recursively find and make WKWebView transparent
unsafe fn make_webview_transparent(view: id) {
    if view == nil {
        return;
    }

    if let Some(webview_class) = Class::get("WKWebView") {
        let is_webview: BOOL = msg_send![view, isKindOfClass: webview_class];
        if is_webview != NO {
            set_webview_transparent(view);
            return;
        }
    }

    let subviews: id = msg_send![view, subviews];
    let count: usize = msg_send![subviews, count];
    for i in 0..count {
        let subview: id = msg_send![subviews, objectAtIndex: i];
        make_webview_transparent(subview);
    }
}

/// Set WKWebView to transparent background
unsafe fn set_webview_transparent(webview: id) {
    // Try private API (most reliable)
    let sel_draws_bg = Sel::register("_setDrawsBackground:");
    let responds: BOOL = msg_send![webview, respondsToSelector: sel_draws_bg];
    if responds != NO {
        let _: () = msg_send![webview, _setDrawsBackground: NO];
    }

    // Try setValue:forKey: as backup
    let key: id = msg_send![class!(NSString), stringWithUTF8String: c"drawsBackground".as_ptr()];
    let no_val: id = msg_send![class!(NSNumber), numberWithBool: NO];
    let _: () = msg_send![webview, setValue: no_val forKey: key];

    // Set layer background to clear
    let _: () = msg_send![webview, setWantsLayer: YES];
    let layer: id = msg_send![webview, layer];
    if layer != nil {
        let clear: id = msg_send![class!(NSColor), clearColor];
        let cg_color: id = msg_send![clear, CGColor];
        let _: () = msg_send![layer, setBackgroundColor: cg_color];
    }
}

// ============================================================================
// View Creation
// ============================================================================

/// Create NSGlassEffectView (macOS 26+ private API)
unsafe fn create_glass_effect_view(
    bounds: NSRect,
    options: &GlassOptions,
) -> Result<(id, Option<id>)> {
    let glass_class = Class::get("NSGlassEffectView").ok_or(Error::ViewCreationFailed)?;

    let glass: id = msg_send![glass_class, alloc];
    let glass: id = msg_send![glass, initWithFrame: bounds];
    let _: () = msg_send![glass, setAutoresizingMask: autoresize_mask()];

    let background = if options.opaque {
        Some(create_background_box(bounds))
    } else {
        None
    };

    Ok((glass, background))
}

/// Create NSVisualEffectView (fallback for older macOS)
unsafe fn create_visual_effect_view(bounds: NSRect) -> (id, Option<id>) {
    let visual: id = msg_send![class!(NSVisualEffectView), alloc];
    let visual: id = msg_send![visual, initWithFrame: bounds];

    let _: () = msg_send![visual, setAutoresizingMask: autoresize_mask()];
    let _: () = msg_send![visual, setBlendingMode: NSVisualEffectBlendingMode::BehindWindow];
    let _: () = msg_send![visual, setMaterial: NSVisualEffectMaterial::UnderWindowBackground];
    let _: () = msg_send![visual, setState: NSVisualEffectState::Active];

    (visual, None)
}

/// Create NSBox for opaque background mode
unsafe fn create_background_box(bounds: NSRect) -> id {
    let bg: id = msg_send![class!(NSBox), alloc];
    let bg: id = msg_send![bg, initWithFrame: bounds];

    let _: () = msg_send![bg, setAutoresizingMask: autoresize_mask()];
    let _: () = msg_send![bg, setBoxType: NS_BOX_CUSTOM];
    let _: () = msg_send![bg, setBorderType: NS_NO_BORDER];

    let color: id = msg_send![class!(NSColor), windowBackgroundColor];
    let _: () = msg_send![bg, setFillColor: color];
    let _: () = msg_send![bg, setWantsLayer: YES];

    bg
}

// ============================================================================
// View Configuration
// ============================================================================

/// Configure view appearance (corner radius, tint color)
unsafe fn configure_view_appearance(glass: id, background: Option<id>, options: &GlassOptions) {
    let _: () = msg_send![glass, setWantsLayer: YES];
    let layer: id = msg_send![glass, layer];

    if layer != nil {
        let _: () = msg_send![layer, setCornerRadius: options.corner_radius];
        let _: () = msg_send![layer, setMasksToBounds: YES];
    }

    if let Some(bg) = background {
        let _: () = msg_send![bg, setWantsLayer: YES];
        let bg_layer: id = msg_send![bg, layer];
        if bg_layer != nil {
            let _: () = msg_send![bg_layer, setCornerRadius: options.corner_radius];
            let _: () = msg_send![bg_layer, setMasksToBounds: YES];
        }
    }

    if let Some(ref hex) = options.tint_color {
        if let Some(color) = color_from_hex(hex) {
            apply_tint_color(glass, layer, color);
        }
    }
}

/// Apply tint color to glass view
unsafe fn apply_tint_color(glass: id, layer: id, color: id) {
    let sel = sel!(setTintColor:);
    let responds: BOOL = msg_send![glass, respondsToSelector: sel];
    if responds != NO {
        let _: () = msg_send![glass, setTintColor: color];
    } else if layer != nil {
        let cg_color: id = msg_send![color, CGColor];
        let _: () = msg_send![layer, setBackgroundColor: cg_color];
    }
}

/// Insert glass view into view hierarchy
unsafe fn insert_glass_into_hierarchy(
    content_view: id,
    glass_view: id,
    background_view: Option<id>,
) {
    if let Some(bg) = background_view {
        let _: () =
            msg_send![content_view, addSubview: bg positioned: NS_WINDOW_BELOW relativeTo: nil];
        let _: () = msg_send![content_view, addSubview: glass_view positioned: NS_WINDOW_ABOVE relativeTo: bg];
    } else {
        let _: () = msg_send![content_view, addSubview: glass_view positioned: NS_WINDOW_BELOW relativeTo: nil];
    }
}

/// Update glass configuration
unsafe fn update_glass_config(
    glass_addr: usize,
    bg_addr: Option<usize>,
    corner_radius: Option<f64>,
    tint_color: Option<String>,
) {
    let glass: id = glass_addr as id;
    let layer: id = msg_send![glass, layer];

    if let Some(radius) = corner_radius {
        if layer != nil {
            let _: () = msg_send![layer, setCornerRadius: radius];
        }

        if let Some(addr) = bg_addr {
            let bg: id = addr as id;
            let bg_layer: id = msg_send![bg, layer];
            if bg_layer != nil {
                let _: () = msg_send![bg_layer, setCornerRadius: radius];
            }
        }
    }

    if let Some(ref hex) = tint_color {
        if let Some(color) = color_from_hex(hex) {
            apply_tint_color(glass, layer, color);
        }
    }
}

// ============================================================================
// Dynamic Property Setting (Experimental APIs)
// ============================================================================

/// Set an integer property on a glass view
fn set_glass_property<R: Runtime>(
    app: &AppHandle<R>,
    view_id: i32,
    key: &str,
    value: i64,
) -> Result<()> {
    let registry = app.state::<GlassViewRegistry>();

    let views = registry.views.lock().unwrap();
    let entry = views.get(&view_id).ok_or(Error::ViewNotFound(view_id))?;
    let glass_addr = entry.glass_view;
    drop(views);

    let key = key.to_string();

    run_on_main_sync(move || unsafe {
        set_view_property(glass_addr, &key, value);
    });

    Ok(())
}

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
