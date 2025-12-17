//! macOS-specific implementation for Liquid Glass effect

mod glass_effect;
mod utils;

// Re-export public API
pub use glass_effect::{is_glass_supported, set_liquid_glass_effect, GlassViewRegistry};
