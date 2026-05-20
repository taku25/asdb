use anyhow::Result;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

use crate::scan::{TextScanner, scan_all};
use crate::storage::{DbState, resolve_db_path};
use crate::transport::{RpcRequest, make_error, make_notification, make_response};

// ──────────────────────────────────────────────
// Default ignore list
// ──────────────────────────────────────────────

fn default_ignore_dirs() -> Vec<String> {
    vec![
        "Binaries", "Intermediate", "Saved", "DerivedDataCache",
        ".git", "node_modules", ".vs", ".idea", "build", "out",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

// ──────────────────────────────────────────────
// Server state
// ──────────────────────────────────────────────

pub struct Server {
    db: Option<DbState>,
    scanners: Vec<TextScanner>,
    root: Option<PathBuf>,
    ignore_dirs: Vec<String>,
    source_dirs: Vec<String>,
    scan_gen: i64,
}

impl Server {
    pub fn new() -> Self {
        Self {
            db: None,
            scanners: Vec::new(),
            root: None,
            ignore_dirs: default_ignore_dirs(),
            source_dirs: Vec::new(),
            scan_gen: 1,
        }
    }

    /// Dispatch one JSON line → returns 0‥N lines to write.
    pub fn handle_line(&mut self, line: &str) -> Vec<String> {
        let req: RpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                return vec![make_error(Value::Null, -32700, format!("parse error: {e}"))];
            }
        };

        let id = req.id.clone().unwrap_or(Value::Null);
        match req.method.as_str() {
            "initialize" => self.handle_initialize(req),
            "ping" => vec![make_response(id, json!({"pong": true}))],
            "file_changed" => self.handle_file_changed(req),
            "shutdown" => vec![make_response(id, json!(null))],
            other => {
                vec![make_error(id, -32601, format!("method not found: {other}"))]
            }
        }
    }

    // ──────────────────────────────────────────
    // initialize
    // ──────────────────────────────────────────

    fn handle_initialize(&mut self, req: RpcRequest) -> Vec<String> {
        let id = req.id.clone().unwrap_or(Value::Null);
        let params = match req.params {
            Some(p) => p,
            None => return vec![make_error(id, -32602, "params required")],
        };

        // Parse params
        let root_path = match params.get("root_path").and_then(|v| v.as_str()) {
            Some(p) => PathBuf::from(p),
            None => return vec![make_error(id, -32602, "root_path required")],
        };

        let source_dirs: Vec<String> = params
            .get("source_dirs")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|e| e.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let ignore_dirs: Vec<String> = {
            let extra: Vec<String> = params
                .get("ignore_dirs")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|e| e.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let mut base = default_ignore_dirs();
            base.extend(extra);
            base
        };

        // Build scanners
        let mut scanners: Vec<TextScanner> = Vec::new();

        if let Some(arr) = params.get("scanners").and_then(|v| v.as_array()) {
            for sc in arr {
                match build_scanner_from_value(sc) {
                    Ok(s) => scanners.push(s),
                    Err(e) => {
                        return vec![make_error(id, -32602, format!("scanner error: {e}"))];
                    }
                }
            }
        }

        // Fallback: built-in C++ scanner when no scanners provided and a grammar_dll given
        if scanners.is_empty() {
            if let Some(dll) = params.get("grammar_dll").and_then(|v| v.as_str()) {
                let query = include_str!("../queries/generic-cpp.scm");
                match TextScanner::new(
                    Path::new(dll),
                    query,
                    vec![".h".into(), ".hpp".into(), ".cpp".into(), ".cc".into(), ".cxx".into()],
                    "builtin-cpp".into(),
                ) {
                    Ok(s) => scanners.push(s),
                    Err(e) => {
                        return vec![make_error(id, -32602, format!("grammar_dll error: {e}"))];
                    }
                }
            }
        }

        if scanners.is_empty() {
            return vec![make_error(id, -32602, "at least one scanner (or grammar_dll) required")];
        }

        // Open DB
        let db_path = resolve_db_path(&root_path);
        let mut db = match DbState::open(&db_path) {
            Ok(d) => d,
            Err(e) => return vec![make_error(id, -32603, format!("db error: {e}"))],
        };

        let _ = db.set_meta("root_path", &root_path.to_string_lossy());

        self.root = Some(root_path.clone());
        self.source_dirs = source_dirs.clone();
        self.ignore_dirs = ignore_dirs.clone();
        self.scanners = scanners;

        // Acknowledge immediately
        let ack = make_response(
            id,
            json!({
                "status": "scanning",
                "db_path": db_path.to_string_lossy().as_ref()
            }),
        );

        // Run scan synchronously (Phase 2 — no async needed)
        let entries = scan_all(&self.scanners, &root_path, &source_dirs, &ignore_dirs);
        let total = entries.len();
        let mut indexed = 0usize;

        let _ = db.conn.execute("BEGIN", []);
        for entry in entries {
            let filepath = entry.path.to_string_lossy().to_string();
            match db.upsert_file(&entry.path, entry.mtime_ms, self.scan_gen) {
                Ok((file_id, needs_rescan)) => {
                    if needs_rescan {
                        let _ = db.replace_symbols(file_id, &filepath, &entry.symbols);
                        indexed += 1;
                    }
                }
                Err(_) => {}
            }
        }
        let _ = db.conn.execute("COMMIT", []);

        let _ = db.set_meta("scan_generation", &self.scan_gen.to_string());
        self.db = Some(db);

        let done = make_notification(
            "scan_complete",
            json!({
                "total_files": total,
                "indexed_files": indexed
            }),
        );

        vec![ack, done]
    }

    // ──────────────────────────────────────────
    // file_changed
    // ──────────────────────────────────────────

    fn handle_file_changed(&mut self, req: RpcRequest) -> Vec<String> {
        let id = req.id.clone().unwrap_or(Value::Null);
        let params = match &req.params {
            Some(p) => p.clone(),
            None => return vec![make_error(id, -32602, "params required")],
        };

        let file_path = match params.get("path").and_then(|v| v.as_str()) {
            Some(p) => PathBuf::from(p),
            None => return vec![make_error(id, -32602, "path required")],
        };

        let db: &mut DbState = match &mut self.db {
            Some(d) => d,
            None => return vec![make_error(id, -32603, "not initialized")],
        };

        let source = match std::fs::read(&file_path) {
            Ok(b) => b,
            Err(e) => return vec![make_error(id, -32603, format!("read file: {e}"))],
        };

        let mtime_ms = std::fs::metadata(&file_path)
            .and_then(|m| m.modified())
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as i64
            })
            .unwrap_or(0);

        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default();

        let scanner = match self.scanners.iter().find(|s| s.extensions.iter().any(|e| e == &ext)) {
            Some(s) => s,
            None => return vec![make_response(id, json!({"status": "skipped"}))],
        };

        let symbols = crate::scan::scan_file(scanner, &source);
        let filepath = file_path.to_string_lossy().to_string();
        self.scan_gen += 1;

        match db.upsert_file(&file_path, mtime_ms, self.scan_gen) {
            Ok((file_id, _)) => {
                let _ = db.replace_symbols(file_id, &filepath, &symbols);
                vec![make_response(id, json!({"status": "ok", "symbols": symbols.len()}))]
            }
            Err(e) => vec![make_error(id, -32603, format!("db error: {e}"))],
        }
    }
}

// ──────────────────────────────────────────────
// Helper: build TextScanner from JSON value
// ──────────────────────────────────────────────

fn build_scanner_from_value(v: &Value) -> Result<TextScanner> {
    let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("unnamed").to_owned();
    let dll = v
        .get("grammar_dll")
        .and_then(|x| x.as_str())
        .ok_or_else(|| anyhow::anyhow!("grammar_dll required"))?;

    let query_content: String = if let Some(qf) = v.get("query_file").and_then(|x| x.as_str()) {
        std::fs::read_to_string(qf)
            .map_err(|e| anyhow::anyhow!("read query_file '{qf}': {e}"))?
    } else if let Some(q) = v.get("query").and_then(|x| x.as_str()) {
        q.to_owned()
    } else {
        include_str!("../queries/generic-cpp.scm").to_owned()
    };

    let extensions: Vec<String> = v
        .get("extensions")
        .and_then(|x| x.as_array())
        .map(|a| a.iter().filter_map(|e| e.as_str().map(String::from)).collect())
        .unwrap_or_else(|| vec![".h".into(), ".hpp".into(), ".cpp".into(), ".cc".into()]);

    TextScanner::new(Path::new(dll), &query_content, extensions, name)
        .map_err(|e| anyhow::anyhow!("{e}"))
}
