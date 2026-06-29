//! Tauri commands orchestrating Smart Organization. These coordinate the pure
//! modules and the database; they contain no business logic of their own.

use std::path::Path;

use tauri::{AppHandle, Manager, State};

use super::{execution, planner, ExecutionResult, OrganizationPlan, UndoResult};
use crate::database::{Database, OrganizationSessionRecord};
use crate::settings;

/// Upper bound on history rows returned to the frontend.
const MAX_HISTORY_LIMIT: i64 = 100;

fn resolve_root(app: &AppHandle, db: &Database) -> Result<std::path::PathBuf, String> {
    let os_default = app.path().download_dir().map_err(|err| err.to_string())?;
    let settings = settings::load(db).map_err(|err| err.to_string())?;
    let remembered = db
        .get_setting(settings::LAST_SCAN_LOCATION_KEY)
        .ok()
        .flatten();
    settings::resolve_root(&settings, remembered.as_deref(), os_default)
}

/// Builds a proposed organization plan from the current inventory. Read-only:
/// it never modifies the filesystem.
#[tauri::command]
pub fn generate_organization_plan(
    app: AppHandle,
    db: State<'_, Database>,
) -> Result<OrganizationPlan, String> {
    let root = resolve_root(&app, &db)?;
    let files = db.list_files().map_err(|err| err.to_string())?;

    Ok(planner::build_plan(&files, &root, &|path: &Path| {
        path.exists()
    }))
}

/// Executes an approved plan: moves files and records the session for undo. The
/// move root is re-resolved server-side, so a tampered `plan.root` cannot
/// redirect moves outside the Downloads folder.
#[tauri::command]
pub fn execute_organization_plan(
    app: AppHandle,
    db: State<'_, Database>,
    plan: OrganizationPlan,
) -> Result<ExecutionResult, String> {
    let root = resolve_root(&app, &db)?;
    let report = execution::execute(&root, &plan.actions);

    let session_id = if report.moves.is_empty() {
        None
    } else {
        let pairs: Vec<(String, String)> = report
            .moves
            .iter()
            .map(|m| (m.source.clone(), m.destination.clone()))
            .collect();
        // Keep the inventory consistent: moved files leave their old path.
        for (source, _) in &pairs {
            let _ = db.remove_file(source);
        }
        Some(
            db.record_organization_session(&root.to_string_lossy(), &pairs)
                .map_err(|err| err.to_string())?,
        )
    };

    Ok(ExecutionResult {
        moved: report.moves.len(),
        skipped: report.skipped,
        failed: report.failed,
        session_id,
        errors: report.errors,
    })
}

/// Returns recent organization sessions, newest first.
#[tauri::command]
pub fn organization_history(
    db: State<'_, Database>,
    limit: i64,
) -> Result<Vec<OrganizationSessionRecord>, String> {
    db.organization_history(limit.clamp(1, MAX_HISTORY_LIMIT))
        .map_err(|err| err.to_string())
}

/// Reverses a recorded session by moving each file back to its original path.
#[tauri::command]
pub fn undo_organization(
    app: AppHandle,
    db: State<'_, Database>,
    session_id: i64,
) -> Result<UndoResult, String> {
    let root = resolve_root(&app, &db)?;
    let moves = db
        .organization_session_moves(session_id)
        .map_err(|err| err.to_string())?;
    let report = execution::undo(&root, &moves);
    db.mark_session_undone(session_id)
        .map_err(|err| err.to_string())?;

    Ok(UndoResult {
        restored: report.restored,
        failed: report.failed,
        errors: report.errors,
    })
}
