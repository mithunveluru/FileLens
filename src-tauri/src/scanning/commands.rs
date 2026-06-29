//! Tauri commands that drive a scan, persist its results, and expose history.
//! Also owns the shared state that lets a running scan be cancelled.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use log::warn;
use tauri::{AppHandle, Emitter, Manager, State};

use super::{scan, ScanOutcome};
use crate::database::{now_ms, Database, ScanRecord};
use crate::settings;

/// Event emitted periodically during a scan; payload is the running file count.
const PROGRESS_EVENT: &str = "scan:progress";

/// Upper bound on how many history rows the frontend may request at once.
const MAX_HISTORY_LIMIT: i64 = 100;

/// Shared scan control: a cancel flag the running walk polls, plus a guard that
/// stops two scans from running (and fighting over the cancel flag) at once.
#[derive(Default)]
pub struct ScanState {
    cancel: Arc<AtomicBool>,
    running: AtomicBool,
}

/// Outcome of one scan run, including timing, used to persist a history row.
struct ScanRun {
    outcome: ScanOutcome,
    root: String,
    started_ms: i64,
    finished_ms: i64,
}

/// Scans the configured Downloads folder, persists the inventory, and returns it.
#[tauri::command]
pub async fn scan_downloads(
    app: AppHandle,
    scan_state: State<'_, ScanState>,
    db: State<'_, Database>,
) -> Result<ScanOutcome, String> {
    let root = resolve_root(&app, &db)?;

    if scan_state.running.swap(true, Ordering::SeqCst) {
        return Err("A scan is already running.".into());
    }

    let result = run_scan(&app, &scan_state, root).await;
    scan_state.running.store(false, Ordering::SeqCst);
    let run = result?;

    if let Err(err) = db.persist_scan(&run.root, run.started_ms, run.finished_ms, &run.outcome) {
        // Persistence failure must not lose the scan the user just ran.
        warn!("failed to persist scan results: {err}");
    }

    // Remember where we scanned so the next launch can reuse it.
    if settings::load(&db)
        .map(|s| s.remember_last_scan_location)
        .unwrap_or(false)
    {
        let _ = db.set_setting(settings::LAST_SCAN_LOCATION_KEY, &run.root);
    }

    Ok(run.outcome)
}

/// Requests cancellation of the in-progress scan, if any.
#[tauri::command]
pub fn cancel_scan(state: State<'_, ScanState>) {
    state.cancel.store(true, Ordering::Relaxed);
}

/// Returns the most recent scans, newest first.
#[tauri::command]
pub fn scan_history(db: State<'_, Database>, limit: i64) -> Result<Vec<ScanRecord>, String> {
    let limit = limit.clamp(1, MAX_HISTORY_LIMIT);
    db.scan_history(limit).map_err(|err| err.to_string())
}

/// Resolves the scan root from settings (override) or the OS Downloads folder.
fn resolve_root(app: &AppHandle, db: &Database) -> Result<PathBuf, String> {
    let os_default = app.path().download_dir().map_err(|err| err.to_string())?;
    let settings = settings::load(db).map_err(|err| err.to_string())?;
    let remembered = db
        .get_setting(settings::LAST_SCAN_LOCATION_KEY)
        .ok()
        .flatten();
    settings::resolve_root(&settings, remembered.as_deref(), os_default)
}

async fn run_scan(app: &AppHandle, state: &ScanState, root: PathBuf) -> Result<ScanRun, String> {
    state.cancel.store(false, Ordering::Relaxed);
    let root_str = root.to_string_lossy().into_owned();

    let cancel = state.cancel.clone();
    let app = app.clone();
    let started_ms = now_ms();
    // The walk is blocking; run it off the async runtime so the UI stays responsive.
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        scan(&root, &cancel, |count| {
            // Progress is best-effort; a failed emit must not abort the scan.
            let _ = app.emit(PROGRESS_EVENT, count);
        })
    })
    .await
    .map_err(|err| format!("Scan task failed: {err}"))?;

    Ok(ScanRun {
        outcome,
        root: root_str,
        started_ms,
        finished_ms: now_ms(),
    })
}
