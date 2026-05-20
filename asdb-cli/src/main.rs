mod config;
mod install;
mod platform;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "asdb-cli",
    about = "asdb Grammar & Plugin Manager",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Grammar DLL の管理 (インストール / 一覧 / 削除)
    Grammar {
        #[command(subcommand)]
        action: GrammarAction,
    },
    /// grammars.toml のデフォルトテンプレートを生成
    Init,
}

#[derive(Subcommand)]
enum GrammarAction {
    /// Grammar DLL をインストール
    Install {
        /// 言語名 (grammars.toml のキー e.g. "cpp")
        lang: Option<String>,
        /// grammars.toml の全言語をインストール
        #[arg(long)]
        all: bool,
    },
    /// インストール済み Grammar 一覧を表示
    List,
    /// Grammar DLL を削除
    Remove { lang: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cmd_init(),
        Commands::Grammar { action } => match action {
            GrammarAction::Install { lang, all } => cmd_install(lang, all),
            GrammarAction::List => cmd_list(),
            GrammarAction::Remove { lang } => cmd_remove(&lang),
        },
    }
}

// ─────────────────────────────────────────
// コマンド実装
// ─────────────────────────────────────────

fn cmd_init() -> Result<()> {
    let path = platform::config_dir().join("grammars.toml");
    if path.exists() {
        println!("Already exists: {}", path.display());
        return Ok(());
    }
    config::write_default(&path)?;
    println!("✅ Created: {}", path.display());
    println!("   Edit it and run `asdb-cli grammar install --all`");
    Ok(())
}

fn cmd_install(lang: Option<String>, all: bool) -> Result<()> {
    let cfg = config::load()?;

    if cfg.grammars.is_empty() {
        let path = platform::config_dir().join("grammars.toml");
        anyhow::bail!(
            "No grammars configured.\nRun `asdb-cli init` to create a template at:\n  {}",
            path.display()
        );
    }

    let install_dir = cfg
        .settings
        .install_dir
        .as_deref()
        .map(platform::expand_tilde)
        .unwrap_or_else(platform::grammar_dir);

    std::fs::create_dir_all(&install_dir)?;

    if all {
        for (name, entry) in &cfg.grammars {
            println!("\n── Installing '{name}'...");
            install::install_grammar(name, entry, &install_dir)?;
        }
    } else if let Some(lang) = lang {
        let entry = cfg.grammars.get(&lang).ok_or_else(|| {
            anyhow::anyhow!(
                "Grammar '{lang}' not found in grammars.toml.\nAvailable: {}",
                cfg.grammars.keys().cloned().collect::<Vec<_>>().join(", ")
            )
        })?;
        println!("\n── Installing '{lang}'...");
        install::install_grammar(&lang, entry, &install_dir)?;
    } else {
        anyhow::bail!("Specify a language name or use --all\nExample: asdb-cli grammar install cpp");
    }

    Ok(())
}

fn cmd_list() -> Result<()> {
    let dir = platform::grammar_dir();
    if !dir.exists() {
        println!("No grammars installed. (dir: {})", dir.display());
        return Ok(());
    }

    println!("Installed grammars: ({})", dir.display());
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let meta_path = entry.path().join("grammar.meta");
        let rev = if meta_path.exists() {
            std::fs::read_to_string(&meta_path)
                .ok()
                .and_then(|s| {
                    s.lines()
                        .find(|l| l.starts_with("rev"))
                        .map(|l| l.split('=').nth(1).unwrap_or("").trim().trim_matches('"').to_string())
                })
                .unwrap_or_else(|| "?".into())
        } else {
            "?".into()
        };

        // DLL ファイルを探してサイズ表示
        let dll_info = std::fs::read_dir(entry.path())
            .ok()
            .and_then(|mut d| {
                d.find_map(|e| {
                    let e = e.ok()?;
                    let n = e.file_name().to_string_lossy().to_string();
                    if n.ends_with(platform::dll_ext()) {
                        let size = e.metadata().ok()?.len();
                        Some(format!("{n} ({:.1} KB)", size as f64 / 1024.0))
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| "(no DLL)".into());

        println!("  {name:<20} rev={rev:<12} {dll_info}");
    }

    Ok(())
}

fn cmd_remove(lang: &str) -> Result<()> {
    let dir = platform::grammar_dir().join(lang);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
        println!("✅ Removed '{lang}'");
    } else {
        println!("Grammar '{lang}' is not installed.");
    }
    Ok(())
}
