import { useState, useEffect } from "react";

import {
  setLiquidGlassEffect,
  isGlassSupported,
  GlassMaterialVariant,
  type LiquidGlassConfig,
} from "tauri-plugin-liquid-glass-api";

import "./App.css";

const VARIANT_NAMES = [
  "Regular",
  "Clear",
  "Dock",
  "AppIcons",
  "Widgets",
  "Text",
  "Avplayer",
  "Facetime",
  "ControlCenter",
  "NotificationCenter",
  "Monogram",
  "Bubbles",
  "Identity",
  "FocusBorder",
  "FocusPlatter",
  "Keyboard",
  "Sidebar",
  "AbuttedSidebar",
  "Inspector",
  "Control",
  "Loupe",
  "Slider",
  "Camera",
  "CartouchePopover",
];

function App() {
  const [supported, setSupported] = useState<boolean | null>(null);
  const [cornerRadius, setCornerRadius] = useState(50);
  const [tintEnabled, setTintEnabled] = useState(false);
  const [tintColor, setTintColor] = useState("#ffffff30");
  const [variant, setVariant] = useState<LiquidGlassConfig["variant"]>(
    GlassMaterialVariant.Bubbles
  );
  const [collapsed, setCollapsed] = useState(true);

  // Apply glass effect whenever settings change
  const applyGlass = async (settings?: {
    cornerRadius?: number;
    tintEnabled?: boolean;
    tintColor?: string;
    variant?: LiquidGlassConfig["variant"];
  }) => {
    const radius = settings?.cornerRadius ?? cornerRadius;
    const tint = settings?.tintEnabled ?? tintEnabled;
    const color = settings?.tintColor ?? tintColor;
    const v = settings?.variant ?? variant;

    try {
      await setLiquidGlassEffect({
        cornerRadius: radius,
        variant: v,
        ...(tint && { tintColor: color }),
      });
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    const init = async () => {
      const isSupported = await isGlassSupported();
      setSupported(isSupported);
      await applyGlass();
    };
    init();
  }, []);

  const updateCornerRadius = async (value: number) => {
    setCornerRadius(value);
    await applyGlass({ cornerRadius: value });
  };

  const updateTintEnabled = async (enabled: boolean) => {
    setTintEnabled(enabled);
    await applyGlass({ tintEnabled: enabled });
  };

  const updateTintColor = async (value: string) => {
    setTintColor(value);
    if (tintEnabled) {
      await applyGlass({ tintColor: value });
    }
  };

  const updateVariant = async (newVariant: LiquidGlassConfig["variant"]) => {
    setVariant(newVariant);
    await applyGlass({ variant: newVariant });
  };

  const reset = async () => {
    const defaults = {
      cornerRadius: 50,
      tintEnabled: false,
      tintColor: "#ffffff30",
      variant: GlassMaterialVariant.Bubbles,
    };
    setCornerRadius(defaults.cornerRadius);
    setTintEnabled(defaults.tintEnabled);
    setTintColor(defaults.tintColor);
    setVariant(defaults.variant);
    await applyGlass(defaults);
  };

  const variantDisabled = supported === false;

  return (
    <div className="container">
      {/* Center hint text */}
      <div className="hint-container">
        <div className="hint-icon">✦</div>
        <div className="hint-title">Liquid Glass</div>
        <div className="hint-text">
          move the window around to see the effect
        </div>
      </div>

      {/* Floating control panel */}
      <div className={`panel ${collapsed ? "collapsed" : ""}`}>
        <div className="panel-header" onClick={() => setCollapsed(!collapsed)}>
          {!collapsed && <span className="panel-title">Controls</span>}
          <span className="collapse-icon">{collapsed ? "+" : "−"}</span>
        </div>

        {!collapsed && (
          <div className="panel-content">
            {variantDisabled && (
              <div className="fallback-notice">
                Liquid Glass not supported. Using NSVisualEffectView fallback.
                Variant selection is disabled.
              </div>
            )}

            <label className="label">
              <span>Radius: {cornerRadius}</span>
              <input
                type="range"
                min="0"
                max="50"
                value={cornerRadius}
                onChange={(e) => updateCornerRadius(Number(e.target.value))}
                className="slider"
              />
            </label>

            <label className={`label ${variantDisabled ? "disabled" : ""}`}>
              <span>Variant</span>
              <select
                value={variant}
                onChange={(e) =>
                  updateVariant(
                    Number(e.target.value) as LiquidGlassConfig["variant"]
                  )
                }
                className="select"
                disabled={variantDisabled}
              >
                {VARIANT_NAMES.map((name, i) => (
                  <option key={i} value={i}>
                    {name}
                  </option>
                ))}
              </select>
            </label>

            <div className="tint-section">
              <label className="checkbox">
                <input
                  type="checkbox"
                  checked={tintEnabled}
                  onChange={(e) => updateTintEnabled(e.target.checked)}
                />
                <span>Tint Overlay</span>
              </label>
              {tintEnabled && (
                <input
                  type="text"
                  value={tintColor}
                  onChange={(e) => updateTintColor(e.target.value)}
                  placeholder="#RRGGBBAA"
                  className="input"
                />
              )}
            </div>

            <button className="button" onClick={reset}>
              Reset
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
