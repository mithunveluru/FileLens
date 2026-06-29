//! Builds an organization plan. Pure: filesystem existence is injected as `exists`.

use std::path::Path;

use super::{
    classifier, ActionStatus, CategoryCount, ConflictStrategy, OrganizationAction,
    OrganizationPlan, PlanSummary, ALL_KINDS,
};
use crate::filesystem::FileEntry;

// Only files directly inside `root` are organized; nested folders are left untouched.
pub fn build_plan(
    files: &[FileEntry],
    root: &Path,
    exists: &dyn Fn(&Path) -> bool,
) -> OrganizationPlan {
    let mut actions: Vec<OrganizationAction> = files
        .iter()
        .filter(|f| is_top_level(&f.path, root))
        .map(|f| action_for(f, root))
        .collect();

    for action in &mut actions {
        action.conflict = exists(Path::new(&action.destination));
    }

    let summary = summarize(&actions);
    OrganizationPlan {
        root: root.to_string_lossy().into_owned(),
        actions,
        summary,
    }
}

fn is_top_level(path: &str, root: &Path) -> bool {
    Path::new(path).parent() == Some(root)
}

fn action_for(file: &FileEntry, root: &Path) -> OrganizationAction {
    let result = classifier::classify(file);
    let name = Path::new(&file.path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let destination = root
        .join(result.kind.folder_name())
        .join(&name)
        .to_string_lossy()
        .into_owned();

    OrganizationAction {
        source: file.path.clone(),
        destination,
        kind: result.kind,
        reason: result.reason,
        conflict: false,
        strategy: ConflictStrategy::KeepBoth,
        status: ActionStatus::Accepted,
    }
}

fn summarize(actions: &[OrganizationAction]) -> PlanSummary {
    let conflicts = actions.iter().filter(|a| a.conflict).count();
    let categories = ALL_KINDS
        .into_iter()
        .filter_map(|kind| {
            let count = actions.iter().filter(|a| a.kind == kind).count();
            (count > 0).then_some(CategoryCount { kind, count })
        })
        .collect();
    PlanSummary {
        total: actions.len(),
        conflicts,
        categories,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organization::FileKind;

    fn file(path: &str, ext: Option<&str>) -> FileEntry {
        FileEntry {
            name: Path::new(path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            extension: ext.map(str::to_string),
            path: path.to_string(),
            size_bytes: 1,
            created_ms: None,
            modified_ms: None,
            mime_type: "application/octet-stream".to_string(),
            is_hidden: false,
        }
    }

    fn never_exists(_: &Path) -> bool {
        false
    }

    #[test]
    fn proposes_destination_in_category_folder() {
        let files = [file("/dl/a.png", Some("png"))];
        let plan = build_plan(&files, Path::new("/dl"), &never_exists);

        assert_eq!(plan.actions.len(), 1);
        let action = &plan.actions[0];
        assert_eq!(action.kind, FileKind::Images);
        assert_eq!(action.destination, "/dl/Images/a.png");
        assert!(!action.conflict);
    }

    #[test]
    fn organizes_only_top_level_files() {
        let files = [
            file("/dl/a.png", Some("png")),
            file("/dl/Images/old.png", Some("png")),
        ];
        let plan = build_plan(&files, Path::new("/dl"), &never_exists);

        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].source, "/dl/a.png");
    }

    #[test]
    fn marks_conflict_when_destination_exists() {
        let files = [file("/dl/a.png", Some("png"))];
        let plan = build_plan(&files, Path::new("/dl"), &|p| {
            p == Path::new("/dl/Images/a.png")
        });

        assert!(plan.actions[0].conflict);
        assert_eq!(plan.summary.conflicts, 1);
    }

    #[test]
    fn is_deterministic_with_per_category_summary() {
        let files = [
            file("/dl/a.png", Some("png")),
            file("/dl/b.pdf", Some("pdf")),
        ];
        let first = build_plan(&files, Path::new("/dl"), &never_exists);
        let second = build_plan(&files, Path::new("/dl"), &never_exists);

        assert_eq!(first.actions.len(), second.actions.len());
        assert_eq!(first.summary.total, 2);
        assert_eq!(first.summary.categories.len(), 2);
    }
}
