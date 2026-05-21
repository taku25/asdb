use rusqlite::params;
use serde_json::{Value, json};

use crate::storage::DbState;

// ──────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────

/// Sanitize user input for use in FTS5 MATCH queries.
/// Strips characters that have special meaning in FTS5.
fn fts_escape(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

// ──────────────────────────────────────────────
// completion
// ──────────────────────────────────────────────

pub fn handle_completion(db: &DbState, params: &Value) -> Value {
    let mode = params.get("mode").and_then(|v| v.as_str()).unwrap_or("prefix");

    match mode {
        "prefix" => completion_prefix(db, params),
        "member_of" => completion_member_of(db, params),
        _ => json!({"items": []}),
    }
}

fn completion_prefix(db: &DbState, params: &Value) -> Value {
    let prefix = match params.get("prefix").and_then(|v| v.as_str()) {
        Some(p) if !p.is_empty() => p,
        _ => return json!({"items": []}),
    };
    let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let escaped = fts_escape(prefix);
    if escaped.is_empty() {
        return json!({"items": []});
    }
    let match_expr = format!("name:{escaped}*");

    let mut stmt = match db.conn.prepare(
        "SELECT name, kind, filepath, start_line FROM symbols_fts
         WHERE symbols_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2",
    ) {
        Ok(s) => s,
        Err(_) => return json!({"items": []}),
    };

    let items: Vec<Value> = match stmt.query_map(params![match_expr, limit], |r| {
            Ok(json!({
                "label":      r.get::<_, String>(0)?,
                "kind":       r.get::<_, String>(1)?,
                "file_path":  r.get::<_, String>(2)?,
                "start_line": r.get::<_, i64>(3)?
            }))
        }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(_) => vec![],
    };

    json!({"items": items})
}

fn completion_member_of(db: &DbState, params: &Value) -> Value {
    let class_name = match params.get("class_name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return json!({"items": []}),
    };
    let access_filter: Vec<&str> = params
        .get("access_filter")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|e| e.as_str()).collect())
        .unwrap_or_else(|| vec!["public", "protected", "private"]);

    // Look up the symbol id for this class
    let symbol_id: Option<i64> = db
        .conn
        .query_row(
            "SELECT s.id FROM symbols s
             JOIN strings n ON s.name_id = n.id
             WHERE n.text = ?1
             LIMIT 1",
            params![class_name],
            |r| r.get(0),
        )
        .ok();

    let symbol_id = match symbol_id {
        Some(id) => id,
        None => return json!({"items": []}),
    };

    let mut stmt = match db.conn.prepare(
        "SELECT n.text, k.text, a.text, rt.text, m.is_static, m.start_line
         FROM members m
         JOIN strings n  ON m.name_id = n.id
         JOIN strings k  ON m.kind_id = k.id
         JOIN strings a  ON m.access_id = a.id
         LEFT JOIN strings rt ON m.return_type_id = rt.id
         WHERE m.symbol_id = ?1",
    ) {
        Ok(s) => s,
        Err(_) => return json!({"items": []}),
    };

    let items: Vec<Value> = match stmt.query_map(params![symbol_id], |r| {
            let access: String = r.get(2)?;
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                access,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, i64>(4)? != 0,
                r.get::<_, i64>(5)?,
            ))
        }) {
        Ok(rows) => rows
            .filter_map(|r| r.ok())
            .filter(|(_, _, access, _, _, _)| access_filter.contains(&access.as_str()))
            .map(|(name, kind, access, return_type, is_static, line)| {
                json!({
                    "label":       name,
                    "kind":        kind,
                    "access":      access,
                    "return_type": return_type,
                    "is_static":   is_static,
                    "start_line":  line
                })
            })
            .collect(),
        Err(_) => vec![],
    };

    json!({"items": items})
}

// ──────────────────────────────────────────────
// search_symbols
// ──────────────────────────────────────────────

pub fn handle_search_symbols(db: &DbState, params: &Value) -> Value {
    // Accept both "query" and "pattern" for flexibility.
    let pattern = params
        .get("query")
        .or_else(|| params.get("pattern"))
        .and_then(|v| v.as_str());
    let pattern = match pattern {
        Some(p) if !p.is_empty() => p,
        _ => return json!({"symbols": []}),
    };
    let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let kind_filter: Vec<&str> = params
        .get("symbol_types")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|e| e.as_str()).collect())
        .unwrap_or_default();

    let escaped = fts_escape(pattern);
    if escaped.is_empty() {
        return json!({"symbols": []});
    }
    let match_expr = format!("name:{escaped}*");

    let mut stmt = match db.conn.prepare(
        "SELECT name, kind, filepath, start_line FROM symbols_fts
         WHERE symbols_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2",
    ) {
        Ok(s) => s,
        Err(_) => return json!({"symbols": []}),
    };

    let symbols: Vec<Value> = match stmt.query_map(params![match_expr, limit], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
            ))
        }) {
        Ok(rows) => rows
            .filter_map(|r| r.ok())
            .filter(|(_, kind, _, _)| {
                kind_filter.is_empty() || kind_filter.contains(&kind.as_str())
            })
            .map(|(name, kind, filepath, start_line)| {
                json!({
                    "name":        name,
                    "symbol_type": kind,
                    "file_path":   filepath,
                    "line":        start_line
                })
            })
            .collect(),
        Err(_) => vec![],
    };

    json!({"symbols": symbols})
}

// ──────────────────────────────────────────────
// goto_definition
// ──────────────────────────────────────────────

pub fn handle_goto_definition(db: &DbState, params: &Value) -> Value {
    let symbol_name = match params.get("symbol_name").and_then(|v| v.as_str()) {
        Some(n) if !n.is_empty() => n,
        _ => return json!(null),
    };

    let escaped = fts_escape(symbol_name);
    if escaped.is_empty() {
        return json!(null);
    }
    // Exact match: FTS5 phrase query.
    let match_expr = format!("\"{escaped}\"");

    let result: Option<(String, i64)> = db
        .conn
        .query_row(
            "SELECT filepath, start_line FROM symbols_fts
             WHERE symbols_fts MATCH 'name:' || ?1
             ORDER BY rank
             LIMIT 1",
            params![match_expr],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();

    match result {
        Some((file_path, line)) => json!({
            "file_path":  file_path,
            "line":       line,
            "character":  0
        }),
        None => json!(null),
    }
}

// ──────────────────────────────────────────────
// get_members
// ──────────────────────────────────────────────

pub fn handle_get_members(db: &DbState, params: &Value) -> Value {
    let class_name = match params.get("class_name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return json!({"members": []}),
    };
    let access_filter: Vec<&str> = params
        .get("access_filter")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|e| e.as_str()).collect())
        .unwrap_or_default();
    let kind_filter: Vec<&str> = params
        .get("member_types")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|e| e.as_str()).collect())
        .unwrap_or_default();

    let symbol_id: Option<i64> = db
        .conn
        .query_row(
            "SELECT s.id FROM symbols s
             JOIN strings n ON s.name_id = n.id
             WHERE n.text = ?1 LIMIT 1",
            params![class_name],
            |r| r.get(0),
        )
        .ok();

    let symbol_id = match symbol_id {
        Some(id) => id,
        None => return json!({"members": []}),
    };

    let mut stmt = match db.conn.prepare(
        "SELECT n.text, k.text, a.text, rt.text, m.is_static, m.start_line
         FROM members m
         JOIN strings n  ON m.name_id = n.id
         JOIN strings k  ON m.kind_id = k.id
         JOIN strings a  ON m.access_id = a.id
         LEFT JOIN strings rt ON m.return_type_id = rt.id
         WHERE m.symbol_id = ?1",
    ) {
        Ok(s) => s,
        Err(_) => return json!({"members": []}),
    };

    let members: Vec<Value> = match stmt.query_map(params![symbol_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, i64>(4)? != 0,
                r.get::<_, i64>(5)?,
            ))
        }) {
        Ok(rows) => rows
            .filter_map(|r| r.ok())
            .filter(|(_, kind, access, _, _, _)| {
                (access_filter.is_empty() || access_filter.contains(&access.as_str()))
                    && (kind_filter.is_empty() || kind_filter.contains(&kind.as_str()))
            })
            .map(|(name, kind, access, return_type, is_static, line)| {
                json!({
                    "name":        name,
                    "member_type": kind,
                    "access":      access,
                    "return_type": return_type,
                    "is_static":   is_static,
                    "start_line":  line
                })
            })
            .collect(),
        Err(_) => vec![],
    };

    json!({"members": members})
}

// ──────────────────────────────────────────────
// get_inheritance
// ──────────────────────────────────────────────

pub fn handle_get_inheritance(db: &DbState, params: &Value) -> Value {
    let class_name = match params.get("class_name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return json!({"parents": [], "children": []}),
    };
    let direction = params.get("direction").and_then(|v| v.as_str()).unwrap_or("parents");

    let symbol_id: Option<i64> = db
        .conn
        .query_row(
            "SELECT s.id FROM symbols s
             JOIN strings n ON s.name_id = n.id
             WHERE n.text = ?1 LIMIT 1",
            params![class_name],
            |r| r.get(0),
        )
        .ok();

    let symbol_id = match symbol_id {
        Some(id) => id,
        None => return json!({"parents": [], "children": []}),
    };

    let mut parents: Vec<String> = Vec::new();
    let mut children: Vec<String> = Vec::new();

    if direction == "parents" || direction == "both" {
        parents = bfs_inheritance(db, symbol_id, true);
    }
    if direction == "children" || direction == "both" {
        children = bfs_inheritance(db, symbol_id, false);
    }

    json!({"parents": parents, "children": children})
}

/// BFS walk of the inheritance table.
/// `upward=true` → walk parent chain; `upward=false` → walk child chain.
fn bfs_inheritance(db: &DbState, start: i64, upward: bool) -> Vec<String> {
    let mut result = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    let mut visited = std::collections::HashSet::new();
    queue.push_back(start);
    visited.insert(start);

    while let Some(current) = queue.pop_front() {
        let query = if upward {
            "SELECT i.parent_id, n.text FROM inheritance i
             JOIN symbols s ON i.parent_id = s.id
             JOIN strings n ON s.name_id = n.id
             WHERE i.child_id = ?1"
        } else {
            "SELECT i.child_id, n.text FROM inheritance i
             JOIN symbols s ON i.child_id = s.id
             JOIN strings n ON s.name_id = n.id
             WHERE i.parent_id = ?1"
        };

        if let Ok(mut stmt) = db.conn.prepare(query) {
            let rows: Vec<(i64, String)> = stmt
                .query_map(params![current], |r| Ok((r.get(0)?, r.get(1)?)))
                .map(|iter| iter.filter_map(|r| r.ok()).collect())
                .unwrap_or_default();
            for (next_id, name) in rows {
                if visited.insert(next_id) {
                    result.push(name);
                    queue.push_back(next_id);
                }
            }
        }
    }

    result
}
