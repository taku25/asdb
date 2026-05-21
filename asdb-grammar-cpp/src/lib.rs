//! C-ABI wrapper that re-exports the tree-sitter C++ grammar as a DLL for asdb.
//!
//! ## Why "asdb_grammar_cpp" and not "tree_sitter_cpp"?
//!
//! The tree-sitter-cpp Rust crate statically links `parser.c` which defines a C
//! function named `tree_sitter_cpp`.  If this cdylib exported a function with the
//! *same* name, the linker would drop the C definition and resolve every internal
//! reference (including the one captured in `tree_sitter_cpp::LANGUAGE`) to our
//! Rust wrapper → infinite recursion at runtime.
//!
//! By using a distinct export name, the C `tree_sitter_cpp` symbol stays intact,
//! `grammar::LANGUAGE.into()` calls the real C parser, and we forward the result.
//!
//! asdb-core's `derive_fn_name("asdb_grammar_cpp.dll")` → `"asdb_grammar_cpp"`,
//! so the symbol lookup in `scan.rs` works automatically.

use tree_sitter_cpp as grammar;

/// Returns the raw `TSLanguage *` for the C++ grammar.
///
/// # Safety
/// The returned pointer is a static reference embedded in this DLL; it is valid
/// for the lifetime of the process.
#[unsafe(no_mangle)]
pub extern "C" fn asdb_grammar_cpp() -> *const () {
    let lang: tree_sitter::Language = grammar::LANGUAGE.into();
    // SAFETY: `Language` is repr(transparent) over `NonNull<TSLanguage>`,
    // which has the same size and alignment as `*const ()`.
    // This mirrors the reverse transmute in asdb-core's scan.rs.
    unsafe { std::mem::transmute(lang) }
}
