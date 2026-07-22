use std::path::Path;

use tauri::{AppHandle, State};

use super::{execution, planner, ExecutionResult, OrganizationPlan, UndoResult};
use crate::database::{Database, OrganizationSessionRecord};
use crate::settings;

const MAX_HISTORY_LIMIT: i64 = 100;

// Read-only: never modifies the filesystem.
#[tauri::command]
pub fn generate_organization_plan(
    app: AppHandle,
    db: State<'_, Database>,
) -> Result<OrganizationPlan, String> {
    let root = settings::commands::active_root(&app, &db)?;
    let files = db.list_files().map_err(|err| err.to_string())?;

    Ok(planner::build_plan(&files, &root, &|path: &Path| {
        path.exists()
    }))
}

// The root is re-resolved server-side so a tampered plan.root cannot redirect moves.
#[tauri::command]
pub fn execute_organization_plan(
    app: AppHandle,
    db: State<'_, Database>,
    plan: OrganizationPlan,
) -> Result<ExecutionResult, String> {
    let root = settings::commands::active_root(&app, &db)?;
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

#[tauri::command]
pub fn organization_history(
    db: State<'_, Database>,
    limit: i64,
) -> Result<Vec<OrganizationSessionRecord>, String> {
    db.organization_history(limit.clamp(1, MAX_HISTORY_LIMIT))
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn undo_organization(
    app: AppHandle,
    db: State<'_, Database>,
    session_id: i64,
) -> Result<UndoResult, String> {
    let root = settings::commands::active_root(&app, &db)?;
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
