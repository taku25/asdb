//! asdb — Phase 1 Spike
//!
//! Grammar DLL を libloading で動的ロードし、tree-sitter で C++ をパース、
//! .scm クエリでクラス名を抽出できることを確認する実験コード。
//!
//! Usage:
//!   asdb <grammar_dll_path> <symbol_name>
//!
//! Example (Windows):
//!   asdb tree-sitter-cpp.dll tree_sitter_cpp
//! Example (Linux/macOS):
//!   asdb libtree-sitter-cpp.so tree_sitter_cpp

use anyhow::{Context, Result};
use libloading::{Library, Symbol};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Parser, Query, QueryCursor};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: asdb <grammar_dll_path> <symbol_name>");
        eprintln!("  e.g.: asdb tree-sitter-cpp.dll tree_sitter_cpp");
        std::process::exit(1);
    }
    let dll_path = &args[1];
    let symbol = format!("{}\0", args[2]);

    println!("Loading grammar DLL: {dll_path}");

    // ① Grammar DLL を libloading で動的ロード
    let language = unsafe { load_language(dll_path, symbol.as_bytes())? };
    println!("✅ Grammar DLL loaded");

    // ② Tree-sitter パーサー生成
    let mut parser = Parser::new();
    parser.set_language(&language).context("Failed to set language")?;

    // ③ C++ ソース文字列をパース
    let source = "class Foo { public: int bar(); };";
    let tree = parser.parse(source, None).context("Failed to parse")?;
    println!("Parse tree:\n{}\n", tree.root_node().to_sexp());

    // ④ .scm クエリでクラス名を抽出
    let query_src = "(class_specifier name: (type_identifier) @class.name)";
    let query = Query::new(&language, query_src).context("Failed to build query")?;

    let mut cursor = QueryCursor::new();
    let name_idx = query
        .capture_index_for_name("class.name")
        .context("capture 'class.name' not found")?;

    let mut found = false;
    let mut matches_iter = cursor.matches(&query, tree.root_node(), source.as_bytes());
    while let Some(m) = matches_iter.next() {
        for cap in m.captures.iter().filter(|c| c.index == name_idx) {
            let text = cap
                .node
                .utf8_text(source.as_bytes())
                .context("utf8 decode failed")?;
            println!("✅ Found class: {text}");
            found = true;
        }
    }

    if !found {
        anyhow::bail!("No class names found — query or DLL may be incorrect");
    }

    println!("\n✅ Phase 1 spike complete! Architecture is valid.");
    Ok(())
}

/// Grammar DLL を動的ロードして tree-sitter Language を返す。
///
/// # Safety
/// - `dll_path` は有効な tree-sitter grammar DLL であること。
/// - `symbol` は `tree_sitter_<lang>\0` 形式の null 終端バイト列。
/// - DLL は `mem::forget` でリークさせプロセス終了まで保持する。
///   (本番実装では ActiveProject 構造体が Library を所有して管理する)
unsafe fn load_language(dll_path: &str, symbol: &[u8]) -> Result<Language> {
    let lib = unsafe { Library::new(dll_path).context("Failed to load DLL")? };

    let func: Symbol<unsafe extern "C" fn() -> *const ()> =
        unsafe { lib.get(symbol).context("Symbol not found in DLL")? };

    let raw = unsafe { func() };
    if raw.is_null() {
        anyhow::bail!("DLL returned null language pointer");
    }

    // tree_sitter::Language は *const TSLanguage の単一フィールド newtype。
    // TSLanguage と () はポインタサイズが同じなので transmute は安全。
    // TODO: tree-sitter が Language::from_raw を pub に昇格したら差し替える。
    let language: Language = unsafe { std::mem::transmute(raw) };

    // DLL をリークしてプロセス終了まで保持
    std::mem::forget(lib);
    Ok(language)
}
