use anyhow::{anyhow, Result};
use log::error;
use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuEvent, SubmenuBuilder};
use tauri_plugin_dialog::DialogExt;

use crate::utils::Settings;

pub fn register_menu(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::new(app.handle())?;

    let files_auto_save_item = CheckMenuItemBuilder::new("Auto save")
        .id("auto-save")
        .checked(settings.auto_save())
        .build(app)?;

    let files_menu = SubmenuBuilder::new(app, "Files")
        .text("open-file", "Open file")
        .separator()
        .item(&files_auto_save_item)
        .build()?;

    let menu = MenuBuilder::new(app).items(&[&files_menu]).build()?;

    app.set_menu(menu)?;
    app.on_menu_event(on_menu_event);

    Ok(())
}

fn on_menu_event(app: &tauri::AppHandle, event: MenuEvent) {
    if let Err(e) = menu_event_handler(app, event) {
        error!("Got error when hanlding menu event: {}", e);
    }
}

fn menu_event_handler(app: &tauri::AppHandle, event: MenuEvent) -> Result<()> {
    match event.id().0.as_str() {
        "open-file" => on_open_file(app)?,
        "auto-save" => on_auto_save_toggle(app)?,
        _ => return Err(anyhow!("Unknown event")),
    }

    Ok(())
}

fn on_open_file(app: &tauri::AppHandle) -> Result<()> {
    let app_clone = app.clone();
    app.dialog()
        .file()
        .add_filter("Grammar file", &["pest"])
        .pick_file(move |path| {
            if let Some(path) = path {
                tauri::async_runtime::spawn(crate::files::change_file(app_clone, path.to_string()));
            }
        });

    Ok(())
}

fn on_auto_save_toggle(app: &tauri::AppHandle) -> Result<()> {
    let mut settings = Settings::new(app)?;
    settings.set_auto_save(!settings.auto_save());

    Ok(())
}
