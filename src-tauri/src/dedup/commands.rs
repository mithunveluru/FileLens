//! IPC surface for duplicate detection. Runs the read-only pipeline over the
//! current inventory; nothing here touches the filesystem beyond reading.

use std::collections::HashSet;
use std::thread::available_parallelism;

use tauri::State;

use super::{detect_duplicates, DuplicateReport};
use crate::database::Database;
use crate::settings;

// Cap disk contention: more threads than this rarely helps a hashing workload
// and just thrashes the drive.
const MAX_WORKERS: usize = 8;

#[tauri::command]
pub fn find_duplicates(db: State<'_, Database>) -> Result<DuplicateReport, String> {
    let settings = settings::load(&db).map_err(|err| err.to_string())?;
    let ignored: HashSet<String> = db
        .ignored_paths()
        .map_err(|err| err.to_string())?
        .into_iter()
        .collect();
    let files: Vec<_> = db
        .list_files()
        .map_err(|err| err.to_string())?
        .into_iter()
        .filter(|f| !settings.is_excluded(f, &ignored))
        .collect();

    let workers = available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(MAX_WORKERS);

    Ok(detect_duplicates(&files, &*db, workers))
}
