mod analysis;
mod cleanup;
mod database;
mod dedup;
mod filesystem;
mod organization;
mod scanning;
mod settings;

use analysis::commands::analyze_downloads;
use cleanup::{file_info, ignore_path, reveal_file, trash_file, unignore_path};
use database::Database;
use dedup::commands::find_duplicates;
use organization::commands::{
    execute_organization_plan, generate_organization_plan, organization_history, undo_organization,
};
use scanning::commands::{cancel_scan, scan_downloads, scan_history, ScanState};
use serde::Serialize;
use settings::commands::{get_settings, save_settings};
use tauri::Manager;

#[derive(Serialize)]
struct AppInfo {
    name: String,
    version: String,
}

/// Application name and version, shown in the UI footer.
#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        name: "File Lens".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Frontend and backend logs share the same sinks.
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(ScanState::default())
        .setup(|app| {
            // A missing data dir falls back to in-memory so startup never aborts.
            let db = app
                .path()
                .app_data_dir()
                .ok()
                .map(|dir| {
                    let _ = std::fs::create_dir_all(&dir);
                    Database::open(&dir.join("file_lens.db"))
                })
                .unwrap_or_else(Database::in_memory);

            // Reconcile the OS autostart entry with the saved preference.
            let launch_on_startup = settings::load(&db)
                .map(|s| s.launch_on_startup)
                .unwrap_or(false);
            settings::commands::apply_launch_on_startup(app.handle(), launch_on_startup);

            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            scan_downloads,
            cancel_scan,
            scan_history,
            analyze_downloads,
            find_duplicates,
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
