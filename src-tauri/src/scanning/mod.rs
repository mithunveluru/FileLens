//! Recursively walks a folder into a file inventory.

pub mod commands;

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use log::warn;
use serde::Serialize;
use walkdir::WalkDir;

use crate::filesystem::{read_entry, FileEntry};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanOutcome {
    pub files: Vec<FileEntry>,
    pub error_count: usize,
    pub cancelled: bool,
    // Set when the walk hit `max_files` and stopped. The caller refuses to
    // persist such a scan, since it almost always means the wrong (huge) root.
    pub limit_exceeded: bool,
}

// Throttled to avoid flooding the IPC bridge.
const PROGRESS_INTERVAL: usize = 100;

// Symlinks are never followed; unreadable entries are logged and skipped; a set
// `cancel` flag stops the walk early without error. Once `max_files` files have
// been collected the walk stops and reports `limit_exceeded`.
pub fn scan(
    root: &Path,
    cancel: &AtomicBool,
    max_files: usize,
    mut on_progress: impl FnMut(usize),
) -> ScanOutcome {
    let mut files = Vec::new();
    let mut error_count = 0usize;

    for entry in WalkDir::new(root).follow_links(false) {
        if cancel.load(Ordering::Relaxed) {
            return ScanOutcome {
                files,
                error_count,
                cancelled: true,
                limit_exceeded: false,
            };
        }
        if files.len() >= max_files {
            return ScanOutcome {
                files,
                error_count,
                cancelled: false,
                limit_exceeded: true,
            };
        }

        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                warn!("skipping unreadable entry: {err}");
                error_count += 1;
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                warn!("skipping {:?}: {err}", entry.path());
                error_count += 1;
                continue;
            }
        };

        match read_entry(entry.path(), &metadata) {
            Some(file) => {
                files.push(file);
                if files.len() % PROGRESS_INTERVAL == 0 {
                    on_progress(files.len());
                }
            }
            None => error_count += 1,
        }
    }

    ScanOutcome {
        files,
        error_count,
        cancelled: false,
        limit_exceeded: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scans_files_recursively_skipping_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("a.txt"), b"hello").unwrap();
        fs::create_dir(root.join("sub")).unwrap();
        fs::write(root.join("sub").join("b.pdf"), b"data").unwrap();

        let outcome = scan(root, &AtomicBool::new(false), usize::MAX, |_| {});

        assert_eq!(outcome.files.len(), 2);
        assert_eq!(outcome.error_count, 0);
        assert!(!outcome.cancelled);
        let names: Vec<&str> = outcome.files.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"a.txt"));
        assert!(names.contains(&"b.pdf"));
    }

    #[test]
    fn empty_directory_yields_an_empty_inventory() {
        let dir = tempfile::tempdir().unwrap();

        let outcome = scan(dir.path(), &AtomicBool::new(false), usize::MAX, |_| {});

        assert!(outcome.files.is_empty());
        assert_eq!(outcome.error_count, 0);
        assert!(!outcome.cancelled);
    }

    #[test]
    fn inventories_unicode_file_names() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("résumé café.pdf"), b"x").unwrap();

        let outcome = scan(dir.path(), &AtomicBool::new(false), usize::MAX, |_| {});

        assert_eq!(outcome.files.len(), 1);
        assert_eq!(outcome.files[0].name, "résumé café.pdf");
        assert_eq!(outcome.files[0].extension.as_deref(), Some("pdf"));
    }

    #[test]
    fn already_cancelled_walk_reports_cancelled() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), b"x").unwrap();

        let outcome = scan(dir.path(), &AtomicBool::new(true), usize::MAX, |_| {});

        assert!(outcome.cancelled);
    }

    #[test]
    fn stops_and_flags_when_file_limit_is_exceeded() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..5 {
            fs::write(dir.path().join(format!("f{i}.txt")), b"x").unwrap();
        }

        let outcome = scan(dir.path(), &AtomicBool::new(false), 2, |_| {});

        assert!(outcome.limit_exceeded);
        assert!(!outcome.cancelled);
        assert_eq!(outcome.files.len(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn does_not_follow_symlinks() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("real.txt");
        fs::write(&target, b"x").unwrap();
        symlink(&target, dir.path().join("link.txt")).unwrap();

        let outcome = scan(dir.path(), &AtomicBool::new(false), usize::MAX, |_| {});

        assert_eq!(outcome.files.len(), 1);
        assert_eq!(outcome.files[0].name, "real.txt");
    }
}
