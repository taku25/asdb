use anyhow::{Context, Result};
use libloading::Library;
use rayon::prelude::*;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Query, QueryCursor};
use walkdir::WalkDir;

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

// ──────────────────────────────────────────────
// Output types
// ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RawSymbol {
    pub name: String,
    pub kind: String,
    pub start_line: u32,
    pub start_byte: usize,
}

// ──────────────────────────────────────────────
// TextScanner
// ──────────────────────────────────────────────

#[allow(dead_code)]
pub struct TextScanner {
    _lib: Library,
    pub language: Language,
    pub query: Query,
    pub extensions: Vec<String>,
    pub name: String,
}

// SAFETY: Library/Language/Query are Send+Sync; QueryCursor is created per call.
unsafe impl Send for TextScanner {}
unsafe impl Sync for TextScanner {}

impl TextScanner {
    pub fn new(
        dll_path: &Path,
        query_content: &str,
        extensions: Vec<String>,
        name: String,
    ) -> Result<Self> {
        let lib = unsafe { Library::new(dll_path) }
            .with_context(|| format!("load DLL: {}", dll_path.display()))?;

        let symbol_name = derive_fn_name(dll_path)?;
        let language: Language = unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn() -> *const ()> =
                lib.get(symbol_name.as_bytes())
                    .with_context(|| format!("symbol '{}' not found", symbol_name))?;
            let raw = func();
            std::mem::transmute(raw)
        };

        let query = Query::new(&language, query_content)
            .with_context(|| format!("compile query for '{name}'"))?;

        Ok(Self { _lib: lib, language, query, extensions, name })
    }
}

/// Derive tree-sitter function name from DLL filename.
/// e.g. `tree-sitter-cpp.dll` → `tree_sitter_cpp`
fn derive_fn_name(path: &Path) -> Result<String> {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .context("DLL has no filename")?;

    // strip "lib" prefix (Linux/macOS)
    let stem = stem.strip_prefix("lib").unwrap_or(stem);
    // "tree-sitter-cpp" → "tree_sitter_cpp"
    let fn_name = stem.replace('-', "_");
    Ok(fn_name)
}

// ──────────────────────────────────────────────
// Per-file scan
// ──────────────────────────────────────────────

pub fn scan_file(scanner: &TextScanner, source: &[u8]) -> Vec<RawSymbol> {
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&scanner.language).is_err() {
        return vec![];
    }

    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return vec![],
    };

    let mut cursor = QueryCursor::new();
    let mut matches_iter = cursor.matches(&scanner.query, tree.root_node(), source);

    let mut symbols = Vec::new();
    while let Some(m) = matches_iter.next() {
        // Collect captures and #set! predicates
        let pattern = m.pattern_index;
        let mut sym_name: Option<String> = None;
        let mut sym_kind: Option<String> = None;
        let mut sym_line = 0u32;
        let mut sym_byte = 0usize;

        for cap in m.captures {
            let cap_name = &scanner.query.capture_names()[cap.index as usize];
            let text = cap.node.utf8_text(source).unwrap_or("").to_owned();

            if *cap_name == "symbol.name" {
                sym_line = cap.node.start_position().row as u32;
                sym_byte = cap.node.start_byte();
                sym_name = Some(text);
            }
        }

        // Extract symbol.type from #set! predicates
        for pred in scanner.query.general_predicates(pattern) {
            if pred.operator.as_ref() == "set!" {
                let args: Vec<&str> = pred
                    .args
                    .iter()
                    .filter_map(|a| {
                        if let tree_sitter::QueryPredicateArg::String(s) = a {
                            Some(s.as_ref())
                        } else {
                            None
                        }
                    })
                    .collect();
                if args.len() == 2 && args[0] == "symbol.type" {
                    sym_kind = Some(args[1].to_owned());
                }
            }
        }

        if let (Some(name), Some(kind)) = (sym_name, sym_kind) {
            symbols.push(RawSymbol { name, kind, start_line: sym_line, start_byte: sym_byte });
        }
    }

    symbols
}

// ──────────────────────────────────────────────
// Parallel directory scan
// ──────────────────────────────────────────────

pub struct ScanEntry {
    pub path: PathBuf,
    pub mtime_ms: i64,
    pub symbols: Vec<RawSymbol>,
}

pub fn scan_all(
    scanners: &[TextScanner],
    root: &Path,
    source_dirs: &[String],
    ignore_dirs: &[String],
) -> Vec<ScanEntry> {
    // Collect candidate files first
    let mut files: Vec<PathBuf> = Vec::new();
    let scan_roots: Vec<PathBuf> = if source_dirs.is_empty() {
        vec![root.to_path_buf()]
    } else {
        source_dirs.iter().map(|d| root.join(d)).collect()
    };

    for scan_root in &scan_roots {
        for entry in WalkDir::new(scan_root).into_iter().filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                return !ignore_dirs.iter().any(|ig| ig == name.as_ref());
            }
            true
        }) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.into_path();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_default();

            if scanners.iter().any(|s| s.extensions.iter().any(|e| e == &ext)) {
                files.push(path);
            }
        }
    }

    // Parallel scan
    files
        .par_iter()
        .filter_map(|path| {
            let mtime_ms = std::fs::metadata(path)
                .and_then(|m| m.modified())
                .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64)
                .unwrap_or(0);

            let source = match std::fs::read(path) {
                Ok(b) => b,
                Err(_) => return None,
            };

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_default();

            let scanner = scanners.iter().find(|s| s.extensions.iter().any(|e| e == &ext))?;
            let symbols = scan_file(scanner, &source);

            Some(ScanEntry { path: path.clone(), mtime_ms, symbols })
        })
        .collect()
}
