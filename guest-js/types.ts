/**
 * Options for configuring the glass effect
 */
export interface GlassOptions {
  /** Corner radius for the glass view in pixels */
  cornerRadius?: number;
  /** Tint color in hex format (#RRGGBB or #RRGGBBAA) */
  tintColor?: string;
  /** Whether to add an opaque background behind the glass */
  opaque?: boolean;
}

/**
 * Glass material variants for NSGlassEffectView
 *
 * These variants control the appearance of the liquid glass effect.
 * Note: These are experimental and may change in future macOS versions.
 */
export const GlassMaterialVariant = {
  Regular: 0,
  Clear: 1,
  Dock: 2,
  AppIcons: 3,
  Widgets: 4,
  Text: 5,
  Avplayer: 6,
  Facetime: 7,
  ControlCenter: 8,
  NotificationCenter: 9,
  Monogram: 10,
  Bubbles: 11,
  Identity: 12,
  FocusBorder: 13,
  FocusPlatter: 14,
  Keyboard: 15,
  Sidebar: 16,
  AbuttedSidebar: 17,
  Inspector: 18,
  Control: 19,
  Loupe: 20,
  Slider: 21,
  Camera: 22,
  CartouchePopover: 23,
} as const;

export type GlassMaterialVariant =
  (typeof GlassMaterialVariant)[keyof typeof GlassMaterialVariant];
