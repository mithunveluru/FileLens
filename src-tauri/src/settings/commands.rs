use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

use super::{load, save, Settings};
use crate::database::Database;

#[tauri::command]
pub fn get_settings(db: State<'_, Database>) -> Result<Settings, String> {
    load(&db).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    db: State<'_, Database>,
    settings: Settings,
) -> Result<(), String> {
    save(&db, &settings)?;
    apply_launch_on_startup(&app, settings.launch_on_startup);
    Ok(())
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
