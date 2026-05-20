use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};

// ──────────────────────────────────────────────
// JSON-RPC types
// ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct RpcResponse<T: Serialize> {
    jsonrpc: &'static str,
    id: Value,
    result: T,
}

#[derive(Debug, Serialize)]
struct RpcError {
    jsonrpc: &'static str,
    id: Value,
    error: ErrorObject,
}

#[derive(Debug, Serialize)]
struct ErrorObject {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize)]
struct RpcNotification<T: Serialize> {
    jsonrpc: &'static str,
    method: &'static str,
    params: T,
}

// ──────────────────────────────────────────────
// Message constructors
// ──────────────────────────────────────────────

pub fn make_response(id: Value, result: Value) -> String {
    serde_json::to_string(&RpcResponse { jsonrpc: "2.0", id, result }).unwrap()
}

pub fn make_error(id: Value, code: i32, message: impl Into<String>) -> String {
    serde_json::to_string(&RpcError {
        jsonrpc: "2.0",
        id,
        error: ErrorObject { code, message: message.into() },
    })
    .unwrap()
}

pub fn make_notification(method: &'static str, params: Value) -> String {
    serde_json::to_string(&RpcNotification { jsonrpc: "2.0", method, params }).unwrap()
}

// ──────────────────────────────────────────────
// I/O helpers
// ──────────────────────────────────────────────

pub fn read_lines(stdin: impl BufRead) -> impl Iterator<Item = String> {
    stdin.lines().filter_map(|l| l.ok()).filter(|l| !l.trim().is_empty())
}

pub fn write_line(line: &str) {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = out.write_all(line.as_bytes());
    let _ = out.write_all(b"\n");
    let _ = out.flush();
}
