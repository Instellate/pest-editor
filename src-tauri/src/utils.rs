use anyhow::Result;
use tauri_plugin_store::StoreExt;

type Store = std::sync::Arc<tauri_plugin_store::Store<tauri::Wry>>;
static SETTINGS_LOCATION: &str = "settings.json";

pub struct Settings {
    store: Store,
    auto_save: bool,
}

impl Settings {
    pub fn new(app: &tauri::AppHandle) -> Result<Self> {
        let store = app.store(SETTINGS_LOCATION)?;
        let auto_save = store
            .get("auto-save")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(Self { store, auto_save })
    }

    pub fn auto_save(&self) -> bool {
        self.auto_save
    }

    pub fn set_auto_save(&mut self, val: bool) {
        self.auto_save = val;
    }
}

impl Drop for Settings {
    fn drop(&mut self) {
        self.store.set("auto-save", self.auto_save);
    }
}
