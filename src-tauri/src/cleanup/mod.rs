//! User-initiated cleanup actions. Nothing here deletes permanently: trashing
//! moves a file to the OS Recycle Bin (recoverable from there), and ignoring is
//! reversible. Trashing is restricted to files inside the Downloads folder.

use std::path::Path;

use log::warn;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::database::Database;
use crate::filesystem::FileEntry;

/// True if `path` lives inside `root`. Both must be absolute (canonicalized).
fn is_within(root: &Path, path: &Path) -> bool {
    path.starts_with(root)
}

/// Moves a Downloads file to the OS Recycle Bin and drops it from the inventory.
#[tauri::command]
pub fn trash_file(app: AppHandle, db: State<'_, Database>, path: String) -> Result<(), String> {
    let root = app
        .path()
        .download_dir()
        .map_err(|err| err.to_string())?
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

/// Opens the system file manager with the file selected.
#[tauri::command]
pub fn reveal_file(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .reveal_item_in_dir(path)
        .map_err(|err| err.to_string())
}

/// Excludes a path from future analysis (reversible via [`unignore_path`]).
#[tauri::command]
pub fn ignore_path(db: State<'_, Database>, path: String) -> Result<(), String> {
    db.add_ignored(&path).map_err(|err| err.to_string())
}

/// Restores a previously ignored path so it is analyzed again.
#[tauri::command]
pub fn unignore_path(db: State<'_, Database>, path: String) -> Result<(), String> {
    db.remove_ignored(&path).map_err(|err| err.to_string())
}

/// Returns full metadata for a file, for the preview panel.
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
