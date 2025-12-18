# Liquid Glass Example App

This example demonstrates how to add the **Liquid Glass** effect to a Tauri application, starting from the standard `create-tauri-app` boilerplate.

> **Note:** The Liquid Glass effect requires **macOS 26+ (Tahoe)** and uses private Apple APIs. On older macOS versions, the plugin falls back to `NSVisualEffectView`.

## Quick Start

```bash
cd examples/liquid-glass-app
pnpm install
pnpm tauri dev
```

## Step-by-Step Guide: Adding Liquid Glass to Your Tauri App

This guide shows the minimal changes needed to enable the liquid glass effect in a standard Tauri React TypeScript app.

---

### Step 1: Update Rust Dependencies (`src-tauri/Cargo.toml`)

Add the liquid glass plugin and enable the `macos-private-api` feature for Tauri:

```toml
[dependencies]
# Enable "macos-private-api" feature for transparent windows
tauri = { version = "2", features = ["macos-private-api"] }

# Add the liquid glass plugin
tauri-plugin-liquid-glass = "0.1"
```

---

### Step 2: Register the Plugin (`src-tauri/src/lib.rs`)

Register the liquid glass plugin in your Tauri builder:

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Add this line to register the liquid glass plugin
        .plugin(tauri_plugin_liquid_glass::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### Step 3: Configure the Window (`src-tauri/tauri.conf.json`)

Enable macOS private API and set the window to be transparent:

```json
{
  "app": {
    "macOSPrivateApi": true,
    "windows": [
      {
        "title": "Your App",
        "width": 800,
        "height": 600,
        "transparent": true,
        "decorations": true,
        "titleBarStyle": "Transparent"
      }
    ]
  }
}
```

---

### Step 4: Add Plugin Permissions (`src-tauri/capabilities/default.json`)

Grant the liquid glass plugin permissions:

```json
{
  "permissions": [
    "core:default",
    "opener:default",
    "liquid-glass:default"
  ]
}
```

---

### Step 5: Make HTML Background Transparent (`index.html`)

Add styles to ensure the HTML/body backgrounds are transparent:

```html
<style>
  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }
  html, body {
    background: transparent !important;
    background-color: transparent !important;
    height: 100%;
  }
  #root {
    height: 100%;
    background: transparent !important;
    background-color: transparent !important;
  }
</style>
```

**Why?**
- The liquid glass effect renders behind the webview
- All layers must be transparent for the effect to be visible

---

### Step 6: Update CSS for Glass Compatibility (`src/App.css`)

Modify your CSS to work with the glass background:

```css
:root {
  /* Use light text colors that work on glass backgrounds */
  color: #ffffff;
  /* Background must be transparent */
  background-color: transparent;
}

/* Add text shadows for better readability */
h1, p {
  text-shadow: 0 1px 3px rgba(0,0,0,0.3);
}

/* Use semi-transparent backgrounds for UI elements */
input, button {
  background-color: rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.2);
}
```

**Tips:**
- Use white or light text colors
- Add text shadows for readability on varying backgrounds
- Use semi-transparent backgrounds with backdrop blur for nested glass effects

---

### Step 7: Apply the Glass Effect (`src/App.tsx`)

Call the plugin to apply the liquid glass effect:

```tsx
import { useEffect } from "react";
import {
  setLiquidGlassEffect,
  isGlassSupported,
  GlassMaterialVariant,
} from "tauri-plugin-liquid-glass-api";

function App() {
  useEffect(() => {
    const init = async () => {
      // Check if liquid glass is supported (macOS 26+)
      const supported = await isGlassSupported();
      console.log("Liquid Glass supported:", supported);

      // Apply the glass effect
      await setLiquidGlassEffect({
        cornerRadius: 24,  // Match your window's corner radius
        // tintColor: "#ffffff30",  // Optional: add a color tint overlay
        // variant: GlassMaterialVariant.Sidebar,  // Optional: material variant (macOS 26+ only)
      });
    };
    init();
  }, []);

  return (
    // Your app content...
  );
}
```

**LiquidGlassConfig Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | `boolean` | `true` | Whether the glass effect is enabled |
| `cornerRadius` | `number` | `0` | Corner radius of the glass effect in pixels |
| `tintColor` | `string` | - | Hex color with alpha (#RRGGBB or #RRGGBBAA) |
| `variant` | `GlassMaterialVariant` | `Regular` | Material variant (macOS 26+ only) |

**Available Functions:**

| Function | Description |
|----------|-------------|
| `isGlassSupported()` | Returns `true` if running on macOS 26+ |
| `setLiquidGlassEffect(config)` | Apply, update, or remove the glass effect |

To disable the effect:

```tsx
await setLiquidGlassEffect({ enabled: false });
```

---

## Summary of Changes

| File | Change |
|------|--------|
| `src-tauri/Cargo.toml` | Add `macos-private-api` feature and liquid-glass plugin |
| `src-tauri/src/lib.rs` | Register `.plugin(tauri_plugin_liquid_glass::init())` |
| `src-tauri/tauri.conf.json` | Add `macOSPrivateApi: true` and `transparent: true` |
| `src-tauri/capabilities/default.json` | Add `liquid-glass:default` permission |
| `index.html` | Make html/body/root backgrounds transparent |
| `src/App.css` | Use transparent backgrounds and light colors |
| `src/App.tsx` | Call `setLiquidGlassEffect()` on mount |

---

## Troubleshooting

### Glass effect not visible?

1. Ensure you're running macOS 26+ (Tahoe) for the full effect, or macOS 10.10+ for the fallback
2. Check that all backgrounds are transparent (html, body, #root, containers)
3. Verify `macOSPrivateApi: true` in `tauri.conf.json`
4. Verify `transparent: true` in window config

### Build errors?

1. Make sure `macos-private-api` feature is enabled in Cargo.toml for tauri
2. Ensure the plugin is registered in lib.rs
3. Check that `liquid-glass:default` permission is added to capabilities

### Effect looks wrong?

- Try adjusting the `cornerRadius` to match your window
- The effect is best seen when dragging the window over varying backgrounds
- Try different `variant` values to see which works best for your app (macOS 26+ only)

### Variant selection disabled in example app?

- This means `isGlassSupported()` returned `false`
- You're likely running on macOS < 26, so the fallback `NSVisualEffectView` is being used
- Variants are only supported on macOS 26+ with `NSGlassEffectView`
