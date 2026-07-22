//! IPC surface for duplicate detection. Runs the read-only pipeline over the
//! current inventory; nothing here touches the filesystem beyond reading.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::available_parallelism;

use tauri::{AppHandle, Emitter, State};

use super::{detect_duplicates, DuplicateReport};
use crate::database::Database;
use crate::settings;

const PROGRESS_EVENT: &str = "dedup:progress";

// Cap disk contention: more threads than this rarely helps a hashing workload
// and just thrashes the drive.
const MAX_WORKERS: usize = 8;

#[derive(Default)]
pub struct DedupState {
    cancel: Arc<AtomicBool>,
    // Guards against two runs fighting over the cancel flag.
    running: AtomicBool,
}

#[tauri::command]
pub async fn find_duplicates(
    app: AppHandle,
    state: State<'_, DedupState>,
    db: State<'_, Database>,
) -> Result<DuplicateReport, String> {
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

    if state.running.swap(true, Ordering::SeqCst) {
        return Err("Duplicate detection is already running.".into());
    }
    state.cancel.store(false, Ordering::Relaxed);

    let workers = available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(MAX_WORKERS);

    let report = run(&app, &db, files, workers, &state.cancel);
    state.running.store(false, Ordering::SeqCst);
    Ok(report)
}

#[tauri::command]
pub fn cancel_duplicate_scan(state: State<'_, DedupState>) {
    state.cancel.store(true, Ordering::Relaxed);
}

// Hashing is blocking work; the scoped pool inside owns its own threads, so the
// only requirement here is that progress emits never abort the run.
fn run(
    app: &AppHandle,
    db: &Database,
    files: Vec<crate::filesystem::FileEntry>,
    workers: usize,
    cancel: &AtomicBool,
) -> DuplicateReport {
    detect_duplicates(&files, db, workers, cancel, &|done| {
        let _ = app.emit(PROGRESS_EVENT, done);
    })
}
