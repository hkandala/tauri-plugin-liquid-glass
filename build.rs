const COMMANDS: &[&str] = &[
    "is_glass_supported",
    "add_glass_effect",
    "configure_glass",
    "set_variant",
    "set_scrim",
    "set_subdued",
    "remove_glass_effect",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./guest-js/index.ts")
        .build();
}
