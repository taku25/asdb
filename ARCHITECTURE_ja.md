# asdb (AST Sitter Database) 設計書 Blueprint

> 作成日: 2026-05-20  
> ステータス: 設計フェーズ (実装前)

---

## 目次

- [1. プロジェクト概要](#1-プロジェクト概要)
- [2. コアアイデンティティ](#2-コアアイデンティティ)
  - [設計根拠: per-project プロセスを採用する理由](#設計根拠-グローバルデーモンを採用しないただし-per-project-プロセスは採用する)
- [3. ディレクトリ構造](#3-ディレクトリ構造)
- [4. プロトコル設計](#4-プロトコル設計)
  - [起動モード](#起動モード)
  - [クライアント接続プロトコル](#クライアント接続プロトコル-lua--typescript-側)
  - [Neovim からの使用例](#neovim-からの使用例-ソケット接続)
  - [VSCode からの使用例](#vscode-からの使用例-ソケット接続)
  - [サーバー側: ライフサイクル管理 (Rust)](#サーバー側-ライフサイクル管理-rust)
  - [ワイヤーフォーマット比較](#ワイヤーフォーマット比較)
  - [Rust トランスポート層設計](#rust-トランスポート層設計)
  - [ストリーミング共通プロトコル](#ストリーミング共通プロトコル)
- [5. JSON-RPC API 仕様 (全メソッド)](#5-json-rpc-api-仕様-全メソッド)
  - [ライフサイクル](#ライフサイクル)
  - [スキャン・更新](#スキャン更新)
  - [補完・シンボル検索](#補完シンボル検索)
  - [サーバー → クライアント 通知](#サーバー--クライアント-通知)
  - [エラーコード定義](#エラーコード定義)
- [6. DB スキーマ設計](#6-db-スキーマ設計)
  - [UNL からの変更点](#unl-からの変更点)
  - [テーブル定義](#テーブル定義)
- [7. `.scm` キャプチャ名 → `ScanOutput` フィールド マッピング規約](#7-scm-キャプチャ名--scanoutput-フィールド-マッピング規約)
- [8. パース パイプライン設計](#8-パース-パイプライン設計)
  - [3種類の DLL を動的ロード](#3種類の-dll-を動的ロード)
  - [プラグインパッケージのイメージ](#プラグインパッケージのイメージ)
  - [統一パースフロー](#統一パースフロー-全パスが-scanoutput--db-ライター)
  - [Scanner DLL の C-ABI インターフェース](#scanner-dll-の-c-abi-インターフェース)
  - [Rust 側の統合設計](#rust-側の統合設計)
  - [`initialize` パラメータ (確定版)](#initialize-パラメータ-確定版)
  - [Rust モジュール構造](#rust-モジュール構造)
  - [拡張ロードマップ](#拡張ロードマップ)
- [9. `.scm` クエリ設計](#9-scm-クエリ設計)
  - [責務の境界線](#責務の境界線)
  - [キャプチャ名規約](#キャプチャ名規約)
  - [`generic-cpp.scm` (汎用 C++ ベースライン)](#generic-cppscm-汎用-c-ベースライン)
  - [`unreal-cpp.scm` (UE 拡張)](#unreal-cppscm-ue-拡張generic-cpp-に追加)
  - [既知の限界](#既知の限界)
- [10. 差分スキャン戦略](#10-差分スキャン戦略)
  - [パス正規化規則](#パス正規化規則-issue-10)
  - [スキャンの種類](#スキャンの種類)
  - [フルスキャン vs 差分スキャンの判定](#フルスキャン-vs-差分スキャンの判定)
  - [起動時 差分スキャン フロー](#起動時-差分スキャン-フロー)
  - [`file_changed` 時フロー](#file_changed-時フロー-エディタ保存イベント)
  - [DB の並列アクセス戦略](#db-の並列アクセス戦略)
  - [DB ライタースレッドの文字列インターニング処理](#db-ライタースレッドの文字列インターニング処理-)
  - [`resolve_type` の内部実装方針](#resolve_type-の内部実装方針-)
  - [`file_opened` + 未保存バッファの扱い](#file_opened--未保存バッファの扱い-)
  - [`transient_trees` サイズ上限と GC 戦略](#transient_trees-サイズ上限と-gc-戦略)
  - [`ignore_dirs` のデフォルト値と結合ルール](#ignore_dirs-のデフォルト値と結合ルール)
  - [エラーハンドリング方針](#エラーハンドリング方針)
  - [ログファイル設計](#ログファイル設計)
- [11. エディタ側 Lua 薄皮 設計 (Neovim)](#11-エディタ側-lua-薄皮-設計-neovim)
  - [設計原則](#設計原則)
  - [責務の全量と行数見積もり](#責務の全量と行数見積もり)
  - [プロジェクトルート特定 (`detect.lua`)](#プロジェクトルート特定-detectlua)
  - [lang_config.json ロードと DLL パス解決 (`config.lua`)](#lang_configjson-ロードと-dll-パス解決-configlua)
  - [プロセス管理 (`process.lua`)](#プロセス管理-processlua)
  - [イベントフック (`events.lua`)](#イベントフック-eventslua)
  - [blink-cmp ソースアダプタ (`source.lua`)](#blink-cmp-ソースアダプタ-sourcelua)
  - [プラグインのディレクトリ構造](#プラグインのディレクトリ構造)
- [12. Grammar Manager (`asdb-cli`)](#12-grammar-manager-asdb-cli)
  - [設計思想](#設計思想)
  - [`grammars.toml` — ユーザー設定ファイル](#grammarstoml--ユーザー設定ファイル)
  - [`asdb-cli` コマンド一覧](#asdb-cli-コマンド一覧)
  - [インストールフロー](#インストールフロー)
  - [インストール後のディレクトリ構造](#インストール後のディレクトリ構造)
  - [`lang_config.json` との連携](#lang_configjson-との連携)
  - [Rust 実装方針](#rust-実装方針)
- [13. 言語拡張仕様 (Plugin System Specification)](#13-言語拡張仕様-plugin-system-specification)
  - [拡張モデル概要](#拡張モデル概要)
  - [`plugins.toml` — ユーザープラグインレジストリ](#pluginstoml--ユーザープラグインレジストリ)
  - [`asdb-cli plugin` コマンド一覧](#asdb-cli-plugin-コマンド一覧)
  - [インストールフロー (type 別)](#インストールフロー-type-別)
  - [`scanners` 配列 — `lang_config.json` の中核](#scanners-配列--lang_configjson-の中核)
  - [`ScanOutput` エンベロープ — DLL の統一返却形式](#scanoutput-エンベロープ--dll-の統一返却形式)
  - [新言語追加の手順 (開発者向け)](#新言語追加の手順-開発者向け)
  - [Scanner DLL 開発者向け要件](#scanner-dll-開発者向け要件)
- [14. 実装ロードマップ](#14-実装ロードマップ)
  - [全フェーズ概観](#全フェーズ概観)
  - [Phase 1: Rust スパイク](#phase-1-rust-スパイク-抽象パース基盤確認)
  - [Phase 2: コア基盤](#phase-2-コア基盤)
  - [Phase 3: クエリエンジン](#phase-3-クエリエンジン)
  - [Phase 4: BinaryScanner](#phase-4-binaryscanner-uasset-scanner-dll)
  - [Phase 5: Lua 薄皮 (asdb.nvim)](#phase-5-lua-薄皮-asdbnvim)
  - [Phase 6: Grammar Manager (`asdb-cli`)](#phase-6-grammar-manager-asdb-cli)
  - [Phase 7: バイナリ配布 + CI/CD](#phase-7-バイナリ配布--cicd)
- [15. 未決事項 (TODO)](#15-未決事項-todo)
- [16. VCS 連携設計](#16-vcs-連携設計)
  - [設計原則](#設計原則-1)
  - [VcsAdapter DLL — C-ABI インターフェース](#vcsadapter-dll--c-abi-インターフェース)
  - [Core 側の VcsAdapter ローダー設計 (Rust)](#core-側の-vcsadapter-ローダー設計-rust)
  - [`lang_config.json` の `vcs_adapter` フィールド](#lang_configjson-の-vcs_adapter-フィールド)
  - [起動時スキャン判定フロー](#起動時スキャン判定フロー)
  - [VCS 別の動作サマリー](#vcs-別の動作サマリー)
  - [reconcile scan とは](#reconcile-scan-とは)
  - [`project_meta` 追加キー](#project_meta-追加キー-1)
  - [プラグイン (Lua) 側の実装量](#プラグイン-lua-側の実装量)
- [17. プロジェクトファイル発見設計 (asdb-discover)](#17-プロジェクトファイル発見設計-asdb-discover)
  - [責務の分離](#責務の分離-1)
  - [asdb-discover コマンド仕様](#asdb-discover-コマンド仕様)
  - [対応プロジェクトタイプ](#対応プロジェクトタイプ)
  - [asdb.nvim との連携フロー](#asdbnvim-との連携フロー)
  - [フォールバック戦略](#フォールバック戦略)
  - [initialize パラメータへの追加](#initialize-パラメータへの追加)

---

## 1. プロジェクト概要

`asdb` は UNL.nvim の次期バージョンとして設計された、**言語不問・エディタ不問の超軽量 AST データベースインフラ**。

従来の UNL が抱えていた以下の問題を解消する：

| 課題 | 解決策 |
|------|--------|
| ユーザーが Rust ビルドを強いられる | プリコンパイル済みバイナリ配布 + C-ABI DLL動的ロード |
| キャッシュフォルダのパスハッシュ管理 | OS 標準キャッシュディレクトリに配置、SHA-256 ハッシュでパス解決 |
| TCPポート管理が必要 / マルチエディタ共有が困難 | Unix ドメインソケット + PIDファイル (1プロジェクト = 1プロセス、複数エディタが共有) |
| UE 固有ロジックがコアに混在 | Tree-sitter `.scm` クエリで完全データ駆動化 |
| メモリのみでは冷起動コスト大 | ローカルSQLite (差分スキャン対応) |

---

## 2. コアアイデンティティ

- **完全データ駆動型**: コア (Rust) は特定言語・環境のロジックを1行も持たない
- **C-ABI 動的ロード**: Tree-sitter DLL は C-ABI なので Rust ABI 問題が発生しない
- **Unix ドメインソケット + PIDファイル (1プロジェクト = 1プロセス)**: TCPポート管理ゼロ。nvim / VSCode / 複数 nvim ウィンドウが同一コアプロセスを共有。スキャンは1プロセスが担当するため調整不要
- **JSON-RPC / MessagePack-RPC 両対応**: VSCode (JSON) と Neovim (msgpack) を同一コアで対応
- **プロジェクトルートゼロ汚染**: DB は OS 標準キャッシュディレクトリ (`~/.cache/asdb/`) に配置。`.gitignore` 設定が不要
- **スキャン中でも補完が止まらない**: text_pool/binary_pool はバックグラウンドスレッドで動作。メインループは常にリクエストを受け付ける

---

### 設計根拠: グローバルデーモンを採用しない、ただし per-project プロセスは採用する

```
// ❌ グローバルデーモン (採用しない)
//    1プロセスが全プロジェクトを管理 → ProjectX のビジー状態が ProjectY をキューで詰まらせる
エディタA (ProjectX スキャン中) ─┐
                                 ├─► asdb-daemon ← クロスプロジェクト ブロッキング
エディタB (ProjectY 補完待ち) ───┘

// ✅ per-project プロセス (採用)
//    1プロジェクト = 1プロセス。複数エディタは同一プロセスにソケット接続
エディタA (ProjectX) ──┐
エディタC (ProjectX) ──┼─► asdb #1 ← ProjectX 専用 (ソケット接続)
エディタD (ProjectX) ──┘

エディタB (ProjectY) ──► asdb #2 ← ProjectY 専用 (別プロセス = 完全独立)
```

- **クロスプロジェクト分離**: ProjectX のフルスキャン中でも ProjectY の補完は別プロセスで即返す
- **同一プロジェクト共有**: 複数エディタが同じ DB・同じキャッシュを共有。スキャンは1回で済む
- **プロセスライフサイクル**: 最後のクライアントが切断してから 60 秒後に自動シャットダウン

---

## 3. ディレクトリ構造

**プロジェクトルートには何も作らない。**  
DB/PID は OS 標準キャッシュディレクトリ、**ソケットは OS ランタイムディレクトリ**に分離配置する。

```
# Linux / macOS
~/.cache/asdb/projects/
├── <sha256_64hex>.db    ← SQLite DB  (パス長制限なし)
└── <sha256_64hex>.pid   ← サーバー情報 (JSON)

$XDG_RUNTIME_DIR/          ← /run/user/1000 など短いパス
└── asdb-<sha256_16hex>.sock   ← Unix ドメインソケット ← sun_path 制限 (108B) 対策

# Windows
%LOCALAPPDATA%\asdb\projects\
├── <sha256_64hex>.db
└── <sha256_64hex>.pid

\\.\pipe\asdb-<sha256_16hex>   ← Windows Named Pipe (primary)
  ※ AF_UNIX on Windows は experimental 扱い (将来 opt-in)
```

> **ソケットパス長の設計根拠:**  
> Unix `sun_path` 制限は **108バイト**。`~/.cache/asdb/.../<64文字>.sock` は典型的なユーザー名でも  
> 制限を超えやすい。`$XDG_RUNTIME_DIR` は `/run/user/1000` (15文字) と短く、16文字の短縮ハッシュで  
> 合計 **38文字以内**に収まる。長いハッシュは DB/PID ファイル名にのみ使用する。

> **PIDファイルのフォーマット:**
> ```json
> {
>   "pid": 12345,
>   "socket": "/run/user/1000/asdb-<16hex>.sock",
>   "server_nonce": "f3a2b1c4-...",
>   "version": "0.1.0",
>   "started_at_ms": 1716193778000
> }
> ```
> `server_nonce` は UUIDv4。PID 再利用を検出するためにサーバーが起動時に生成する (後述)。

> **Git を汚さない**: プロジェクトルートへの書き込みは一切しないため `.gitignore` 設定が不要。  
> **読み取り専用ディレクトリへの対応**: OS ライブラリやサードパーティソースも安全に開ける。  
> **パス正規化**: `root_path` は必ず `canonicalize()` + Windows lowercase 正規化 を適用してからハッシュ (後述)。  
> **1プロジェクト = 1プロセス**: PIDファイルで生存確認 + nonce 検証。接続できれば既存プロセスを再利用。  
> **書き込み権限がない場合**: 後述のフォールバックパスで自動的に代替ディレクトリを試みる。

---

## 4. プロトコル設計

### 起動モード

```bash
# Linux / macOS 通常モード
asdb --socket /run/user/1000/asdb-<16hex>.sock \
          --pid-file ~/.cache/asdb/projects/<64hex>.pid

# Windows 通常モード (Named Pipe)
asdb --pipe \\.\pipe\asdb-<16hex> \
          --pid-file %LOCALAPPDATA%\asdb\projects\<64hex>.pid

# デバッグ専用 Stdio モード ⚠️ クライアント共有・broadcast なし。単一接続のみ。
asdb --stdio --protocol json      # JSON-RPC
asdb --stdio --protocol msgpack   # MessagePack-RPC
```

> **`--stdio` の制約**: client_count 管理なし、グレースシャットダウンなし、scan_complete broadcast なし。  
> 接続が閉じると即プロセス終了。Phase 1 デバッグ・CI・VSCode 最小構成でのみ使用する。

### クライアント接続プロトコル (Lua / TypeScript 側)

```
クライアント起動フロー:

  1. root_path を正規化 → SHA-256 → full_hash(64hex) + short_hash(16hex) を計算
  2. <cache>/<full_hash>.pid を読む

     ┌─ PIDファイルなし
     │    → [新規起動]
     │
     ├─ PIDファイルあり + PID が死んでいる (kill -0 / OpenProcess 失敗)
     │    → .pid を削除 (ソケット/パイプは削除しない: 別 server が使っている可能性)
     │    → [新規起動]
     │
     └─ PIDファイルあり + PID が生きている
          → pid_data.socket (または pipe) に接続試行
          → 接続成功 → initialize 送信 → レスポンスの nonce と pid_data.server_nonce を照合
            ├─ 一致 → 正常接続、client_id 取得
            └─ 不一致 → PID 再利用を検出。[新規起動] へ (上書き)
          → 接続失敗 (stale socket) → .pid を削除 → [新規起動]

  [新規起動]:
    asdb --socket (または --pipe) + --pid-file をデタッチ起動
    → .pid が出現するまでポーリング (100ms 間隔, 最大 5 秒)

    ⚠️ 起動競合の対策 (複数クライアントが同時に "PIDなし" を検出した場合):
    → asdb 側では bind()/CreateNamedPipe() が最初の1つにしか成功しない
    → 失敗した asdb は起動を中断し exit(0)
    → クライアントは .pid のポーリング中に "まだ出ないな" → 既存 .pid を読んで [接続] に進む
    → .pid の書き込みは atomic: "<hash>.pid.tmp" に書き rename で上書き (partial read 防止)

    → 5 秒でタイムアウト → エラー通知のみ (--stdio fallback は行わない。二重スキャン防止)
```

**Lua 実装スケルトン:**

```lua
-- asdb/client.lua
local M = {}

function M.connect_or_spawn(root_path)
  local norm = M.normalize_path(root_path)        -- canonicalize + Windows lowercase
  local full_hash  = M.sha256_hex(norm)            -- 64文字: DB/PID ファイル名用
  local short_hash = full_hash:sub(1, 16)          -- 16文字: ソケット/パイプ名用
  local pid_path   = M.cache_dir() .. "/asdb/projects/" .. full_hash .. ".pid"
  local sock_path  = M.runtime_socket_path(short_hash)  -- XDG_RUNTIME_DIR or Named Pipe

  local pid_data = M.read_pid_file(pid_path)
  if pid_data and M.is_pid_alive(pid_data.pid) then
    local ok, client = pcall(M.connect_socket, pid_data.socket)
    if ok then
      -- nonce 検証: initialize レスポンスと照合
      local res = client:initialize({ root_path = norm, config = { ... } })
      if res.server_nonce == pid_data.server_nonce then
        return client  -- ✅ 正常接続
      end
      -- PID 再利用を検出 → 落として新規起動へ
      client:close()
    end
    vim.fn.delete(pid_path)  -- stale PID を削除 (ソケットは削除しない)
  end

  -- デタッチ起動
  M.spawn_detached({ "asdb", "--socket", sock_path, "--pid-file", pid_path })
  local client = M.wait_for_pid_and_connect(pid_path, 5000)
  if not client then
    error("[asdb] サーバーの起動がタイムアウトしました (5 秒)")
  end
  return client
end

function M.normalize_path(path)
  local p = vim.fn.resolve(vim.fn.fnamemodify(path, ":p"))  -- canonicalize
  if vim.fn.has("win32") == 1 then p = p:lower() end        -- Windows: lowercase
  return p
end
```

### Neovim からの使用例 (ソケット接続)

```lua
-- ソケット接続後は vim.rpcrequest / vim.rpcnotify の channel として使用
local chan = vim.fn.sockconnect("pipe", sock_path, { rpc = true })
vim.rpcrequest(chan, "initialize", { root_path = root_path, config = { ... } })
vim.rpcnotify(chan, "file_changed", { file_path = "/path/to/file.cpp" })
```

### VSCode からの使用例 (ソケット接続)

```typescript
import * as net from "net";
const socket = net.createConnection(sockPath);
// 改行区切り JSON-RPC を socket に流す (--stdio 時と同一フォーマット)
```

### サーバー側: ライフサイクル管理 (Rust)

```rust
// main.rs
fn main() {
    match parse_args() {
        Mode::Socket { socket_path, pid_path } => run_socket_mode(socket_path, pid_path),
        Mode::Pipe   { pipe_name,   pid_path } => run_pipe_mode(pipe_name, pid_path),   // Windows
        Mode::Stdio  { protocol }              => run_stdio_mode(protocol),              // デバッグ専用
    }
}

pub struct AsdbServer {
    client_count:       AtomicI32,
    shutdown_generation: AtomicU64,   // グレースタイマーの競合防止用 (issue #7)
    clients:            Mutex<HashMap<String, ClientSender>>, // broadcast 用 (issue #8)
    // ...
}

// クライアントごとの通知チャネル (bounded queue) - issue #8
struct ClientSender {
    tx: tokio::sync::mpsc::Sender<RpcMessage>,   // capacity: 16
}

fn run_socket_mode(socket_path: PathBuf, pid_path: PathBuf) {
    let nonce    = Uuid::new_v4().to_string();   // PID reuse 検出用 nonce - issue #4
    let listener = UnixListener::bind(&socket_path)?;
    // atomic write: .pid.tmp → rename → .pid  (partial read 防止)
    write_pid_file_atomic(&pid_path, &socket_path, &nonce)?;
    let server = Arc::new(AsdbServer::new(nonce));

    for stream in listener.incoming() {
        let server = server.clone();
        tokio::spawn(async move {
            // 新接続でシャットダウン世代をインクリメント (グレースタイマーをキャンセル) - issue #7
            server.shutdown_generation.fetch_add(1, Ordering::SeqCst);
            server.client_count.fetch_add(1, Ordering::SeqCst);

            handle_client(server.clone(), stream).await;  // ブロック

            let remaining = server.client_count.fetch_sub(1, Ordering::SeqCst) - 1;
            if remaining == 0 {
                let gen = server.shutdown_generation.load(Ordering::SeqCst);
                let srv = server.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(60)).await;
                    // 世代が変わっていなければ (新接続がなかった) シャットダウン
                    if srv.shutdown_generation.load(Ordering::SeqCst) == gen {
                        srv.initiate_shutdown();
                    }
                });
            }
        });
    }
}

// broadcast (scan_complete 等) - issue #8
// per-client bounded queue (cap=16)。queue full 時は最古を drop してから enqueue (best-effort)
impl AsdbServer {
    pub fn broadcast(&self, msg: RpcMessage) {
        let clients = self.clients.lock();
        for sender in clients.values() {
            // try_send: full なら drop (notification は best-effort)
            let _ = sender.tx.try_send(msg.clone());
        }
    }
}
```

> **shutdown(force:false) の接続クローズ:** サーバーはクライアント登録解除後にその TCP/Unix コネクションを  
> 即座に close する。close 後の追加リクエストは OS が ECONNRESET/EPIPE で拒否するため  
> double-decrement は発生しない。 — issue #6

> **グレースシャットダウン競合防止:** 新接続があるたびに `shutdown_generation` をインクリメントする。  
> 60 秒タイマー発火時に世代が変わっていれば何もしない → 競合なし。 — issue #7

> **broadcast backpressure:** 通知は per-client bounded channel (cap=16) を経由する。  
> queue full 時は最古エントリを drop して新しい通知を優先する (スキャン進捗は最新が重要)。  
> request/response パスと notification パスは別チャネルなので通知遅延が補完応答を詰まらせない。 — issue #8

// シャットダウン時のクリーンアップ
fn on_shutdown(socket_path: &Path, pid_path: &Path) {
    let _ = std::fs::remove_file(socket_path);
    let _ = std::fs::remove_file(pid_path);
}

### ワイヤーフォーマット比較

| | JSON-RPC 2.0 | MessagePack-RPC |
|---|---|---|
| フレーミング | 改行区切り (`\n`) | 自己区切り (msgpack 配列) |
| リクエスト | `{"jsonrpc":"2.0","id":1,"method":"...","params":{}}` | `[0, msgid, "method", [params]]` |
| レスポンス | `{"jsonrpc":"2.0","id":1,"result":{}}` | `[1, msgid, null, result]` |
| エラー | `{"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"..."}}` | `[1, msgid, "error message", null]` |
| 通知(S→C) | `{"jsonrpc":"2.0","method":"...","params":{}}` | `[2, "method", [params]]` |

> **ソケット接続時のプロトコル選択:** クライアントは接続直後の最初のバイトでプロトコルを自動判別する  
> (`{` で始まれば JSON, 0x90-0x9f / 0xdc-0xdd で始まれば MessagePack)。  
> `--stdio` モードは `--protocol` フラグで明示指定。

### Rust トランスポート層設計

```rust
// 共通中間表現
// ⚠️ params を serde_json::Value にしない:
//   msgpack は JSON にない型 (binary blob, u64 > i64::MAX 等) を持つ。
//   プロトコル境界ではプロトコル固有の型を使い、ハンドラ層で共通型に変換する。
pub enum RpcId {
    Number(u64),    // JSON-RPC 2.0: id は string | number | null
    String(String),
}

pub enum RpcMessage {
    Request      { id: RpcId, method: String, params: ParamValue },
    Response     { id: RpcId, result: Result<ParamValue, RpcError> },
    Notification { method: String, params: ParamValue },
}

// プロトコル固有のパラメータ型
pub enum ParamValue {
    Json(serde_json::Value),     // JSON-RPC トランスポート用
    Msgpack(rmpv::Value),        // msgpack-RPC トランスポート用
}

impl ParamValue {
    // ハンドラ層で共通の serde_json::Value へ変換 (受信 params のデコード用)
    pub fn into_json(self) -> serde_json::Value { /* ... */ }
}

// ⚠️ レスポンスを返す時は ParamValue 経由 (= DOM 変換) を経由しない。
//    Rust の応答構造体 (CompletionResponse 等) から直接シリアライズすることで
//    DOM アロケーションのオーバーヘッドを排除する。
//
//    JSON トランスポート:   serde_json::to_writer(&mut stdout, &response)
//    Msgpack トランスポート: rmp_serde::encode::write(&mut stdout, &response)
//
// Transport::send は RpcMessage::Response の result に SerializedValue を持たせるか、
// または write_response<T: Serialize>(&mut self, id, result: &T) で直接書く。

// トランスポート抽象 (コアロジックはこのトレイトだけを知る)
pub trait Transport: Send {
    fn recv(&mut self) -> anyhow::Result<RpcMessage>;
    fn send(&mut self, msg: RpcMessage) -> anyhow::Result<()>;
    // レスポンス送信は T: Serialize を直接書き出す (DOM 中間変換なし)
    fn write_response<T: serde::Serialize>(&mut self, id: RpcId, result: &T) -> anyhow::Result<()>;
}

pub struct JsonTransport   { reader: BufReader<Box<dyn Read + Send>>, writer: BufWriter<Box<dyn Write + Send>> }
pub struct MsgpackTransport{ reader: BufReader<Box<dyn Read + Send>>, writer: BufWriter<Box<dyn Write + Send>> }
// SocketTransport: UnixStream を Read+Write として扱う。プロトコルは自動判別
pub struct SocketTransport { stream: BufReader<UnixStream>, protocol: Protocol }

fn main() {
    match parse_args() {
        Mode::Socket { socket_path, pid_path } => {
            // ソケットモード: 各接続を独立タスクで処理
            for stream in UnixListener::bind(&socket_path)?.incoming() {
                tokio::spawn(handle_socket_client(server.clone(), stream?));
            }
        }
        Mode::Stdio { protocol } => {
            let transport: Box<dyn Transport> = match protocol {
                Protocol::Msgpack => Box::new(MsgpackTransport::new(stdin(), stdout())),
                Protocol::Json    => Box::new(JsonTransport::new(stdin(), stdout())),
            };
            AsdbServer::new(transport).run();
        }
    }
}
```

### ストリーミング共通プロトコル

`find_usages` 等のストリーミング対応メソッドは以下のパターンに従う。

```
Client                          Server
  │                               │
  │── Request (stream:true) ─────►│ ハンドラがスレッドをスポーン
  │                               │
  │◄─ Notification (chunk 0) ─────│ chunk_size 件ごとに送信
  │◄─ Notification (chunk 1) ─────│
  │       ...                     │
  │◄─ Response (done/cancelled) ──│ 完了またはキャンセル後に元 id へ返す
  │                               │
  │── Notification (cancel) ─────►│ (オプション: ユーザーが中断)
```

**Rust ハンドラのスケルトン:**

```rust
fn handle_find_usages_stream(
    transport: Arc<Mutex<dyn Transport>>,
    id: RpcId,
    params: FindUsagesParams,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) {
    let chunk_size = params.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
    let rows = db.query_usages(&params);   // Iterator<Item = UsageRow>

    let mut chunk: Vec<UsageItem> = Vec::with_capacity(chunk_size);
    let mut total = 0usize;
    let mut chunk_index = 0u32;

    for row in rows {
        if *cancel_rx.borrow() {
            transport.lock().write_response(id, &FindUsagesFinal { total, status: "cancelled" });
            return;
        }
        chunk.push(row.into());
        total += 1;
        if chunk.len() >= chunk_size {
            transport.lock().send_notification("find_usages/chunk", &FindUsagesChunk {
                req_id: &id, chunk_index, items: std::mem::take(&mut chunk),
            });
            chunk_index += 1;
        }
    }
    if !chunk.is_empty() {
        transport.lock().send_notification("find_usages/chunk", &FindUsagesChunk {
            req_id: &id, chunk_index, items: chunk,
        });
    }
    transport.lock().write_response(id, &FindUsagesFinal { total, status: "done" });
}
```

**`lang_config.json` でのデフォルト値:**
```json
"streaming": {
  "chunk_size": 10
}
```

---

## 5. JSON-RPC API 仕様 (全メソッド)

> MessagePack-RPC は同一メソッド名・同一パラメータ構造。シリアライズ形式のみ異なる。

### 【ライフサイクル】

#### `initialize` — プロジェクト登録・スキャン開始

```json
// Request params  (lang_config.json の core セクションをそのまま転送する)
{
  "root_path": "/path/to/MyProject",
  "config": {
    "ignore_dirs": ["Binaries", "Intermediate", "Saved", ".git"],

    "scanners": [
      {
        "name":        "unreal_cpp",
        "extensions":  [".h", ".cpp", ".inl"],
        "source_dirs": ["Source", "Plugins"],
        "grammar_dll": "/path/to/tree-sitter-unreal-cpp.dll",
        "query_file":  "/path/to/unreal-cpp.scm"
      },
      {
        "name":        "unreal_assets",
        "extensions":  [".uasset", ".umap"],
        "source_dirs": ["Content"],
        "scanner_dll": "/path/to/asdb-scanner-ue-assets.dll"
      }
    ],

    "sub_root_markers": [
      { "pattern": "*.Build.cs", "name_from": "stem" },
      { "pattern": "*.uplugin",  "name_from": "stem" }
    ],

    "lang_hints": {
      "type_strip_keywords": ["virtual","static","inline","FORCEINLINE","const"],
      "api_macro_pattern":   "[A-Z0-9_]+_API",
      "header_extensions":   [".h",".hpp",".inl"],
      "source_extensions":   [".cpp",".cc",".c"]
    }
  }
}

// Response (即時返却 — スキャンはバックグラウンド)
{
  "status":        "scanning",      // "ready" | "scanning"
  "client_id":     "client-001",    // このクライアントの識別子 (サーバーが採番)
  "client_count":  2,               // 現在接続中のクライアント数 (デバッグ用)
  "server_nonce":  "f3a2b1c4-...",  // PID 再利用検出用。PIDファイルの nonce と照合すること
  "db_path":       "~/.cache/asdb/projects/<sha256>.db",
  "db_mode":       "persistent",    // "persistent" | "temp" | "memory"
  "file_count":    1842,            // 前回スキャン時のファイル数 (初回は 0)
  "changed_files": 12               // 差分スキャン対象数
}
// クライアントは server_nonce と pid_file.server_nonce を照合し、不一致なら PID 再利用として切断・再起動
```

> **べき等性**: `root_path` が同じリクエストが再送された場合、設定更新 + 差分スキャン再起動として扱う。

#### `shutdown` — クライアント切断 / サーバー停止

```json
// params: { "force": false }   (デフォルト: false)
// Response: { "status": "unregistered" }   force:false — このクライアントを登録解除
//            { "status": "shutdown" }       force:true  — 他クライアントを問わず即時シャットダウン

// force:false の挙動:
//   client_count-- → 0 になったら 60 秒の猶予後に自動シャットダウン
//   猶予中に新クライアントが接続したらタイマーキャンセル (サーバー継続稼働)
```

#### `ping` — 死活確認

```json
// params: { "pid": 12345 }   (省略可)
// Response: { "pong": true, "uptime_ms": 3421 }
```

---

### 【スキャン・更新】

#### `file_changed` — 保存イベント時の差分更新

```json
// params
{ "file_path": "/path/to/MyClass.cpp" }

// Response
{ "status": "updated", "symbols_updated": 3 }
// または
{ "status": "queued" }  // 別の更新が進行中の場合
```

#### `file_opened` — オンデマンド優先パース

バッファオープン時に呼び出し、そのファイルを優先スキャンする。

```json
// params
{
  "file_path": "/path/to/MyClass.h",
  "content":   "..."   // オプション: 未保存バッファ内容
}

// Response
{ "status": "parsed", "symbols_found": 5 }
```

#### `rescan` — 強制フルリスキャン

```json
// params: {}
// Response: { "status": "scanning" }
// 完了時に scan_complete 通知
```

---

### 【補完・シンボル検索】

#### `completion` — 補完候補取得 (2モード)

**モード1: プレフィックス検索** (ユーザーがクラス名・関数名を直接入力中)

```json
// params
{
  "mode":      "prefix",
  "prefix":    "AMyChar",
  "file_path": "/path/to/MyClass.cpp",
  "line":      42,
  "character": 15
}
```

**モード2: メンバー補完** (`->` `.` `::` 検出後。Lua が trigger を判定してから呼ぶ)

```json
// params
{
  "mode":          "member_of",
  "class_name":    "AMyCharacter",     // resolve_type で解決済みの型名
  "filter":        "instance",         // "instance" | "static_or_enum"
  "access_filter": ["public", "protected"]
}
```

**共通レスポンス**

```json
{
  "items": [
    {
      "label":      "BeginPlay",
      "kind":       "function",
      "detail":     "void BeginPlay()",
      "access":     "protected",
      "class_name": "AActor",
      "deprecated": false
    }
  ]
}
```

> `kind` の値: `"function"` | `"property"` | `"class"` | `"struct"` | `"enum"` | `"enum_value"` | `"field"`

#### `resolve_type` — 式の型解決 (メンバー補完の前段)

```json
// params
{
  "symbol_name": "MyObj",       // カーソル左側の識別子
  "file_path":   "/path/to/MyClass.cpp",
  "line":        42,
  "character":   15
}

// Response
{
  "raw_type":  "AMyCharacter*",  // 宣言そのまま
  "base_type": "AMyCharacter",   // ポインタ/参照/テンプレートを除いた型名
  "kind":      "class"           // "class" | "struct" | "enum" | "builtin" | "unknown"
}
```

> **Lua 側の補完ルーティング全体像** (triggers は `lang_config.json` から読む):
> ```
> カーソル前テキストを走査
>     │
>     ├─ trigger "->" or "." を検出
>     │       → resolve_type("MyObj") → base_type 取得
>     │       → completion(mode="member_of", class_name=base_type, filter="instance")
>     │
>     ├─ trigger "::" を検出
>     │       → completion(mode="member_of", class_name="MyClass", filter="static_or_enum")
>     │         ("::" の左辺はクラス名直指定なので resolve_type 不要)
>     │
>     └─ トリガーなし (通常入力)
>             → completion(mode="prefix", prefix="入力中のテキスト")
> ```

#### `search_symbols` — プレフィックス/パターン検索

```json
// params
{
  "pattern":      "AMyChar",
  "symbol_types": ["class", "struct"],  // 省略可 = 全種
  "source_roots": ["MyGame", "MyEditor"], // 省略可 = 全ルート (UNL の modules フィルタ相当)
  "limit":        50
}

// Response
{
  "symbols": [
    {
      "name":        "AMyCharacter",
      "symbol_type": "class",
      "source_root": "MyGame",
      "file_path":   "/path/to/MyCharacter.h",
      "line":        15
    }
  ]
}
```

#### `get_symbol` — シンボル詳細取得

```json
// params
{ "name": "AMyCharacter", "namespace": null }

// Response
{
  "name":         "AMyCharacter",
  "symbol_type":  "class",
  "base_classes": ["ACharacter"],
  "file_path":    "/path/to/MyCharacter.h",
  "line_start":   15,
  "line_end":     180,
  "flags":        { "uclass": true, "blueprintable": true },
  "members":      [ ... ]
}
```

#### `get_members` — クラスメンバー取得 (アクセス制御フィルタ付き)

```json
// params
{
  "class_name":    "AMyCharacter",
  "access_filter": ["public", "protected"],  // 省略 = 全アクセス
  "member_types":  ["function", "property"], // 省略 = 全種
  "recursive":     true
}

// Response
{
  "members": [
    {
      "name":        "BeginPlay",
      "member_type": "function",
      "return_type": "void",
      "value_type":  null,
      "access":      "protected",
      "is_static":   false,
      "flags":       { "ufunction": true, "blueprint_native_event": true },
      "file_path":   "/path/to/MyCharacter.cpp",
      "line_start":  42,
      "defined_in":  "AMyCharacter"
    }
  ]
}
```

#### `get_inheritance` — 継承ツリー取得

```json
// params
{
  "class_name": "AMyCharacter",
  "direction":  "parents"   // "parents" | "children" | "both"
}

// Response
{
  "parents":  ["ACharacter", "APawn", "AActor"],
  "children": ["ABoss", "APlayer"]
}
```

#### `goto_definition` — 定義ジャンプ

```json
// params
{
  "file_path": "/path/to/MyClass.cpp",
  "content":   "...",
  "line":      55,
  "character": 20
}

// Response
{
  "file_path": "/path/to/MyCharacter.h",
  "line":      15,
  "character": 0
}
```

#### `find_usages` — シンボル参照一覧

`stream: false`（省略時デフォルト）は全件同期レスポンス。`stream: true` を指定するとストリーミングモードに切り替わる。

```json
// params (同期モード: stream 省略 or false)
{
  "symbol_name": "BeginPlay",
  "class_name":  "AMyCharacter",  // 省略可
  "file_path":   null             // 省略可: 特定ファイル内のみ
}

// Response (同期モード)
{
  "usages": [
    { "file_path": "/path/to/X.cpp", "line": 88, "preview": "Super::BeginPlay();" }
  ]
}

// params (ストリーミングモード)
{
  "symbol_name": "BeginPlay",
  "class_name":  "AMyCharacter",  // 省略可
  "file_path":   null,            // 省略可
  "stream":      true,
  "chunk_size":  10               // 省略可: デフォルト 10 (lang_config で変更可)
}

// ① Server → Client: Notification (chunk_size 件ごとに複数回送信)
// method: "find_usages/chunk"
{
  "req_id":      42,
  "chunk_index": 0,               // 0-based、順序保証
  "items": [
    { "file_path": "/path/to/X.cpp", "line": 88, "preview": "Super::BeginPlay();" }
  ]
}

// ② Server → Client: 最終 Response (元リクエストの id に対して返す)
{ "total": 47, "status": "done" }

// ② キャンセル時
{ "total": 23, "status": "cancelled" }

// Client → Server: キャンセル通知 (任意タイミングで送信可)
// method: "find_usages/cancel"
{ "req_id": 42 }
```

> **Lua 側の責務:** `find_usages/chunk` 通知を受け取るたびに Quickfix / Picker へ追記。`status: "done"` で確定表示。キャンセルはユーザーが picker を閉じた瞬間に `find_usages/cancel` を送信。

#### `search_files` — ファイル名検索

```json
// params
{
  "pattern":      "Character",
  "source_roots": ["MyGame"],  // 省略可 = 全ルート
  "limit":        30
}

// Response
{
  "files": [
    { "file_path": "/path/to/MyCharacter.h",  "source_root": "MyGame", "is_header": true },
    { "file_path": "/path/to/MyCharacter.cpp", "source_root": "MyGame", "is_header": false }
  ]
}
```

#### `list_source_roots` — source_root 一覧取得 (UNL の `GetModules` / `GetComponents` 相当)

```json
// params: {}  (なし)

// Response
{
  "source_roots": [
    {
      "name":        "MyGame",
      "path":        "/path/to/project/Source/MyGame",
      "marker_file": "MyGame.Build.cs",
      "file_count":  142
    },
    {
      "name":        "MyEditor",
      "path":        "/path/to/project/Source/MyEditor",
      "marker_file": "MyEditor.Build.cs",
      "file_count":  38
    },
    {
      "name":        "MyPlugin",
      "path":        "/path/to/project/Plugins/MyPlugin/Source/MyPlugin",
      "marker_file": "MyPlugin.uplugin",
      "file_count":  61
    }
  ]
}
```

#### `get_file_symbols` — ファイル内シンボル一覧

```json
// params
{ "file_path": "/path/to/MyCharacter.h" }

// Response
{
  "symbols": [
    { "name": "AMyCharacter", "symbol_type": "class",  "line_start": 15 },
    { "name": "FMyStruct",    "symbol_type": "struct", "line_start": 200 }
  ]
}
```

---

#### `search_assets` — Blueprint/uasset アセット検索

`.uasset` / `.umap` バイナリから解析した Blueprint クラスを検索する。  
Soft Object Reference パスの補完 (`/Game/Characters/BP_...`) にも使用。

```json
// params
{
  "pattern":   "BP_My",       // asset_name or asset_path の前方一致 or LIKE
  "kind":      "blueprint",   // 省略 = 全種。"blueprint" | "map" | "data_asset"
  "limit":     50
}

// Response
{
  "assets": [
    {
      "asset_path":    "/Game/Characters/BP_MyCharacter",
      "asset_name":    "BP_MyCharacter",
      "parent_class":  "AMyCharacter",    // C++ 側シンボルに解決済み (解決できなければ raw パス)
      "kind":          "blueprint"
    }
  ]
}
```

> `parent_class` が C++ シンボルに解決されていれば `get_members` で Blueprint クラスのメンバーも取得可能。

#### `grep_assets` — アセットバイナリ内バイト列検索

UNL の `GrepAssets` 相当。バイナリファイルを高速検索。  
ソフト参照先アセットパスの全文探索等に使用。

```json
// params
{ "pattern": "BP_MyCharacter", "limit": 100 }

// Response
{ "files": ["/path/to/SomeMap.umap", "/path/to/DataTable.uasset"] }
```

**実装方針 (mmap 安全性):**

mmap は高速だが、スキャン中にファイルが削除・切り詰められた場合に SIGBUS (Linux) / ACCESS_VIOLATION (Windows) が発生する。Rust の `catch_unwind` では OS シグナル起因のクラッシュは吸収できない。

```
対策: grep_assets には通常のバッファ読み込みを使用する (mmap は使わない)
理由:
  - uasset は既にスキャン済み (assets テーブルに NameMap が格納済み)
  - grep_assets はテキストパターンマッチのため、NameMap 列の FTS5 / LIKE 検索で代替可能
  - ファイル直接読み込みが必要なケースは assets にない文字列を探す場合のみ
    → その場合も BufReader + read_to_end で安全に読む

将来的に mmap による高速化が必要になった場合:
  - memmap2 クレートを使用 (unsafe を最小化)
  - ファイルを開く前に File::open の権限チェック
  - map_copy() ではなく map() (読み取り専用) を使用
  - SIGBUS 対策: ファイルサイズ確認後にアクセス範囲を制限
```

---

### 【サーバー → クライアント 通知】

| 通知メソッド | params | 意味 |
|---|---|---|
| `scan_progress` | `{ "stage": "parse", "current": 120, "total": 1842, "message": "..." }` | スキャン進捗 (**全クライアントへ broadcast**) |
| `scan_complete` | `{ "duration_ms": 4200, "files_scanned": 1842, "symbols_found": 28341 }` | スキャン完了 (**全クライアントへ broadcast**) |
| `asset_scan_complete` | `{ "assets_scanned": 3241 }` | アセットスキャン完了 (**全クライアントへ broadcast**) |
| `ready` | `{ "file_count": 1842 }` | 初回スキャン完了・クエリ受付開始 (**全クライアントへ broadcast**) |

---

### エラーコード定義

| コード | 定数名 | 意味 |
|---|---|---|
| `-32700` | `PARSE_ERROR` | リクエストのデシリアライズ失敗 |
| `-32600` | `INVALID_REQUEST` | 必須フィールド欠落 |
| `-32601` | `METHOD_NOT_FOUND` | 未知のメソッド |
| `-32602` | `INVALID_PARAMS` | パラメータ型・値不正 |
| `-32000` | `NOT_INITIALIZED` | `initialize` 前にクエリが来た |
| `-32001` | `SCAN_IN_PROGRESS` | スキャン中のため応答不可 |
| `-32002` | `DB_ERROR` | SQLite 操作エラー |
| `-32003` | `PARSER_LOAD_FAILED` | DLL 動的ロード失敗 |

---

## 6. DB スキーマ設計

### UNL からの変更点

| UNL | asdb | 理由 |
|-----|------|------|
| `modules` + `components` | `source_roots` | UE 固有概念を排除 |
| `classes` | `symbols` | 命名を言語中立に |
| DBパスにハッシュ値 | OS 標準キャッシュ + SHA-256 ハッシュ | Git を汚さず、読み取り専用ディレクトリも安全 |
| キャッシュフォルダ固定 | プラットフォーム別 XDG ディレクトリ | `~/.cache/asdb/` / `%LOCALAPPDATA%\asdb\` |

### テーブル定義

```sql
-- ① 文字列インターニング
CREATE TABLE strings (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    text TEXT NOT NULL UNIQUE
);

-- ② ディレクトリ階層木
CREATE TABLE directories (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER,
    name_id   INTEGER NOT NULL,
    UNIQUE(parent_id, name_id),
    FOREIGN KEY(parent_id) REFERENCES directories(id) ON DELETE CASCADE,
    FOREIGN KEY(name_id)   REFERENCES strings(id)
);

-- ③ スキャン対象ルート (modules/components の代替)
--    initialize.source_dirs エントリ + sub_root_markers で自動検出されたサブルート
CREATE TABLE source_roots (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    name_id        INTEGER NOT NULL,    -- 表示名。sub_root_markers 検出時はファイル名ステム ("MyGame", "MyPlugin" 等)
    dir_id         INTEGER NOT NULL,
    marker_file_id INTEGER,             -- 検出トリガーとなったファイル名 ("MyGame.Build.cs" 等)。手動登録は NULL
    UNIQUE(dir_id),
    FOREIGN KEY(name_id)        REFERENCES strings(id),
    FOREIGN KEY(dir_id)         REFERENCES directories(id) ON DELETE CASCADE,
    FOREIGN KEY(marker_file_id) REFERENCES strings(id)
);

-- ④ ファイル (mtime 差分スキャン用)
CREATE TABLE files (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    directory_id   INTEGER NOT NULL,
    filename_id    INTEGER NOT NULL,
    extension_id   INTEGER,            -- 拡張子 (".h", ".cpp" 等)
    mtime_ms       INTEGER,            -- Unix timestamp (ミリ秒) ← 秒精度では同一秒内の複数変更を取りこぼす
    file_hash_id   INTEGER,            -- SHA256 文字列
    source_root_id INTEGER,
    is_header      INTEGER DEFAULT 0,
    scan_generation INTEGER DEFAULT 0, -- クラッシュ中断スキャン検出用 (issue #9)
    UNIQUE(directory_id, filename_id),
    FOREIGN KEY(directory_id)   REFERENCES directories(id) ON DELETE CASCADE,
    FOREIGN KEY(filename_id)    REFERENCES strings(id),
    FOREIGN KEY(extension_id)   REFERENCES strings(id),
    FOREIGN KEY(file_hash_id)   REFERENCES strings(id),
    FOREIGN KEY(source_root_id) REFERENCES source_roots(id)
);

-- ⑤ モジュール/名前空間 階層 (C++ namespace / Rust mod / Java package 等)
CREATE TABLE modules (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id       INTEGER,           -- 階層: Foo → Foo::Bar → Foo::Bar::Baz
    name_id         INTEGER NOT NULL,
    module_type_id  INTEGER NOT NULL,  -- strings: 'namespace'|'package'|'module'
    file_id         INTEGER,           -- 定義ファイル (nullable: 複数ファイルに分散する場合あり)
    UNIQUE(parent_id, name_id),
    FOREIGN KEY(parent_id)      REFERENCES modules(id) ON DELETE CASCADE,
    FOREIGN KEY(name_id)        REFERENCES strings(id),
    FOREIGN KEY(module_type_id) REFERENCES strings(id),
    FOREIGN KEY(file_id)        REFERENCES files(id) ON DELETE SET NULL
);

-- ⑥ シンボル (旧 classes)
CREATE TABLE symbols (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    name_id        INTEGER NOT NULL,
    module_id      INTEGER,            -- modules テーブルを参照
    file_id        INTEGER,
    line_start     INTEGER,
    line_end       INTEGER,
    symbol_type_id INTEGER NOT NULL,   -- strings: 'class'|'struct'|'enum'|'interface'|...
    is_final       INTEGER DEFAULT 0,
    flags_id       INTEGER,            -- strings: .scm が返す追加メタ (JSON 文字列)
    FOREIGN KEY(name_id)        REFERENCES strings(id),
    FOREIGN KEY(module_id)      REFERENCES modules(id) ON DELETE SET NULL,
    FOREIGN KEY(file_id)        REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY(symbol_type_id) REFERENCES strings(id),
    FOREIGN KEY(flags_id)       REFERENCES strings(id)
);

-- ⑦ 継承関係
CREATE TABLE inheritance (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    child_id       INTEGER NOT NULL,
    parent_name_id INTEGER NOT NULL,
    parent_id      INTEGER,            -- 解決後の symbols.id (nullable)
    FOREIGN KEY(child_id)       REFERENCES symbols(id) ON DELETE CASCADE,
    FOREIGN KEY(parent_name_id) REFERENCES strings(id),
    FOREIGN KEY(parent_id)      REFERENCES symbols(id) ON DELETE SET NULL
);

-- ⑧ メンバー (関数・プロパティ・フィールド)
CREATE TABLE members (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol_id        INTEGER NOT NULL,
    name_id          INTEGER NOT NULL,
    member_type_id   INTEGER,          -- strings: 'function'|'property'|'field'|...
    type_id          INTEGER,          -- 型名
    return_type_id   INTEGER,
    access_id        INTEGER,          -- strings: 'public'|'protected'|'private'
    flags_id         INTEGER,          -- strings: .scm が返す追加メタ (JSON 文字列)
    is_static        INTEGER DEFAULT 0,
    line_start       INTEGER,
    line_end         INTEGER,
    file_id          INTEGER,          -- 実装ファイル (.cpp 等)
    FOREIGN KEY(symbol_id)      REFERENCES symbols(id) ON DELETE CASCADE,
    FOREIGN KEY(name_id)        REFERENCES strings(id),
    FOREIGN KEY(member_type_id) REFERENCES strings(id),
    FOREIGN KEY(type_id)        REFERENCES strings(id),
    FOREIGN KEY(return_type_id) REFERENCES strings(id),
    FOREIGN KEY(access_id)      REFERENCES strings(id),
    FOREIGN KEY(flags_id)       REFERENCES strings(id),
    FOREIGN KEY(file_id)        REFERENCES files(id) ON DELETE CASCADE
);

-- ⑨ enum 値
CREATE TABLE enum_values (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    enum_id INTEGER NOT NULL,
    name_id INTEGER NOT NULL,
    line    INTEGER,
    file_id INTEGER,
    FOREIGN KEY(enum_id) REFERENCES symbols(id) ON DELETE CASCADE,
    FOREIGN KEY(name_id) REFERENCES strings(id),
    FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE
);

-- ⑩ シンボル参照 (call graph)
CREATE TABLE symbol_calls (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL,
    line    INTEGER NOT NULL,
    name_id INTEGER NOT NULL,
    FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY(name_id) REFERENCES strings(id)
);

-- ⑪ #include グラフ
CREATE TABLE file_includes (
    file_id          INTEGER NOT NULL,
    include_path_id  INTEGER NOT NULL,
    base_filename_id INTEGER NOT NULL,
    resolved_file_id INTEGER,
    FOREIGN KEY(file_id)          REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY(include_path_id)  REFERENCES strings(id),
    FOREIGN KEY(base_filename_id) REFERENCES strings(id),
    FOREIGN KEY(resolved_file_id) REFERENCES files(id) ON DELETE SET NULL
);

-- ⑫ FTS5 全文検索インデックス
--    name / symbol_type はテキストのまま保持 (FTS5 は文字列インデックスのため)
CREATE VIRTUAL TABLE symbols_fts USING fts5(
    name,
    symbol_type,
    rowid_ref UNINDEXED
);

-- ⑬ アセット (Blueprint/uasset バイナリ解析結果)
CREATE TABLE assets (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id         INTEGER NOT NULL UNIQUE,
    asset_path_id   INTEGER NOT NULL,  -- strings: /Game/Characters/BP_MyChar (UE ロジカルパス)
    asset_name_id   INTEGER NOT NULL,  -- strings: BP_MyChar
    parent_class_id INTEGER,           -- strings: /Script/Engine.Character など
    parent_id       INTEGER,           -- 解決後の symbols.id (nullable)
    flags_id        INTEGER,           -- strings: JSON (Blueprint種別等)
    FOREIGN KEY(file_id)          REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY(asset_path_id)    REFERENCES strings(id),
    FOREIGN KEY(asset_name_id)    REFERENCES strings(id),
    FOREIGN KEY(parent_class_id)  REFERENCES strings(id),
    FOREIGN KEY(parent_id)        REFERENCES symbols(id) ON DELETE SET NULL,
    FOREIGN KEY(flags_id)         REFERENCES strings(id)
);

-- ⑭ アセット内関数 (Blueprint 関数ノード)
CREATE TABLE asset_functions (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id INTEGER NOT NULL,
    path_id  INTEGER NOT NULL,   -- strings: /Game/Characters/BP_MyChar:FunctionName
    FOREIGN KEY(asset_id) REFERENCES assets(id) ON DELETE CASCADE,
    FOREIGN KEY(path_id)  REFERENCES strings(id)
);

-- ⑮ アセット依存関係 (インポートグラフ)
CREATE TABLE asset_imports (
    asset_id       INTEGER NOT NULL,
    import_path_id INTEGER NOT NULL,   -- strings: /Script/... or /Game/... パス
    PRIMARY KEY(asset_id, import_path_id),
    FOREIGN KEY(asset_id)       REFERENCES assets(id) ON DELETE CASCADE,
    FOREIGN KEY(import_path_id) REFERENCES strings(id)
);

-- ⑯ プロジェクトメタ (キーバリューストア。値は JSON/テキストのため strings を経由しない)
CREATE TABLE project_meta (
    key   TEXT PRIMARY KEY,
    value TEXT
);
-- 格納キー:
--   db_version                   : スキーマバージョン (整数)
--   last_full_scan_at            : ISO8601 タイムスタンプ
--   scanners_config              : initialize で受け取った scanners 配列 (JSON)
--   ignore_dirs                  : JSON配列 ["Binaries","Intermediate",...]
--   current_scan_generation      : 進行中スキャンの世代番号 (INTEGER 文字列)
--   committed_scan_generation    : 最後に完了したスキャンの世代番号 (INTEGER 文字列)
--
-- クラッシュ検出 (issue #9):
--   起動時に current_scan_generation != committed_scan_generation なら
--   前回スキャンが中断されたと判断 → フルスキャンを強制する
--
-- ⚠️ scan_status / scan_pid / scan_started_ms は不要:
--    1プロジェクト = 1プロセスのため、DB にスキャン調整用キーは一切不要
```

---

## 7. `.scm` キャプチャ名 → `ScanOutput` フィールド マッピング規約

`.scm` クエリのキャプチャは、Core が**内部的に `ScanOutput` を組み立てる**際のフィールドマッピングとして機能する。  
すべてのパスが最終的に `ScanOutput` → DB ライタースレッドを通る（§8 参照）。

| キャプチャ名 | `ScanOutput.symbols[]` の対応フィールド |
|---|---|
| `@symbol.name` | `name` |
| `@symbol.type` | `symbol_type` (`class`/`struct`/`enum`/...) |
| `@symbol.module` | `module` (所属モジュール名、完全修飾可: `Foo::Bar`) |
| `@symbol.base` | `base` (継承元クラス名) |
| `@symbol.flags` | `flags` (JSON 文字列) |
| `@module.name` | モジュール名 (単一セグメント) |
| `@module.parent` | 親モジュール名 (完全修飾: `Foo::Bar`) |
| `@module.type` | `module_type` (`namespace`/`package`/`module`) |
| `@member.name` | `members[].name` |
| `@member.type` | `members[].member_type` |
| `@member.value_type` | `members[].type` |
| `@member.return_type` | `members[].return_type` |
| `@member.access` | `members[].access` |
| `@member.flags` | `members[].flags` (JSON 文字列) |
| `@enum_value.name` | (enum の) `members[].name` |

> **経路の統一**: Tree-sitter パスでも Scanner DLL パスでも、最終出力は同じ `ScanOutput` 構造体。  
> Core の DB ライタースレッドは `ScanOutput` だけを知っており、データの出どころを区別しない (§8)。

> **module 階層の組み立て**: `@module.name` + `@module.parent` を受け取った Core が `modules` テーブルに  
> 階層を自動構築する。`GameplayTags` のような namespace ベースの階層構造は PostScanner 不要で表現可能。

### `.scm` 記述例 (Unreal Engine C++)

```scheme
; unreal.scm
; ⚠️ #set! の引数に @ は付けない (キャプチャ参照ではなくプロパティ設定のため)

; モジュール/名前空間 (C++ namespace → modules テーブルに階層構築)
(namespace_definition
  name: (namespace_identifier) @module.name
  (#set! module.type "namespace"))

(class_specifier
  name: (type_identifier) @symbol.name
  (#set! symbol.type "class")
  base_class_clause: (base_class_clause
    (base_class_specifier
      name: (qualified_identifier) @symbol.base)))

(struct_specifier
  name: (type_identifier) @symbol.name
  (#set! symbol.type "struct"))

(enum_specifier
  name: (type_identifier) @symbol.name
  (#set! symbol.type "enum")
  body: (enumerator_list
    (enumerator
      name: (identifier) @enum_value.name)))

(function_definition
  declarator: (function_declarator
    declarator: (identifier) @member.name)
  (#set! member.type "function"))
```

---

## 8. パース パイプライン設計

### 3種類の DLL を動的ロード

```
asdb が実行時に動的ロードするもの:

  ① Grammar DLL  (Tree-sitter パーサー)
     tree-sitter-cpp.dll            ← 汎用 C++
     tree-sitter-unreal-cpp.dll     ← UE C++ (カスタムノード追加)
     tree-sitter-csharp.dll         ← C# (将来)

  ② Scanner DLL  (バイナリ/カスタム解析)
     asdb-scanner-ue-assets.dll     ← uasset バイナリ専用
     asdb-scanner-csharp.dll        ← C# 専用処理 (将来)

  ③ VcsAdapter DLL  (VCS 状態監視・差分取得)
     asdb-vcs-git.dll               ← Git
     asdb-vcs-svn.dll               ← SVN
     asdb-vcs-p4.dll                ← Perforce
     ※ 1プロジェクト = 1 DLL。lang_config.json の vcs_adapter で指定
```

**フレーバーは `lang_config.json` の組み合わせで決まる:**

| フレーバー | grammar_dll | query_file | scanner_dll | vcs_adapter_dll |
|---|---|---|---|---|
| UE C++ + Git | `tree-sitter-unreal-cpp.dll` | `unreal-cpp.scm` | なし | `asdb-vcs-git.dll` |
| Generic C++ + SVN | `tree-sitter-cpp.dll` | `generic-cpp.scm` | なし | `asdb-vcs-svn.dll` |
| UE Assets + P4 | なし | なし | `asdb-scanner-ue-assets.dll` | `asdb-vcs-p4.dll` |

> `asdb` はいずれのフレーバーも知らない。  
> VCS 固有の処理は完全に VcsAdapter DLL に委譲する。  
> **未コミット変更の検知は既存の `file_changed` RPC が担うため、VcsAdapter は commit/sync 操作のみを追う。**

---

### プラグインパッケージのイメージ

> **lspconfig と同じ思想**: 言語ごとの設定を `lang_config.json` 1ファイルに集約。  
> エディタプラグイン本体はこれを読み込むだけで、コードを一切変更せずに新言語対応できる。

```
asdb-plugin-unreal/              ← Neovim / VSCode プラグインに同梱
├── tree-sitter-unreal-cpp.dll   ① Grammar DLL
├── unreal-cpp.scm               ② Query file
├── asdb-scanner-ue-assets.dll   ③ Scanner DLL (uasset バイナリ用)
└── lang_config.json             ④ 言語設定（エディタ非依存）

asdb-plugin-generic-cpp/
├── tree-sitter-cpp.dll
├── generic-cpp.scm
└── lang_config.json
```

**`lang_config.json` の構造:**

```json
{
  "language": "unreal_cpp",

  "core": {
    "ignore_dirs": ["Binaries", "Intermediate", "Saved"],
    "sub_root_markers": [
      { "pattern": "*.Build.cs", "name_from": "stem" },
      { "pattern": "*.uplugin",  "name_from": "stem" }
    ],
    "scanners": [
      {
        "name":        "unreal_cpp",
        "extensions":  [".h", ".cpp", ".inl"],
        "source_dirs": ["Source", "Plugins"],
        "grammar_dll": "${PLUGIN_DIR}/tree-sitter-unreal-cpp.dll",
        "query_file":  "${PLUGIN_DIR}/unreal-cpp.scm"
      },
      {
        "name":        "unreal_assets",
        "extensions":  [".uasset", ".umap"],
        "source_dirs": ["Content"],
        "scanner_dll": "${PLUGIN_DIR}/asdb-scanner-ue-assets.dll"
      }
    ],
    "vcs_adapter": {
      "dll": "${PLUGIN_DIR}/asdb-vcs-git.dll"
    },
    "lang_hints": {
      "type_strip_keywords": ["virtual", "static", "inline", "FORCEINLINE", "const"],
      "api_macro_pattern":   "[A-Z0-9_]+_API",
      "header_extensions":   [".h", ".hpp", ".inl"],
      "source_extensions":   [".cpp", ".cc", ".c"]
    }
  },

  "editor": {
    "triggers": [
      { "pattern": "->", "action": "member_of", "filter": "instance",       "resolve_left": true  },
      { "pattern": ".",  "action": "member_of", "filter": "instance",       "resolve_left": true  },
      { "pattern": "::", "action": "member_of", "filter": "static_or_enum", "resolve_left": false }
    ]
  }
}
```

> `core` セクション → `initialize` リクエストにそのまま転送 (asdb が解釈)  
> `editor` セクション → エディタプラグインがローカルで保持し、補完ルーティングに使用  
> 新言語対応 = `lang_config.json` を書くだけ。Lua/TS のコード変更不要。

---

### 統一パースフロー (全パスが `ScanOutput` → DB ライター)

すべての解析経路の**出力は `ScanOutput` に統一**。DB ライタースレッドはこれだけを知る。

```
ファイル content
    │
    ▼
┌───────────────────────────────────────────────────────────┐
│  EP1: FileClassifier  (scanners 配列の extensions で振り分け)│
│                                                           │
│  .h/.cpp/.inl  → TextScanner へ                           │
│  .uasset/.umap → BinaryScanner へ                         │
│  *.generated.h → Skip                                     │
└──────────────┬─────────────────────────┬──────────────────┘
               │ Text                    │ Binary
               ▼                         ▼
┌──────────────────────────┐  ┌─────────────────────────────┐
│  TextScanner             │  │  BinaryScanner (scanner_dll) │
│                          │  │                             │
│  ① Grammar DLL ABI 確認  │  │  DLL の scan() を呼び出す    │
│    ts_language_version() │  │  → ScanOutput JSON を受け取る│
│    ≥ MIN_COMPATIBLE か検証│  │  → コールバックで JSON 受け取る  │
│    NG: エラーログ + skip  │  │                             │
│                          │  └──────────┬──────────────────┘
│  ② Grammar DLL でパース   │             │ ScanOutput
│    → tree_sitter::Tree   │             │
│                          │             │
│  ③ .scm クエリ実行        │             │
│    キャプチャ → RawSymbol │             │
│                          │             │
│  ④ RawSymbol → ScanOutput│             │
│    を Core が内部組み立て │             │
└──────────────┬───────────┘             │
               │ ScanOutput              │
               └────────────┬────────────┘
                            │ ScanOutput (統一)
                            ▼
               ┌────────────────────────┐
               │  DB ライタースレッド    │
               │  (channel 経由)        │
               │                       │
               │  "symbols" セクション  │
               │  → symbols / members  │
               │     inheritance       │
               │     modules ✅        │  ← namespace/package/mod 階層を自動構築
               │     enum_values       │
               │     symbols_fts ✅    │  ← FTS5 も同一トランザクションで更新
               │                       │
               │  "assets" セクション   │
               │  → assets             │
               │     asset_functions   │
               │     asset_imports     │
               └────────────────────────┘
```

> **Grammar ABI チェック (必須)**  
> Grammar DLL ロード直後に `ts_language_version(lang)` を取得し、  
> `asdb` にリンクされた `TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION` と比較。  
> 不一致の場合はそのスキャナーを無効化してエラーログを出す (クラッシュしない)。

---

### Scanner DLL の C-ABI インターフェース

Tree-sitter DLL と同方針 (C-ABI = Rust ABI 問題ゼロ)。  
結果は **JSON 文字列 (`ScanOutput`)** で返すことで、複雑な構造体受け渡しを回避：

```c
// asdb_scanner_plugin.h

typedef struct {
    // 静的文字列のみ可 (ヒープ確保禁止)
    const char* plugin_name;

    // このファイルを処理すべきか (1=yes, 0=skip)
    int (*should_run)(const char* file_path);

    // スキャンして ScanOutput JSON をコールバック経由で返す
    // ⚠️ scan() は複数スレッドから同時に呼ばれる。
    //    内部でグローバル可変状態を持つ場合は自前でロックすること。
    // ⚠️ コールバック内で JSON 文字列を必要なだけコピーすること。
    //    コールバックから戻った直後に DLL が json_ptr を解放してよい。
    //    → DLL とコアが異なるアロケータ (CRT) でもクラッシュしない。
    void (*scan)(
        const char*           file_path,
        const uint8_t*        content,
        size_t                content_len,
        void (*callback)(const char* json_ptr, size_t json_len, void* userdata),
        void*                 userdata
    );

    // DLL 解放前に呼ぶクリーンアップ関数
    void (*deinit)(void);

} asdb_scanner_plugin_t;

// DLL がエクスポートするエントリーポイント
// 返却ポインタの所有権は DLL 側が持つ。deinit() 後は無効になる。
asdb_scanner_plugin_t* asdb_scanner_init();
```

`scan()` が返す `ScanOutput` JSON フォーマット:

```json
{
  "version": 1,
  "symbols": [
    {
      "name":        "BS2GameplayTags",
      "symbol_type": "namespace",
      "line_start":  10,
      "line_end":    50,
      "members": [
        {
          "name":        "E000100",
          "member_type": "property",
          "access":      "public",
          "return_type": "FGameplayTag",
          "flags":       "{\"static\":true}",
          "line_start":  12
        }
      ]
    }
  ],
  "assets": [
    {
      "asset_path":   "/Game/Characters/BP_MyCharacter",
      "asset_name":   "BP_MyCharacter",
      "parent_class": "/Script/Engine.Character",
      "functions":    ["/Game/Characters/BP_MyCharacter:Jump"],
      "imports":      ["/Script/Engine.Character"]
    }
  ]
}
```

> - `"symbols"` のみ: テキスト系 PostScanner DLL
> - `"assets"` のみ: バイナリ系 Scanner DLL (uasset 等)
> - 両方: 混在ファイルタイプを扱う DLL (将来拡張)
> - **後方互換**: `"symbols"` だけ返す既存 DLL はそのまま動く (`version` フィールドも省略可)

---

### Rust 側の統合設計

TextScanner と BinaryScanner を **同一 Trait で透過的に扱う**:

```rust
// scan/scanner.rs

pub struct ScanInput<'a> {
    pub file_path: &'a str,
    pub content:   &'a [u8],
    pub tree:      Option<&'a tree_sitter::Tree>, // TextScanner のみ Some
}

pub trait Scanner: Send + Sync {
    fn name(&self) -> &'static str;
    fn should_run(&self, file_path: &str) -> bool;
    fn scan(&self, input: &ScanInput) -> ScanOutput;
}

// ─── TextScanner: Grammar DLL + .scm
struct TextScanner {
    grammar:    libloading::Library,   // Grammar DLL
    query:      tree_sitter::Query,    // コンパイル済み .scm
}
impl Scanner for TextScanner { /* captures → ScanOutput 変換 */ }

// ─── BinaryScanner: Scanner DLL (uasset 等)
struct BinaryScanner {
    _lib:   libloading::Library,
    plugin: *const asdb_scanner_plugin_t,
}

impl Drop for BinaryScanner {
    fn drop(&mut self) {
        unsafe { ((*self.plugin).deinit)() };
    }
}

impl Scanner for BinaryScanner {
    fn scan(&self, input: &ScanInput) -> ScanOutput {
        let mut result: Option<ScanOutput> = None;

        // コールバック: DLL が JSON を生成した直後に呼ばれる。
        // この時点で json_ptr はまだ DLL 側メモリ — 即コピーして所有権をコアに移す。
        // ⚠️ extern "C" コールバック内でパニックすると C スタックを巻き戻して UB になる。
        //    catch_unwind でパニックを吸収し、結果を None のままにして安全にフォールバックする。
        extern "C" fn on_result(
            json_ptr: *const c_char,
            json_len: usize,
            userdata: *mut c_void,
        ) {
            let _ = std::panic::catch_unwind(|| {
                let out = unsafe { &mut *(userdata as *mut Option<ScanOutput>) };
                let bytes = unsafe { std::slice::from_raw_parts(json_ptr as *const u8, json_len) };
                *out = serde_json::from_slice(bytes).ok();
            });
        }

        unsafe {
            ((*self.plugin).scan)(
                file_path_c,
                input.content.as_ptr(),
                input.content.len(),
                on_result,
                &mut result as *mut _ as *mut c_void,
            );
        }
        result.unwrap_or_default()
    }
}
```

```rust
// scan/pipeline.rs
// initialize 時に scanners 配列を読んでルーターを構築

pub fn build_scanners(scanners_config: &[ScannerConfig]) -> Vec<(Vec<String>, Box<dyn Scanner>)> {
    scanners_config.iter().map(|cfg| {
        let scanner: Box<dyn Scanner> = if let Some(ref dll) = cfg.grammar_dll {
            Box::new(TextScanner::load(dll, &cfg.query_file.unwrap())?)
        } else {
            Box::new(BinaryScanner::load(&cfg.scanner_dll.unwrap())?)
        };
        (cfg.extensions.clone(), scanner)
    }).collect()
}
```

---

### `initialize` パラメータ (確定版)

```json
{
  "method": "initialize",
  "params": {
    "root_path": "/path/to/MyProject",
    "config": {
      "ignore_dirs": ["Binaries", "Intermediate", "Saved", ".git"],

      "scanners": [
        {
          "name":        "unreal_cpp",
          "extensions":  [".h", ".cpp", ".inl"],
          "source_dirs": ["Source", "Plugins"],
          "source_files_list": "",
          "grammar_dll": "/path/to/tree-sitter-unreal-cpp.dll",
          "query_file":  "/path/to/unreal-cpp.scm"
        },
        {
          "name":        "unreal_assets",
          "extensions":  [".uasset", ".umap"],
          "source_dirs": ["Content"],
          "source_files_list": "",
          "scanner_dll": "/path/to/asdb-scanner-ue-assets.dll"
        }
      ],

      "sub_root_markers": [
        { "pattern": "*.Build.cs", "name_from": "stem" },
        { "pattern": "*.uplugin",  "name_from": "stem" }
      ],

      "lang_hints": {
        "type_strip_keywords": [
          "virtual", "static", "inline", "FORCEINLINE",
          "FORCEINLINE_DEBUGGABLE", "const", "constexpr",
          "friend", "explicit", "override", "final"
        ],
        "api_macro_pattern": "[A-Z0-9_]+_API",
        "header_extensions": [".h", ".hpp", ".inl"],
        "source_extensions": [".cpp", ".cc", ".c"]
      }
    }
  }
}
```

> `lang_config.json` の `core` セクションがそのまま `config` に対応する。  
> エディタ側は `lang_config.json` を読み込み、DLL パスを絶対パスに解決してから送信する。

---

### Rust モジュール構造

```
asdb/
├── src/
│   ├── main.rs
│   ├── transport/        ← JSON-RPC / MsgPack (言語知識ゼロ)
│   ├── storage/          ← SQLite 層 (言語知識ゼロ)
│   ├── query/            ← クエリ実行エンジン (言語知識ゼロ)
│   └── scan/
│       ├── mod.rs        ← パイプライン orchestrator
│       ├── text.rs       ← TextScanner: Grammar DLL + .scm
│       ├── binary.rs     ← BinaryScanner: scanner_dll C-ABI ラッパー
│       └── pipeline.rs   ← scanners 配列 → Scanner ルーター構築
```

---

### 拡張ロードマップ

| バージョン | メカニズム | 適用ケース |
|---|---|---|
| **v1** | `grammar_dll` + `.scm` + `modules` | 標準テキスト言語 (C++, C#, Rust 等) |
| **v1.5** | + `scanner_dll` | バイナリフォーマット (uasset 等)、特殊テキスト |
| **v2** | + `lua_hooks` (mlua, ~300KB) | コンパイル不要の独自マクロ対応 |

---

## 9. `.scm` クエリ設計

### 責務の境界線

```
.scm ファイルが知っていること          Rust コアが知っていること
─────────────────────────              ─────────────────────────
・どのノードタイプがシンボルか         ・declarator フィールドの
・どのノードタイプがメンバーか           チェーンを辿る方法
・#set! によるメタデータ分類           ・access_specifier 兄弟を
・言語固有の特殊ノード名                 遡ってアクセスレベルを決定する方法
  (unreal_class_declaration 等)       ・バイト範囲の包含でメンバーと
                                        シンボルを紐づける方法
                                      ・compound_statement 内か否かの判定
```

### キャプチャ名規約

```scheme
;; ─── シンボル ────────────────────────────────────────────────
@symbol.name         → symbols.name_id (必須)
;; #set! プレディケートで付与:
;;   symbol.type     → symbols.symbol_type ("class"|"struct"|"enum"|...)
;;   symbol.flags    → symbols.flags (JSON 文字列)
;;   symbol.module   → symbols.module_id (モジュール名で解決)

;; ─── モジュール/名前空間 ──────────────────────────────────────
@module.name         → modules.name_id (必須: 単一セグメント)
@module.parent       → modules.parent_id (完全修飾名で解決: "Foo::Bar")
;; #set! プレディケートで付与:
;;   module.type     → modules.module_type ("namespace"|"package"|"module")

;; ─── 継承 ────────────────────────────────────────────────────
@symbol.base         → inheritance.parent_name_id
;; 紐づけはバイト範囲の包含で解決

;; ─── メンバー ────────────────────────────────────────────────
@member.node         → members の親ノード (Rust が名前・型を抽出)
;; #set! プレディケートで付与:
;;   member.type     → members.member_type ("function"|"field"|"type_alias")
;;   member.flags    → members.flags (JSON 文字列)
;; ※ アクセスレベルは Rust が access_specifier 兄弟から自動判定

;; ─── enum 値 ─────────────────────────────────────────────────
@enum_value.name     → enum_values.name_id

;; ─── 呼び出しグラフ ──────────────────────────────────────────
@call.name           → symbol_calls.name_id

;; ─── インクルード ─────────────────────────────────────────────
@include.path        → file_includes.include_path_id
```

### `generic-cpp.scm` (汎用 C++ ベースライン)

```scheme
;; ============================================================
;; SECTION: symbols
;; ============================================================

(class_specifier  name: (type_identifier) @symbol.name  body: (_)
  (#set! symbol.type "class"))

(struct_specifier  name: (type_identifier) @symbol.name  body: (_)
  (#set! symbol.type "struct"))

(enum_specifier  name: (type_identifier) @symbol.name
  (#set! symbol.type "enum"))

(namespace_definition  name: (namespace_identifier) @module.name
  (#set! module.type "namespace"))

(preproc_def          name: (identifier) @symbol.name  (#set! symbol.type "define"))
(preproc_function_def name: (identifier) @symbol.name  (#set! symbol.type "define"))

;; ============================================================
;; SECTION: members
;; @member.node を受け取った Rust が宣言子チェーンを辿って名前/型を抽出
;; ============================================================

(function_definition)  @member.node  (#set! member.type "function")
(declaration)          @member.node  (#set! member.type "field")
(field_declaration)    @member.node  (#set! member.type "field")
(alias_declaration)    @member.node  (#set! member.type "type_alias")

;; ============================================================
;; SECTION: enum_values
;; ============================================================

(enumerator  name: (identifier) @enum_value.name)

;; ============================================================
;; SECTION: calls
;; ============================================================

(call_expression
  function: [
    (identifier) @call.name
    (field_expression field: (field_identifier) @call.name)
  ])

;; ============================================================
;; SECTION: includes
;; ============================================================

(preproc_include
  path: [(string_literal) @include.path (system_lib_string) @include.path])
```

### `unreal-cpp.scm` (UE 拡張、generic-cpp に追加)

```scheme
;; ============================================================
;; SECTION: symbols  (UE 専用ノード — tree-sitter-unreal-cpp が提供)
;; ============================================================

(unreal_class_declaration  name: (type_identifier) @symbol.name
  (#set! symbol.type "UCLASS"))

(unreal_struct_declaration  name: (_) @symbol.name
  (#set! symbol.type "USTRUCT"))

(unreal_enum_declaration  name: (_) @symbol.name
  (#set! symbol.type "UENUM"))

;; DECLARE_DELEGATE_*, DECLARE_EVENT_* 系
;; (unreal_declaration_macro ノードに対応)
(unreal_declaration_macro) @member.node
  (#set! member.type "function" member.flags "delegate")

;; ============================================================
;; SECTION: members
;; ============================================================

(unreal_function_declaration) @member.node
  (#set! member.type "function" member.flags "UFUNCTION")
```

### 既知の限界

| 限界 | 対応方針 |
|---|---|
| `UE_DEFINE_GAMEPLAY_TAG` namespace 階層 | Phase 2: `ue_gameplay_tags` PostScanner |
| `UE_DECLARE_GAMEPLAY_TAG_EXTERN` | 同上 |
| 複数 `.scm` ファイルの合成 | 将来: `query_files: [...]` で複数指定 |
| C# (.Target.cs 等) | 別 `csharp.scm` + `tree-sitter-c-sharp.dll` で対応予定 |

---

## 10. 差分スキャン戦略

### パス正規化規則 (issue #10)

同一プロジェクトを symlink 越しに開いた場合や Windows の大文字小文字揺れで別 DB が生成されないよう、ハッシュ計算前に必ず以下の正規化を行う。

```rust
fn normalize_root(raw: &str) -> anyhow::Result<PathBuf> {
    let abs = std::fs::canonicalize(raw)?;  // symlink 解決 + 絶対パス化
    #[cfg(target_os = "windows")]
    let abs = PathBuf::from(abs.to_string_lossy().to_lowercase());  // Windows: 大文字小文字を統一
    Ok(abs)
}

// 使用例:
//   normalize_root("/home/user/../user/project") → "/home/user/project"
//   normalize_root("C:\\Users\\Foo\\PROJECT")    → "c:\\users\\foo\\project"
//   normalize_root("/tmp/link_to_project")       → "/home/user/project"  (symlink 解決)
fn project_hash(root: &Path) -> String {
    let normalized = root.to_string_lossy();
    let digest = sha256::digest(normalized.as_bytes());
    digest  // 64 hex chars (full hash → DB/PID ファイル名)
}

fn project_short_hash(root: &Path) -> String {
    project_hash(root)[..16].to_string()  // 16 hex chars (socket/pipe 名)
}
```

> **Lua 側でも同様に正規化する** (§4 の `connect_or_spawn()` 内)  
> `vim.fn.fnamemodify(root, ":p")` でシンボリックリンク解決 +  
> Windows では `root:lower()` でハッシュを計算すること。

### スキャンの種類

| 種類 | トリガー | 対象 |
|---|---|---|
| **フルスキャン** | 初回起動 / DBバージョン更新 / 設定変更検知 | 全ファイル |
| **差分スキャン** | 再起動時 (DBあり) | 変更ファイルのみ |
| **単ファイル再スキャン** | `file_changed` 通知 | 1ファイル |
| **強制フルスキャン** | `rescan` リクエスト | 全ファイル |

---

### フルスキャン vs 差分スキャンの判定

起動時に `project_meta` テーブルの値と現在の設定を比較し、フルスキャンが必要か判断する：

```
initialize 受信 (クライアント登録)
    │
    ▼
root_path を正規化 (canonicalize + Windows lowercase) → SHA-256
    │
    ▼
XDG キャッシュパス解決 → DB を開く (なければ作成 → フルスキャン確定)
    │
    ▼
project_meta を読み込む
    │
    ├─ current_scan_generation != committed_scan_generation
    │    → 前回スキャンがクラッシュ中断 → フルスキャン (issue #9)
    │
    ├─ db_version が現在のコードと異なる → フルスキャン
    ├─ query_file_hash が変わった        → フルスキャン
    ├─ parser_dll_hash が変わった        → フルスキャン
    └─ 問題なし                          → 差分スキャン

// ⚠️ 1プロジェクト = 1プロセスのため "scanning_by_peer" ケース (旧ケースB/C) は存在しない
```

> **`project_meta` に保存するもの:**  
> `db_version`, `query_file_hash` (SHA256), `parser_dll_hash` (SHA256),  
> `last_full_scan_at`, `source_dirs` (JSON), `ignore_dirs` (JSON),  
> `current_scan_generation`, `committed_scan_generation`

**scan_generation フロー (issue #9):**

```
フルスキャン開始:
  gen = current_scan_generation + 1
  UPDATE project_meta SET value = gen WHERE key = 'current_scan_generation'
  (committed_scan_generation はまだ更新しない)
  files テーブルの各行に scan_generation = gen を記録しながら INSERT

フルスキャン完了:
  UPDATE project_meta SET value = gen WHERE key = 'committed_scan_generation'

次回起動時:
  current(gen=5) != committed(gen=4) → 中断を検出 → フルスキャン強制
  current(gen=5) == committed(gen=5) → 正常 → 差分スキャン
```

フルスキャン時は既存の `symbols`, `members`, `files` 等を全 DELETE してから再スキャン。

---

### 起動時 差分スキャン フロー

```
0. DB パス解決 (フォールバック付き):

   以下の優先順位で順に試み、最初に成功したパスを使用する。
   `initialize` レスポンスの `db_path` と `db_mode` で結果をクライアントに通知。

   ┌──────────────────────────────────────────────────────────────┐
   │ 優先度 1: XDG 標準キャッシュ (通常ケース)                    │
   │   Linux/macOS: ~/.cache/asdb/projects/<sha256>.db            │
   │   Windows:     %LOCALAPPDATA%\asdb\projects\<sha256>.db      │
   │   db_mode: "persistent"                                      │
   ├──────────────────────────────────────────────────────────────┤
   │ 優先度 2: 一時ディレクトリ (XDG が書き込み不可の場合)        │
   │   Linux/macOS: $TMPDIR/asdb/<sha256>.db                      │
   │   Windows:     %TEMP%\asdb\<sha256>.db                       │
   │   db_mode: "temp"  (OS 再起動 or /tmp クリアで消える可能性)  │
   ├──────────────────────────────────────────────────────────────┤
   │ 優先度 3: インメモリ DB (一時 dir も書き込み不可の場合)       │
   │   SQLite ":memory:"                                          │
   │   db_mode: "memory" (プロセス再起動で消える)                 │
   └──────────────────────────────────────────────────────────────┘

   いずれの場合もスキャン・補完は動作する。
   db_mode が "temp" または "memory" の場合、Lua 側は警告通知を表示する。

```rust
   // storage/db_path.rs
   pub enum DbMode { Persistent, Temp, Memory }

   pub fn resolve_db_path(root_path: &Path) -> (Option<PathBuf>, DbMode) {
       let hash = sha256_hex(root_path.to_string_lossy().as_bytes());
       let filename = format!("{}.db", hash);

       // 優先度 1: XDG cache
       if let Some(cache_dir) = platform_cache_dir() {
           let path = cache_dir.join("asdb").join("projects").join(&filename);
           if ensure_dir_writable(&path).is_ok() {
               return (Some(path), DbMode::Persistent);
           }
       }
       // 優先度 2: TMPDIR
       if let Some(tmp_dir) = std::env::temp_dir().canonicalize().ok() {
           let path = tmp_dir.join("asdb").join(&filename);
           if ensure_dir_writable(&path).is_ok() {
               return (Some(path), DbMode::Temp);
           }
       }
       // 優先度 3: メモリ
       (None, DbMode::Memory)
   }
```

   `initialize` レスポンスへの反映:
   ```json
   {
     "db_path": "/home/user/.cache/asdb/projects/abc123.db",
     "db_mode": "persistent"   // "persistent" | "temp" | "memory"
   }
   ```

   Lua 側警告ロジック:
   ```lua
   if result.db_mode == "temp" then
     vim.notify("[asdb] DB はTMPDIRに保存されています。再起動時に再スキャンが必要になる場合があります。", vim.log.levels.WARN)
   elseif result.db_mode == "memory" then
     vim.notify("[asdb] DB はメモリのみです。エディタを閉じるとキャッシュが消えます。", vim.log.levels.WARN)
   end
   ```

   > **読み書き対称性:** `resolve_db_path()` は起動時に**1回だけ**呼ばれる。  
   > 戻り値の `(PathBuf, DbMode)` に基づいて SQLite 接続を1本確立し、  
   > 以降の **全 SQL 操作（SELECT / INSERT / UPDATE / DELETE）はこの1本の接続のみを使用する**。  
   > 「書き込みフォールバック」ではなく「接続確立フォールバック」であり、読み込みも自動的に同 tier を使う。

   > **`DbMode::Memory` のマルチプロセス制約:**  
   > `:memory:` DB はプロセス内でしか参照できないため、複数のエディタが同一プロジェクトを開いた場合に  
   > **各プロセスが独立した `:memory:` DB を持つ**。WAL による共有は不可。  
   > → 各プロセスが独立してフルスキャンを実行する。  
   > → `db_mode: "memory"` の場合は Lua 側で追加警告を表示する:  
   > ```lua
   > -- memory モードでは複数ウィンドウ間の補完同期が行われないことを警告
   > vim.notify("[asdb] メモリモード: 複数のエディタウィンドウで開いている場合、補完は同期されません。", vim.log.levels.WARN)
   > ```  
   > この制約は「書き込み権限が一切ない環境」という極端なケースに限定され、通常の使用には影響しない。

1. DB の files テーブルから全 (path, mtime_ms) を HashMap に取得

2. source_dirs を walk (ignore_dirs を除外) してディスク上のファイル一覧を列挙

3. ファイルごとに分類:
   ┌─────────────────────────────────────────────────────────┐
   │  ディスクにあり DB にもある                              │
   │    mtime_ms が同じ  → スキップ (変更なし)                 │
   │    mtime_ms が違う  → "変更" リストへ                      │
   │                                                         │
   │  ディスクにあり DB にない  → "追加" リストへ             │
   │  DB にあるがディスクにない → "削除" リストへ             │
   └─────────────────────────────────────────────────────────┘

4. 削除ファイルを先に処理 (FK CASCADE で symbols も消える):
     DELETE FROM files WHERE path = ?
     DELETE FROM symbols_fts WHERE rowid IN (...)  ← FTS5 手動同期

5. **sub_root_markers によるサブルート自動検出 (フルスキャン時のみ)**:
     ウォーク中にマーカーパターンにマッチするファイルを発見したら
     そのディレクトリを source_roots に UPSERT
       name = マーカーファイルのステム ("MyGame.Build.cs" → "MyGame")
       marker_file = ファイル名
     配下のファイルは files.source_root_id にその id を設定

6. 変更・追加ファイルをスキャナー種別ごとに **2つの独立したスレッドプール**で並列パース:

     ```
     ┌──────────────────────────────────────────────────────────┐
     │  text_pool (CPU-bound: TextScanner)                      │
     │  ワーカー数: num_cpus / 2                                │
     │  対象: .h / .cpp / .inl 等の tree-sitter パース          │
     └─────────────────────┬────────────────────────────────────┘
                           │ mpsc channel (ScanOutput)
     ┌──────────────────────────────────────────────────────────┐
     │  binary_pool (I/O-bound: BinaryScanner)                  │
     │  ワーカー数: num_cpus (I/O 待ちが多いため多め)           │
     │  対象: .uasset / .umap 等のバイナリパース                │
     └─────────────────────┬────────────────────────────────────┘
                           │ 同じ mpsc channel に合流
                           ▼
                     DB ライタースレッド (1本)
     ```

     - 両プールは **同時スタート** (逐次ではない)
     - DB ライタースレッドは両プールから流れてくる ScanOutput を統一処理
     - ワーカー数はフルスキャン開始時に `initialize` パラメータで上書き可 (将来対応)

7. DB ライタースレッドが channel を受信し、ファイル単位のトランザクションで書き込み:
     BEGIN
       DELETE FROM symbols WHERE file_id = ?   ← 変更ファイルの場合
       DELETE FROM symbols_fts WHERE rowid IN (...) ← FTS5 手動同期
       INSERT INTO files ...
       INSERT INTO symbols ... + INSERT INTO symbols_fts ... ← 必ずセット
       INSERT INTO members ...
     COMMIT

8. **text_pool 完了を待機** (binary_pool は継続実行中でも可)
     → 継承テーブルの再解決パス (ソーススキャン完了が前提):
     ```sql
     UPDATE inheritance
     SET parent_id = (SELECT id FROM symbols WHERE name = parent_name)
     WHERE parent_id IS NULL;
     ```
     → `assets.parent_id` の遅延解決:
     ```sql
     UPDATE assets
     SET parent_id = (SELECT id FROM symbols WHERE name = assets.parent_name)
     WHERE parent_id IS NULL;
     ```

9. **binary_pool 完了を待機** (text_pool より後に終わった場合)

10. 全スキャン完了 → `scan_complete` 通知を送信
```

**channel を経由した理由:** SQLite は基本的に1ライタースレッド推奨。  
パース (CPU bound) と書き込み (I/O bound) を分離することで両方の効率を最大化。

---

### `file_changed` 時フロー (エディタ保存イベント)

```
file_changed 受信 { file_path: "/abs/path/to/Foo.h" }
    │
    ▼
1. file_path が source_dirs 配下かつ ignore_dirs 外か確認
   → 対象外なら即 return

2. BEGIN TRANSACTION
   DELETE FROM files WHERE path = ?
     ↳ ON DELETE CASCADE で symbols / members / enum_values も連鎖削除
   -- FTS5 も手動で同期 (CASCADE 対象外のため)
   DELETE FROM symbols_fts WHERE rowid IN
     (SELECT id FROM symbols WHERE file_id = old_file_id)
   COMMIT

3. ファイルを再パース → ScanOutput

4. BEGIN TRANSACTION
   INSERT INTO files (path, mtime_ms, ...)
   INSERT INTO symbols ...  → symbols_fts にも同時 INSERT
   INSERT INTO members ...
   INSERT INTO inheritance (WHERE parent_name 解決可能なもの)
   COMMIT

5. 継承テーブルの再解決パス (名前ベース):
   -- 削除したシンボルを parent としていた未解決の参照を再リンク
   UPDATE inheritance
   SET parent_id = (SELECT id FROM symbols WHERE name = parent_name)
   WHERE parent_id IS NULL

6. files.mtime_ms を新しい値で更新済み
```

> **継承参照の保護:** `file_changed` で親クラスの定義ファイルが再スキャンされた場合、  
> `parent_id` が旧 `symbol_id` を指したまま stale になる可能性がある。  
> FK 制約は `ON DELETE SET NULL` とし、ステップ 5 で名前ベース再リンクすることで整合性を回復。

> **FTS5 の手動同期:** `symbols_fts` は仮想テーブルのため CASCADE が効かない。  
> INSERT / DELETE は必ず `symbols` と同一トランザクションで行うこと。

---

### DB の並列アクセス戦略

```sql
-- DB 初期化時に WAL モードと Busy Timeout を設定
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;  -- WAL なら FULL より高速で安全
PRAGMA busy_timeout = 5000;   -- 5秒待機 (複数エディタが同一 DB を開いた場合の競合解決)
```

| 操作 | スレッド | 備考 |
|---|---|---|
| TextScanner (ソーススキャン) | text_pool (CPU-bound, n/2 ワーカー) | tree-sitter パース |
| BinaryScanner (アセットスキャン) | binary_pool (I/O-bound, n ワーカー) | 両プール同時スタート |
| DB 書き込み | DB ライタースレッド (1本) | 両プールの channel を統合受信 |
| completion / search | クエリスレッド (リクエスト毎) | WAL で読み込みは並列OK |
| file_changed | 専用キュー経由でライタースレッドへ | スキャン中でも受付 |
| 複数エディタ接続 | 同一プロセス内の複数クライアントタスク | DB は1プロセスが独占アクセス。WAL による外部プロセス競合は発生しない |

> **WAL の役割 (単一プロセス内)**: 読み取りスレッド (補完クエリ) とライタースレッド (スキャン書き込み) の並列動作を保証するためにのみ使用。複数プロセス間の共有は不要。

> **接続確立の対称性:** `resolve_db_path()` は起動時に1回だけ実行される。以降の SELECT / INSERT / UPDATE / DELETE はすべてこの1本の接続を使う。「書き込みフォールバック」ではなく「**接続確立フォールバック**」であり、読み込みも同 tier が自動的に適用される。

> **`DbMode::Memory` のマルチクライアント制約:** `:memory:` DB はプロセス内でしか参照できないが、  
> 1プロジェクト = 1プロセスの設計により全クライアントが同じメモリ DB を共有する (問題なし)。

---

### DB ライタースレッドの文字列インターニング処理 ③

`ScanOutput` は生文字列 (Raw String) で届く。DB ライタースレッドがインターニングを担当する。

```rust
// storage/intern.rs

pub struct StringIntern {
    cache: HashMap<String, i64>,  // 文字列 → strings.id のインメモリキャッシュ
}

impl StringIntern {
    // 文字列を strings テーブルに登録し、id を返す
    // キャッシュヒットなら SQL を発行しない (スキャン高速化の要)
    pub fn intern(&mut self, conn: &Connection, text: &str) -> rusqlite::Result<i64> {
        if let Some(&id) = self.cache.get(text) {
            return Ok(id);
        }
        conn.execute("INSERT OR IGNORE INTO strings (text) VALUES (?1)", [text])?;
        let id: i64 = conn.query_row(
            "SELECT id FROM strings WHERE text = ?1", [text], |r| r.get(0)
        )?;
        self.cache.insert(text.to_string(), id);
        Ok(id)
    }
}
```

DB ライタースレッドは `StringIntern` をスレッドローカルに保持し、全フィールドに適用する:

```
ScanOutput.symbols[i].symbol_type (生文字列)
    → intern("class") → symbol_type_id = 3
    → INSERT INTO symbols (..., symbol_type_id) VALUES (..., 3)
```

- キャッシュはフルスキャン開始時に空の状態で初期化し、スキャン完了まで保持
- `file_changed` 時も同じ `StringIntern` を使いまわす (プロセス寿命と同じ)
- `"class"`, `"function"`, `"public"` 等の頻出文字列は初期化時にプリロード可能 (将来最適化)

---

### `resolve_type` の内部実装方針 ④

`resolve_type` はカーソル左側の識別子の型を解決し、メンバー補完の前段として呼ばれる。  
**2フェーズ実装**を採用する:

```
Phase 3 実装: DB ルックアップのみ
    "MyObj" → symbols WHERE name = "MyObj" AND symbol_type IN ('class', 'struct')
    → ヒット → base_type = "MyObj" を返す
    → ミス → kind = "unknown" を返す

Phase 4+ 実装: バッファ内 tree-sitter クエリを追加
    symbols WHERE name = "MyObj" でミスした場合:
        → file_path のファイルを再パース (またはキャッシュ済みの Tree を使用)
        → .scm クエリで変数宣言を探す:
            (declaration declarator: (init_declarator declarator: (identifier) @var.name)
              type: (_) @var.type (#eq? @var.name "MyObj"))
        → @var.type の文字列から lang_hints.type_strip_keywords を除去して base_type を抽出
```

**Phase 4+ 変数宣言キャプチャ用 `.scm` (C++):**

```query
; resolve_type_cpp.scm
; 変数宣言から (変数名 → 型名) のペアを一本釣りする

; パターン A: 通常の型宣言  例: MyClass* Obj = ...;
(declaration
  type: [
    (type_identifier)        @var.type
    (qualified_identifier)   @var.type
    (template_type)          @var.type
    (pointer_declarator (type_identifier) @var.type)
  ]
  declarator: [
    (identifier)                         @var.name
    (pointer_declarator (identifier)     @var.name)
    (reference_declarator (identifier)   @var.name)
    (init_declarator declarator: (identifier)         @var.name)
    (init_declarator declarator: (pointer_declarator (identifier) @var.name))
  ]
)

; パターン B: TObjectPtr<T> 等のテンプレート  例: TObjectPtr<AMyCharacter> Obj;
(declaration
  type: (template_type
    name:      (_) @template.outer   ; TObjectPtr, TArray 等
    arguments: (template_argument_list (type_descriptor (type_identifier) @var.type))
  )
  declarator: [
    (identifier)                     @var.name
    (init_declarator declarator: (identifier) @var.name)
  ]
)

; パターン C: auto  例: auto Obj = Cast<AMyCharacter>(Other);
; auto は tree-sitter が "auto" を type_identifier として返すため、
; Cast<T> の T を別途抽出する必要がある → Phase 5 以降のスコープ
```

> **`.scm` の配置:** `lang_config.json` の `resolve_type_query` フィールドで指定。  
> 省略した場合は Phase 3 (DB ルックアップのみ) で動作する。

> **Phase 3 の限界:** ローカル変数・パラメータの型は解決不可。`AMyCharacter* Obj;` の `Obj` → `unknown`。  
> メンバー変数はクラスの `members` テーブルから解決できる場合がある (将来: `get_members` 経由で型を検索)。  
> Phase 3 の現実的カバー率: `::` 演算子 (クラス名直打ち) ≒ 100%、`->` / `.` ≒ 40〜60%。

---

### `file_opened` + 未保存バッファの扱い ⑤

`file_opened` の `content` フィールドは**インメモリ一時パース**に使用する。DB への書き込みは行わない。

```
file_opened 受信 { file_path, content: "..." (未保存バッファ内容) }
    │
    ▼
1. content を tree-sitter でパース → Tree をメモリ上に保持
2. ActiveProject.transient_trees: HashMap<PathBuf, TransientTree> に格納
3. レスポンス: { "status": "parsed", "symbols_found": N }

completion / resolve_type リクエスト受信時:
    → file_path が transient_trees に存在する場合、DB の代わりに TransientTree を参照
    → 存在しない場合は DB を参照 (通常フロー)

file_changed 受信 (保存イベント):
    → transient_trees から該当エントリを削除
    → ディスクファイルを再パース → DB に書き込み (通常の file_changed フロー)
```

> **用途:** LSP の `textDocument/didChange` 相当。保存前でも補完精度を維持する。  
> **メモリ管理:** 後述の GC 戦略を参照。

---

### `transient_trees` サイズ上限と GC 戦略

`transient_trees` は開いているファイルのインメモリ tree-sitter ツリーを保持する。
大規模ファイルや多くのバッファが同時に開かれた場合のメモリ爆食いを防ぐためにサイズ上限と GC を設ける。

```rust
struct TransientTree {
    tree:          tree_sitter::Tree,
    source_bytes:  Vec<u8>,     // クエリ実行に必要なソース本体
    size_bytes:    usize,       // source_bytes.len()
    last_accessed: Instant,     // LRU 更新用
}

struct TransientTreeStore {
    trees:          HashMap<PathBuf, TransientTree>,
    total_bytes:    usize,
    max_bytes:      usize,      // デフォルト: 50MB (initialize で変更可)
    max_entries:    usize,      // デフォルト: 50 ファイル
}
```

**GC トリガー条件 (いずれかを満たしたら発動):**

| トリガー | 説明 |
|---|---|
| `total_bytes > max_bytes` | 合計サイズが上限を超えた |
| `trees.len() > max_entries` | エントリ数が上限を超えた |
| `file_changed` 受信 | 保存されたエントリを即削除 (GC ではなく通常削除) |
| `shutdown` | 全エントリを解放 |

**GC 戦略: LRU エビクション**

```
GC 発動時:
    1. last_accessed が最も古いエントリから削除
    2. total_bytes が max_bytes * 0.8 (80%) を下回るまで繰り返す
    3. エビクトされたエントリへの次回アクセス → DB フォールバック (補完精度は低下するが動作は継続)
```

**単一ファイルが上限超えの場合 (例: 自動生成コードの巨大ヘッダー):**

```
file_opened 時に content が max_bytes / 2 を超える場合:
    → transient_trees に格納しない
    → レスポンス: { "status": "skipped", "reason": "file_too_large" }
    → DB フォールバックで動作継続
```

**`initialize` パラメータでの設定:**
```json
"transient_trees": {
  "max_mb":      50,   // 合計上限 (MB)
  "max_entries": 50    // ファイル数上限
}
```

---

```
rescan 受信 { "mode": "full" | "incremental" }
    │
    ├─ "full"        → project_meta をリセット → フルスキャン開始
    └─ "incremental" → 差分スキャンを再実行 (mtime 比較から)
```

既に進行中のスキャンがある場合は現在のスキャンをキャンセルして新しいスキャンを開始。

---

### `ignore_dirs` のデフォルト値と結合ルール

```json
["Binaries", "Intermediate", "Saved", "DerivedDataCache",
 ".git", "node_modules", ".vs", ".idea"]
```

`initialize` で `ignore_dirs` を渡した場合は **デフォルトとマージ** (重複排除)。  
`ignore_dirs_override: true` を指定した場合はデフォルトを**完全に上書き**する。

```json
// 通常: デフォルト + 追加
{ "ignore_dirs": ["ThirdParty", "Marketplace"] }
// → デフォルト8件 + ThirdParty + Marketplace = 10件

// 完全上書き: カスタム環境でデフォルトが邪魔な場合
{ "ignore_dirs": ["node_modules"], "ignore_dirs_override": true }
// → node_modules のみ (Binaries 等も対象になるので注意)
```

> **`ignore_dirs_override` の必要性判断:** 決定。デフォルトマージのみでは対応できないケース（例: Intermediate を解析したい場合）があるため、`ignore_dirs_override: true` フラグを正式サポートする。省略時デフォルト `false`。

---

### エラーハンドリング方針

| ケース | 挙動 |
|---|---|
| 1ファイルのパース失敗 | そのファイルをスキップしてスキャン継続。`scan_progress` に error フィールドで報告 |
| DBへの書き込み失敗 | ロールバック後、次のファイルへ継続 |
| XDG キャッシュへの書き込み権限なし | TMPDIR → `:memory:` のフォールバックパスを試みる (§10 スキャンフロー ステップ0 参照) |
| Scanner DLL のロード失敗 | エラーログ + その DLL をスキップして継続 |

---

### ログファイル設計

```
# Linux / macOS
~/.cache/asdb/logs/<sha256_of_root_path>.log

# Windows
%LOCALAPPDATA%\asdb\logs\<sha256>.log
```

- **ファイル名は DB と同一の sha256 ハッシュ** → プロジェクトと 1:1 対応、管理が容易
- ローテーション: 起動時に前回ログを `.log.1` にリネーム、上限 3世代 (`.log`, `.log.1`, `.log.2`)
- ログレベル: `error` / `warn` / `info` / `debug`  
  デフォルト: `info`。`initialize` の `log_level` フィールドで変更可
- 書式: `[ISO8601] [LEVEL] [module] message`  
  例: `[2026-05-20T08:00:01Z] [INFO] [scanner] scanned 1842 files in 3.2s`
- `db_mode: "memory"` の場合はログもメモリバッファに保持し、`shutdown` 時に捨てる

---

## 11. エディタ側 Lua 薄皮 設計 (Neovim)

### 設計原則

> **Lua側はつなぎ役に徹する。判断・処理はすべて `asdb` 側。**

Lua が知っていること:
- どのディレクトリをプロジェクトルートとみなすか
- どのプロジェクトタイプか (マーカーファイルだけで判断)
- プラグイン同梱 DLL のパス

Lua が**やらないこと:**
- ファイルのパース・解析
- シンボルのフィルタリング・加工
- プロセス間のデータ変換 (そのまま横流し)

---

### 責務の全量と行数見積もり

| モジュール | 責務 | 概算行数 |
|---|---|---|
| `detect.lua` | プロジェクトルート特定・タイプ判定 | ~35行 |
| `process.lua` | `asdb` プロセスの起動・終了・RPC送受信 | ~60行 |
| `init.lua` | `initialize` ペイロード組み立て＆送信 | ~40行 |
| `events.lua` | `BufWritePost` 等のエディタイベント → RPC通知 | ~25行 |
| `source.lua` | blink-cmp ソースアダプタ | ~55行 |
| **合計** | | **~215行** |

---

### プロジェクトルート特定 (`detect.lua`)

```lua
-- カレントファイルから上位ディレクトリを辿り、マーカーファイルを探す
local MARKERS = {
  unreal = { "*.uproject" },
  generic_cpp = { "CMakeLists.txt", "compile_commands.json", ".clangd" },
  csharp = { "*.sln", "*.csproj" },
}

local function find_root(bufnr)
  local path = vim.api.nvim_buf_get_name(bufnr)
  return vim.fs.root(path, function(name, _)
    -- いずれかのマーカーにマッチすれば root 確定
    for _, markers in pairs(MARKERS) do
      for _, m in ipairs(markers) do
        if vim.fn.glob(name .. "/" .. m) ~= "" then return true end
      end
    end
    return false
  end)
end

local function detect_type(root)
  if vim.fn.glob(root .. "/*.uproject") ~= "" then return "unreal" end
  if vim.fn.filereadable(root .. "/CMakeLists.txt") == 1 then return "generic_cpp" end
  if vim.fn.glob(root .. "/*.sln") ~= "" then return "csharp" end
  return "generic_cpp"  -- フォールバック
end
```

---

### lang_config.json ロードと DLL パス解決 (`config.lua`)

`lang_config.json` の `core` セクションをそのまま `initialize` に転送するだけ。  
DLL パスはプラグインディレクトリからの**相対パス固定**で `core` に追記する。

```lua
local plugin_dir = vim.fn.fnamemodify(debug.getinfo(1).source:sub(2), ":h:h:h")
-- plugin_dir = ~/.local/share/nvim/lazy/asdb.nvim/

local function load_lang_config(lang_name)
  -- 例: "unreal" → asdb-plugin-unreal/lang_config.json
  local path = plugin_dir .. "/plugins/asdb-plugin-" .. lang_name .. "/lang_config.json"
  local f = io.open(path, "r")
  if not f then return nil end
  local cfg = vim.json.decode(f:read("*a"))
  f:close()

  -- DLL パスを絶対パスに解決 (lang_config.json はパス名のみ持つ)
  local bin_dir = plugin_dir .. "/plugins/asdb-plugin-" .. lang_name .. "/"
  local ext = jit.os == "Windows" and ".dll" or jit.os == "OSX" and ".dylib" or ".so"

  cfg.core.parser_dll   = bin_dir .. "tree-sitter-" .. lang_name:gsub("_", "-") .. ext
  cfg.core.query_file   = bin_dir .. lang_name:gsub("_", "-") .. ".scm"
  cfg.core.scanner_dlls = vim.tbl_map(function(name)
    return bin_dir .. name .. ext
  end, cfg.core.scanner_dll_names or {})

  return cfg
end
```

---

### プロセス管理 (`process.lua`)

```lua
local function start(root, lang_cfg)
  local job_id = vim.fn.jobstart(
    { "asdb", "--protocol", "msgpack" },
    { rpc = true, on_exit = function() M.job_id = nil end }
  )
  M.job_id    = job_id
  M.triggers  = lang_cfg.editor.triggers  -- triggers をローカルに保持

  -- core セクションをそのまま initialize に転送
  vim.rpcnotify(job_id, "initialize", {
    root_path = root,
    config    = lang_cfg.core,
  })
end

local function stop()
  if M.job_id then
    vim.rpcnotify(M.job_id, "shutdown", {})
    vim.fn.jobstop(M.job_id)
  end
end
```

`rpc = true` を指定するだけで Neovim が msgpack-rpc を自動処理。  
JSON のパース/シリアライズコードは**1行も書かない**。

---

### イベントフック (`events.lua`)

```lua
-- BufWritePost → file_changed 通知 (保存後即トリガー)
vim.api.nvim_create_autocmd("BufWritePost", {
  callback = function(ev)
    if not M.job_id then return end
    local path = vim.api.nvim_buf_get_name(ev.buf)
    -- 拡張子チェックのみ (重い処理なし)
    if path:match("%.[ch]$") or path:match("%.cpp$") or path:match("%.inl$") then
      vim.rpcnotify(M.job_id, "file_changed", { file_path = path })
    end
  end,
})

-- VimLeavePre → graceful shutdown
vim.api.nvim_create_autocmd("VimLeavePre", {
  callback = function() process.stop() end,
})
```

---

### blink-cmp ソースアダプタ (`source.lua`)

trigger ルーティングを `lang_config.json` の `editor.triggers` で行う。  
重い処理はゼロ。カーソル前テキストの正規表現マッチのみ。

```lua
local function detect_trigger(line, triggers)
  for _, t in ipairs(triggers) do
    -- カーソル直前に trigger pattern があるか
    local pat = vim.pesc(t.pattern)
    local left = line:match("([%w_]+)" .. pat .. "%s*$")
    if left then return t, left end
  end
  return nil, nil
end

function source:get_completions(ctx, callback)
  if not process.job_id then return callback({ items = {}, isIncomplete = false }) end

  local line     = ctx.cursor_before_line
  local triggers = process.triggers  -- lang_config.json の editor.triggers

  local trigger, left_expr = detect_trigger(line, triggers)

  local params
  if trigger and trigger.resolve_left then
    -- Step 1: resolve_type (同期的に先行実行)
    local ok, res = pcall(vim.rpcrequest, process.job_id, "resolve_type", {
      symbol_name = left_expr,
      file_path   = vim.api.nvim_buf_get_name(0),
      line        = ctx.cursor.row,
      character   = ctx.cursor.col,
    })
    if not ok or not res or res.kind == "unknown" then
      return callback({ items = {}, isIncomplete = false })
    end
    -- Step 2: member_of 補完
    params = {
      mode         = "member_of",
      class_name   = res.base_type,
      filter       = trigger.filter,
      access_filter = { "public", "protected" },
    }
  elseif trigger and not trigger.resolve_left then
    -- "::" 等: 左辺がクラス名直指定
    params = {
      mode      = "member_of",
      class_name = left_expr,
      filter    = trigger.filter,
    }
  else
    -- 通常プレフィックス補完
    params = {
      mode   = "prefix",
      prefix = line:match("[%w_]+$") or "",
    }
  end

  vim.rpcrequest_async(process.job_id, "completion", params, function(err, result)
    if err or not result then return callback({ items = {}, isIncomplete = false }) end
    local items = vim.tbl_map(function(item)
      return {
        label         = item.label,
        kind          = item.kind,
        detail        = item.detail,
        documentation = item.documentation,
        insertText    = item.insert_text,
      }
    end, result.items or {})
    callback({ items = items, isIncomplete = false })
  end)
end
```

---

### プラグインのディレクトリ構造

```
asdb.nvim/
├── lua/
│   └── asdb/
│       ├── init.lua       ← setup() エントリーポイント
│       ├── detect.lua     ← ルート特定・タイプ判定
│       ├── config.lua     ← lang_config.json ロード・DLL パス解決
│       ├── process.lua    ← プロセス管理・RPC
│       ├── events.lua     ← オートコマンド登録
│       └── source.lua     ← blink-cmp ソースアダプタ
├── plugins/               ← 言語プラグインパッケージ
│   ├── asdb-plugin-unreal/
│   │   ├── tree-sitter-unreal-cpp.dll
│   │   ├── unreal-cpp.scm
│   │   ├── asdb-scanner-ue.dll
│   │   └── lang_config.json
│   └── asdb-plugin-generic-cpp/
│       ├── tree-sitter-cpp.dll
│       ├── generic-cpp.scm
│       └── lang_config.json
├── bin/                   ← プリコンパイル済みバイナリ同梱
│   ├── asdb          (Linux)
│   ├── asdb.exe      (Windows)
│   ├── tree-sitter-unreal-cpp.so / .dll / .dylib
│   └── asdb-scanner-ue.so / .dll / .dylib
└── queries/
    ├── unreal-cpp.scm
    └── generic-cpp.scm
```

> `bin/` に OS 別バイナリを同梱することで、ユーザーは Rust のビルド環境不要。  
> GitHub Releases から OS 別にダウンロードする仕組みを lazy.nvim の `build` フックで自動化予定。

---

## 12. Grammar Manager (`asdb-cli`)

`nvim-treesitter` の `:TSInstall` 相当の機能を **`asdb-cli`** バイナリとして提供。  
設定ファイル (`grammars.toml`) に言語 → Tree-sitter リポジトリ URL を書くだけで、  
DLL のダウンロード/ビルド/配置を自動化する。

---

### 設計思想

| 課題 | 解決策 |
|------|--------|
| ユーザーが DLL を手動ビルドするのは辛い | CLI ツールが全自動化 |
| コンパイラがない環境 | **プリビルド優先**: GitHub Releases から取得。失敗時のみビルドフォールバック |
| `tree-sitter generate` が必要か | `src/parser.c` が存在する場合は不要。なければ `tree-sitter` CLI に委譲 |
| クロスプラットフォーム | `.dll` / `.so` / `.dylib` を OS 検出して出力 |

---

### `grammars.toml` — ユーザー設定ファイル

場所: `~/.config/asdb/grammars.toml`  
(プロジェクト固有の上書き: `<project_root>/.asdb/grammars.toml` または `~/.config/asdb/<project_hash>/grammars.toml`)

```toml
# asdb grammar 管理設定

[grammars.cpp]
url     = "https://github.com/tree-sitter/tree-sitter-cpp"
rev     = "v0.22.0"     # タグ or コミットハッシュ or ブランチ名
prebuilt = true         # true: GitHub Releases から取得を優先

[grammars.unreal_cpp]
url     = "https://github.com/your-org/tree-sitter-unreal-cpp"
rev     = "main"
prebuilt = false        # ソースからビルド (プライベートリポジトリ等)

[grammars.csharp]
url     = "https://github.com/tree-sitter/tree-sitter-c-sharp"
rev     = "v0.21.3"
prebuilt = true

# インストール先 (省略時はデフォルト)
[settings]
install_dir = "~/.local/share/asdb/grammars"   # Linux/macOS
# install_dir = "%LOCALAPPDATA%/asdb/grammars"  # Windows
```

---

### `asdb-cli` コマンド一覧

```
asdb-cli grammar install <lang>       Grammar DLL をインストール
asdb-cli grammar install --all        grammars.toml の全言語をインストール
asdb-cli grammar update  <lang>       最新リビジョンに更新
asdb-cli grammar remove  <lang>       Grammar DLL を削除
asdb-cli grammar list                 インストール済み一覧 + バージョン表示
asdb-cli grammar check                grammars.toml の設定を検証 (URL疎通チェック等)
```

---

### インストールフロー

```
asdb-cli grammar install unreal_cpp
        │
        ├─ [1] grammars.toml を読む
        │       → url = "...", rev = "main", prebuilt = false
        │
        ├─ [2] prebuilt = true の場合:
        │       GitHub Releases API: GET /repos/<owner>/<repo>/releases/latest
        │       → アセット一覧から OS に対応する DLL を特定
        │           Linux:   tree-sitter-unreal-cpp-linux-x86_64.so
        │           Windows: tree-sitter-unreal-cpp-windows-x86_64.dll
        │           macOS:   tree-sitter-unreal-cpp-macos-aarch64.dylib
        │       → ダウンロード → SHA256 検証 → install_dir へ配置
        │       → 失敗時はビルドフォールバックへ
        │
        └─ [3] ビルドフォールバック (prebuilt = false or ダウンロード失敗):
                git clone --depth 1 --branch <rev> <url> → tempdir
                        │
                        ├─ src/parser.c が存在する? → YES: そのまま使用
                        │                              NO:  tree-sitter generate を実行
                        │
                        └─ C コンパイル (Rust の std::process::Command で cc/cl.exe 呼び出し)
                                gcc -O2 -shared -fPIC src/parser.c [src/scanner.c] \
                                    -o ~/.local/share/asdb/grammars/unreal_cpp/tree-sitter-unreal-cpp.so
                                → install_dir へ配置
                                → tempdir クリーンアップ
```

---

### インストール後のディレクトリ構造

```
~/.local/share/asdb/grammars/
├── cpp/
│   ├── tree-sitter-cpp.so         ← Grammar DLL
│   ├── grammar.meta               ← インストール情報 (url, rev, build_date, sha256)
│   └── queries/                   ← .scm ファイル (リポジトリの queries/ フォルダをコピー)
│       └── highlights.scm
├── unreal_cpp/
│   ├── tree-sitter-unreal-cpp.so
│   ├── grammar.meta
│   └── queries/
│       └── unreal-cpp.scm
└── csharp/
    ├── tree-sitter-c-sharp.so
    └── ...
```

**`grammar.meta` の内容:**

```toml
url        = "https://github.com/tree-sitter/tree-sitter-cpp"
rev        = "v0.22.0"
sha256     = "abc123..."
installed_at = "2026-05-20T14:32:00Z"
source     = "prebuilt"   # "prebuilt" | "built_from_source"
```

---

### `lang_config.json` との連携

Plugin Manager でインストールした DLL は `lang_config.json` の `scanners[].grammar_dll` / `scanner_dll` フィールドで参照できる。  
`"${ASDB_PLUGIN_DIR}"` 変数で install_dir への絶対パスを解決:

```json
{
  "core": {
    "scanners": [
      {
        "name":        "unreal_cpp",
        "extensions":  [".h", ".cpp", ".inl"],
        "grammar_dll": "${ASDB_PLUGIN_DIR}/grammar/unreal_cpp/tree-sitter-unreal-cpp.dll",
        "query_file":  "${ASDB_PLUGIN_DIR}/grammar/unreal_cpp/queries/unreal-cpp.scm"
      },
      {
        "name":        "unreal_assets",
        "extensions":  [".uasset", ".umap"],
        "grammar_dll": null,
        "scanner_dll": "${ASDB_PLUGIN_DIR}/scanner/unreal_assets/asdb-scanner-ue-assets.dll"
      }
    ]
  }
}
```

> `ASDB_PLUGIN_DIR` デフォルト: `~/.local/share/asdb/plugins`

---

### Rust 実装方針

```toml
# asdb-cli/Cargo.toml
[dependencies]
toml     = "0.8"
reqwest  = { version = "0.12", features = ["blocking", "json"] }
tempfile = "3"
sha2     = "0.10"
git2     = "0.19"
```

```
asdb/
├── Cargo.toml          (workspace)
├── asdb/          ← メイン Stdio サーバー
├── asdb-cli/           ← Plugin Manager ユーティリティ
└── asdb-lib/           ← 共有ライブラリ (DB, scanner, types)
```

---

## 13. 言語拡張仕様 (Plugin System Specification)

asdb に新しい言語・フォーマットのサポートを追加するための**公式仕様**。  
この仕様に従うだけで、asdb のコード変更なしに任意の言語を対応できる。

---

### 拡張モデル概要

```
言語拡張 = プラグインパッケージ (DLL + 設定ファイル)
        │
        ├─ テキスト言語 (C++, C#, Python...)
        │       Grammar DLL (tree-sitter C-ABI) + .scm クエリファイル
        │       オプション: PostScanner DLL (Rust C-ABI)
        │
        └─ バイナリフォーマット (uasset, umap, ...)
                Scanner DLL (Rust C-ABI) のみ
                Grammar DLL 不要
```

---

### `plugins.toml` — ユーザープラグインレジストリ

場所: `~/.config/asdb/plugins.toml`

```toml
# asdb プラグイン管理設定

# ────────────────────────────────────────────────
# type = "grammar"  : Tree-sitter Grammar DLL
#   ビルド手段: gcc -shared src/parser.c  (C コンパイラ)
#   フォールバック: C コンパイラがあれば自前ビルド可
# ────────────────────────────────────────────────
[plugins.unreal_cpp_grammar]
type     = "grammar"
url      = "https://github.com/your-org/tree-sitter-unreal-cpp"
rev      = "main"
prebuilt = true          # GitHub Releases から DL 優先

[plugins.cpp_grammar]
type     = "grammar"
url      = "https://github.com/tree-sitter/tree-sitter-cpp"
rev      = "v0.22.0"
prebuilt = true

# ────────────────────────────────────────────────
# type = "scanner"  : バイナリ/特殊解析 Scanner DLL (Rust 製)
#   ビルド手段: cargo build --release --lib
#   フォールバック: Rust ツールチェーンが必要。prebuilt 強く推奨
# ────────────────────────────────────────────────
[plugins.unreal_assets_scanner]
type     = "scanner"
url      = "https://github.com/your-org/asdb-scanner-ue-assets"
rev      = "v0.1.0"
prebuilt = true          # Rust ビルドは重いので prebuilt 必須推奨

[settings]
install_dir = "~/.local/share/asdb/plugins"
```

---

### インストール後のディレクトリ構造

```
~/.local/share/asdb/plugins/
├── grammar/
│   ├── unreal_cpp_grammar/
│   │   ├── tree-sitter-unreal-cpp.so   ← Grammar DLL
│   │   ├── plugin.meta                 ← インストール情報
│   │   └── queries/
│   │       └── unreal-cpp.scm
│   └── cpp_grammar/
│       ├── tree-sitter-cpp.so
│       └── queries/
└── scanner/
    └── unreal_assets_scanner/
        ├── asdb-scanner-ue-assets.so   ← Scanner DLL
        └── plugin.meta
```

**`plugin.meta` の内容 (TOML):**

```toml
name         = "unreal_assets_scanner"
type         = "scanner"
url          = "https://github.com/your-org/asdb-scanner-ue-assets"
rev          = "v0.1.0"
sha256       = "abc123..."
installed_at = "2026-05-20T14:50:00Z"
source       = "prebuilt"   # "prebuilt" | "built_from_source"
```

---

### `asdb-cli plugin` コマンド一覧

```
asdb-cli plugin install <name>        プラグインをインストール
asdb-cli plugin install --all         plugins.toml の全プラグインをインストール
asdb-cli plugin update  <name>        最新リビジョンに更新
asdb-cli plugin remove  <name>        プラグインを削除
asdb-cli plugin list                  インストール済み一覧 + type/rev 表示
asdb-cli plugin check                 plugins.toml の設定を検証
```

---

### インストールフロー (type 別)

```
asdb-cli plugin install <name>
        │
        ├─ plugins.toml から type, url, rev, prebuilt を読む
        │
        ├─ [共通 Step 1] prebuilt = true の場合:
        │       GitHub Releases API → OS/arch に対応するアセットを特定・DL
        │       → SHA256 検証 → install_dir/<type>/<name>/ に配置
        │       → 成功: 完了 ✅
        │       → 失敗: フォールバックへ
        │
        ├─ [grammar フォールバック]
        │       git clone → src/parser.c を確認 (なければ tree-sitter generate)
        │       → gcc -O2 -shared -fPIC src/parser.c [src/scanner.c] -o <name>.so
        │       → queries/ フォルダをコピー → 配置
        │
        └─ [scanner フォールバック]
                git clone → cargo build --release --lib
                Rust が見つからない場合:
                  → エラー終了 "Rust toolchain not found.
                               This plugin requires a prebuilt binary.
                               Wait for an official release or install Rust."
```

---

### `scanners` 配列 — `lang_config.json` の中核

`lang_config.json` の `core.scanners` が **ファイルタイプ → ハンドラ** のマッピング。  
Core はこの配列をイテレートして拡張子でルーティングする。

```json
{
  "core": {
    "source_dirs": ["Source", "Plugins"],
    "ignore_dirs": ["Binaries", "Intermediate", "Saved"],

    "scanners": [
      {
        "name":         "unreal_cpp",
        "extensions":   [".h", ".cpp", ".inl"],
        "grammar_dll":  "${ASDB_PLUGIN_DIR}/grammar/unreal_cpp_grammar/tree-sitter-unreal-cpp.dll",
        "query_file":   "${ASDB_PLUGIN_DIR}/grammar/unreal_cpp_grammar/queries/unreal-cpp.scm"
      },
      {
        "name":         "unreal_assets",
        "extensions":   [".uasset", ".umap"],
        "asset_dirs":   ["Content"],
        "grammar_dll":  null,
        "scanner_dll":  "${ASDB_PLUGIN_DIR}/scanner/unreal_assets_scanner/asdb-scanner-ue-assets.dll"
      }
    ],

    "sub_root_markers": [
      { "pattern": "*.Build.cs", "name_from": "stem" },
      { "pattern": "*.uplugin",  "name_from": "stem" }
    ]
  }
}
```

| フィールド | 必須 | 説明 |
|---|---|---|
| `name` | ✅ | スキャナー識別子 |
| `extensions` | ✅ | このスキャナーが処理するファイル拡張子 |
| `source_files_list` | — | 改行区切りファイルパス一覧の **テキストファイルパス** (`asdb-discover` が書き出す。指定時は `source_dirs` より優先) |
| `source_dirs` | — | スキャン対象ディレクトリ (`source_files_list` が空のときの glob フォールバック) |
| `grammar_dll` | — | Tree-sitter Grammar DLL パス (text パス) |
| `query_file` | — | `.scm` クエリファイルパス (`grammar_dll` がある場合必須) |
| `scanner_dll` | — | バイナリ/特殊スキャナー DLL パス (`grammar_dll` の代替) |

> `grammar_dll` と `scanner_dll` は**排他**。両方指定した場合はエラー。

---

### `ScanOutput` エンベロープ — DLL の統一返却形式

全 Scanner DLL (grammar post-scanner・binary scanner 共通) が返す JSON フォーマット:

```json
{
  "version": 1,

  "symbols": [
    {
      "name": "AMyCharacter", "symbol_type": "class",
      "line_start": 10, "line_end": 50,
      "base": "ACharacter",
      "members": [ ... ],
      "flags": { "uclass": true }
    }
  ],

  "assets": [
    {
      "asset_path":   "/Game/Characters/BP_MyCharacter",
      "asset_name":   "BP_MyCharacter",
      "parent_class": "/Script/Engine.Character",
      "functions":    ["/Game/Characters/BP_MyCharacter:Jump"],
      "imports":      ["/Script/Engine.Character"]
    }
  ]
}
```

| セクション | 書き込み先テーブル | 返す DLL |
|---|---|---|
| `symbols` | `symbols` / `members` / `inheritance` / `enum_values` | TextScanner (grammar_dll + .scm) |
| `assets` | `assets` / `asset_functions` / `asset_imports` | BinaryScanner (uasset 等) |

- **後方互換**: `"symbols"` だけ返す既存 DLL はそのまま動く
- **Core は中身を解釈しない**: セクション名でルーティングするだけ
- 将来的に `"file_includes"` 等のセクションを追加することで拡張可能

---

### 新言語追加の手順 (開発者向け)

1. Tree-sitter grammar を書くか既存のものを使う
2. `.scm` クエリファイルを作成 (DB マッピング規約に従う)
3. バイナリ対応が必要なら BinaryScanner DLL を実装 (`ScanOutput.assets` を返す)
4. `plugins.toml` にエントリを追加
5. `asdb-cli plugin install <name>` でインストール
6. `lang_config.json` の `scanners` 配列にエントリを追加
7. 完了 — asdb の変更は一切不要

---

### Scanner DLL 開発者向け要件

| 要件 | 詳細 |
|------|------|
| **スレッド安全性 (必須)** | `scan()` は rayon スレッドプールから並列呼び出しされる。内部にグローバル可変状態がある場合は `Mutex` 等で自前ロックすること |
| **`plugin_name` は静的文字列** | ヒープ確保した文字列は禁止。`"my_scanner"` のような文字列リテラルへのポインタのみ |
| **`deinit()` の実装 (必須)** | `asdb` は `dlclose()` 前に `deinit()` を呼ぶ。ここでヒープ解放・スレッド停止等を行うこと |
| **コールバック内でデータをコピー** | `scan()` はコールバック関数を受け取り、JSON 生成後にコールバックを呼ぶ。コールバックから戻り次第 json_ptr を解放してよい。コアはコールバック内で即コピーするため **アロケータの違い・CRT の違いは無問題** |
| **`ScanOutput` JSON のみ返す** | コールバックに渡す文字列は必ず有効な `ScanOutput` JSON であること。エラー時は `{"version":1,"symbols":[],"assets":[]}` を渡す |

---

## 14. 実装ロードマップ

### 全フェーズ概観

```
Phase 1: Rust スパイク ─── libloading + Tree-sitter DLL 動作確認
Phase 2: コア基盤 ──────── Stdio RPC + SQLite + 差分スキャン
Phase 3: クエリエンジン ── completion / search_symbols 等の実装
Phase 4: BinaryScanner ── uasset Scanner DLL 実装
Phase 5: Lua 薄皮 ──────── asdb.nvim + blink-cmp ソース
Phase 5.5: asdb-discover ─ プロジェクトファイル発見 CLI (別リポジトリ)
Phase 6: Grammar Manager ─ asdb-cli + grammars.toml ビルド自動化
Phase 7: バイナリ配布 ───── cargo-dist + CI/CD + lazy.nvim build フック
```

---

### Phase 1: Rust スパイク (抽象パース基盤確認)

**目的:** `libloading` + Tree-sitter C-ABI が実際に動くことを確認する。設計上の賭けをここで回収。

**Cargo.toml の主要依存:**

```toml
[dependencies]
libloading = "0.8"
tree-sitter = "0.23"
```

**タスク:**

1. `cargo new asdb --bin`
2. `libloading` で手元の `tree-sitter-cpp.dll`/`.so` を動的ロード
3. `ts_parser_new()` → `ts_parser_set_language()` → `ts_parser_parse_string()` の呼び出しを確認
4. `.scm` クエリ文字列をハードコードして `ts_query_new()` → `ts_query_cursor_exec()` でノード抽出
5. キャプチャ名 (`@symbol.name` 等) が取れることを確認

**完了条件:** `"class Foo {};"` という文字列をパースして `Foo` という名前が取れること。

---

### Phase 2: コア基盤

**目的:** Stdio RPC + SQLite + 差分スキャンの骨格を作る。

**追加依存:**

```toml
serde        = { version = "1", features = ["derive"] }
serde_json   = "1"
rmp-serde    = "1"          # MessagePack-RPC
rusqlite     = { version = "0.31", features = ["bundled"] }
tokio        = { version = "1", features = ["full"] }
rayon        = "1"          # 並列スキャン
tracing      = "0.1"
tracing-subscriber = "0.3"
```

**タスク:**

1. **Transport 層**
   - `--protocol json` / `--protocol msgpack` の起動フラグ解析
   - `Transport` trait + `JsonTransport` / `MsgpackTransport` の実装
   - `stdin` から改行区切り (JSON) または自己区切り (msgpack) で読み込むイベントループ

2. **`initialize` ハンドラ**
   - `ActiveProject` 構造体をメモリ上に生成
   - XDG キャッシュパス解決 + SQLite DB 初期化 (スキーマ適用)
   - `project_meta` への設定情報書き込み

3. **フルスキャン**
   - `source_dirs` を `ignore_dirs` を除外しながら `walkdir` で列挙
   - `rayon` スレッドプールで並列パース → channel 経由で DB ライタースレッドへ
   - ファイル単位トランザクションで `files` / `symbols` / `members` に INSERT
   - 完了後 `scan_complete` 通知

4. **差分スキャン**
   - 起動時に `project_meta` の `query_file_hash` / `parser_dll_hash` を比較
   - `files` テーブルの mtime と照合して 追加/変更/削除 を分類
   - 変更分のみ再スキャン

5. **`file_changed` ハンドラ**
   - `DELETE FROM files WHERE path = ?` (CASCADE)
   - 再パース → INSERT

**完了条件:** `initialize` 送ったら XDG キャッシュ配下に `<sha256>.db` が生成され、シンボルが入っていること。

---

### Phase 3: クエリエンジン

**目的:** `completion` / `search_symbols` / `get_members` 等の主要クエリを実装。

**タスク:**

1. **`completion`**
   - `prefix` で `symbols_fts` を前方一致検索 (`symbol MATCH 'Foo*'`)
   - `workspace_id` (= root_path) でスコープ隔離
   - LSP CompletionItem 番号 (`kind`) を asdb 側で決定して返す

2. **`search_symbols`**
   - FTS5 + LIKE のハイブリッド検索
   - `symbol_type` / `namespace` フィルタ対応

3. **`get_members`**
   - `symbol_id` から `members` テーブルを引く
   - `access` フィルタ (public/protected/private) 対応

4. **`get_inheritance`**
   - `inheritance` テーブルを BFS で辿り継承ツリーを返す

5. **`goto_definition`**
   - `symbols` / `members` から `file_id` + `line_start` を返す

6. **`ping`** (疎通確認用、最初に実装して動作検証に使う)

**完了条件:** Neovim から直接 `vim.rpcrequest` で叩いてシンボルが返ってくること。

---

### Phase 4: BinaryScanner (uasset Scanner DLL)

**目的:** `.uasset` / `.umap` バイナリの解析 → `assets` / `asset_functions` / `asset_imports` テーブルへの格納。

**タスク:**

1. `asdb-scanner-ue-assets` crate を作成 (`cdylib` ターゲット)
2. `ue_asset_parser` モジュール実装 (uasset バイナリフォーマット解析):
   - `FPackageFileSummary` ヘッダー読み取り (magic: `0x9E2A83C1`)
   - `NameMap` → `ExportMap` → `ImportMap` の順に解析
   - Export エントリから `asset_name`, `parent_class`, Blueprint 関数名を抽出
   - Import エントリから `/Script/Engine.Character` 等の依存パスを抽出
3. `ScanOutput` の `assets` セクションに変換して返す
4. UE4 vs UE5 の `legacy_ver` 分岐対応:
   - `legacy_ver >= -7`: UE4 系フォーマット
   - `legacy_ver == -9`: UE5.6+ 新フォーマット (`VersioningInfo` ブロック追加)

**`assets.parent_id` 解決タイミング:**  
`parent_class` フィールドはスキャン時点では raw 文字列 (`/Script/Engine.Character`)。  
`source_dirs` スキャン完了後に後処理バッチで解決する:

```sql
-- ソーススキャン完了後に一括解決
UPDATE assets
SET parent_id = (
    SELECT s.id FROM symbols s
    JOIN strings st ON s.name_id = st.id
    WHERE st.text = assets.parent_class_raw
)
WHERE parent_id IS NULL;
```

**完了条件:** `search_assets("BP_My")` で Blueprint クラスと `parent_class` が返ること。

---

### Phase 5: Lua 薄皮 (asdb.nvim)

**目的:** Neovim から実際に使えるプラグインにする。

**タスク:**

1. `detect.lua` — `vim.fs.root` でプロジェクトルート特定
2. `process.lua` — `jobstart({rpc=true})` でプロセス起動・`initialize` 送信
3. `events.lua` — `BufWritePost` autocmd で `file_changed` 通知
4. `source.lua` — blink-cmp ソースアダプタ登録
5. `init.lua` — `require("asdb").setup({})` インターフェース
6. スキャン中インジケータ (`scan_progress` 通知を受けて lualine 等に表示)

**完了条件:** UE プロジェクトを開いて補完が動くこと。

---

### Phase 6: Grammar Manager (`asdb-cli`)

**目的:** ユーザーが `grammars.toml` を書くだけで Grammar DLL を自動取得・ビルドできるようにする。

**タスク:**

1. `asdb-cli/` crate を workspace に追加 (`asdb`, `asdb-cli`, `asdb-lib` の3構成)
2. `grammars.toml` パーサー実装 (`toml` crate)
3. `grammar install` — プリビルド DL (reqwest) + ビルドフォールバック (cc/gcc 呼び出し)
4. `grammar list` / `grammar update` / `grammar remove` 実装
5. `grammar.meta` (TOML) の書き込み・読み込み (url, rev, sha256, installed_at)
6. `lang_config.json` の `${ASDB_GRAMMAR_DIR}` 変数解決を `asdb` / `config.lua` に追加

**完了条件:** `asdb-cli grammar install unreal_cpp` 1コマンドで DLL が `~/.local/share/asdb/grammars/` に配置されること。

---

### Phase 7: バイナリ配布 + CI/CD

**目的:** ユーザーが Rust/C コンパイラ環境なしで使えるようにする。

**戦略: `cargo-dist` + GitHub Actions ネイティブマトリックスビルド**

> クロスコンパイルより**各 OS のネイティブランナーで直接ビルド**する方が確実。  
> `rusqlite` (bundled SQLite) や Grammar DLL の C コードがあるため、クロスコンパイルは不安定。

**セットアップ:**

```bash
cargo install cargo-dist
cargo dist init   # dist.toml 生成 + GitHub Actions YAML 自動生成
```

```toml
# Cargo.toml [workspace.metadata.dist]
[workspace.metadata.dist]
cargo-dist-version = "0.22"
ci = ["github"]
targets = [
  "x86_64-unknown-linux-gnu",
  "x86_64-pc-windows-msvc",
  "aarch64-apple-darwin",   # Apple Silicon
  "x86_64-apple-darwin",    # Intel Mac
]
installers = []   # lazy.nvim 側で独自 DL するため不要
```

**タスク:**

1. `cargo dist init` で GitHub Actions ワークフロー自動生成
2. タグ push (`v*`) → 4プラットフォーム並列ビルド → GitHub Releases に自動アップロード
3. 成果物: `asdb` + `asdb-cli` + `dist-manifest.json` (全アセット URL + SHA256 入り)
4. 公式 Grammar DLL (`asdb-plugin-unreal` 等) も同一マトリックスで C コンパイル:

```yaml
# Grammar DLL を各 OS ネイティブでビルド
- name: Build Grammar DLL (Linux)
  if: matrix.os == 'ubuntu-latest'
  run: |
    gcc -O2 -shared -fPIC \
      tree-sitter-unreal-cpp/src/parser.c \
      -o tree-sitter-unreal-cpp-linux-x86_64.so
```

5. `asdb.nvim` の `install.lua` — `dist-manifest.json` を読んで OS/arch に対応するアセットを自動 DL:

```lua
-- lazy.nvim spec
{
  "your-org/asdb.nvim",
  build = function()
    require("asdb.install").fetch()
    -- dist-manifest.json 取得 → OS/arch 検出 → 対応バイナリ DL → SHA256 検証 → bin/ 配置
  end,
}
```

6. `db_version` 変更時の自動マイグレーション or 再スキャン通知

**完了条件:** `lazy.nvim` で install するだけで全プラットフォームで動くこと。

**全体リリースフロー:**

```
git tag v0.2.0 && git push --tags
        ↓
cargo-dist が自動生成した GitHub Actions が起動
        ↓
ubuntu / windows / macos-arm / macos-intel で並列ビルド
        ↓
GitHub Releases に asdb, asdb-cli, Grammar DLL, dist-manifest.json をアップロード
        ↓
ユーザー: lazy.nvim install → build フックが manifest を読んで自動 DL + SHA256 検証
```

---

---

## 16. VCS 連携設計

### 設計原則

> **VCS 固有ロジックはすべて VcsAdapter DLL に委譲する。**  
> Core は DLL が返す **opaque token** と **変更ファイルリスト** だけを受け取り、スキャン戦略を決定する。  
> Git/SVN/P4 の内部ファイルフォーマットや CLI コマンドは Core に一行も入らない。  
> **未コミット変更は既存の `file_changed` RPC が担う → VcsAdapter は commit/sync 操作のみを追う。**

| 役割 | 担当 |
|---|---|
| opaque token 取得・変更ファイル列挙・VCS 状態監視 | VcsAdapter DLL |
| token 比較・スキャン戦略決定・reconcile/full/incremental 実行 | Core |
| `vcs_adapter.dll` パスを `lang_config.json` に記述 | Plugin |
| VCS 変化の通知 (オプション: DLL が直接監視するので不要な場合が多い) | Plugin (thin) |

---

### VcsAdapter DLL — C-ABI インターフェース

Scanner DLL と同じ **callback + コピー** パターン。Core がバッファを所有しないため Windows CRT 問題が発生しない。

```c
// ─────────────────────────────────────────────
// Callback 型定義
// ─────────────────────────────────────────────
// callback に渡された char* は呼び出し中のみ有効 (Core は即座にコピーすること)
// encoding: UTF-8, len: byte 数, NUL 終端に依存しない
typedef void* VcsWatchHandle;
typedef void (*VcsRefCallback)    (const char* ref,  size_t len, void* ud);
typedef void (*VcsChangesCallback)(const char* json, size_t len, void* ud);
typedef void (*VcsWatchCallback)  (const char* new_ref, size_t len, void* ud);

// ─────────────────────────────────────────────
// 必須エクスポート
// ─────────────────────────────────────────────

// ABI バージョン (Core はロード直後にチェックし、非対応なら DLL を拒否する)
uint32_t vcs_abi_version(void);     // 現バージョン: 1

// DLL メタデータ + capabilities (JSON 文字列)
// {
//   "name":                    "git",        // "git"|"svn"|"p4"
//   "version":                 "0.1.0",
//   "supports_watch":          true,         // vcs_start_watch が実装されているか
//   "recommended_poll_ms":     0,            // 0 = poll 不要 (watch 使用), >0 = poll 推奨間隔
//   "cheap_current_ref":       true          // vcs_get_current_ref が軽量か (常時ポーリング可否)
// }
void vcs_capabilities(VcsRefCallback cb, void* ud);

// 現在の VCS state token を取得
// token は commit/sync 操作を追う opaque 文字列 (SHA/revision/CL番号)
// 失敗時は空文字列を返す
void vcs_get_current_ref(const char* root, VcsRefCallback cb, void* ud);

// 2つの token 間の変更ファイルリストを取得
// JSON 返却形式:
// {
//   "ok":         true,
//   "confidence": "complete|partial|unknown",  // complete = 全変更を正確に列挙
//   "basis":      "commit_range|sync_range|unknown",
//   "changes": [
//     { "path": "Source/Foo.h",     "status": "modified"                       },
//     { "path": "Source/New.cpp",   "status": "added"                          },
//     { "path": "Source/Del.h",     "status": "deleted"                        },
//     { "path": "Source/New2.h",    "status": "renamed", "old_path": "Old.h"   }
//   ]
// }
// path は project root からの相対パス, '/' 区切り, UTF-8
// 失敗時: { "ok": false, "error_kind": "timeout|auth|not_repo|unsupported|io", "message": "..." }
void vcs_get_changed_files(
    const char* root,
    const char* old_ref,
    const char* new_ref,
    VcsChangesCallback cb,
    void* ud
);

// ─────────────────────────────────────────────
// オプションエクスポート (supports_watch: true の DLL のみ)
// ─────────────────────────────────────────────

// VCS 状態ファイルを監視するバックグラウンドスレッドを起動
// 変化検知時に new_ref を引数として callback を呼ぶ (内部スレッドから呼ばれる)
// 戻り値: ハンドル (失敗時 NULL)
VcsWatchHandle vcs_start_watch(const char* root, VcsWatchCallback cb, void* ud);

// 監視スレッドを停止 (内部スレッドを join するまでブロック)
// この関数が返った後、callback は絶対に呼ばれない (ABI 契約)
// NULL 渡し・二重呼び出しは安全に無視する
// ⚠️ Core は vcs_stop_watch 完了前に DLL を unload してはいけない
void vcs_stop_watch(VcsWatchHandle handle);
```

---

### Core 側の VcsAdapter ローダー設計 (Rust)

```rust
// callback は DLL の内部スレッドから呼ばれるため、
// Tokio ランタイムに直接触れず mpsc channel に投げるだけにする
let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

let tx_clone = tx.clone();
// callback: new_ref を channel に送るだけ
let watch_handle = unsafe {
    vcs_start_watch(root_cstr.as_ptr(), |ref_ptr, len, _ud| {
        let _ = std::panic::catch_unwind(|| {          // FFI boundary: パニック防護
            // ⚠️ token サイズ上限: 256 バイトを超えるものは粗悪 DLL からの異常値として無視
            if len > 256 { return; }
            let new_ref = std::str::from_utf8(
                std::slice::from_raw_parts(ref_ptr as *const u8, len)
            ).unwrap_or("").to_string();
            let _ = tx_clone.send(new_ref);
        });
    }, std::ptr::null_mut())
};

// Tokio タスクで channel を受け取り、スキャン処理を実行
tokio::spawn(async move {
    while let Some(new_ref) = rx.recv().await {
        // vcs_get_changed_files は blocking → spawn_blocking に移譲
        let changes = tokio::task::spawn_blocking(move || {
            get_changed_files_sync(&root, &last_ref, &new_ref)
        }).await?;

        server.trigger_scan(changes).await;
    }
});
```

> `vcs_get_changed_files` は P4/HTTP など時間のかかる場合があるため、必ず `spawn_blocking` で呼ぶ。  
> タイムアウト (デフォルト 30 秒) を設け、超過時は `confidence: unknown` として reconcile scan に落ちる。

**シャットダウン時の `vcs_stop_watch` タイムアウト:**

```rust
// vcs_stop_watch は "内部スレッドを join するまでブロック" の契約だが、
// ネットワーク I/O でスタックする SVN/P4 DLL がハングしてシャットダウン自体が止まる可能性がある
// → tokio::time::timeout でラップし、3秒超過したらリソースリークを受け入れて強制終了
async fn shutdown_vcs_watch(watch_handle: VcsWatchHandle) {
    let result = tokio::time::timeout(
        Duration::from_secs(3),
        tokio::task::spawn_blocking(move || {
            unsafe { vcs_stop_watch(watch_handle) };
        }),
    ).await;

    if result.is_err() {
        // タイムアウト: DLL スレッドのハングを検出。
        // vcs_stop_watch が戻らないため DLL のメモリ解放は諦め、
        // プロセスを強制終了して OS にクリーンアップを委ねる。
        log::error!("vcs_stop_watch timed out — forcing process exit");
        std::process::exit(1);
    }
}
```

---

### `lang_config.json` の `vcs_adapter` フィールド

```json
{
  "core": {
    "scanners":    [ ... ],
    "vcs_adapter": {
      "dll": "${PLUGIN_DIR}/asdb-vcs-git.dll"
    }
  }
}
```

> `vcs_adapter` を省略するか DLL が見つからない場合は **mtime + reconcile scan** のみで動作する  
> (VCS なし / VCS 不明プロジェクトでも正常動作することを保証)

---

### 起動時スキャン判定フロー

```
initialize 受信
    │
    ▼
vcs_adapter.dll をロード → vcs_abi_version() 確認
    │
    ├── ABI 不一致 → DLL 拒否。VCS なし扱いで継続
    │
    ▼
vcs_get_current_ref(root) → current_token (fast path)
    │
    ├── project_meta["last_vcs_token"] == current_token
    │     → token 変化なし → reconcile scan (起動中の変化を捕捉)
    │
    ├── project_meta["last_vcs_token"] != current_token
    │     → vcs_get_changed_files(old, new) [spawn_blocking]
    │          ├── ok + confidence=complete → incremental scan
    │          ├── ok + confidence=partial  → incremental + reconcile
    │          └── ok=false / unknown       → full scan
    │
    └── last_vcs_token なし (初回) → full scan
    │
    ▼
スキャン完了後:
  UPDATE project_meta SET last_vcs_token = current_token
  UPDATE project_meta SET committed_scan_generation = current_scan_generation
    │
    ▼
vcs_start_watch(root, callback) を起動 (supports_watch=true の場合)
supports_watch=false の場合:
  capabilities["recommended_poll_ms"] 間隔で vcs_get_current_ref をポーリング
```

---

### VCS 別の動作サマリー

| VCS | token 例 | supports_watch | 変更ファイル取得 | confidence |
|---|---|---|---|---|
| **Git** | `"a3f2b1c4"` (HEAD SHA) | ✅ (`.git/refs/` 監視) | `git diff --name-status` | complete |
| **SVN** | `"1234"` (revision) | △ (`.svn/wc.db` polling) | `svn diff --summarize` | complete |
| **P4 (sync wrapper)** | `"12345"` (CL番号) | ❌ | `p4 describe <cl>` | complete |
| **P4 (通常)** | `"12345"` | ❌ | 困難 | unknown → full scan |
| **なし / 不明** | `""` | ❌ | なし | — (reconcile のみ) |

> **P4 の運用方針:** P4 DLL が `confidence: unknown` を返す場合は Core が自動で reconcile/full scan に落ちる。  
> P4 sync を DLL が直接 wrap できる環境では `confidence: complete` が返せる。

---

### reconcile scan とは

```
DB の既知ファイル一覧 vs 実ファイル一覧 を照合:

  ① DB にあるが実ファイルが消えた  → DB から DELETE
  ② 実ファイルにあるが DB にない   → パース → INSERT
  ③ mtime/size が変化している      → 旧シンボル DELETE → 再パース → INSERT
  ④ 変化なし                       → スキップ

フルスキャンとの違い:
  ・変化のないファイルはパースをスキップ → CPU 負荷が大幅に低い
  ・mtime だけに頼らず削除ファイルも確実に掃除できる
  ・vcs_sync 通知が来なかった場合 (起動時 Core 停止中の変化) の安全網として機能
```

---

### `project_meta` 追加キー

| key | value 例 | 説明 |
|---|---|---|
| `vcs_type` | `"git"` | capabilities から取得した VCS 名 |
| `last_vcs_token` | `"a3f2b1c4..."` | スキャン完了時に記録した token |

`last_vcs_token` は **`committed_scan_generation` 更新と同じタイミング** で書き込む。  
スキャン中断時は古い token のまま → 次回起動で再スキャンを保証。

---

### プラグイン (Lua) 側の実装量

VcsAdapter DLL がファイル監視とコマンド実行を担うため、プラグイン側は最小限。

```lua
-- lang_config.json に vcs_adapter.dll を指定するだけで VCS 連携が完成する
-- プラグインが VCS コマンドを実行する必要はない

-- オプション: ユーザーが手動で強制リフレッシュしたい場合の窓口だけ用意
vim.api.nvim_create_user_command("AsdbRefresh", function(opts)
  local force = opts.bang
  client:notify("vcs_sync", {
    new_token      = "",            -- Core が DLL から取得する
    changed_files  = {},
    scan_mode      = force and "full" or "reconcile",
    confidence     = "unknown",
  })
end, { bang = true, desc = "Refresh asdb index (! = full scan)" })
```

> `vcs_sync` RPC は引き続き存在するが、通常運用では Plugin から送る必要はない。  
> DLL の `vcs_start_watch` / ポーリングが自動で変化を検知し Core を動かす。

| 役割 | 担当 |
|---|---|
| VCS タイプ自動検出 (ファイル存在チェックのみ) | Core |
| `last_vcs_token` の保存・比較 | Core |
| 起動時・接続時の reconcile scan | Core |
| VCS コマンド実行・token 取得・変更ファイル列挙 | Plugin |
| ブランチ切り替え/rebase/sync 後の通知 | Plugin |

---

### 対応 VCS と token 定義

| VCS | token の例 | plugin が取得する方法 |
|---|---|---|
| **Git** | `"a3f2b1c4"` (HEAD SHA 40桁) | `git rev-parse HEAD` |
| **SVN** | `"1234"` (リビジョン番号) | `svnversion .` |
| **Perforce (P4)** | `"12345"` (changelist 番号) | `p4 changes -m1 //...@have` |
| **None** | `""` | なし (mtime/reconcile のみ) |

Core は token の中身を解釈しない。**「前回と違う文字列か？」だけを判定する。**

---

### 新規 RPC: `vcs_sync`

Plugin が VCS 変化を検知したときに送る通知。

```json
{
  "jsonrpc": "2.0",
  "method": "vcs_sync",
  "params": {
    "new_token": "a3f2b1c4d5e6...",
    "changed_files": [
      { "path": "/abs/path/to/file1.h",   "status": "modified" },
      { "path": "/abs/path/to/file2.cpp",  "status": "added"    },
      { "path": "/abs/path/to/old.h",      "status": "deleted"  },
      { "path": "/abs/path/to/new.h",      "old_path": "/abs/path/to/renamed.h", "status": "renamed" }
    ],
    "scan_mode":  "incremental",  // "incremental" | "reconcile" | "full"
    "confidence": "complete"      // "complete" | "partial" | "unknown"
  }
}
```

**`changed_files.status` 値:**

| status | DB への処置 |
|---|---|
| `modified` | 旧シンボルを DELETE → 再パース → INSERT |
| `added` | パース → INSERT |
| `deleted` | 旧シンボルを DELETE (再パースなし) |
| `renamed` | `old_path` のシンボルを `path` に UPDATE + 再パース |

**`scan_mode` / `confidence` の組み合わせ:**

| confidence | changed_files | Core の動作 |
|---|---|---|
| `complete` | あり | `incremental`: changed_files のみ再スキャン |
| `partial`  | あり | `incremental` + その後 reconcile |
| `unknown`  | なし | `reconcile` (全ファイルの mtime/size 照合) |
| (任意)     | なし | `reconcile` |
| (任意)     | `force_full_scan` 相当 | `full`: スキャン強制 |

> **reconcile とは:** DB に記録されているファイル一覧と実ファイル一覧を照合し、  
> 消えたファイルのシンボルを DELETE、新規ファイルを追加スキャン、mtime/size が変化したファイルを再スキャンする処理。  
> フルスキャンより軽量だが mtime 差分スキャンより信頼性が高い。

---

### VCS タイプ自動検出 (Core)

```rust
fn detect_vcs(root: &Path) -> VcsType {
    if root.join(".git").exists()                  { VcsType::Git }
    else if root.join(".svn").exists()             { VcsType::Svn }
    else if root.join(".p4config").exists()
         || root.join(".p4ignore").exists()        { VcsType::P4  }
    else                                           { VcsType::None }
}
// ⚠️ Core は .git/HEAD や packed-refs などの VCS 内部ファイルを読まない
// ⚠️ git/svn/p4 のサブプロセスを Core が起動しない
// ⚠️ detect_vcs の結果は initialize レスポンスに含め、Plugin がコマンド選択に使う
```

**`initialize` レスポンスへの追加フィールド:**

```json
{
  "status":      "scanning",
  "vcs_type":    "git",          // ← 追加: "git" | "svn" | "p4" | "none"
  "vcs_token":   "a3f2b1c4...",  // ← 追加: 前回記録した last_vcs_token (plugin の参照用)
  "client_id":   "client-001",
  ...
}
```

Plugin は `vcs_type` を見て使うコマンドを決定し、`vcs_token` を見て前回から変化があったか判断する。

---

### 起動時スキャン判定フロー

```
initialize 受信
    │
    ▼
detect_vcs(root) → vcs_type を project_meta に保存
    │
    ▼
project_meta["last_vcs_token"] を読む
    │
    ├── なし (初回) → フルスキャン → last_vcs_token = ""
    │
    └── あり → reconcile scan を実行 (ファイル追加・削除・mtime変化を捕捉)
                 ※ vcs_sync が来れば incremental に切り替わる
```

> **なぜ起動時は reconcile か?**  
> Core 停止中に `git pull` / `svn update` / `p4 sync` が走り、  
> vcs_sync 通知が飛んでこなかったケース (別エディタが担当していた等) に対応するため。  
> mtime 差分だけでは削除ファイルの掃除ができない。

---

### Plugin 側実装 (Neovim Lua)

Plugin 側は「VCS コマンドを叩いて `vcs_sync` を送るだけ」の薄い実装で済む。

```lua
-- git: commit / pull / rebase / branch switch 後に発火
local function on_git_change(old_token, new_token)
  -- git diff --name-status <old>..<new> で変更ファイル列挙
  local ok, changes = pcall(git_diff_name_status, old_token, new_token)
  if ok then
    client:notify("vcs_sync", {
      new_token    = new_token,
      changed_files = changes,   -- { path, status, old_path? }
      scan_mode    = "incremental",
      confidence   = "complete",
    })
  else
    -- rebase / force-push 等で old_token が見つからない場合
    client:notify("vcs_sync", {
      new_token    = new_token,
      changed_files = {},
      scan_mode    = "full",
      confidence   = "unknown",
    })
  end
end

-- HEAD の変化を検知 (polling または uv.fs_event で .git/refs を監視)
local function poll_git_head()
  local new_token = vim.fn.system("git rev-parse HEAD"):gsub("%s+$", "")
  if new_token ~= cached_token then
    on_git_change(cached_token, new_token)
    cached_token = new_token
  end
end
```

**VCS 別の実装量目安:**

| VCS | Plugin 実装規模 | changed_files 取得コマンド |
|---|---|---|
| Git | ~50行 | `git diff --name-status <old>..<new>` |
| SVN | ~40行 | `svn diff --summarize -r<old>:<new>` |
| P4 (sync wrapper) | ~60行 | `p4 describe <cl>` or `p4 sync -n` preview |
| P4 (通知なし) | 不要 | confidence=unknown → reconcile |

> **P4 の注意点:** P4 はローカルファイルだけで完全な差分を得にくい。  
> Plugin が `p4 sync` を wrap する形でのみ `confidence: complete` になる。  
> それ以外は `confidence: unknown` として Core が reconcile/full scan に落ちる運用を想定。

---

### `project_meta` 追加キー

| key | value 例 | 説明 |
|---|---|---|
| `vcs_type` | `"git"` | 自動検出した VCS タイプ |
| `last_vcs_token` | `"a3f2b1c4..."` | 最後に正常完了したスキャン時の token |

`last_vcs_token` は **スキャン完了後** に更新する  
(`committed_scan_generation` と同じタイミング)。  
スキャン中断時は古い token のまま → 次回起動で再スキャンを保証。

---

### マルチエディタ環境での安全性

複数エディタが同一 Core プロセスに接続している場合、どれかのエディタが `vcs_sync` を送らなかったとしても、**Core の起動時 reconcile scan** が補完する。

```
エディタA (nvim)  ─── git pull → vcs_sync 送信 ───► Core がスキャン
エディタB (VSCode) ──接続時→ initialize 受信 → last_vcs_token 照合済み → 追加スキャン不要
エディタC (nvim2)  ──接続時→ last_vcs_token が latest token と一致 → スキャンスキップ
```

> エディタAが切断した後でエディタBが接続してきた場合も、  
> Core は既にスキャン完了済みなので再スキャン不要。

- [x] `.scm` クエリ → DB マッピングの完全仕様 (フェーズ3で詳細化)
- [x] 書き込み権限がない場合のフォールバックパス → §10 スキャンフロー ステップ0・§3・§5 initialize レスポンス更新済み (`DbMode::Persistent/Temp/Memory` 3段フォールバック、Lua 警告通知付き)
- [x] `ignore_dirs_override` フィールドの必要性検討 → 正式サポート決定。`ignore_dirs_override: true` でデフォルトを完全上書き可能 (§10 ignore_dirs 節更新済み)
- [x] `find_usages_async` などのストリーミングレスポンス設計 → §4 ストリーミング共通プロトコル・§5 `find_usages` spec 拡張済み (`chunk_size` 設定可)
- [x] ログファイルの配置場所 → `~/.cache/asdb/logs/<sha256>.log`、3世代ローテーション、`log_level` 設定可 (§10 エラーハンドリング後に追加済み)
- [x] 複数エディタウィンドウが同一プロジェクトを開いた場合の DB ロック戦略 → WAL モード + `busy_timeout` 5秒で解決済み
- [x] `assets` テーブルの `parent_id` 解決タイミング → §14 Phase 4 に記載 (ソーススキャン完了後 UPDATE)
- [x] アセットスキャンとソーススキャンの並列実行戦略 → §10 スキャンフロー更新済み (text_pool/binary_pool 分離、両プール同時スタート、text_pool 完了後に deferred UPDATE)
- [x] `grep_assets` の mmap unsafe 使用についての安全性レビュー → mmap 不使用に決定 (SIGBUS/ACCESS_VIOLATION 回避)。BufReader 読み込み + FTS5/LIKE 検索を優先 (§5 grep_assets 節更新済み)
- [ ] INI/Config 解析: `DefaultGame.ini` 等 (現状スコープ外。需要があれば別 BinaryScanner として追加)
- [x] `initialize` params を `scanners` 配列形式に統一 → §5 修正済み
- [x] `resolve_type` 内部実装方針 → §10 に記載 (Phase 3: DB only / Phase 4+: buffer tree-sitter)
- [x] `file_opened` + 未保存バッファの扱い → §10 に記載 (transient_trees)
- [x] DB ライターの文字列インターニング手順 → §10 に記載 (StringIntern キャッシュ)
- [x] uasset BinaryScanner 設計 → §14 Phase 4 に記載 (uasset フォーマット解析 + UE4/UE5 分岐)
- [x] Phase 4 `resolve_type` バッファ内 tree-sitter クエリ用の `.scm` 定義 (変数宣言キャプチャ) → §10 resolve_type 設計に `resolve_type_cpp.scm` 追加済み (パターンA/B/C、auto は Phase 5 スコープ)
- [x] `transient_trees` のサイズ上限値と GC 戦略 → §10 に LRU エビクション設計追加済み (デフォルト 50MB / 50ファイル、単一ファイル上限超えはスキップ)
- [x] `mtime` → `mtime_ms` (ミリ秒精度) → §6 `files` テーブル・§10 差分スキャンフロー修正済み
- [x] `extern "C" fn on_result` に `catch_unwind` 追加 → §8 修正済み (パニック UB 防止)
- [x] レスポンス直列化を DOM 中間変換なし (`T: Serialize` → `rmp_serde` / `serde_json` 直書き) → §4 修正済み
- [x] `source_files: []` (RPCで巨大配列を送る設計) → `source_files_list` (テキストファイルパスを渡す設計) に変更 → §8・§17 修正済み

---

## 17. プロジェクトファイル発見設計 (asdb-discover)

### 責務の分離

| コンポーネント | 責務 |
|---|---|
| `asdb` (Core) | 渡されたファイルリストを index して検索・補完を返す |
| `asdb-discover` | プロジェクトファイル (Cargo.toml / .sln 等) を解析して **ファイルリストをテキストファイルに書き出す** |
| `asdb.nvim` / `asdb-vscode` | `asdb-discover` を呼んでテキストファイルパスを `initialize` RPC に乗せる |

> **Core はプロジェクトファイル形式を一切知らない。**  
> `asdb-discover` は `asdb` と **別リポジトリ** として配布される独立 CLI ツール。  
> 「ファイルのリストアップ」という泥臭いビルドシステム固有の処理を完全に切り出すことで、  
> Core は「渡されたリストをただ無心で最速パースするだけのピュアな筋肉」であり続ける。

---

### なぜ RPC で配列を送らないのか (source_files_list の設計根拠)

UE プロジェクトは 5 万ファイルを超えることがある。  
`source_files: ["/path/1.cpp", "/path/2.cpp", ...]` という JSON 配列は **数 MB〜数十 MB** に膨れ上がる。

| 問題 | 影響 |
|---|---|
| Lua 側の巨大テーブルを MessagePack にシリアライズ | エディタが一瞬フリーズ (Jank) |
| Rust 側で数 MB の JSON を一括デシリアライズ | 起動直後に不要な CPU スパイク + 大量アロケーション |

**解決策: テキストファイル経由のリスト渡し**

```json
{
  "scanners": [
    {
      "name": "unreal_cpp",
      "extensions": [".h", ".cpp"],
      "source_files_list": "/home/user/.cache/asdb/tmp/abc123_files.txt",
      "grammar_dll": "..."
    }
  ]
}
```

- RPC ペイロードは常に数キロバイト以内 → エディタ側ゼロ負荷
- Core は `BufReader` で 1 行ずつ読み込みながら `rayon` のキューに流せる → メモリ効率最大化

---

### asdb-discover コマンド仕様

```bash
# compile_commands.json から C++ ソースファイルリスト
asdb-discover --type compile-commands build/compile_commands.json

# Cargo.toml からワークスペース全 Rust ソース
asdb-discover --type cargo Cargo.toml

# .sln から C# ソースファイルリスト
asdb-discover --type msbuild MyGame.sln

# .vcxproj 直接指定
asdb-discover --type vcxproj MyGame.vcxproj

# go.mod から Go ソース
asdb-discover --type go-mod go.mod

# *.uproject (UE) からソースファイルリスト
asdb-discover --type uproject MyGame.uproject
```

**出力: テキストファイルに改行区切りで書き出し、パスを標準出力に返す**

```bash
$ asdb-discover --type cargo Cargo.toml
/home/user/.cache/asdb/tmp/f3a2b1c4_files.txt

$ cat /home/user/.cache/asdb/tmp/f3a2b1c4_files.txt
/path/to/src/main.rs
/path/to/src/lib.rs
/path/to/src/parser.rs
...
```

> テキストファイルは `~/.cache/asdb/tmp/<hash>_files.txt` に配置。  
> `asdb` が `initialize` 完了後にクリーンアップするか、OS の tmp クリーンアップに任せる。

---

### 対応プロジェクトタイプ

| `--type` | 入力ファイル | 内部処理 | 備考 |
|---|---|---|---|
| `compile-commands` | `compile_commands.json` | JSON 直接パース | CMake / Meson / Ninja / Bear が生成 |
| `cargo` | `Cargo.toml` | `cargo metadata --no-deps` 呼び出し | Rust 標準 |
| `msbuild` | `.sln` / `.vcxproj` | MSBuild XML パース | C# / C++ Visual Studio |
| `vcxproj` | `.vcxproj` | MSBuild XML パース | C++ プロジェクト単体 |
| `go-mod` | `go.mod` | `go list ./...` 呼び出し | Go 標準 |
| `uproject` | `*.uproject` | `Source/` ディレクトリスキャン + `.Build.cs` 列挙 | Unreal Engine |

> **車輪の再発明なし:** `cargo metadata` / `go list` など既存 CLI を最大限活用。  
> XML パースが必要なのは `.sln` / `.vcxproj` のみ。それ以外は既存ツールの出力を読む。

---

### asdb.nvim との連携フロー

```
nvim でプロジェクトを開く
    ↓
detect.lua がプロジェクトルートと種別を特定 (*.uproject / Cargo.toml / .sln 等)
    ↓
asdb-discover --type <type> <project_file> を非同期実行
    ↓
標準出力からテキストファイルパスを受け取る
    ↓
initialize RPC の scanners[].source_files_list にパスをセットして送信
    ↓
asdb Core が BufReader でストリーム読み込みしながらスキャン開始
```

**Lua 側のイメージ (asdb.nvim):**

```lua
-- detect.lua がプロジェクトタイプを判別
local project_type, project_file = detect.find_project(root)

-- asdb-discover を呼び出し (非同期)
local list_path = vim.fn.system({
  "asdb-discover", "--type", project_type, project_file
}):gsub("%s+$", "")  -- trim trailing newline

-- initialize RPC に source_files_list を追加して送信
client.initialize({
  root_path = root,
  config = {
    scanners = {
      {
        name = "cpp",
        extensions = { ".h", ".cpp" },
        source_files_list = list_path,  -- ← テキストファイルパスだけ渡す
        grammar_dll = grammar_dll_path,
        query_file = query_file_path,
      }
    }
  }
})
```

---

### フォールバック戦略

```
Priority 1: source_files_list が指定されている → テキストファイルをストリーム読み込み
Priority 2: source_dirs が指定されている      → glob スキャン (従来通り)
Priority 3: 両方なし                          → root_path 以下を ignore_dirs 除外で全探索
```

> `source_files_list` と `source_dirs` は排他ではない。  
> `source_files_list` のファイルが存在しない場合は `source_dirs` にフォールバックする。  
> Core はファイルが存在しないときに警告ログを出力し、`scan_progress` 通知でエディタに伝える。

---

### Core 側の読み込み実装イメージ (Rust)

```rust
// source_files_list が指定されている場合、BufReader で1行ずつ rayon に流す
if let Some(list_path) = scanner_config.source_files_list {
    let file = BufReader::new(File::open(&list_path)?);
    file.lines()
        .filter_map(|l| l.ok())
        .filter(|l| !l.is_empty())
        .par_bridge()           // BufReader の行を rayon の並列イテレータに変換
        .for_each(|path| scan_file(path, &tx));
} else {
    // source_dirs glob フォールバック
    walkdir::WalkDir::new(&root).into_iter()
        .filter_entry(|e| !is_ignored(e, &ignore_dirs))
        .par_bridge()
        .for_each(|entry| scan_file(entry.path(), &tx));
}
```

> `par_bridge()` により、テキストファイル読み込みのスループット律速なしに  
> 並列パースが走る。メモリ上に全ファイルパスを展開する必要がない。

