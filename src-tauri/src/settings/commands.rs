use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;

use super::{load, resolve_root, save, Settings, LAST_SCAN_LOCATION_KEY};
use crate::database::Database;

// The one root resolver. Every command that reads or writes user files goes
// through this, so scanning, organizing, and trashing can never disagree.
pub fn active_root(app: &AppHandle, db: &Database) -> Result<PathBuf, String> {
    let os_default = app.path().download_dir().map_err(|err| err.to_string())?;
    let settings = load(db).map_err(|err| err.to_string())?;
    let remembered = db.get_setting(LAST_SCAN_LOCATION_KEY).ok().flatten();
    resolve_root(&settings, remembered.as_deref(), os_default)
}

#[tauri::command]
pub fn get_settings(db: State<'_, Database>) -> Result<Settings, String> {
    load(&db).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    db: State<'_, Database>,
    settings: Settings,
) -> Result<Settings, String> {
    let stored = save(&db, &settings)?;
    apply_launch_on_startup(&app, stored.launch_on_startup);
    Ok(stored)
}

// Best-effort: a failure to update the OS autostart entry is logged, not propagated.
pub fn apply_launch_on_startup(app: &AppHandle, enabled: bool) {
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    if let Err(err) = result {
        log::warn!("could not update launch-on-startup: {err}");
    }
}
