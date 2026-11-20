//! Minimal FFI type definitions replacing the `cocoa` crate.
//!
//! The `cocoa` crate transitively depends on `block 0.1`, which is unmaintained
//! and triggers a future-incompatibility warning ("static of uninhabited type")
//! that will become a hard error in a future Rust release. Only a handful of
//! types and constants from `cocoa` were used, so they are defined here instead.

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use objc::runtime::Object;

/// Objective-C object pointer (`id` in Objective-C).
pub type id = *mut Object;

/// The null object pointer (`nil` in Objective-C).
pub const nil: id = std::ptr::null_mut();

/// NSPoint (same layout as CGPoint)
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NSPoint {
    pub x: f64,
    pub y: f64,
}

/// NSSize (same layout as CGSize)
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NSSize {
    pub width: f64,
    pub height: f64,
}

/// NSRect (same layout as CGRect)
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NSRect {
    pub origin: NSPoint,
    pub size: NSSize,
}

// NSAutoresizingMaskOptions
pub const NSViewWidthSizable: u64 = 2;
pub const NSViewHeightSizable: u64 = 16;

/// NSVisualEffectBlendingMode
/// <https://developer.apple.com/documentation/appkit/nsvisualeffectview/blendingmode>
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
pub enum NSVisualEffectBlendingMode {
    BehindWindow = 0,
    WithinWindow = 1,
}

/// NSVisualEffectState
/// <https://developer.apple.com/documentation/appkit/nsvisualeffectview/state>
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
pub enum NSVisualEffectState {
    FollowsWindowActiveState = 0,
    Active = 1,
    Inactive = 2,
}

/// NSVisualEffectMaterial (only the variant used by this plugin)
/// <https://developer.apple.com/documentation/appkit/nsvisualeffectview/material>
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NSVisualEffectMaterial {
    UnderWindowBackground = 21,
}
