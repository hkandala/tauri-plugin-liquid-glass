//! Glass backend implementations for different macOS versions

use cocoa::appkit::{
    NSViewHeightSizable, NSViewWidthSizable, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState,
};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::NSRect;
use objc::runtime::{Class, Object, Sel, BOOL};
use objc::{class, msg_send, sel, sel_impl};

use super::registry::ViewHandle;
use super::utils::glass_class_available;
use crate::error::{Error, Result};

// ============================================================================
// Constants
// ============================================================================

/// NSAutoresizingMaskOptions (combined for convenience)
fn autoresize_mask() -> u64 {
    NSViewWidthSizable | NSViewHeightSizable
}

// ============================================================================
// Glass Backend Trait - Strategy Pattern for Glass View Types
// ============================================================================

/// Backend trait for creating and configuring glass effect views.
///
/// This abstracts the differences between NSGlassEffectView (macOS 26+)
/// and NSVisualEffectView (fallback for older versions).
///
/// # Safety
/// All methods must be called on the main thread.
pub trait GlassBackend {
    /// Create a new glass effect view with the given bounds
    ///
    /// # Safety
    /// Must be called on the main thread
    unsafe fn create_view(&self, bounds: NSRect) -> Result<id>;

    /// Apply tint color to the glass view
    ///
    /// Returns the tint overlay handle if one was created (for NSVisualEffectView fallback)
    ///
    /// # Safety
    /// - Must be called on the main thread
    /// - `view` and `layer` must be valid Objective-C objects
    unsafe fn apply_tint(
        &self,
        view: id,
        layer: id,
        color: id,
        existing_overlay: Option<ViewHandle>,
    ) -> Option<ViewHandle>;

    /// Clear tint color from the glass view
    ///
    /// # Safety
    /// - Must be called on the main thread
    /// - `view` must be a valid Objective-C object
    unsafe fn clear_tint(&self, view: id, existing_overlay: Option<ViewHandle>);

    /// Set the glass material variant
    ///
    /// # Safety
    /// - Must be called on the main thread
    /// - `view` must be a valid Objective-C object
    unsafe fn set_variant(&self, view: id, variant: i64);
}

// ============================================================================
// Native Glass Backend (macOS 26+ NSGlassEffectView)
// ============================================================================

/// Backend implementation using NSGlassEffectView (macOS 26+)
struct NativeGlassBackend;

impl GlassBackend for NativeGlassBackend {
    unsafe fn create_view(&self, bounds: NSRect) -> Result<id> {
        let glass_class = Class::get("NSGlassEffectView").ok_or(Error::ViewCreationFailed)?;

        let glass: id = msg_send![glass_class, alloc];
        let glass: id = msg_send![glass, initWithFrame: bounds];
        let _: () = msg_send![glass, setAutoresizingMask: autoresize_mask()];

        Ok(glass)
    }

    unsafe fn apply_tint(
        &self,
        view: id,
        _layer: id,
        color: id,
        _existing_overlay: Option<ViewHandle>,
    ) -> Option<ViewHandle> {
        // NSGlassEffectView has native tint support
        let _: () = msg_send![view, setTintColor: color];
        None
    }

    unsafe fn clear_tint(&self, view: id, _existing_overlay: Option<ViewHandle>) {
        let _: () = msg_send![view, setTintColor: nil];
    }

    unsafe fn set_variant(&self, view: id, variant: i64) {
        set_view_property(view, "variant", variant);
    }
}

// ============================================================================
// Visual Effect Backend (Fallback for older macOS)
// ============================================================================

/// Backend implementation using NSVisualEffectView (fallback)
struct VisualEffectBackend;

impl GlassBackend for VisualEffectBackend {
    unsafe fn create_view(&self, bounds: NSRect) -> Result<id> {
        let visual: id = msg_send![class!(NSVisualEffectView), alloc];
        let visual: id = msg_send![visual, initWithFrame: bounds];

        let _: () = msg_send![visual, setAutoresizingMask: autoresize_mask()];
        let _: () = msg_send![visual, setBlendingMode: NSVisualEffectBlendingMode::BehindWindow];
        let _: () = msg_send![visual, setMaterial: NSVisualEffectMaterial::UnderWindowBackground];
        let _: () = msg_send![visual, setState: NSVisualEffectState::Active];

        Ok(visual)
    }

    unsafe fn apply_tint(
        &self,
        view: id,
        layer: id,
        color: id,
        existing_overlay: Option<ViewHandle>,
    ) -> Option<ViewHandle> {
        // NSVisualEffectView doesn't support tint - use overlay subview
        let overlay: id = if let Some(handle) = existing_overlay {
            // Reuse existing overlay
            handle.as_id()
        } else {
            // Create new overlay view
            let bounds: NSRect = msg_send![view, bounds];
            let overlay: id = msg_send![class!(NSView), alloc];
            let overlay: id = msg_send![overlay, initWithFrame: bounds];
            let _: () = msg_send![overlay, setAutoresizingMask: autoresize_mask()];
            let _: () = msg_send![overlay, setWantsLayer: YES];
            let _: () = msg_send![view, addSubview: overlay];
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

        Some(ViewHandle::new(overlay))
    }

    unsafe fn clear_tint(&self, _view: id, existing_overlay: Option<ViewHandle>) {
        if let Some(handle) = existing_overlay {
            let overlay = handle.as_id();
            let _: () = msg_send![overlay, removeFromSuperview];
        }
    }

    unsafe fn set_variant(&self, _view: id, _variant: i64) {
        // NSVisualEffectView doesn't support variants - no-op
    }
}

// ============================================================================
// Backend Selection
// ============================================================================

/// Get the appropriate glass backend for the current macOS version
pub fn get_backend() -> Box<dyn GlassBackend> {
    if glass_class_available() {
        Box::new(NativeGlassBackend)
    } else {
        Box::new(VisualEffectBackend)
    }
}

// ============================================================================
// Dynamic Property Setting (Experimental APIs)
// ============================================================================

/// Set property on view using selector lookup
///
/// # Safety
/// - Must be called on the main thread
/// - `view` must be a valid Objective-C object
unsafe fn set_view_property(view: id, key: &str, value: i64) {
    let obj = view;

    // Try private setter: set_<key>:
    let private_sel = Sel::register(&format!("set_{}:", key));
    if try_send_i64(obj, private_sel, value) {
        return;
    }

    // Try public setter: setKey:
    let public_sel = Sel::register(&format!(
        "set{}{}:",
        key.chars().next().unwrap().to_uppercase(),
        &key[1..]
    ));
    try_send_i64(obj, public_sel, value);
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
