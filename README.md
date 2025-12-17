# tauri-plugin-liquid-glass

macOS 26+ Liquid Glass effect support for Tauri v2 applications.

This plugin provides native macOS Liquid Glass effects using the private `NSGlassEffectView` API available in macOS 26 (Tahoe) and later. On older macOS versions, it falls back to `NSVisualEffectView`.

## Features

- Native macOS 26+ Liquid Glass effect
- Graceful fallback to `NSVisualEffectView` on older macOS
- 24 material variants (experimental)
- Configurable corner radius and tint color
- Opaque background mode
- Safe no-op on non-macOS platforms

## Requirements

- Tauri v2.0+
- macOS 26+ for Liquid Glass effect (falls back to vibrancy on older versions)
- Rust 1.77+

## Installation

### Rust

Add the plugin to your `Cargo.toml`:

```toml
[dependencies]
tauri-plugin-liquid-glass = { git = "https://github.com/hkandala/tauri-plugin-liquid-glass" }
```

### JavaScript/TypeScript

```bash
npm install tauri-plugin-liquid-glass-api
# or
yarn add tauri-plugin-liquid-glass-api
# or
pnpm add tauri-plugin-liquid-glass-api
```

## Setup

### 1. Register the plugin in your Tauri app

```rust
// src-tauri/src/main.rs
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_liquid_glass::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 2. Configure permissions

Add the plugin permissions to your capability file:

```json
// src-tauri/capabilities/default.json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "liquid-glass:default"
  ]
}
```

### 3. Enable transparent window

In your `tauri.conf.json`, enable window transparency:

```json
{
  "app": {
    "windows": [
      {
        "transparent": true
      }
    ]
  }
}
```

And in your HTML/CSS:

```css
html, body {
  background: transparent;
}
```

## Usage

### TypeScript API

```typescript
import {
  isGlassSupported,
  addGlassEffect,
  configureGlass,
  removeGlassEffect,
  unstable_setVariant,
  GlassMaterialVariant,
} from "tauri-plugin-liquid-glass-api";

// Check if liquid glass is supported
const supported = await isGlassSupported();

// Apply glass effect with options
const viewId = await addGlassEffect({
  cornerRadius: 24,
  tintColor: "#ffffff20",
  opaque: false,
});

// Update configuration
await configureGlass(viewId, 32, "#00000010");

// Set material variant (experimental)
await unstable_setVariant(viewId, GlassMaterialVariant.Dock);

// Remove glass effect
await removeGlassEffect(viewId);
```

### Using the Convenience Class

```typescript
import liquidGlass, { GlassMaterialVariant } from "tauri-plugin-liquid-glass-api";

// Apply glass
await liquidGlass.apply({
  cornerRadius: 24,
  tintColor: "#ffffff20",
});

// Update variant
await liquidGlass.unstable_setVariant(GlassMaterialVariant.Sidebar);

// Remove when done
await liquidGlass.remove();
```

## API Reference

### Stable APIs

| Function | Description |
|----------|-------------|
| `isGlassSupported()` | Check if liquid glass is supported |
| `addGlassEffect(options?)` | Add glass effect to current window |
| `configureGlass(viewId, cornerRadius?, tintColor?)` | Update glass configuration |
| `removeGlassEffect(viewId)` | Remove glass effect |

### Experimental APIs (unstable_)

These APIs use private Apple APIs and may change in future macOS versions:

| Function | Description |
|----------|-------------|
| `unstable_setVariant(viewId, variant)` | Set material variant (0-23) |
| `unstable_setScrim(viewId, enabled)` | Enable/disable scrim overlay |
| `unstable_setSubdued(viewId, enabled)` | Enable/disable subdued state |

### GlassOptions

```typescript
interface GlassOptions {
  cornerRadius?: number;  // Corner radius in pixels
  tintColor?: string;     // Hex color (#RRGGBB or #RRGGBBAA)
  opaque?: boolean;       // Add opaque background behind glass
}
```

### GlassMaterialVariant

24 available variants:

- `Regular` (0) - Default
- `Clear` (1)
- `Dock` (2)
- `AppIcons` (3)
- `Widgets` (4)
- `Text` (5)
- `Avplayer` (6)
- `Facetime` (7)
- `ControlCenter` (8)
- `NotificationCenter` (9)
- `Monogram` (10)
- `Bubbles` (11)
- `Identity` (12)
- `FocusBorder` (13)
- `FocusPlatter` (14)
- `Keyboard` (15)
- `Sidebar` (16)
- `AbuttedSidebar` (17)
- `Inspector` (18)
- `Control` (19)
- `Loupe` (20)
- `Slider` (21)
- `Camera` (22)
- `CartouchePopover` (23)

## Example

See the [examples/tauri-app](./examples/tauri-app) directory for a complete example application.

To run the example:

```bash
cd examples/tauri-app
npm install
npm run tauri dev
```

## Platform Support

| Platform | Support |
|----------|---------|
| macOS 26+ | Full Liquid Glass effect |
| macOS 10.10-25 | Fallback to NSVisualEffectView |
| Windows | No-op (returns -1 for viewId) |
| Linux | No-op (returns -1 for viewId) |

## Notes

- **Private API**: This plugin uses Apple's private `NSGlassEffectView` API, which is not officially documented and may change in future macOS versions.
- **App Store**: Using private APIs may affect App Store approval. Consider using only the fallback `NSVisualEffectView` for production apps.
- **Thread Safety**: All native operations are automatically dispatched to the main thread.

## Credits

Inspired by [electron-liquid-glass](https://github.com/meridiuslabs/electron-liquid-glass) by Meridius Labs.

## License

MIT License - see [LICENSE](./LICENSE) for details.
