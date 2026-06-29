//! Conflict resolution for destinations that already exist at execution time.
//! Pure: the "is this path taken?" check is injected, so the naming logic is
//! unit-tested without touching disk.

use std::path::{Path, PathBuf};

use super::ConflictStrategy;

/// Computes the final destination for a move, applying `strategy` only if the
/// intended `dest` is already taken. Returns `None` when the move should be
/// skipped.
pub fn resolve_destination(
    dest: &Path,
    strategy: ConflictStrategy,
    taken: &dyn Fn(&Path) -> bool,
) -> Option<PathBuf> {
    if !taken(dest) {
        return Some(dest.to_path_buf());
    }
    match strategy {
        ConflictStrategy::Skip => None,
        ConflictStrategy::Replace => Some(dest.to_path_buf()),
        // Rename and KeepBoth both preserve the existing file by choosing a new,
        // unused numbered name.
        ConflictStrategy::Rename | ConflictStrategy::KeepBoth => Some(unique_name(dest, taken)),
    }
}

fn unique_name(dest: &Path, taken: &dyn Fn(&Path) -> bool) -> PathBuf {
    let parent = dest.parent().unwrap_or_else(|| Path::new(""));
    let stem = dest
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let extension = dest.extension().map(|e| e.to_string_lossy().into_owned());

    let mut counter = 1u32;
    loop {
        let name = match &extension {
            Some(ext) => format!("{stem} ({counter}).{ext}"),
            None => format!("{stem} ({counter})"),
        };
        let candidate = parent.join(name);
        if !taken(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_destination_when_free() {
        let dest = Path::new("/dl/Images/a.png");
        let resolved = resolve_destination(dest, ConflictStrategy::Rename, &|_| false);
        assert_eq!(resolved, Some(dest.to_path_buf()));
    }

    #[test]
    fn skip_returns_none() {
        let resolved =
            resolve_destination(Path::new("/dl/a.png"), ConflictStrategy::Skip, &|_| true);
        assert_eq!(resolved, None);
    }

    #[test]
    fn replace_keeps_destination() {
        let dest = Path::new("/dl/a.png");
        let resolved = resolve_destination(dest, ConflictStrategy::Replace, &|_| true);
        assert_eq!(resolved, Some(dest.to_path_buf()));
    }

    #[test]
    fn rename_finds_first_free_numbered_name() {
        let dest = Path::new("/dl/Images/a.png");
        let taken =
            |p: &Path| p == Path::new("/dl/Images/a.png") || p == Path::new("/dl/Images/a (1).png");
        let resolved = resolve_destination(dest, ConflictStrategy::KeepBoth, &taken);
        assert_eq!(resolved, Some(PathBuf::from("/dl/Images/a (2).png")));
    }
}
