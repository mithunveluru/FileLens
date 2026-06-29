mod analysis;
mod cleanup;
mod database;
mod filesystem;
mod organization;
mod scanning;
mod settings;

use analysis::commands::analyze_downloads;
use cleanup::{file_info, ignore_path, reveal_file, trash_file, unignore_path};
use database::Database;
use organization::commands::{
    execute_organization_plan, generate_organization_plan, organization_history, undo_organization,
};
use scanning::commands::{cancel_scan, scan_downloads, scan_history, ScanState};
use serde::Serialize;
use settings::commands::{get_settings, save_settings};
use tauri::Manager;

/// Application identity returned to the frontend via the `app_info` command.
#[derive(Serialize)]
struct AppInfo {
    name: String,
    version: String,
}

/// Smoke-test command proving the React <-> Rust IPC bridge is wired up.
#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        name: "Download Doctor".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Unified logging: frontend (tauri-plugin-log) and backend logs share
        // the same sinks. `LevelFilter::Info` is the floor for shipped builds.
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .manage(ScanState::default())
        .setup(|app| {
            // Open the database in the per-user app-data dir, creating it on first run.
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db = Database::open(&dir.join("download_doctor.db"))?;
            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            scan_downloads,
            cancel_scan,
            scan_history,
            analyze_downloads,
            trash_file,
            reveal_file,
            ignore_path,
            unignore_path,
            file_info,
            get_settings,
            save_settings,
            generate_organization_plan,
            execute_organization_plan,
            organization_history,
            undo_organization
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
