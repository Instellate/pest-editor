use log::error;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tauri_specta::Event;
use tokio::sync::Mutex;

use crate::utils::Settings;

#[derive(Default)]
pub struct FileState {
    /// The path to the currently viewed grammar file
    pub grammar_file_path: Mutex<Option<String>>,
    pub unsaved_content: Mutex<Option<String>>,
    pub original_content: Mutex<Option<String>>,
    pub unsaved_input: Mutex<Option<String>>,
    pub original_input: Mutex<Option<String>>,
}

#[tauri::command]
#[specta::specta]
pub async fn update_grammar_content(
    app: tauri::AppHandle,
    state: tauri::State<'_, FileState>,
    content: String,
) -> Result<(), String> {
    if state
        .original_content
        .lock()
        .await
        .as_ref()
        .map(|v| v.eq(&content))
        .unwrap_or(false)
    {
        *state.unsaved_content.lock().await = None;
        return Ok(());
    }

    let settings = Settings::new(&app).map_err(|_| String::from("Couldn't initialize settings"))?;
    if !settings.auto_save() {
        let mut unsaved_content = state.unsaved_content.lock().await;
        *unsaved_content = Some(content);
        return Ok(());
    }

    let grammar_file_path = state.grammar_file_path.lock().await;
    let Some(path) = grammar_file_path.as_ref() else {
        return Ok(());
    };

    tokio::fs::write(path, content)
        .await
        .map_err(|_| String::from("Couldn't write file content"))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn save_grammar_content(state: tauri::State<'_, FileState>) -> Result<(), String> {
    let grammar_file_path = state.grammar_file_path.lock().await;
    let mut unsaved_content = state.unsaved_content.lock().await;

    let Some(path) = grammar_file_path.as_ref() else {
        todo!("Implement dialog for where to save file");
    };

    let Some(content) = unsaved_content.take() else {
        return Ok(());
    };

    tokio::fs::write(path, content)
        .await
        .map_err(|_| String::from("Couldn't write file content"))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn update_grammar_input(
    app: tauri::AppHandle,
    state: tauri::State<'_, FileState>,
    content: String,
) -> Result<(), String> {
    if state
        .original_input
        .lock()
        .await
        .as_ref()
        .map(|v| *v == content)
        .unwrap_or(false)
    {
        *state.unsaved_input.lock().await = None;
        return Ok(());
    }

    let settings = Settings::new(&app).map_err(|_| String::from("Couldn't initialize settings"))?;
    if !settings.auto_save() {
        let mut unsaved_input = state.unsaved_input.lock().await;
        *unsaved_input = Some(content);
        return Ok(());
    }

    let grammar_file_path = state.grammar_file_path.lock().await;
    let Some(path) = grammar_file_path.as_ref() else {
        return Ok(());
    };

    let store = app
        .store("inputs.json")
        .map_err(|_| String::from("Couldn't get store"))?;
    store.set(path, content);

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn save_grammar_input(
    app: tauri::AppHandle,
    state: tauri::State<'_, FileState>,
) -> Result<(), String> {
    let grammar_file_path = state.grammar_file_path.lock().await;
    let mut unsaved_input = state.unsaved_input.lock().await;

    let Some(path) = grammar_file_path.as_ref() else {
        todo!("Implement dialog for where to save file");
    };

    let Some(content) = unsaved_input.take() else {
        return Ok(());
    };

    let store = app
        .store("inputs.json")
        .map_err(|_| String::from("Couldn't get store"))?;
    store.set(path, content);

    Ok(())
}

#[derive(Serialize, Deserialize, Clone, specta::Type, tauri_specta::Event)]
pub struct ChangeFileEvent {
    grammar: String,
    input: String,
}

impl ChangeFileEvent {
    pub fn new(grammar: String, input: String) -> Self {
        Self { grammar, input }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn change_file(app: tauri::AppHandle, path: String) {
    if let Err(e) = change_file_internal(app, path).await {
        error!("Got error when trying to change file: {}", e);
    }
}

async fn change_file_internal(app: tauri::AppHandle, path: String) -> anyhow::Result<()> {
    let store = app.store("inputs.json")?;

    let content = tokio::fs::read_to_string(path.to_string()).await?;
    let input = store
        .get(&path)
        .and_then(|v| v.as_str().map(|v| v.to_string()))
        .unwrap_or_else(String::new);

    let state: tauri::State<'_, FileState> = app.state();
    let mut grammar_file_path = state.grammar_file_path.lock().await;
    *grammar_file_path = Some(path);

    *state.original_content.lock().await = Some(content.clone());
    *state.unsaved_content.lock().await = None;

    *state.original_input.lock().await = Some(input.clone());
    *state.unsaved_input.lock().await = None;

    ChangeFileEvent::new(content, input)
        .emit(&app)
        .map_err(anyhow::Error::from)
}
