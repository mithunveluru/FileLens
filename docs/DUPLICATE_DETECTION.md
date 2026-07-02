# Duplicate Detection

File Lens reports two files as duplicates only when their contents are
cryptographically identical. Detection is a read-only pipeline: it never moves,
modifies, or deletes anything. Removing a copy is a separate, explicit user
action.

## Why a staged pipeline

Hashing every file would be slow and wasteful. Instead the candidate set is
narrowed by progressively more expensive stages, so the costly work runs only on
files that could actually be duplicates:

```
inventory
  → size groups        (free: metadata already scanned)
  → sampled fingerprint (three 64 KB reads)
  → full BLAKE3 hash    (streamed, cached)
  → verified groups
```

Each stage is a small, independently tested unit (`candidates_by_size`,
`fingerprint`, `hash_file`), composed by `detect_duplicates`. New match
strategies can be added as stages without touching the existing ones.

### Stage 1 — size grouping

Two byte-identical files must have the same size, so files with a unique size
can never be duplicates and are dropped immediately. This alone eliminates the
large majority of files for essentially no cost, because sizes come from the
scan that already happened. Zero-byte files are excluded: they are trivially
"identical" but free no space and would only add noise.

### Stage 2 — sampled fingerprint

For the size collisions that remain, a cheap signature is computed from the
first, middle, and last 64 KB of each file (`fingerprint`). Identical files
always share a fingerprint, so any difference here proves the files differ and
lets us skip full hashing. This is a filter, not a verdict — distinct files can
share a fingerprint, which the next stage resolves.

### Stage 3 — full hash verification

Only files that still collide are hashed in full with **BLAKE3**. Files sharing
a full hash are the verified duplicate groups. BLAKE3 is preferred for speed;
MD5 and SHA-1 are deliberately avoided because they are not collision-resistant.

## Why hashing is streamed

Files are read in fixed 128 KB chunks and fed to the hasher incrementally
(`hash_file`), so memory use is constant regardless of file size. A multi-gigabyte
file hashes in the same memory footprint as a small one.

## Why the cache is safe

Full hashes are cached in SQLite keyed by absolute path, and a cached hash is
reused only when the file's **size and modified time both match** what was
recorded. Any edit changes at least one of those, so a stale hash can never be
returned — invalidation is automatic and needs no separate bookkeeping. A cache
miss simply recomputes; a cache failure only costs a rehash and never fails the
scan.

## Concurrency

Hashing and fingerprinting run on a bounded worker pool (`parallel_map`): a fixed
number of threads drain a shared queue, rather than one thread per file. The
worker count is capped (`MAX_WORKERS`) because a hashing workload is disk-bound
and more threads mostly thrash the drive. Because each stage regroups its results
by key, result order does not matter and no ordering is imposed.

## Error handling

A file that cannot be read (permission denied, deleted mid-scan, locked) is
recorded in `DuplicateReport.errors` and dropped from grouping — it is simply not
claimed as a duplicate. One failure never aborts the run; the report is always
partial-safe.

## Result model

Detection returns structured domain objects, never a bare boolean:

- `DuplicateReport` — groups plus totals (`redundantFiles`, `reclaimableBytes`)
  and pipeline stats (`filesHashed`, `cacheHits`).
- `DuplicateGroup` — one set of identical files, its `VerificationStatus`, copy
  count, and reclaimable bytes (size × (copies − 1)).
- `DuplicateCandidate` — a single file's path, size, and modified time.

`VerificationStatus` currently only ever takes the value `Verified`; the weaker
variants exist so future strategies (for example perceptual image similarity)
can be added without changing the model. Only `Verified` is ever presented to the
user as an actual duplicate.
