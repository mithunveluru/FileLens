-- Connection pragmas (foreign_keys is per-connection; journal_mode persists).
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

-- One row per scan run: the history.
CREATE TABLE IF NOT EXISTS scans (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    root_path   TEXT    NOT NULL,
    started_ms  INTEGER NOT NULL,
    finished_ms INTEGER,
    file_count  INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    cancelled   INTEGER NOT NULL DEFAULT 0
);

-- The current file inventory. `path` is the natural key, so a rescan upserts
-- instead of duplicating. `last_scan_id` records the scan that most recently
-- saw the file, which lets a complete scan prune files deleted from disk.
CREATE TABLE IF NOT EXISTS files (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    path         TEXT    NOT NULL UNIQUE,
    name         TEXT    NOT NULL,
    extension    TEXT,
    size_bytes   INTEGER NOT NULL,
    created_ms   INTEGER,
    modified_ms  INTEGER,
    mime_type    TEXT    NOT NULL,
    is_hidden    INTEGER NOT NULL,
    last_scan_id INTEGER NOT NULL REFERENCES scans(id)
);

-- Indexes for the queries Phase 3/4 will run (largest, oldest, by type).
CREATE INDEX IF NOT EXISTS idx_files_size      ON files(size_bytes);
CREATE INDEX IF NOT EXISTS idx_files_modified  ON files(modified_ms);
CREATE INDEX IF NOT EXISTS idx_files_extension ON files(extension);
CREATE INDEX IF NOT EXISTS idx_files_last_scan ON files(last_scan_id);

-- Key/value user settings (used from Phase 6).
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Paths the user has chosen to ignore (used from Phase 5/6).
CREATE TABLE IF NOT EXISTS ignored_paths (
    path       TEXT PRIMARY KEY,
    created_ms INTEGER NOT NULL
);

-- One row per executed Smart Organization session, for history and undo.
CREATE TABLE IF NOT EXISTS organization_sessions (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    created_ms INTEGER NOT NULL,
    root       TEXT    NOT NULL,
    move_count INTEGER NOT NULL,
    undone     INTEGER NOT NULL DEFAULT 0
);

-- The moves performed in a session; enough to reverse them (undo).
CREATE TABLE IF NOT EXISTS organization_moves (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  INTEGER NOT NULL REFERENCES organization_sessions(id) ON DELETE CASCADE,
    source      TEXT    NOT NULL,
    destination TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_org_moves_session ON organization_moves(session_id);

-- Persistent content-hash cache for duplicate detection. A hash is reused only
-- when size and modified time still match, so an edited file is rehashed
-- automatically. Keyed by path since one path holds one current hash.
CREATE TABLE IF NOT EXISTS hash_cache (
    path        TEXT    PRIMARY KEY,
    size_bytes  INTEGER NOT NULL,
    modified_ms INTEGER,
    algo        TEXT    NOT NULL,
    hash        TEXT    NOT NULL
);
