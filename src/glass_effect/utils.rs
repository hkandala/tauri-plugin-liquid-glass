//! Utility functions for macOS native code

use cocoa::base::id;
use dispatch::Queue;
use objc::runtime::{Class, BOOL};
use objc::{class, msg_send, sel, sel_impl};

/// Execute a closure on the main thread synchronously.
///
/// This is necessary because all NSView operations must be performed on the main thread.
/// If already on the main thread, the closure is executed directly.
pub fn run_on_main_sync<F, R>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    if is_main_thread() {
        f()
    } else {
        use std::sync::mpsc;
        let (tx, rx) = mpsc::channel();

        Queue::main().exec_async(move || {
            let result = f();
            let _ = tx.send(result);
        });

        rx.recv()
            .expect("Failed to receive result from main thread")
    }
}

/// Check if the current thread is the main thread
fn is_main_thread() -> bool {
    unsafe {
        let is_main: BOOL = msg_send![class!(NSThread), isMainThread];
        is_main != cocoa::base::NO
    }
}

/// Parse hex color string to NSColor
///
/// Supports #RRGGBB and #RRGGBBAA formats
pub fn color_from_hex(hex: &str) -> Option<id> {
    let hex = hex.trim().trim_start_matches('#');

    if hex.len() != 6 && hex.len() != 8 {
        return None;
    }

    let rgba = u32::from_str_radix(hex, 16).ok()?;

    let (r, g, b, a) = if hex.len() == 6 {
        (
            ((rgba >> 16) & 0xFF) as f64 / 255.0,
            ((rgba >> 8) & 0xFF) as f64 / 255.0,
            (rgba & 0xFF) as f64 / 255.0,
            1.0,
        )
    } else {
        (
            ((rgba >> 24) & 0xFF) as f64 / 255.0,
            ((rgba >> 16) & 0xFF) as f64 / 255.0,
            ((rgba >> 8) & 0xFF) as f64 / 255.0,
            (rgba & 0xFF) as f64 / 255.0,
        )
    };

    unsafe {
        let color: id = msg_send![
            class!(NSColor),
            colorWithRed: r
            green: g
            blue: b
            alpha: a
        ];
        Some(color)
    }
}

/// Check if NSGlassEffectView class is available
pub fn glass_class_available() -> bool {
    Class::get("NSGlassEffectView").is_some()
}
