import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LiquidGlassConfig, GlassMaterialVariant } from "./types";

export { LiquidGlassConfig, GlassMaterialVariant };

const PLUGIN_NAME = "liquid-glass";

/**
 * Check if liquid glass effect is supported on the current platform
 *
 * @returns true if running on macOS 26+ with NSGlassEffectView available
 */
export async function isGlassSupported(): Promise<boolean> {
  return invoke<boolean>(`plugin:${PLUGIN_NAME}|is_glass_supported`);
}

/**
 * Set liquid glass effect on the current window
 *
 * @param config Configuration for the glass effect. All fields are optional with sensible defaults.
 *
 * @example
 * ```typescript
 * // Enable with defaults
 * await setLiquidGlassEffect({});
 *
 * // Enable with custom settings
 * await setLiquidGlassEffect({
 *   cornerRadius: 24,
 *   tintColor: '#ffffff20'
 * });
 *
 * // Disable the effect
 * await setLiquidGlassEffect({ enabled: false });
 * ```
 */
export async function setLiquidGlassEffect(
  config: LiquidGlassConfig = {}
): Promise<void> {
  const window = getCurrentWindow();
  return invoke(`plugin:${PLUGIN_NAME}|set_liquid_glass_effect`, {
    window: window.label,
    config,
  });
}
