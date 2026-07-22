//! User settings as one JSON document; serde defaults keep it forward-compatible.

pub mod commands;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::analysis::AnalysisConfig;
use crate::database::Database;
use crate::filesystem::FileEntry;

const SETTINGS_KEY: &str = "app";

// Separate key so saving settings never clobbers it.
pub const LAST_SCAN_LOCATION_KEY: &str = "last_scan_location";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// None/empty means the OS Downloads folder.
    pub downloads_folder: Option<String>,
    pub age_threshold_days: i64,
    pub large_file_min_mb: u64,
    pub ignored_folders: Vec<String>,
    pub ignored_extensions: Vec<String>,
    /// "system", "light", or "dark".
    pub theme: String,
    pub auto_scan_on_startup: bool,
    pub launch_on_startup: bool,
    pub remember_last_scan_location: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            downloads_folder: None,
            age_threshold_days: 90,
            large_file_min_mb: 100,
            ignored_folders: Vec::new(),
            ignored_extensions: Vec::new(),
            theme: "system".to_string(),
            auto_scan_on_startup: true,
            launch_on_startup: false,
            remember_last_scan_location: true,
        }
    }
}

impl Settings {
    // A threshold of zero or less flags every file; clamp before persisting.
    fn sanitized(&self) -> Settings {
        Settings {
            age_threshold_days: self.age_threshold_days.max(1),
            large_file_min_mb: self.large_file_min_mb.max(1),
            ..self.clone()
        }
    }

    pub fn analysis_config(&self) -> AnalysisConfig {
        AnalysisConfig {
            large_file_min_bytes: self.large_file_min_mb.saturating_mul(1024 * 1024),
            old_file_max_age_days: self.age_threshold_days,
        }
    }

    pub fn is_excluded(&self, file: &FileEntry, ignored_paths: &HashSet<String>) -> bool {
        if ignored_paths.contains(&file.path) {
            return true;
        }
        if let Some(ext) = file.extension.as_deref() {
            let ignored_ext = self
                .ignored_extensions
                .iter()
                .any(|e| e.trim_start_matches('.').eq_ignore_ascii_case(ext));
            if ignored_ext {
                return true;
            }
        }
        // `Path::starts_with` is component-aware, so "/dl/foo" does not match
        // "/dl/foobar" and it works across path separators.
        self.ignored_folders
            .iter()
            .any(|dir| Path::new(&file.path).starts_with(dir))
    }
}

// Preference order: configured override, remembered location, OS Downloads.
pub fn resolve_root(
    settings: &Settings,
    remembered: Option<&str>,
    os_default: PathBuf,
) -> Result<PathBuf, String> {
    if let Some(folder) = settings
        .downloads_folder
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let path = PathBuf::from(folder);
        return if path.is_dir() {
            Ok(path)
        } else {
            Err("The configured Downloads folder does not exist.".into())
        };
    }

    if settings.remember_last_scan_location {
        if let Some(remembered) = remembered.map(str::trim).filter(|s| !s.is_empty()) {
            let path = PathBuf::from(remembered);
            if path.is_dir() {
                return Ok(path);
            }
        }
    }

    Ok(os_default)
}

pub fn load(db: &Database) -> rusqlite::Result<Settings> {
    Ok(match db.get_setting(SETTINGS_KEY)? {
        Some(json) => serde_json::from_str(&json).unwrap_or_default(),
        None => Settings::default(),
    })
}

// Returns what was actually stored so the caller reflects any clamping.
pub fn save(db: &Database, settings: &Settings) -> Result<Settings, String> {
    let settings = settings.sanitized();
    let json = serde_json::to_string(&settings).map_err(|err| err.to_string())?;
    db.set_setting(SETTINGS_KEY, &json)
        .map_err(|err| err.to_string())?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, ext: Option<&str>) -> FileEntry {
        FileEntry {
            name: path.rsplit('/').next().unwrap().to_string(),
            extension: ext.map(str::to_string),
            path: path.to_string(),
            size_bytes: 1,
            created_ms: None,
            modified_ms: None,
            mime_type: "application/octet-stream".to_string(),
            is_hidden: false,
        }
    }

    #[test]
    fn analysis_config_converts_mb_to_bytes() {
        let settings = Settings {
            large_file_min_mb: 100,
            age_threshold_days: 30,
            ..Default::default()
        };
        let config = settings.analysis_config();
        assert_eq!(config.large_file_min_bytes, 100 * 1024 * 1024);
        assert_eq!(config.old_file_max_age_days, 30);
    }

    #[test]
    fn save_clamps_thresholds_that_would_flag_everything() {
        let db = Database::in_memory();
        let stored = save(
            &db,
            &Settings {
                age_threshold_days: 0,
                large_file_min_mb: 0,
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(stored.age_threshold_days, 1);
        assert_eq!(stored.large_file_min_mb, 1);

        let reloaded = load(&db).unwrap();
        assert_eq!(reloaded.age_threshold_days, 1);
        assert_eq!(reloaded.large_file_min_mb, 1);

        let negative = save(
            &db,
            &Settings {
                age_threshold_days: -30,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(negative.age_threshold_days, 1);
    }

    #[test]
    fn resolve_root_prefers_remembered_location_then_falls_back() {
        let dir = tempfile::tempdir().unwrap();
        let remembered = dir.path().join("remembered");
        let os_default = dir.path().join("downloads");
        std::fs::create_dir(&remembered).unwrap();
        std::fs::create_dir(&os_default).unwrap();

        let on = Settings {
            remember_last_scan_location: true,
            ..Default::default()
        };
        assert_eq!(
            resolve_root(&on, remembered.to_str(), os_default.clone()).unwrap(),
            remembered
        );

        let off = Settings {
            remember_last_scan_location: false,
            ..Default::default()
        };
        assert_eq!(
            resolve_root(&off, remembered.to_str(), os_default.clone()).unwrap(),
            os_default
        );

        assert_eq!(
            resolve_root(&on, Some("/no/such/folder"), os_default.clone()).unwrap(),
            os_default
        );
    }

    #[test]
    fn is_excluded_covers_paths_extensions_and_folders() {
        let settings = Settings {
            ignored_extensions: vec![".ISO".to_string(), "torrent".to_string()],
            ignored_folders: vec!["/dl/keep".to_string()],
            ..Default::default()
        };
        let ignored: HashSet<String> = ["/dl/secret.txt".to_string()].into_iter().collect();

        assert!(settings.is_excluded(&file("/dl/secret.txt", Some("txt")), &ignored));
        assert!(settings.is_excluded(&file("/dl/a.iso", Some("iso")), &ignored));
        assert!(settings.is_excluded(&file("/dl/keep/a.txt", Some("txt")), &ignored));
        assert!(!settings.is_excluded(&file("/dl/keepsake/a.txt", Some("txt")), &ignored));
        assert!(!settings.is_excluded(&file("/dl/normal.txt", Some("txt")), &ignored));
    }
}
