# Liquid Glass Example App

This example demonstrates how to add the **Liquid Glass** effect to a Tauri application, starting from the standard `create-tauri-app` boilerplate.

> **Note:** The Liquid Glass effect requires **macOS 26+ (Tahoe)** and uses private Apple APIs.

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
tauri-plugin-liquid-glass = "0.1"  # or use path for local development
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
        "decorations": true
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
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [glassViewId, setGlassViewId] = useState<number>(-1);

  // Apply glass effect on mount
  useEffect(() => {
    applyGlassEffect();
  }, []);

  const applyGlassEffect = async () => {
    try {
      // Remove existing effect if re-applying
      if (glassViewId >= 0) {
        await invoke("plugin:liquid-glass|remove_glass_effect", { 
          viewId: glassViewId 
        });
      }

      // Apply the liquid glass effect
      const id = await invoke<number>("plugin:liquid-glass|add_glass_effect", {
        options: {
          cornerRadius: 24,           // Match your window's corner radius
          // tintColor: "#ffffff30",  // Optional: add a color tint overlay
        },
      });

      setGlassViewId(id);
    } catch (error) {
      console.error("Failed to apply glass effect:", error);
    }
  };

  return (
    // Your app content...
  );
}
```

**Available Options:**
- `cornerRadius` - The corner radius of the glass effect (default: 0)
- `tintColor` - Optional hex color with alpha for a tint overlay (format: `#RRGGBBAA`)

**Available Commands:**
- `plugin:liquid-glass|add_glass_effect` - Apply the glass effect, returns a view ID
- `plugin:liquid-glass|remove_glass_effect` - Remove the effect using the view ID
- `plugin:liquid-glass|configure_glass` - Update options (cornerRadius, tintColor)
- `plugin:liquid-glass|set_variant` - Change the glass variant style

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
| `src/App.tsx` | Call `add_glass_effect` on mount |

---

## Troubleshooting

### Glass effect not visible?
1. Ensure you're running macOS 26+ (Tahoe)
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
