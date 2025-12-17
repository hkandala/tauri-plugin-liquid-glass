# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Check Rust plugin compilation
cargo check

# Build Rust plugin
cargo build

# Build TypeScript API
npm run build

# Run example app
cd examples/liquid-glass-app && npm install && npm run tauri dev
```

## Architecture

This is a **Tauri v2 plugin** that provides macOS Liquid Glass effects using native Objective-C APIs.

### Core Plugin (Rust)

- `src/lib.rs` - Plugin entry point, registers commands and manages `GlassViewRegistry` state
- `src/commands.rs` - 7 Tauri commands exposed to JavaScript (all platform-gated with `#[cfg(target_os = "macos")]`)
- `src/models.rs` - `GlassOptions` struct and `GlassMaterialVariant` enum (24 variants)
- `src/error.rs` - Plugin error types implementing `Serialize` for JS interop

### macOS Native Layer (`src/macos/`)

- `glass_effect.rs` - Core implementation using `NSGlassEffectView` (private API) with `NSVisualEffectView` fallback
- `utils.rs` - `run_on_main_thread()` for thread safety, `color_from_hex()` for color parsing

**Key pattern**: Raw pointer addresses (`usize`) are used instead of `id` types when crossing thread boundaries since Objective-C pointers are not `Send`.

### TypeScript API (`guest-js/`)

- `index.ts` - Exports stable APIs and `unstable_`-prefixed experimental APIs
- `types.ts` - TypeScript interfaces matching Rust types

### Permissions

- `permissions/default.toml` - Default permission set
- `build.rs` - Auto-generates permission files via `tauri-plugin` crate

## Key Technical Details

- Uses `objc` + `cocoa` crates for Objective-C bridging (not the newer `objc2`)
- `NSGlassEffectView` is a private Apple API only available on macOS 26+
- All native view operations dispatch to main thread via `dispatch_sync`
- View registry stores pointer addresses as `usize` for thread-safe storage
- Non-macOS platforms return safe no-op values (-1 for viewId, false for support checks)
