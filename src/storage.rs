use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ──────────────────────────────────────────────
// DB path resolution
// ──────────────────────────────────────────────

pub fn resolve_db_path(root: &Path) -> PathBuf {
    let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let hash = hex::encode(Sha256::digest(canonical.to_string_lossy().as_bytes()));
    let short = &hash[..16];

    let base = if cfg!(windows) {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
    } else {
        std::env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_default();
                PathBuf::from(home).join(".cache")
            })
    };

    base.join("asdb").join("projects").join(format!("{short}.db"))
}

// ──────────────────────────────────────────────
// Schema
// ──────────────────────────────────────────────

pub fn apply_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
        PRAGMA foreign_keys=ON;

        CREATE TABLE IF NOT EXISTS project_meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS strings (
            id   INTEGER PRIMARY KEY,
            text TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS directories (
            id        INTEGER PRIMARY KEY,
            parent_id INTEGER REFERENCES directories(id),
            name_id   INTEGER NOT NULL REFERENCES strings(id),
            UNIQUE(parent_id, name_id)
        );

        CREATE TABLE IF NOT EXISTS files (
            id              INTEGER PRIMARY KEY,
            directory_id    INTEGER NOT NULL REFERENCES directories(id),
            filename_id     INTEGER NOT NULL REFERENCES strings(id),
            mtime_ms        INTEGER NOT NULL DEFAULT 0,
            scan_generation INTEGER NOT NULL DEFAULT 0,
            UNIQUE(directory_id, filename_id)
        );

        CREATE TABLE IF NOT EXISTS symbols (
            id            INTEGER PRIMARY KEY,
            file_id       INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
            name_id       INTEGER NOT NULL REFERENCES strings(id),
            kind_id       INTEGER NOT NULL REFERENCES strings(id),
            start_line    INTEGER NOT NULL DEFAULT 0,
            start_byte    INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS symbols_file_idx ON symbols(file_id);
        CREATE INDEX IF NOT EXISTS symbols_name_idx ON symbols(name_id);

        CREATE VIRTUAL TABLE IF NOT EXISTS symbols_fts USING fts5(
            name,
            kind,
            filepath,
            content='',
            tokenize='unicode61'
        );
        ",
    )
    .context("apply_schema")
}

// ──────────────────────────────────────────────
// DbState — open connection + string/dir caches
// ──────────────────────────────────────────────

pub struct DbState {
    pub conn: Connection,
    strings: HashMap<String, i64>,
    dirs: HashMap<PathBuf, i64>,
}

impl DbState {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("create db dir")?;
        }
        let conn = Connection::open(path).context("open db")?;
        apply_schema(&conn)?;
        Ok(Self { conn, strings: HashMap::new(), dirs: HashMap::new() })
    }

    /// Get or insert a string, return its id.
    pub fn intern(&mut self, text: &str) -> Result<i64> {
        if let Some(&id) = self.strings.get(text) {
            return Ok(id);
        }
        self.conn.execute("INSERT OR IGNORE INTO strings(text) VALUES(?1)", params![text])?;
        let id: i64 =
            self.conn.query_row("SELECT id FROM strings WHERE text=?1", params![text], |r| {
                r.get(0)
            })?;
        self.strings.insert(text.to_owned(), id);
        Ok(id)
    }

    /// Get or insert directory node for `path`, return its id.
    pub fn intern_dir(&mut self, path: &Path) -> Result<i64> {
        if let Some(&id) = self.dirs.get(path) {
            return Ok(id);
        }

        let id = if let Some(parent) = path.parent() {
            let parent_id = self.intern_dir(parent)?;
            let seg = path.file_name().unwrap_or_default().to_string_lossy();
            let name_id = self.intern(&seg)?;
            self.conn.execute(
                "INSERT OR IGNORE INTO directories(parent_id, name_id) VALUES(?1, ?2)",
                params![parent_id, name_id],
            )?;
            self.conn.query_row(
                "SELECT id FROM directories WHERE parent_id=?1 AND name_id=?2",
                params![parent_id, name_id],
                |r| r.get(0),
            )?
        } else {
            // filesystem root
            let seg = path.to_string_lossy();
            let name_id = self.intern(&seg)?;
            self.conn.execute(
                "INSERT OR IGNORE INTO directories(parent_id, name_id) VALUES(NULL, ?1)",
                params![name_id],
            )?;
            self.conn.query_row(
                "SELECT id FROM directories WHERE parent_id IS NULL AND name_id=?1",
                params![name_id],
                |r| r.get(0),
            )?
        };

        self.dirs.insert(path.to_path_buf(), id);
        Ok(id)
    }

    /// Upsert a file row; return (file_id, needs_rescan).
    pub fn upsert_file(&mut self, path: &Path, mtime_ms: i64, scan_gen: i64) -> Result<(i64, bool)> {
        let dir = path.parent().unwrap_or(Path::new("."));
        let dir_id = self.intern_dir(dir)?;
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let name_id = self.intern(&name)?;

        // check existing
        let existing: Option<(i64, i64, i64)> = self
            .conn
            .query_row(
                "SELECT id, mtime_ms, scan_generation FROM files
                 WHERE directory_id=?1 AND filename_id=?2",
                params![dir_id, name_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .ok();

        if let Some((id, old_mtime, _)) = existing {
            if old_mtime == mtime_ms {
                return Ok((id, false));
            }
            self.conn.execute(
                "UPDATE files SET mtime_ms=?1, scan_generation=?2 WHERE id=?3",
                params![mtime_ms, scan_gen, id],
            )?;
            Ok((id, true))
        } else {
            self.conn.execute(
                "INSERT INTO files(directory_id, filename_id, mtime_ms, scan_generation)
                 VALUES(?1, ?2, ?3, ?4)",
                params![dir_id, name_id, mtime_ms, scan_gen],
            )?;
            Ok((self.conn.last_insert_rowid(), true))
        }
    }

    /// Replace all symbols for a file.
    pub fn replace_symbols(
        &mut self,
        file_id: i64,
        filepath: &str,
        symbols: &[crate::scan::RawSymbol],
    ) -> Result<()> {
        self.conn.execute("DELETE FROM symbols WHERE file_id=?1", params![file_id])?;

        for sym in symbols {
            let name_id = self.intern(&sym.name)?;
            let kind_id = self.intern(&sym.kind)?;
            self.conn.execute(
                "INSERT INTO symbols(file_id, name_id, kind_id, start_line, start_byte)
                 VALUES(?1, ?2, ?3, ?4, ?5)",
                params![file_id, name_id, kind_id, sym.start_line, sym.start_byte],
            )?;
            self.conn.execute(
                "INSERT INTO symbols_fts(name, kind, filepath) VALUES(?1, ?2, ?3)",
                params![sym.name, sym.kind, filepath],
            )?;
        }
        Ok(())
    }

    pub fn get_meta(&self, key: &str) -> Option<String> {
        self.conn
            .query_row("SELECT value FROM project_meta WHERE key=?1", params![key], |r| r.get(0))
            .ok()
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO project_meta(key, value) VALUES(?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
}
