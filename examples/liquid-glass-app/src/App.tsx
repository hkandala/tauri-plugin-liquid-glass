import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

const VARIANT_NAMES = [
  "Regular", "Clear", "Dock", "AppIcons", "Widgets", "Text",
  "Avplayer", "Facetime", "ControlCenter", "NotificationCenter",
  "Monogram", "Bubbles", "Identity", "FocusBorder", "FocusPlatter",
  "Keyboard", "Sidebar", "AbuttedSidebar", "Inspector", "Control",
  "Loupe", "Slider", "Camera", "CartouchePopover",
];

function App() {
  const [viewId, setViewId] = useState<number>(-1);
  const [cornerRadius, setCornerRadius] = useState(24);
  const [tintEnabled, setTintEnabled] = useState(false);
  const [tintColor, setTintColor] = useState("#ffffff30");
  const [variant, setVariant] = useState(2);
  const [collapsed, setCollapsed] = useState(false);

  useEffect(() => {
    applyGlass();
  }, []);

  const applyGlass = async (reset = false) => {
    try {
      if (viewId >= 0) {
        await invoke("plugin:liquid-glass|remove_glass_effect", { viewId });
      }

      // Use default values if resetting, otherwise use current state
      const radius = reset ? 24 : cornerRadius;
      const tint = reset ? false : tintEnabled;
      const color = reset ? "#ffffff30" : tintColor;

      if (reset) {
        setCornerRadius(24);
        setTintEnabled(false);
        setTintColor("#ffffff30");
        setVariant(2);
      }

      const id = await invoke<number>("plugin:liquid-glass|add_glass_effect", {
        options: {
          cornerRadius: radius,
          ...(tint && { tintColor: color }),
        },
      });
      setViewId(id);

      // Apply the variant (default is Dock = 2)
      const variantToApply = reset ? 2 : variant;
      await invoke("plugin:liquid-glass|set_variant", {
        viewId: id,
        variant: variantToApply,
      });
    } catch (e) {
      console.error(e);
    }
  };

  const updateCornerRadius = async (value: number) => {
    setCornerRadius(value);
    if (viewId < 0) return;
    try {
      await invoke("plugin:liquid-glass|configure_glass", {
        viewId,
        cornerRadius: value,
        ...(tintEnabled && { tintColor }),
      });
    } catch (e) {
      console.error(e);
    }
  };

  const updateTintEnabled = async (enabled: boolean) => {
    setTintEnabled(enabled);
    if (viewId < 0) return;
    try {
      await invoke("plugin:liquid-glass|configure_glass", {
        viewId,
        cornerRadius,
        tintColor: enabled ? tintColor : "#00000000",
      });
    } catch (e) {
      console.error(e);
    }
  };

  const updateTintColor = async (value: string) => {
    setTintColor(value);
    if (viewId < 0 || !tintEnabled) return;
    try {
      await invoke("plugin:liquid-glass|configure_glass", {
        viewId,
        cornerRadius,
        tintColor: value,
      });
    } catch (e) {
      console.error(e);
    }
  };

  const updateVariant = async (newVariant: number) => {
    setVariant(newVariant);
    if (viewId < 0) return;
    try {
      await invoke("plugin:liquid-glass|set_variant", {
        viewId,
        variant: newVariant,
      });
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="container">
      {/* Center hint text */}
      <div className="hint-container">
        <div className="hint-icon">✦</div>
        <div className="hint-title">Liquid Glass</div>
        <div className="hint-text">
          Drag the title bar to see the effect
        </div>
      </div>

      {/* Floating control panel */}
      <div className="panel">
        <div className="panel-header" onClick={() => setCollapsed(!collapsed)}>
          <span className="panel-title">Controls</span>
          <span className="collapse-icon">{collapsed ? "+" : "−"}</span>
        </div>

        {!collapsed && (
          <div className="panel-content">
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

            <label className="label">
              <span>Variant</span>
              <select
                value={variant}
                onChange={(e) => updateVariant(Number(e.target.value))}
                className="select"
              >
                {VARIANT_NAMES.map((name, i) => (
                  <option key={i} value={i}>{name}</option>
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

            <button className="button" onClick={() => applyGlass(true)}>
              Reset
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
