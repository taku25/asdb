/// OS 別のデフォルトパス解決
use std::path::PathBuf;

pub fn grammar_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| "C:/Users/Public".into());
        PathBuf::from(base).join("asdb").join("grammars")
    }
    #[cfg(not(target_os = "windows"))]
    {
        home_dir().join(".local").join("share").join("asdb").join("grammars")
    }
}

pub fn config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("APPDATA").unwrap_or_else(|_| "C:/Users/Public".into());
        PathBuf::from(base).join("asdb")
    }
    #[cfg(not(target_os = "windows"))]
    {
        // XDG_CONFIG_HOME を優先
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg).join("asdb")
        } else {
            home_dir().join(".config").join("asdb")
        }
    }
}

/// ビルド済み DLL のファイル拡張子
pub fn dll_ext() -> &'static str {
    if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    }
}

/// GitHub Releases アセット名に含まれる OS + アーキテクチャのサフィックス
pub fn os_arch_tag() -> String {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };
    format!("{os}-{arch}")
}

/// `~/` を展開する簡易ユーティリティ
pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        home_dir().join(rest)
    } else {
        PathBuf::from(path)
    }
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}
