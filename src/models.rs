use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Options for configuring the glass effect
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlassOptions {
    /// Corner radius for the glass view in pixels
    #[serde(default)]
    pub corner_radius: f64,

    /// Tint color in hex format (#RRGGBB or #RRGGBBAA)
    #[serde(default)]
    pub tint_color: Option<String>,

    /// Whether to add an opaque background behind the glass
    #[serde(default)]
    pub opaque: bool,
}

/// Glass material variants for NSGlassEffectView
///
/// These variants control the appearance of the liquid glass effect.
/// Note: These are experimental and may change in future macOS versions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(i64)]
pub enum GlassMaterialVariant {
    #[default]
    Regular = 0,
    Clear = 1,
    Dock = 2,
    AppIcons = 3,
    Widgets = 4,
    Text = 5,
    Avplayer = 6,
    Facetime = 7,
    ControlCenter = 8,
    NotificationCenter = 9,
    Monogram = 10,
    Bubbles = 11,
    Identity = 12,
    FocusBorder = 13,
    FocusPlatter = 14,
    Keyboard = 15,
    Sidebar = 16,
    AbuttedSidebar = 17,
    Inspector = 18,
    Control = 19,
    Loupe = 20,
    Slider = 21,
    Camera = 22,
    CartouchePopover = 23,
}
