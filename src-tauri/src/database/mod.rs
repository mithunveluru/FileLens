//! SQLite persistence. A single connection guarded by a `Mutex` is managed in
//! Tauri state; for a desktop app's low write concurrency that is simpler and
//! plenty fast.
//
// ponytail: single Mutex<Connection>; switch to an r2d2 pool only if lock
// contention ever shows up in practice.

use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::filesystem::FileEntry;
use crate::scanning::ScanOutcome;

pub struct Database {
    conn: Mutex<Connection>,
}

/// One past organization session, for the history view. Serialized as camelCase.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationSessionRecord {
    pub id: i64,
    pub created_ms: i64,
    pub root: String,
    pub move_count: i64,
    pub undone: bool,
}

/// One past scan run, for the history view. Serialized as camelCase.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRecord {
    pub id: i64,
    pub root_path: String,
    pub started_ms: i64,
    pub finished_ms: Option<i64>,
    pub file_count: i64,
    pub error_count: i64,
    pub cancelled: bool,
}

impl Database {
    /// Opens (creating if needed) the database file and applies the schema.
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open(path)?)
    }

    fn from_connection(conn: Connection) -> rusqlite::Result<Self> {
        conn.execute_batch(include_str!("schema.sql"))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Persists a scan and its files in a single transaction, returning the new
    /// scan id. Files are upserted by path (no duplicates); a *complete* scan
    /// also prunes files it did not see, since those are gone from disk. A
    /// cancelled scan is partial, so it never prunes.
    pub fn persist_scan(
        &self,
        root: &str,
        started_ms: i64,
        finished_ms: i64,
        outcome: &ScanOutcome,
    ) -> rusqlite::Result<i64> {
        let mut conn = self.conn.lock().expect("db mutex poisoned");
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT INTO scans (root_path, started_ms, finished_ms, file_count, error_count, cancelled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                root,
                started_ms,
                finished_ms,
                outcome.files.len() as i64,
                outcome.error_count as i64,
                outcome.cancelled,
            ],
        )?;
        let scan_id = tx.last_insert_rowid();

        {
            let mut stmt = tx.prepare(
                "INSERT INTO files
                    (path, name, extension, size_bytes, created_ms, modified_ms, mime_type, is_hidden, last_scan_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(path) DO UPDATE SET
                    name=excluded.name,
                    extension=excluded.extension,
                    size_bytes=excluded.size_bytes,
                    created_ms=excluded.created_ms,
                    modified_ms=excluded.modified_ms,
                    mime_type=excluded.mime_type,
                    is_hidden=excluded.is_hidden,
                    last_scan_id=excluded.last_scan_id",
            )?;
            for file in &outcome.files {
                stmt.execute(params![
                    file.path,
                    file.name,
                    file.extension,
                    file.size_bytes as i64,
                    file.created_ms,
                    file.modified_ms,
                    file.mime_type,
                    file.is_hidden,
                    scan_id,
                ])?;
            }
        }

        if !outcome.cancelled {
            tx.execute(
                "DELETE FROM files WHERE last_scan_id != ?1",
                params![scan_id],
            )?;
        }

        tx.commit()?;
        Ok(scan_id)
    }

    /// Returns the full current file inventory, for the analysis engine.
    pub fn list_files(&self) -> rusqlite::Result<Vec<FileEntry>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT path, name, extension, size_bytes, created_ms, modified_ms, mime_type, is_hidden
             FROM files",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(FileEntry {
                path: row.get(0)?,
                name: row.get(1)?,
                extension: row.get(2)?,
                size_bytes: row.get::<_, i64>(3)? as u64,
                created_ms: row.get(4)?,
                modified_ms: row.get(5)?,
                mime_type: row.get(6)?,
                is_hidden: row.get::<_, i64>(7)? != 0,
            })
        })?;
        rows.collect()
    }

    /// Returns a single file by path, if it is in the inventory.
    pub fn get_file(&self, path: &str) -> rusqlite::Result<Option<FileEntry>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.query_row(
            "SELECT path, name, extension, size_bytes, created_ms, modified_ms, mime_type, is_hidden
             FROM files WHERE path = ?1",
            [path],
            |row| {
                Ok(FileEntry {
                    path: row.get(0)?,
                    name: row.get(1)?,
                    extension: row.get(2)?,
                    size_bytes: row.get::<_, i64>(3)? as u64,
                    created_ms: row.get(4)?,
                    modified_ms: row.get(5)?,
                    mime_type: row.get(6)?,
                    is_hidden: row.get::<_, i64>(7)? != 0,
                })
            },
        )
        .optional()
    }

    /// Removes a file from the inventory (e.g. after it was moved to the trash).
    pub fn remove_file(&self, path: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute("DELETE FROM files WHERE path = ?1", [path])?;
        Ok(())
    }

    /// Reads a raw setting value by key.
    pub fn get_setting(&self, key: &str) -> rusqlite::Result<Option<String>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
            row.get(0)
        })
        .optional()
    }

    /// Writes a raw setting value, replacing any existing one.
    pub fn set_setting(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    /// Adds a path to the ignore list (no-op if already present).
    pub fn add_ignored(&self, path: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "INSERT OR IGNORE INTO ignored_paths (path, created_ms) VALUES (?1, ?2)",
            params![path, now_ms()],
        )?;
        Ok(())
    }

    /// Removes a path from the ignore list.
    pub fn remove_ignored(&self, path: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute("DELETE FROM ignored_paths WHERE path = ?1", [path])?;
        Ok(())
    }

    /// Returns every ignored path.
    pub fn ignored_paths(&self) -> rusqlite::Result<Vec<String>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn.prepare("SELECT path FROM ignored_paths")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect()
    }

    /// Records an executed organization session and its moves in one
    /// transaction, returning the new session id.
    pub fn record_organization_session(
        &self,
        root: &str,
        moves: &[(String, String)],
    ) -> rusqlite::Result<i64> {
        let mut conn = self.conn.lock().expect("db mutex poisoned");
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO organization_sessions (created_ms, root, move_count) VALUES (?1, ?2, ?3)",
            params![now_ms(), root, moves.len() as i64],
        )?;
        let session_id = tx.last_insert_rowid();
        {
            let mut stmt = tx.prepare(
                "INSERT INTO organization_moves (session_id, source, destination) VALUES (?1, ?2, ?3)",
            )?;
            for (source, destination) in moves {
                stmt.execute(params![session_id, source, destination])?;
            }
        }
        tx.commit()?;
        Ok(session_id)
    }

    /// Returns recent organization sessions, newest first.
    pub fn organization_history(
        &self,
        limit: i64,
    ) -> rusqlite::Result<Vec<OrganizationSessionRecord>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, created_ms, root, move_count, undone
             FROM organization_sessions ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit], |row| {
            Ok(OrganizationSessionRecord {
                id: row.get(0)?,
                created_ms: row.get(1)?,
                root: row.get(2)?,
                move_count: row.get(3)?,
                undone: row.get::<_, i64>(4)? != 0,
            })
        })?;
        rows.collect()
    }

    /// Returns the (source, destination) moves recorded for a session.
    pub fn organization_session_moves(
        &self,
        session_id: i64,
    ) -> rusqlite::Result<Vec<(String, String)>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare("SELECT source, destination FROM organization_moves WHERE session_id = ?1")?;
        let rows = stmt.query_map([session_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect()
    }

    /// Marks a session as undone.
    pub fn mark_session_undone(&self, session_id: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "UPDATE organization_sessions SET undone = 1 WHERE id = ?1",
            [session_id],
        )?;
        Ok(())
    }

    /// Returns the most recent scans, newest first.
    pub fn scan_history(&self, limit: i64) -> rusqlite::Result<Vec<ScanRecord>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, root_path, started_ms, finished_ms, file_count, error_count, cancelled
             FROM scans ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit], |row| {
            Ok(ScanRecord {
                id: row.get(0)?,
                root_path: row.get(1)?,
                started_ms: row.get(2)?,
                finished_ms: row.get(3)?,
                file_count: row.get(4)?,
                error_count: row.get(5)?,
                cancelled: row.get::<_, i64>(6)? != 0,
            })
        })?;
        rows.collect()
    }
}

/// Current time as Unix epoch milliseconds.
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Database {
        fn in_memory() -> Self {
            Self::from_connection(Connection::open_in_memory().unwrap()).unwrap()
        }

        fn file_count(&self) -> i64 {
            self.conn
                .lock()
                .unwrap()
                .query_row("SELECT COUNT(*) FROM files", [], |r| r.get(0))
                .unwrap()
        }

        fn size_of(&self, path: &str) -> Option<i64> {
            self.conn
                .lock()
                .unwrap()
                .query_row(
                    "SELECT size_bytes FROM files WHERE path = ?1",
                    [path],
                    |r| r.get(0),
                )
                .ok()
        }
    }

    fn file(path: &str, size: u64) -> FileEntry {
        FileEntry {
            name: path.rsplit('/').next().unwrap().to_string(),
            extension: None,
            path: path.to_string(),
            size_bytes: size,
            created_ms: None,
            modified_ms: Some(1),
            mime_type: "application/octet-stream".to_string(),
            is_hidden: false,
        }
    }

    fn outcome(files: Vec<FileEntry>, cancelled: bool) -> ScanOutcome {
        ScanOutcome {
            files,
            error_count: 0,
            cancelled,
        }
    }

    #[test]
    fn persists_scan_and_lists_history() {
        let db = Database::in_memory();
        let id = db
            .persist_scan("/dl", 1, 2, &outcome(vec![file("/dl/a", 10)], false))
            .unwrap();

        let history = db.scan_history(10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, id);
        assert_eq!(history[0].file_count, 1);
        assert_eq!(db.file_count(), 1);
    }

    #[test]
    fn rescan_upserts_without_duplicating() {
        let db = Database::in_memory();
        db.persist_scan("/dl", 1, 2, &outcome(vec![file("/dl/a", 10)], false))
            .unwrap();
        db.persist_scan("/dl", 3, 4, &outcome(vec![file("/dl/a", 99)], false))
            .unwrap();

        assert_eq!(db.file_count(), 1);
        assert_eq!(db.size_of("/dl/a"), Some(99));
    }

    #[test]
    fn complete_rescan_prunes_missing_files() {
        let db = Database::in_memory();
        db.persist_scan(
            "/dl",
            1,
            2,
            &outcome(vec![file("/dl/a", 1), file("/dl/b", 2)], false),
        )
        .unwrap();
        // Second scan no longer sees /dl/b -> it is pruned.
        db.persist_scan("/dl", 3, 4, &outcome(vec![file("/dl/a", 1)], false))
            .unwrap();

        assert_eq!(db.file_count(), 1);
        assert_eq!(db.size_of("/dl/b"), None);
    }

    #[test]
    fn get_and_remove_file() {
        let db = Database::in_memory();
        db.persist_scan("/dl", 1, 2, &outcome(vec![file("/dl/a", 10)], false))
            .unwrap();

        assert_eq!(
            db.get_file("/dl/a").unwrap().map(|f| f.size_bytes),
            Some(10)
        );
        assert!(db.get_file("/dl/missing").unwrap().is_none());

        db.remove_file("/dl/a").unwrap();
        assert!(db.get_file("/dl/a").unwrap().is_none());
        assert_eq!(db.file_count(), 0);
    }

    #[test]
    fn records_organization_session_with_moves() {
        let db = Database::in_memory();
        let moves = vec![
            ("/dl/a.png".to_string(), "/dl/Images/a.png".to_string()),
            ("/dl/b.pdf".to_string(), "/dl/Documents/b.pdf".to_string()),
        ];
        let id = db.record_organization_session("/dl", &moves).unwrap();
        assert!(id > 0);

        let count: i64 = db
            .conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM organization_moves WHERE session_id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn settings_get_set_upsert() {
        let db = Database::in_memory();
        assert!(db.get_setting("app").unwrap().is_none());

        db.set_setting("app", "{\"a\":1}").unwrap();
        assert_eq!(db.get_setting("app").unwrap().as_deref(), Some("{\"a\":1}"));

        db.set_setting("app", "{\"a\":2}").unwrap();
        assert_eq!(db.get_setting("app").unwrap().as_deref(), Some("{\"a\":2}"));
    }

    #[test]
    fn ignored_paths_add_list_remove() {
        let db = Database::in_memory();
        db.add_ignored("/dl/a").unwrap();
        db.add_ignored("/dl/a").unwrap(); // idempotent
        db.add_ignored("/dl/b").unwrap();
        assert_eq!(db.ignored_paths().unwrap().len(), 2);

        db.remove_ignored("/dl/a").unwrap();
        assert_eq!(db.ignored_paths().unwrap(), vec!["/dl/b".to_string()]);
    }

    #[test]
    fn cancelled_rescan_does_not_prune() {
        let db = Database::in_memory();
        db.persist_scan(
            "/dl",
            1,
            2,
            &outcome(vec![file("/dl/a", 1), file("/dl/b", 2)], false),
        )
        .unwrap();
        // Cancelled partial scan only saw /dl/a, but must not delete /dl/b.
        db.persist_scan("/dl", 3, 4, &outcome(vec![file("/dl/a", 1)], true))
            .unwrap();

        assert_eq!(db.file_count(), 2);
    }
}
