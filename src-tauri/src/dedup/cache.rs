//! Persistent hash storage behind a trait so the pipeline depends on an
//! abstraction, not on SQLite. Reuse is safe because a hash is returned only
//! when the file's size and modified time are unchanged; any edit changes at
//! least one and forces a rehash.

use super::hash::ALGO;
use crate::database::Database;

// (path, size, modified_ms, hash)
pub type CacheEntry = (String, u64, Option<i64>, String);

pub trait HashCache: Sync {
    fn get(&self, path: &str, size: u64, modified_ms: Option<i64>) -> Option<String>;
    fn put(&self, path: &str, size: u64, modified_ms: Option<i64>, hash: &str);

    // One write per entry by default; implementors with a cheaper batch override.
    fn put_many(&self, entries: &[CacheEntry]) {
        for (path, size, modified_ms, hash) in entries {
            self.put(path, *size, *modified_ms, hash);
        }
    }
}

impl HashCache for Database {
    fn get(&self, path: &str, size: u64, modified_ms: Option<i64>) -> Option<String> {
        self.cached_hash(path, size, modified_ms, ALGO)
            .ok()
            .flatten()
    }

    fn put(&self, path: &str, size: u64, modified_ms: Option<i64>, hash: &str) {
        // A cache write failure only costs a future rehash; never fail the scan.
        if let Err(err) = self.store_hash(path, size, modified_ms, ALGO, hash) {
            log::warn!("failed to cache hash for {path}: {err}");
        }
    }

    // One transaction for the batch instead of a commit per file.
    fn put_many(&self, entries: &[CacheEntry]) {
        if let Err(err) = self.store_hashes(entries, ALGO) {
            log::warn!("failed to cache {} hashes: {err}", entries.len());
        }
    }
}

#[cfg(test)]
pub mod test_cache {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::HashCache;

    // (size, modified_ms, hash) — the fields that make a cached hash valid.
    type Entry = (u64, Option<i64>, String);

    #[derive(Default)]
    pub struct MemoryCache {
        entries: Mutex<HashMap<String, Entry>>,
        pub hits: Mutex<usize>,
    }

    impl HashCache for MemoryCache {
        fn get(&self, path: &str, size: u64, modified_ms: Option<i64>) -> Option<String> {
            let entries = self.entries.lock().unwrap();
            match entries.get(path) {
                Some((s, m, h)) if *s == size && *m == modified_ms => {
                    *self.hits.lock().unwrap() += 1;
                    Some(h.clone())
                }
                _ => None,
            }
        }

        fn put(&self, path: &str, size: u64, modified_ms: Option<i64>, hash: &str) {
            self.entries
                .lock()
                .unwrap()
                .insert(path.to_string(), (size, modified_ms, hash.to_string()));
        }
    }
}
