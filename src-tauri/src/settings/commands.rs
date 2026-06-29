//! Tauri commands for reading and writing user settings.

use tauri::State;

use super::{load, save, Settings};
use crate::database::Database;

/// Returns the current settings (defaults if none saved yet).
#[tauri::command]
pub fn get_settings(db: State<'_, Database>) -> Result<Settings, String> {
    load(&db).map_err(|err| err.to_string())
}

/// Persists the given settings.
#[tauri::command]
pub fn save_settings(db: State<'_, Database>, settings: Settings) -> Result<(), String> {
    save(&db, &settings)
}
