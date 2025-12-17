const COMMANDS: &[&str] = &["is_glass_supported", "set_liquid_glass_effect"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./guest-js/index.ts")
        .build();
}
