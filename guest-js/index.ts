import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { GlassOptions, GlassMaterialVariant } from "./types";

export { GlassOptions, GlassMaterialVariant };

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
 * Add a glass effect to the current window
 *
 * @param options Glass effect configuration options
 * @returns View ID for future operations (-1 if not supported)
 */
export async function addGlassEffect(options?: GlassOptions): Promise<number> {
  const window = getCurrentWindow();
  return invoke<number>(`plugin:${PLUGIN_NAME}|add_glass_effect`, {
    window: window.label,
    options,
  });
}

/**
 * Configure an existing glass view
 *
 * @param viewId The view ID returned from addGlassEffect
 * @param cornerRadius Optional corner radius in pixels
 * @param tintColor Optional hex color (#RRGGBB or #RRGGBBAA)
 */
export async function configureGlass(
  viewId: number,
  cornerRadius?: number,
  tintColor?: string
): Promise<void> {
  return invoke(`plugin:${PLUGIN_NAME}|configure_glass`, {
    viewId,
    cornerRadius,
    tintColor,
  });
}

/**
 * Remove a glass effect from a window
 *
 * @param viewId The view ID to remove
 */
export async function removeGlassEffect(viewId: number): Promise<void> {
  return invoke(`plugin:${PLUGIN_NAME}|remove_glass_effect`, {
    viewId,
  });
}

// ============================================================================
// Experimental (unstable) APIs - may change in future versions
// ============================================================================

/**
 * Set the glass material variant (experimental)
 *
 * This uses private Apple APIs and may change in future macOS versions.
 *
 * @param viewId The view ID
 * @param variant The material variant
 */
export async function unstable_setVariant(
  viewId: number,
  variant: GlassMaterialVariant
): Promise<void> {
  return invoke(`plugin:${PLUGIN_NAME}|set_variant`, {
    viewId,
    variant,
  });
}

/**
 * Enable or disable scrim overlay (experimental)
 *
 * This uses private Apple APIs and may change in future macOS versions.
 *
 * @param viewId The view ID
 * @param enabled Whether scrim is enabled
 */
export async function unstable_setScrim(
  viewId: number,
  enabled: boolean
): Promise<void> {
  return invoke(`plugin:${PLUGIN_NAME}|set_scrim`, {
    viewId,
    enabled,
  });
}

/**
 * Enable or disable subdued state (experimental)
 *
 * This uses private Apple APIs and may change in future macOS versions.
 *
 * @param viewId The view ID
 * @param enabled Whether subdued state is enabled
 */
export async function unstable_setSubdued(
  viewId: number,
  enabled: boolean
): Promise<void> {
  return invoke(`plugin:${PLUGIN_NAME}|set_subdued`, {
    viewId,
    enabled,
  });
}

// ============================================================================
// Convenience class for managing a glass view
// ============================================================================

/**
 * Convenience class for managing a single glass view
 *
 * @example
 * ```typescript
 * import liquidGlass from 'tauri-plugin-liquid-glass-api';
 *
 * // Apply glass effect
 * await liquidGlass.apply({ cornerRadius: 24, tintColor: '#ffffff20' });
 *
 * // Change variant
 * await liquidGlass.unstable_setVariant(GlassMaterialVariant.Dock);
 *
 * // Remove when done
 * await liquidGlass.remove();
 * ```
 */
export class LiquidGlass {
  private viewId: number = -1;

  /**
   * Check if glass effect is active
   */
  get isActive(): boolean {
    return this.viewId >= 0;
  }

  /**
   * Get the current view ID
   */
  get id(): number {
    return this.viewId;
  }

  /**
   * Apply glass effect to the current window
   */
  async apply(options?: GlassOptions): Promise<void> {
    if (this.isActive) {
      await this.remove();
    }
    this.viewId = await addGlassEffect(options);
  }

  /**
   * Remove the glass effect
   */
  async remove(): Promise<void> {
    if (this.isActive) {
      await removeGlassEffect(this.viewId);
      this.viewId = -1;
    }
  }

  /**
   * Update glass configuration
   */
  async configure(cornerRadius?: number, tintColor?: string): Promise<void> {
    if (this.isActive) {
      await configureGlass(this.viewId, cornerRadius, tintColor);
    }
  }

  /**
   * Set material variant (experimental)
   */
  async unstable_setVariant(variant: GlassMaterialVariant): Promise<void> {
    if (this.isActive) {
      await unstable_setVariant(this.viewId, variant);
    }
  }

  /**
   * Set scrim state (experimental)
   */
  async unstable_setScrim(enabled: boolean): Promise<void> {
    if (this.isActive) {
      await unstable_setScrim(this.viewId, enabled);
    }
  }

  /**
   * Set subdued state (experimental)
   */
  async unstable_setSubdued(enabled: boolean): Promise<void> {
    if (this.isActive) {
      await unstable_setSubdued(this.viewId, enabled);
    }
  }
}

export default new LiquidGlass();
