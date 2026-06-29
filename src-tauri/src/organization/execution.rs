//! Applies an approved plan to the filesystem. This is the only organization
//! module that touches disk. It validates every move stays inside the Downloads
//! root, resolves conflicts, and continues past recoverable per-file errors,
//! collecting results for reporting and history.

use std::fs;
use std::path::{Component, Path, PathBuf};

use super::{conflict, ActionStatus, OrganizationAction};

/// A path is a safe descendant of `root` only if it is lexically under it *and*
/// contains no `..` component. The `..` guard matters because the plan crosses
/// an untrusted IPC boundary: `Path::starts_with` alone would accept
/// `<root>/../escape`, which resolves outside the Downloads folder.
fn is_safe_descendant(root: &Path, path: &Path) -> bool {
    path.starts_with(root) && !path.components().any(|c| c == Component::ParentDir)
}

/// A move that completed successfully — enough to reverse it later (undo).
pub struct ExecutedMove {
    pub source: String,
    pub destination: String,
}

/// The result of executing a plan.
pub struct ExecutionReport {
    pub moves: Vec<ExecutedMove>,
    pub skipped: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// Executes the accepted actions, keeping every move inside `root`. A failure on
/// one file is recorded and the rest continue.
pub fn execute(root: &Path, actions: &[OrganizationAction]) -> ExecutionReport {
    let mut report = ExecutionReport {
        moves: Vec::new(),
        skipped: 0,
        failed: 0,
        errors: Vec::new(),
    };

    for action in actions {
        if action.status != ActionStatus::Accepted {
            report.skipped += 1;
            continue;
        }
        match apply(root, action) {
            Ok(Some(destination)) => {
                report.moves.push(ExecutedMove {
                    source: action.source.clone(),
                    destination,
                });
            }
            Ok(None) => report.skipped += 1,
            Err(err) => {
                report.failed += 1;
                report.errors.push(format!("{}: {err}", action.source));
            }
        }
    }

    report
}

/// Performs one move. Returns the final destination, or `None` if a conflict
/// strategy chose to skip it.
fn apply(root: &Path, action: &OrganizationAction) -> Result<Option<String>, String> {
    let source = Path::new(&action.source);
    let intended = PathBuf::from(&action.destination);

    if !is_safe_descendant(root, source) || !is_safe_descendant(root, &intended) {
        return Err("refusing to move outside the Downloads folder".into());
    }
    if !source.exists() {
        return Err("source no longer exists".into());
    }

    let final_dest =
        match conflict::resolve_destination(&intended, action.strategy, &|p| p.exists()) {
            Some(dest) => dest,
            None => return Ok(None),
        };

    move_file(source, &final_dest).map_err(|err| err.to_string())?;
    Ok(Some(final_dest.to_string_lossy().into_owned()))
}

/// The result of undoing a session.
pub struct UndoReport {
    pub restored: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// Reverses recorded moves (destination back to source). Never overwrites a
/// file that has since reappeared at the original location.
pub fn undo(root: &Path, moves: &[(String, String)]) -> UndoReport {
    let mut report = UndoReport {
        restored: 0,
        failed: 0,
        errors: Vec::new(),
    };

    for (source, destination) in moves {
        let src = Path::new(source);
        let dst = Path::new(destination);

        let outcome = if !is_safe_descendant(root, src) || !is_safe_descendant(root, dst) {
            Err("refusing to move outside the Downloads folder".to_string())
        } else if !dst.exists() {
            Err("the moved file no longer exists".to_string())
        } else if src.exists() {
            Err("the original location is now occupied".to_string())
        } else {
            move_file(dst, src).map_err(|err| err.to_string())
        };

        match outcome {
            Ok(()) => report.restored += 1,
            Err(err) => {
                report.failed += 1;
                report.errors.push(format!("{destination}: {err}"));
            }
        }
    }

    report
}

fn move_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    // For the Replace strategy the destination may exist; remove it so rename
    // works across platforms. Renamed/KeepBoth destinations are always free.
    if destination.exists() {
        fs::remove_file(destination)?;
    }
    if fs::rename(source, destination).is_ok() {
        return Ok(());
    }
    // Cross-filesystem rename fails; fall back to copy + remove.
    fs::copy(source, destination)?;
    fs::remove_file(source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organization::{ConflictStrategy, FileKind};

    fn action(source: &str, destination: &str) -> OrganizationAction {
        OrganizationAction {
            source: source.to_string(),
            destination: destination.to_string(),
            kind: FileKind::Other,
            reason: String::new(),
            conflict: false,
            strategy: ConflictStrategy::KeepBoth,
            status: ActionStatus::Accepted,
        }
    }

    #[test]
    fn moves_accepted_file_creating_folder() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let source = root.join("a.txt");
        fs::write(&source, b"x").unwrap();
        let dest = root.join("Documents").join("a.txt");

        let report = execute(
            root,
            &[action(source.to_str().unwrap(), dest.to_str().unwrap())],
        );

        assert_eq!(report.moves.len(), 1);
        assert!(dest.exists());
        assert!(!source.exists());
    }

    #[test]
    fn skips_unaccepted_actions() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("a.txt");
        fs::write(&source, b"x").unwrap();
        let mut act = action(
            source.to_str().unwrap(),
            dir.path().join("Documents/a.txt").to_str().unwrap(),
        );
        act.status = ActionStatus::Skipped;

        let report = execute(dir.path(), &[act]);

        assert_eq!(report.skipped, 1);
        assert!(source.exists());
    }

    #[test]
    fn keep_both_renames_on_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let source = root.join("a.txt");
        fs::write(&source, b"new").unwrap();
        let dest_dir = root.join("Documents");
        fs::create_dir_all(&dest_dir).unwrap();
        fs::write(dest_dir.join("a.txt"), b"existing").unwrap();

        let report = execute(
            root,
            &[action(
                source.to_str().unwrap(),
                dest_dir.join("a.txt").to_str().unwrap(),
            )],
        );

        assert_eq!(report.moves.len(), 1);
        assert!(dest_dir.join("a.txt").exists()); // original kept
        assert!(dest_dir.join("a (1).txt").exists()); // both kept
    }

    #[test]
    fn missing_source_is_recorded_and_others_continue() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let present = root.join("b.txt");
        fs::write(&present, b"x").unwrap();

        let report = execute(
            root,
            &[
                action(
                    root.join("ghost.txt").to_str().unwrap(),
                    root.join("Documents/ghost.txt").to_str().unwrap(),
                ),
                action(
                    present.to_str().unwrap(),
                    root.join("Documents/b.txt").to_str().unwrap(),
                ),
            ],
        );

        assert_eq!(report.failed, 1);
        assert_eq!(report.moves.len(), 1);
    }

    #[test]
    fn refuses_destination_outside_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let source = root.join("a.txt");
        fs::write(&source, b"x").unwrap();

        let report = execute(root, &[action(source.to_str().unwrap(), "/tmp/evil/a.txt")]);

        assert_eq!(report.failed, 1);
        assert!(source.exists());
    }

    #[test]
    fn refuses_directory_traversal_destination() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let source = root.join("a.txt");
        fs::write(&source, b"x").unwrap();
        // Lexically "under" root via starts_with, but escapes through "..".
        let escaping = root.join("..").join("escaped.txt");

        let report = execute(
            root,
            &[action(source.to_str().unwrap(), escaping.to_str().unwrap())],
        );

        assert_eq!(report.failed, 1);
        assert!(source.exists());
        assert!(!escaping.exists());
    }

    #[test]
    fn undo_restores_a_moved_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let source = root.join("a.txt");
        let dest = root.join("Documents").join("a.txt");
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::write(&dest, b"x").unwrap();

        let report = undo(
            root,
            &[(
                source.to_string_lossy().into_owned(),
                dest.to_string_lossy().into_owned(),
            )],
        );

        assert_eq!(report.restored, 1);
        assert!(source.exists());
        assert!(!dest.exists());
    }

    #[test]
    fn undo_does_not_overwrite_a_reappeared_original() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let source = root.join("a.txt");
        let dest = root.join("Documents").join("a.txt");
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::write(&dest, b"moved").unwrap();
        fs::write(&source, b"new file with same name").unwrap();

        let report = undo(
            root,
            &[(
                source.to_string_lossy().into_owned(),
                dest.to_string_lossy().into_owned(),
            )],
        );

        assert_eq!(report.failed, 1);
        assert_eq!(fs::read(&source).unwrap(), b"new file with same name");
    }
}
