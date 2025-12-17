//! macOS-specific implementation for Liquid Glass effect

mod glass_effect;
mod utils;

// Re-export public API
pub use glass_effect::{
    add_glass_effect,
    configure_glass,
    is_glass_supported,
    remove_glass_effect,
    set_scrim,
    set_subdued,
    set_variant,
    GlassViewRegistry,
};
