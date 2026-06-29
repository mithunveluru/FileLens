//! User settings, persisted as a single JSON document in the `settings` table.
//! `#[serde(default)]` makes old stored documents forward-compatible: new fields
//! fall back to their defaults instead of failing to parse.

pub mod commands;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::analysis::AnalysisConfig;
use crate::database::Database;
use crate::filesystem::FileEntry;

const SETTINGS_KEY: &str = "app";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Override for the scan root; `None`/empty means the OS Downloads folder.
    pub downloads_folder: Option<String>,
    pub age_threshold_days: i64,
    pub large_file_min_mb: u64,
    pub ignored_folders: Vec<String>,
    pub ignored_extensions: Vec<String>,
    /// `"system"`, `"light"`, or `"dark"` — applied by the frontend.
    pub theme: String,
    pub auto_scan_on_startup: bool,
    /// Start File Lens automatically when the user logs in (OS-level autostart).
    pub launch_on_startup: bool,
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
        }
    }
}

impl Settings {
    /// Builds the analysis thresholds from the user's settings.
    pub fn analysis_config(&self) -> AnalysisConfig {
        AnalysisConfig {
            large_file_min_bytes: self.large_file_min_mb.saturating_mul(1024 * 1024),
            old_file_max_age_days: self.age_threshold_days,
        }
    }

    /// True if a file should be excluded from analysis: explicitly ignored, an
    /// ignored extension, or inside an ignored folder.
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

/// Resolves the scan/organization root: the configured override if set and
/// valid, otherwise the OS Downloads folder passed in by the caller.
pub fn resolve_root(settings: &Settings, os_default: PathBuf) -> Result<PathBuf, String> {
    match settings
        .downloads_folder
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(folder) => {
            let path = PathBuf::from(folder);
            if path.is_dir() {
                Ok(path)
            } else {
                Err("The configured Downloads folder does not exist.".into())
            }
        }
        None => Ok(os_default),
    }
}

/// Loads settings, falling back to defaults if absent or unparseable.
pub fn load(db: &Database) -> rusqlite::Result<Settings> {
    Ok(match db.get_setting(SETTINGS_KEY)? {
        Some(json) => serde_json::from_str(&json).unwrap_or_default(),
        None => Settings::default(),
    })
}

/// Persists settings as JSON.
pub fn save(db: &Database, settings: &Settings) -> Result<(), String> {
    let json = serde_json::to_string(settings).map_err(|err| err.to_string())?;
    db.set_setting(SETTINGS_KEY, &json)
        .map_err(|err| err.to_string())
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
    fn is_excluded_covers_paths_extensions_and_folders() {
        let settings = Settings {
            ignored_extensions: vec![".ISO".to_string(), "torrent".to_string()],
            ignored_folders: vec!["/dl/keep".to_string()],
            ..Default::default()
        };
        let ignored: HashSet<String> = ["/dl/secret.txt".to_string()].into_iter().collect();

        assert!(settings.is_excluded(&file("/dl/secret.txt", Some("txt")), &ignored));
        assert!(settings.is_excluded(&file("/dl/a.iso", Some("iso")), &ignored)); // case + dot insensitive
        assert!(settings.is_excluded(&file("/dl/keep/a.txt", Some("txt")), &ignored));
        assert!(!settings.is_excluded(&file("/dl/keepsake/a.txt", Some("txt")), &ignored)); // not a prefix match
        assert!(!settings.is_excluded(&file("/dl/normal.txt", Some("txt")), &ignored));
    }
}
