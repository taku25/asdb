/// grammars.toml のデシリアライズ
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

use crate::platform;

#[derive(Debug, Deserialize)]
pub struct GrammarsConfig {
    #[serde(default)]
    pub grammars: HashMap<String, GrammarEntry>,
    #[serde(default)]
    pub settings: Settings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GrammarEntry {
    /// Tree-sitter リポジトリの URL (GitHub)
    pub url: String,
    /// チェックアウトするタグ / コミット / ブランチ
    pub rev: String,
    /// true: GitHub Releases からプリビルド DLL を取得。失敗時はビルドフォールバック
    #[serde(default = "default_true")]
    pub prebuilt: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct Settings {
    /// Grammar DLL のインストール先 (省略時は OS 標準)
    pub install_dir: Option<String>,
}

fn default_true() -> bool {
    true
}

/// `~/.config/asdb/grammars.toml` を読み込む
pub fn load() -> Result<GrammarsConfig> {
    let path = platform::config_dir().join("grammars.toml");
    if !path.exists() {
        // ファイルがなければ空の設定を返す (grammar install 時にガイドを出す)
        return Ok(GrammarsConfig {
            grammars: HashMap::new(),
            settings: Settings::default(),
        });
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))
}

/// デフォルトの `grammars.toml` テンプレートを書き出す
pub fn write_default(path: &std::path::Path) -> Result<()> {
    let template = r#"# asdb Grammar Manager 設定
# `asdb-cli grammar install <lang>` でここに書いた Grammar DLL をインストールします

[grammars.cpp]
url      = "https://github.com/tree-sitter/tree-sitter-cpp"
rev      = "v0.23.4"
prebuilt = true

[grammars.c_sharp]
url      = "https://github.com/tree-sitter/tree-sitter-c-sharp"
rev      = "v0.21.3"
prebuilt = true

# [grammars.unreal_cpp]
# url      = "https://github.com/your-org/tree-sitter-unreal-cpp"
# rev      = "main"
# prebuilt = false

[settings]
# install_dir = "~/.local/share/asdb/grammars"
"#;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, template)?;
    Ok(())
}
