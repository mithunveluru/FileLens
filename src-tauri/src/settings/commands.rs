//! Tauri commands for reading and writing user settings.

use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

use super::{load, save, Settings};
use crate::database::Database;

/// Returns the current settings (defaults if none saved yet).
#[tauri::command]
pub fn get_settings(db: State<'_, Database>) -> Result<Settings, String> {
    load(&db).map_err(|err| err.to_string())
}

/// Persists the given settings and syncs OS-level launch-on-startup to match.
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

/// Best-effort sync of the OS autostart entry; a failure here must not fail the
/// save, so it is logged rather than propagated.
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
