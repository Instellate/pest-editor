// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod parsing;
use parsing::{find_rule_references, get_all_rules, parse_input, update_pest_grammar};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder =
        tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
            update_pest_grammar,
            parse_input,
            find_rule_references,
            get_all_rules,
        ]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/lib/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            update_pest_grammar,
            parse_input,
            find_rule_references,
            get_all_rules,
        ])
        .setup(move |app| {
            builder.mount_events(app);

            Ok(())
        })
        .manage(parsing::ParserState::default())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
