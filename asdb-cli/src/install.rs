/// Grammar DLL のインストール処理
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::config::GrammarEntry;
use crate::platform::{dll_ext, os_arch_tag};

// ─────────────────────────────────────────
// 公開エントリポイント
// ─────────────────────────────────────────

pub fn install_grammar(name: &str, entry: &GrammarEntry, install_dir: &Path) -> Result<()> {
    let grammar_dir = install_dir.join(name);
    std::fs::create_dir_all(&grammar_dir)?;

    if entry.prebuilt {
        match try_prebuilt(name, entry, &grammar_dir) {
            Ok(_) => {
                println!("✅ Installed '{name}' from prebuilt release");
                return Ok(());
            }
            Err(e) => {
                eprintln!("⚠️  Prebuilt download failed: {e}");
                eprintln!("   Falling back to build from source...");
            }
        }
    }

    build_from_source(name, entry, &grammar_dir)
}

// ─────────────────────────────────────────
// プリビルド取得
// ─────────────────────────────────────────

fn try_prebuilt(_name: &str, entry: &GrammarEntry, dest: &Path) -> Result<()> {
    let (owner, repo) = parse_github_url(&entry.url)?;
    let tag = &entry.rev;
    let suffix = os_arch_tag();
    let ext = dll_ext();

    // GitHub Releases API
    let api_url = format!(
        "https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}"
    );
    println!("  Fetching release info: {api_url}");

    let response: serde_json::Value = ureq::get(&api_url)
        .set("User-Agent", "asdb-cli/0.1")
        .call()
        .context("GitHub API request failed")?
        .into_json()
        .context("Failed to parse GitHub API response")?;

    let assets = response["assets"]
        .as_array()
        .context("No assets array in release response")?;

    // OS+アーキテクチャにマッチするアセットを探す
    let asset = assets
        .iter()
        .find(|a| {
            let asset_name = a["name"].as_str().unwrap_or("");
            asset_name.contains(&suffix) && asset_name.ends_with(ext)
        })
        .with_context(|| {
            format!(
                "No prebuilt asset found matching '{suffix}.{ext}'. \
                 Available: {}",
                assets
                    .iter()
                    .filter_map(|a| a["name"].as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;

    let download_url = asset["browser_download_url"]
        .as_str()
        .context("Missing browser_download_url")?;

    println!("  Downloading: {download_url}");
    let bytes = ureq::get(download_url)
        .set("User-Agent", "asdb-cli/0.1")
        .call()
        .context("Download failed")?
        .into_reader()
        .pipe_bytes()?;

    let dll_name = format!("{repo}.{ext}");
    let dll_path = dest.join(&dll_name);
    std::fs::write(&dll_path, &bytes)?;

    write_meta(dest, entry, "prebuilt", &hex_sha256(&bytes))?;
    Ok(())
}

// ─────────────────────────────────────────
// ソースからビルド
// ─────────────────────────────────────────

fn build_from_source(name: &str, entry: &GrammarEntry, dest: &Path) -> Result<()> {
    let tmpdir = tempfile::TempDir::new()?;
    let src_dir = tmpdir.path().join(name);

    println!("  Cloning {} @ {}...", entry.url, entry.rev);
    let status = std::process::Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--branch",
            &entry.rev,
            &entry.url,
            src_dir.to_str().unwrap(),
        ])
        .status()
        .context("git not found — please install Git")?;

    anyhow::ensure!(status.success(), "git clone failed (exit {})", status);

    let repo_name = repo_name_from_url(&entry.url);
    let ext = dll_ext();
    let dll_path = dest.join(format!("{repo_name}.{ext}"));

    // Windows では npm スクリプトが .cmd ファイルなので tree-sitter.cmd を使う
    #[cfg(target_os = "windows")]
    let tree_sitter_bin = "tree-sitter.cmd";
    #[cfg(not(target_os = "windows"))]
    let tree_sitter_bin = "tree-sitter";

    let output = std::process::Command::new(tree_sitter_bin)
        .args(["build", "--output", dll_path.to_str().unwrap()])
        .current_dir(&src_dir)
        .output()
        .context("tree-sitter CLI not found — please install: npm install -g tree-sitter-cli")?;

    if !output.status.success() {
        anyhow::bail!(
            "tree-sitter build failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // queries/ フォルダをコピー
    let queries_src = src_dir.join("queries");
    if queries_src.exists() {
        copy_dir_all(&queries_src, &dest.join("queries"))?;
        println!("  Copied queries/ folder");
    }

    let dll_bytes = std::fs::read(&dll_path)?;
    write_meta(dest, entry, "built_from_source", &hex_sha256(&dll_bytes))?;
    println!("✅ Built and installed '{name}'");
    Ok(())
}

// ─────────────────────────────────────────
// ヘルパー
// ─────────────────────────────────────────

fn parse_github_url(url: &str) -> Result<(String, String)> {
    let url = url.trim_end_matches('/').trim_end_matches(".git");
    let parts: Vec<&str> = url.split('/').collect();
    let n = parts.len();
    anyhow::ensure!(n >= 2, "Cannot parse GitHub URL: {url}");
    Ok((parts[n - 2].to_string(), parts[n - 1].to_string()))
}

fn repo_name_from_url(url: &str) -> &str {
    url.trim_end_matches('/')
        .trim_end_matches(".git")
        .split('/')
        .last()
        .unwrap_or("grammar")
}

fn hex_sha256(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex::encode(h.finalize())
}

fn write_meta(dir: &Path, entry: &GrammarEntry, source: &str, sha256: &str) -> Result<()> {
    let content = format!(
        "url          = \"{}\"\nrev          = \"{}\"\nsha256       = \"{}\"\nsource       = \"{}\"\n",
        entry.url, entry.rev, sha256, source
    );
    std::fs::write(dir.join("grammar.meta"), content)?;
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

// ureq のレスポンスボディを Vec<u8> に読み込むヘルパー trait
trait ReadBytes {
    fn pipe_bytes(self) -> Result<Vec<u8>>;
}

impl ReadBytes for Box<dyn std::io::Read + Send + Sync + 'static> {
    fn pipe_bytes(mut self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut self, &mut buf)?;
        Ok(buf)
    }
}
