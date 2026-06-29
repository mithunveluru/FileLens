use std::collections::HashSet;

use tauri::State;

use super::{report, AnalysisInput, AnalysisReport};
use crate::database::{now_ms, Database};
use crate::settings;

#[tauri::command]
pub fn analyze_downloads(db: State<'_, Database>) -> Result<AnalysisReport, String> {
    let settings = settings::load(&db).map_err(|err| err.to_string())?;
    let config = settings.analysis_config();
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

    let input = AnalysisInput {
        files: &files,
        config: &config,
        now_ms: now_ms(),
    };
    Ok(report(&input))
}
