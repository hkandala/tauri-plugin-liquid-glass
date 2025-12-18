//! Glass view registry for tracking created views by window label

use std::collections::HashMap;
use std::sync::Mutex;

use cocoa::base::id;

use crate::error::{Error, Result};

// ============================================================================
// View Handle - Type-safe wrapper for raw pointer addresses
// ============================================================================

/// A thread-safe handle to an NSView stored as a raw pointer address.
///
/// # Safety
/// All actual view operations must be performed on the main thread via `run_on_main_sync`.
#[derive(Clone, Copy, Debug)]
pub struct ViewHandle(usize);

impl ViewHandle {
    /// Create a new ViewHandle from an Objective-C id
    pub fn new(view: id) -> Self {
        Self(view as usize)
    }

    /// Convert back to an Objective-C id
    ///
    /// # Safety
    /// - Must be called on the main thread
    /// - The underlying view must still be valid
    pub unsafe fn as_id(self) -> id {
        self.0 as id
    }
}

// ============================================================================
// Glass View Entry
// ============================================================================

/// Entry for tracking a glass view.
pub struct GlassViewEntry {
    pub glass_view: ViewHandle,
    /// Tint overlay view for NSVisualEffectView fallback (NSGlassEffectView has native tint support)
    pub tint_overlay: Option<ViewHandle>,
}

// SAFETY: GlassViewEntry stores ViewHandle which contains usize values (raw pointer addresses).
// All actual view operations are performed on the main thread via run_on_main_sync.
unsafe impl Send for GlassViewEntry {}
unsafe impl Sync for GlassViewEntry {}

// ============================================================================
// Glass View Registry
// ============================================================================

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

impl GlassViewRegistry {
    /// Check if a window has a registered glass view
    pub fn contains(&self, label: &str) -> Result<bool> {
        self.views
            .lock()
            .map(|views| views.contains_key(label))
            .map_err(|_| Error::RegistryLockFailed)
    }

    /// Insert a new glass view entry
    pub fn insert(
        &self,
        label: String,
        glass_view: ViewHandle,
        tint_overlay: Option<ViewHandle>,
    ) -> Result<()> {
        self.views
            .lock()
            .map(|mut views| {
                views.insert(
                    label,
                    GlassViewEntry {
                        glass_view,
                        tint_overlay,
                    },
                );
            })
            .map_err(|_| Error::RegistryLockFailed)
    }

    /// Get a glass view entry by label
    pub fn get(&self, label: &str) -> Result<Option<(ViewHandle, Option<ViewHandle>)>> {
        self.views
            .lock()
            .map(|views| views.get(label).map(|e| (e.glass_view, e.tint_overlay)))
            .map_err(|_| Error::RegistryLockFailed)
    }

    /// Remove a glass view entry and return it
    pub fn remove(&self, label: &str) -> Result<Option<(ViewHandle, Option<ViewHandle>)>> {
        self.views
            .lock()
            .map(|mut views| views.remove(label).map(|e| (e.glass_view, e.tint_overlay)))
            .map_err(|_| Error::RegistryLockFailed)
    }

    /// Update the tint overlay for an existing entry
    pub fn update_tint(&self, label: &str, tint: Option<ViewHandle>) -> Result<()> {
        self.views
            .lock()
            .map(|mut views| {
                if let Some(entry) = views.get_mut(label) {
                    entry.tint_overlay = tint;
                }
            })
            .map_err(|_| Error::RegistryLockFailed)
    }
}
