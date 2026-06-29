//! Turns a single file on disk into typed metadata.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

// camelCase to match the TypeScript FileEntry.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub extension: Option<String>,
    pub path: String,
    pub size_bytes: u64,
    pub created_ms: Option<i64>,
    pub modified_ms: Option<i64>,
    pub mime_type: String,
    pub is_hidden: bool,
}

// Caller must skip directories and symlinks first.
pub fn read_entry(path: &Path, metadata: &std::fs::Metadata) -> Option<FileEntry> {
    let name = path.file_name()?.to_string_lossy().into_owned();
    let extension = path
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase());
    let mime_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();
    // Dotfile check covers Linux/macOS; Windows hidden attributes are not yet handled.
    let is_hidden = name.starts_with('.');

    Some(FileEntry {
        name,
        extension,
        path: path.to_string_lossy().into_owned(),
        size_bytes: metadata.len(),
        created_ms: metadata.created().ok().and_then(system_time_to_millis),
        modified_ms: metadata.modified().ok().and_then(system_time_to_millis),
        mime_type,
        is_hidden,
    })
}

fn system_time_to_millis(time: SystemTime) -> Option<i64> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn reads_extension_mime_and_hidden_flag() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".secret.JSON");
        fs::write(&path, b"{}").unwrap();
        let metadata = fs::metadata(&path).unwrap();

        let entry = read_entry(&path, &metadata).unwrap();

        assert_eq!(entry.extension.as_deref(), Some("json"));
        assert!(entry.is_hidden);
        assert_eq!(entry.mime_type, "application/json");
        assert_eq!(entry.size_bytes, 2);
    }

    #[test]
    fn extensionless_file_has_no_extension_and_octet_stream_mime() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("README");
        fs::write(&path, b"x").unwrap();
        let metadata = fs::metadata(&path).unwrap();

        let entry = read_entry(&path, &metadata).unwrap();

        assert_eq!(entry.extension, None);
        assert_eq!(entry.mime_type, "application/octet-stream");
        assert!(!entry.is_hidden);
    }
}
