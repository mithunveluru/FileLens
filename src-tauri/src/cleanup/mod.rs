//! User cleanup actions; trashing moves files to the OS Recycle Bin and stays within Downloads.

use std::path::Path;

use log::warn;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::database::Database;
use crate::filesystem::FileEntry;
use crate::settings;

// Both paths must already be absolute (canonicalized) for this to be safe.
fn is_within(root: &Path, path: &Path) -> bool {
    path.starts_with(root)
}

#[tauri::command]
pub fn trash_file(app: AppHandle, db: State<'_, Database>, path: String) -> Result<(), String> {
    // Same root the scan used, so a configured folder is trashable.
    let root = settings::commands::active_root(&app, &db)?
        .canonicalize()
        .map_err(|err| err.to_string())?;
    let target = Path::new(&path)
        .canonicalize()
        .map_err(|_| "File no longer exists.".to_string())?;

    if !is_within(&root, &target) {
        return Err("Refusing to trash a file outside the Downloads folder.".into());
    }

    trash::delete(&target)
        .map_err(|err| format!("Could not move file to the Recycle Bin: {err}"))?;

    // Keep the inventory consistent with disk; a failure here is not fatal.
    if let Err(err) = db.remove_file(&path) {
        warn!("trashed {path} but failed to remove it from the inventory: {err}");
    }
    Ok(())
}

#[tauri::command]
pub fn reveal_file(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .reveal_item_in_dir(path)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn ignore_path(db: State<'_, Database>, path: String) -> Result<(), String> {
    db.add_ignored(&path).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn unignore_path(db: State<'_, Database>, path: String) -> Result<(), String> {
    db.remove_ignored(&path).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn file_info(db: State<'_, Database>, path: String) -> Result<Option<FileEntry>, String> {
    db.get_file(&path).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_within_detects_paths_inside_and_outside_root() {
        let root = Path::new("/home/u/Downloads");
        assert!(is_within(root, Path::new("/home/u/Downloads/a.txt")));
        assert!(is_within(root, Path::new("/home/u/Downloads/sub/b.txt")));
        assert!(!is_within(root, Path::new("/etc/passwd")));
        assert!(!is_within(root, Path::new("/home/u/Documents/c.txt")));
    }
}
