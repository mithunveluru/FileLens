//! Verified duplicate detection.
//!
//! A staged pipeline turns the file inventory into cryptographically verified
//! duplicate groups. Each stage cuts the candidate set before the next, more
//! expensive one runs, so full hashing touches only files that survived the
//! cheaper filters:
//!
//!   size groups  ->  sampled fingerprint  ->  full BLAKE3 hash  ->  groups
//!
//! Only files sharing a full hash are reported as duplicates; nothing is ever
//! called a duplicate on the strength of name, extension, or size alone. The
//! pipeline is strictly read-only.

pub mod cache;
pub mod commands;
mod fingerprint;
mod hash;

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;

use serde::Serialize;

use crate::filesystem::FileEntry;
use cache::HashCache;

/// How sure we are that a group is a real duplicate set. Only `Verified` is
/// produced today; the weaker variants are reserved for future strategies
/// (e.g. perceptual image matching) so the model need not change to add them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // weaker variants are reserved for future match strategies
pub enum VerificationStatus {
    Verified,
    PossibleDuplicate,
    SimilarMetadata,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateCandidate {
    pub path: String,
    pub size_bytes: u64,
    pub modified_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    pub hash: String,
    pub status: VerificationStatus,
    pub size_bytes: u64,
    pub copies: usize,
    /// Bytes freed if all but one copy were removed. The decision is the user's;
    /// this is only the arithmetic.
    pub reclaimable_bytes: u64,
    pub files: Vec<DuplicateCandidate>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateReport {
    pub groups: Vec<DuplicateGroup>,
    pub total_groups: usize,
    pub redundant_files: usize,
    pub reclaimable_bytes: u64,
    pub files_hashed: usize,
    pub cache_hits: usize,
    pub errors: Vec<String>,
    // Set when the user stopped the run; the groups are then partial.
    pub cancelled: bool,
}

/// Runs the full pipeline over an already-scanned inventory (the scanner and
/// metadata extractor are reused, not rebuilt). `workers` bounds concurrency.
/// A set `cancel` flag stops the run early and marks the report partial;
/// `on_progress` receives the running count of files through the hash stage.
pub fn detect_duplicates(
    files: &[FileEntry],
    cache: &dyn HashCache,
    workers: usize,
    cancel: &AtomicBool,
    on_progress: &(dyn Fn(usize) + Sync),
) -> DuplicateReport {
    let mut errors = Vec::new();

    let size_survivors = candidates_by_size(files);
    let fingerprint_survivors = fingerprint_stage(size_survivors, workers, cancel, &mut errors);
    let hashed = hash_stage(
        fingerprint_survivors,
        cache,
        workers,
        cancel,
        on_progress,
        &mut errors,
    );

    build_report(
        hashed.groups,
        hashed.files_hashed,
        hashed.cache_hits,
        errors,
        cancel.load(Ordering::Relaxed),
    )
}

// Emitting per file would flood the IPC bridge on a large candidate set.
const PROGRESS_INTERVAL: usize = 25;

// Stage 1 — candidate builder. Files with a unique size cannot have a
// byte-identical twin, so only size collisions survive. Zero-byte files are
// dropped: they are trivially "identical" but free no space and only add noise.
fn candidates_by_size(files: &[FileEntry]) -> Vec<&FileEntry> {
    let mut by_size: HashMap<u64, Vec<&FileEntry>> = HashMap::new();
    for file in files.iter().filter(|f| f.size_bytes > 0) {
        by_size.entry(file.size_bytes).or_default().push(file);
    }
    by_size
        .into_values()
        .filter(|group| group.len() >= 2)
        .flatten()
        .collect()
}

// Stage 2 — fingerprint. Split size groups by a cheap sampled signature; only
// files whose fingerprints still collide need full hashing.
fn fingerprint_stage<'a>(
    candidates: Vec<&'a FileEntry>,
    workers: usize,
    cancel: &AtomicBool,
    errors: &mut Vec<String>,
) -> Vec<&'a FileEntry> {
    let fingerprints = parallel_map(candidates, workers, cancel, |file| {
        (
            file,
            fingerprint::fingerprint(Path::new(&file.path), file.size_bytes),
        )
    });

    let mut groups: HashMap<(u64, String), Vec<&FileEntry>> = HashMap::new();
    for (file, result) in fingerprints {
        match result {
            Ok(fp) => groups.entry((file.size_bytes, fp)).or_default().push(file),
            Err(err) => errors.push(format!("{}: {err}", file.path)),
        }
    }
    groups
        .into_values()
        .filter(|group| group.len() >= 2)
        .flatten()
        .collect()
}

struct Hashed {
    groups: HashMap<String, Vec<DuplicateCandidate>>,
    files_hashed: usize,
    cache_hits: usize,
}

// Stage 3 — verify. Full streaming hash (cache-aware); group by hash. A hashing
// failure drops that one file and is recorded, never aborting the run.
fn hash_stage(
    candidates: Vec<&FileEntry>,
    cache: &dyn HashCache,
    workers: usize,
    cancel: &AtomicBool,
    on_progress: &(dyn Fn(usize) + Sync),
    errors: &mut Vec<String>,
) -> Hashed {
    let hashed_count = AtomicUsize::new(0);
    let cache_hits = AtomicUsize::new(0);
    let processed = AtomicUsize::new(0);

    let results = parallel_map(candidates, workers, cancel, |file| {
        let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
        if done.is_multiple_of(PROGRESS_INTERVAL) {
            on_progress(done);
        }
        let outcome = match cache.get(&file.path, file.size_bytes, file.modified_ms) {
            Some(cached) => {
                cache_hits.fetch_add(1, Ordering::Relaxed);
                Ok(cached)
            }
            None => hash::hash_file(Path::new(&file.path)).inspect(|digest| {
                hashed_count.fetch_add(1, Ordering::Relaxed);
                cache.put(&file.path, file.size_bytes, file.modified_ms, digest);
            }),
        };
        (file, outcome)
    });

    let mut groups: HashMap<String, Vec<DuplicateCandidate>> = HashMap::new();
    for (file, outcome) in results {
        match outcome {
            Ok(hash) => groups.entry(hash).or_default().push(DuplicateCandidate {
                path: file.path.clone(),
                size_bytes: file.size_bytes,
                modified_ms: file.modified_ms,
            }),
            Err(err) => errors.push(format!("{}: {err}", file.path)),
        }
    }

    Hashed {
        groups,
        files_hashed: hashed_count.into_inner(),
        cache_hits: cache_hits.into_inner(),
    }
}

fn build_report(
    hash_groups: HashMap<String, Vec<DuplicateCandidate>>,
    files_hashed: usize,
    cache_hits: usize,
    errors: Vec<String>,
    cancelled: bool,
) -> DuplicateReport {
    let mut groups: Vec<DuplicateGroup> = hash_groups
        .into_iter()
        .filter(|(_, files)| files.len() >= 2)
        .map(|(hash, files)| {
            let size_bytes = files[0].size_bytes;
            let copies = files.len();
            DuplicateGroup {
                hash,
                status: VerificationStatus::Verified,
                size_bytes,
                copies,
                reclaimable_bytes: size_bytes * (copies as u64 - 1),
                files,
            }
        })
        .collect();

    // Biggest reclaimable payoff first; stable output for the UI.
    groups.sort_by_key(|g| std::cmp::Reverse(g.reclaimable_bytes));

    DuplicateReport {
        total_groups: groups.len(),
        redundant_files: groups.iter().map(|g| g.copies - 1).sum(),
        reclaimable_bytes: groups.iter().map(|g| g.reclaimable_bytes).sum(),
        files_hashed,
        cache_hits,
        errors,
        cancelled,
        groups,
    }
}

// A bounded worker pool: `workers` threads drain a shared queue. Result order is
// not preserved because every stage regroups by key afterwards.
fn parallel_map<T, R, F>(items: Vec<T>, workers: usize, cancel: &AtomicBool, f: F) -> Vec<R>
where
    T: Send,
    R: Send,
    F: Fn(T) -> R + Sync,
{
    if items.is_empty() {
        return Vec::new();
    }
    let workers = workers.clamp(1, items.len());
    let queue = Mutex::new(items.into_iter());
    let results = Mutex::new(Vec::new());

    thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                let next = queue.lock().expect("queue mutex poisoned").next();
                match next {
                    Some(item) => {
                        let mapped = f(item);
                        results.lock().expect("results mutex poisoned").push(mapped);
                    }
                    None => break,
                }
            });
        }
    });

    results.into_inner().expect("results mutex poisoned")
}

#[cfg(test)]
mod tests {
    use super::cache::test_cache::MemoryCache;
    use super::*;
    use std::fs;

    fn entry(path: &str, size: u64, modified_ms: Option<i64>) -> FileEntry {
        FileEntry {
            name: Path::new(path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            extension: None,
            path: path.to_string(),
            size_bytes: size,
            created_ms: None,
            modified_ms,
            mime_type: "application/octet-stream".to_string(),
            is_hidden: false,
        }
    }

    fn write(dir: &Path, name: &str, bytes: &[u8]) -> FileEntry {
        let path = dir.join(name);
        fs::write(&path, bytes).unwrap();
        entry(path.to_str().unwrap(), bytes.len() as u64, Some(1))
    }

    fn detect(files: &[FileEntry], cache: &dyn HashCache, workers: usize) -> DuplicateReport {
        detect_duplicates(files, cache, workers, &AtomicBool::new(false), &|_| {})
    }

    fn run(files: &[FileEntry]) -> DuplicateReport {
        detect(files, &MemoryCache::default(), 4)
    }

    #[test]
    fn identical_content_under_different_names_is_a_verified_duplicate() {
        let dir = tempfile::tempdir().unwrap();
        let a = write(dir.path(), "a.txt", b"shared payload");
        let b = write(dir.path(), "copy-of-a.txt", b"shared payload");

        let report = run(&[a, b]);

        assert_eq!(report.total_groups, 1);
        assert_eq!(report.groups[0].status, VerificationStatus::Verified);
        assert_eq!(report.groups[0].copies, 2);
        assert_eq!(report.redundant_files, 1);
    }

    #[test]
    fn same_size_different_content_is_not_a_duplicate() {
        let dir = tempfile::tempdir().unwrap();
        let a = write(dir.path(), "a.bin", b"AAAAAAAA");
        let b = write(dir.path(), "b.bin", b"BBBBBBBB");

        let report = run(&[a, b]);

        assert_eq!(report.total_groups, 0);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn same_name_different_content_in_subfolders_is_not_a_duplicate() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("one")).unwrap();
        fs::create_dir(dir.path().join("two")).unwrap();
        let a = write(dir.path(), "one/report.txt", b"first version.");
        let b = write(dir.path(), "two/report.txt", b"second versio!");

        let report = run(&[a, b]);

        assert_eq!(report.total_groups, 0);
    }

    #[test]
    fn reports_copies_and_reclaimable_space_keeping_one() {
        let dir = tempfile::tempdir().unwrap();
        let payload = vec![42u8; 1000];
        let a = write(dir.path(), "a", &payload);
        let b = write(dir.path(), "b", &payload);
        let c = write(dir.path(), "c", &payload);

        let report = run(&[a, b, c]);

        assert_eq!(report.groups[0].copies, 3);
        assert_eq!(report.reclaimable_bytes, 2000);
        assert_eq!(report.redundant_files, 2);
    }

    #[test]
    fn empty_files_are_not_reported() {
        let dir = tempfile::tempdir().unwrap();
        let a = write(dir.path(), "a", b"");
        let b = write(dir.path(), "b", b"");

        let report = run(&[a, b]);

        assert_eq!(report.total_groups, 0);
    }

    #[test]
    fn unchanged_files_reuse_cached_hashes_on_second_run() {
        let dir = tempfile::tempdir().unwrap();
        let a = write(dir.path(), "a", b"cache me");
        let b = write(dir.path(), "b", b"cache me");
        let cache = MemoryCache::default();

        let first = detect(&[a.clone(), b.clone()], &cache, 2);
        assert_eq!(first.files_hashed, 2);
        assert_eq!(first.cache_hits, 0);

        let second = detect(&[a, b], &cache, 2);
        assert_eq!(second.files_hashed, 0);
        assert_eq!(second.cache_hits, 2);
        assert_eq!(second.total_groups, 1);
    }

    #[test]
    fn editing_a_file_invalidates_its_cached_hash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a");
        fs::write(&path, b"original").unwrap();
        let cache = MemoryCache::default();

        let v1 = entry(path.to_str().unwrap(), 8, Some(1));
        detect(&[v1], &cache, 1);

        // Same path, new size and mtime -> cache must not match.
        fs::write(&path, b"the file has grown").unwrap();
        let v2 = entry(path.to_str().unwrap(), 18, Some(2));
        let report = detect(&[v2], &cache, 1);
        assert_eq!(report.cache_hits, 0);
    }

    #[test]
    fn a_deleted_file_is_recorded_and_others_still_verify() {
        let dir = tempfile::tempdir().unwrap();
        let a = write(dir.path(), "a", b"present twins");
        let b = write(dir.path(), "b", b"present twins");
        let ghost = entry(dir.path().join("ghost").to_str().unwrap(), 13, Some(1));

        let report = run(&[a, b, ghost]);

        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.total_groups, 1);
        assert_eq!(report.groups[0].copies, 2);
    }

    #[test]
    fn a_cancelled_run_is_flagged_and_reports_no_groups() {
        let dir = tempfile::tempdir().unwrap();
        let a = write(dir.path(), "a", b"twins");
        let b = write(dir.path(), "b", b"twins");

        let report = detect_duplicates(
            &[a, b],
            &MemoryCache::default(),
            2,
            &AtomicBool::new(true),
            &|_| {},
        );

        assert!(report.cancelled);
        assert_eq!(report.total_groups, 0);
    }

    #[test]
    fn progress_is_reported_during_hashing() {
        let dir = tempfile::tempdir().unwrap();
        let payload = vec![3u8; 64];
        let files: Vec<FileEntry> = (0..PROGRESS_INTERVAL * 2)
            .map(|i| write(dir.path(), &format!("f{i}"), &payload))
            .collect();
        let seen = Mutex::new(Vec::new());

        detect_duplicates(
            &files,
            &MemoryCache::default(),
            2,
            &AtomicBool::new(false),
            &|n| seen.lock().unwrap().push(n),
        );

        assert!(!seen.into_inner().unwrap().is_empty());
    }

    #[test]
    fn unicode_names_hash_and_group_correctly() {
        let dir = tempfile::tempdir().unwrap();
        let a = write(dir.path(), "résumé café.txt", "même contenu".as_bytes());
        let b = write(dir.path(), "copie.txt", "même contenu".as_bytes());

        let report = run(&[a, b]);

        assert_eq!(report.total_groups, 1);
        assert_eq!(report.groups[0].copies, 2);
    }
}
