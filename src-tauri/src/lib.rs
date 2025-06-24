pub(crate) mod files;
pub(crate) mod menu;
pub(crate) mod parsing;
pub(crate) mod utils;

use crate::files::{
    change_file, save_grammar_content, save_grammar_input, update_grammar_content,
    update_grammar_input, ChangeFileEvent,
};
use crate::menu::register_menu;
use crate::parsing::{find_rule_references, get_all_rules, parse_input, update_pest_grammar};
use log::error;
use std::sync::Mutex;
use tauri::{Manager, Window, WindowEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_store::StoreExt;
use tauri_specta::collect_events;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            update_pest_grammar,
            parse_input,
            find_rule_references,
            get_all_rules,
            update_grammar_content,
            save_grammar_content,
            update_grammar_input,
            save_grammar_input,
            change_file,
        ])
        .events(collect_events![ChangeFileEvent]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/lib/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            update_pest_grammar,
            parse_input,
            find_rule_references,
            get_all_rules,
            update_grammar_content,
            save_grammar_content,
            update_grammar_input,
            save_grammar_input,
            change_file,
        ])
        .setup(move |app| {
            builder.mount_events(app);
            register_menu(app)?;

            let args = std::env::args().skip(1);
            for arg in args {
                if arg.starts_with("-") {
                    continue;
                }

                let path = std::path::Path::new(&arg);
                if path.exists() {
                    let state_store = app.store("state.json")?;
                    state_store.set("last-file", arg);
                }
                break;
            }

            Ok(())
        })
        .on_window_event(on_window_event)
        .manage(parsing::ParserState::default())
        .manage(WindowEventState::default())
        .manage(files::FileState::default())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct WindowEventState {
    is_closing: Mutex<bool>,
}

impl Default for WindowEventState {
    fn default() -> Self {
        Self {
            is_closing: Mutex::new(false),
        }
    }
}

fn on_window_event(window: &Window, event: &WindowEvent) {
    let app = window.app_handle();
    let state: tauri::State<WindowEventState> = app.state();

    if let WindowEvent::CloseRequested { api, .. } = event {
        let is_closing = state.is_closing.lock().unwrap();
        if *is_closing {
            return;
        }

        let file_state: tauri::State<files::FileState> = app.state();
        if file_state.unsaved_content.blocking_lock().is_none()
            && file_state.unsaved_input.blocking_lock().is_none()
        {
            return;
        }

        let win_clone = window.clone();
        api.prevent_close();
        app.dialog()
            .message("You have unsaved changes, are you sure you want to exit?")
            .kind(MessageDialogKind::Info)
            .buttons(tauri_plugin_dialog::MessageDialogButtons::YesNo)
            .title("Unsaved changes")
            .show(move |result| {
                let state: tauri::State<'_, WindowEventState> = win_clone.state();
                let file_state: tauri::State<files::FileState> = win_clone.state();
                let mut is_closing = state.is_closing.lock().unwrap();

                if result {
                    if file_state.unsaved_content.blocking_lock().is_some() {
                        tauri::async_runtime::block_on(crate::save_grammar_content(
                            file_state.clone(),
                        ))
                        .unwrap();
                    }

                    if file_state.unsaved_input.blocking_lock().is_some() {
                        tauri::async_runtime::block_on(crate::save_grammar_input(
                            win_clone.app_handle().clone(),
                            file_state.clone(),
                        ))
                        .unwrap();
                    }
                }

                let path = file_state.grammar_file_path.blocking_lock();
                if let Some(path) = path.as_ref() {
                    let state_store = win_clone.store("state.json").expect("Expected state store");
                    state_store.set("last-file", path.clone());
                }

                *is_closing = true;
                if let Err(e) = win_clone.close() {
                    error!("Got error when trying to close window: {}", e);
                }
            });
    }
}
