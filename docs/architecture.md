# 概要設計（アーキテクチャ）

> **最終更新:** 2026-11-02 · **状態:** 設計段階。実装が始まったら冒頭に「現状」節を足し、差分はそちらを正とする（photocraft の architecture.md と同じ運用）。

## 1. 方針

1. **エンジン優先・ヘッドレス優先。** すべての機能は GUI なしで動く（テスト、CLI、MCP）。GUI はエンジンのクライアントの 1 つ。
2. **すべてはコマンド。** ユーザー操作は安定した `CommandId`（例 `label.rating.set`）と serde で直列化できるパラメータを持つ。
   メニュー、ショートカット、コンテキストメニュー、CLI、コントロールチャネル、MCP は同じコマンドを呼ぶ。
3. **UI スレッドは描画だけ。** ファイル走査、デコード、DB、外部プロセス、ファイル操作はワーカー。UI はフレーム毎にイベントを drain する。
4. **ファイルが正、DB はキャッシュ。** レーティング・ラベル・キーワード・IPTC はファイル（埋め込み XMP / サイドカー）が正本。
   DB は検索と表示の高速化のための索引で、消しても再構築できる。例外はファイルに書けない情報（コレクション、スタック、
   手動並び順、キーワード辞書）で、これらは別ファイルのユーザーデータとして扱い、バックアップする（[data-model.md](data-model.md)）。
5. **pure Rust。** C/C++ 依存は I/O 端の feature の裏に限る（photocraft の原則 7）。
6. **編集機能は持たない。** 画素処理は Craft Apps に委譲する。Pier が書くのはメタデータとファイル操作だけ。

## 2. ワークスペース構成

```text
Pier/
├─ Cargo.toml                 [workspace] resolver = "3", edition = "2024", rust-version = "1.95"
├─ rust-toolchain.toml        channel = "1.95.0", components = ["rustfmt", "clippy"]
├─ crates/
│  ├─ core/        pier-core          L0  AssetId、パス正規化（NFC/NFD）、ファイル種別判定（拡張子＋マジックバイト）、エラー型、時刻
│  ├─ meta/        pier-meta          L0  EXIF / IPTC-IIM / XMP の読み書き、埋め込み領域の差し替え（ワークスペース依存なし、単体公開可）
│  ├─ decode/      pier-decode        L1  フォーマット別デコーダの集約。埋め込みサムネイル抽出、縮小デコード、ページ数
│  ├─ catalog/     pier-catalog       L1  キャッシュ DB（redb）とユーザーデータストア、スキーマ移行
│  ├─ fs/          pier-fs            L1  走査、監視、コピー／移動／ゴミ箱／リネーム、関連ファイル解決、アトミック書き込み、Undo ジャーナル
│  ├─ render/      pier-render        L2  サムネイル／プレビュー生成ジョブ（優先度キュー、キャンセル、色変換、キャッシュ書き込み）
│  ├─ search/      pier-search        L2  フィルター集計、検索条件 AST、スマートコレクション評価、並べ替え
│  ├─ craft-bridge/pier-craft-bridge  L2  Craft Apps 検出、起動、ファイル引き渡し、CLI 実行、インストール誘導情報
│  ├─ engine/      pier-engine        L3  Session、コマンドレジストリ、履歴（Undo/Redo）、ジョブ管理
│  ├─ ui-egui/     pier-ui-egui       L4  egui シェル。パネル、ワークスペース、ダイアログ、テーマ、オフスクリーンスナップショット
│  ├─ automation/  pier-automation    L4  JSON コントロールチャネルのハンドラ、MCP サーバー
│  └─ testkit/     pier-testkit       --  合成ファイル生成器、一時フォルダ、ゴールデン比較
├─ apps/
│  ├─ pier/        デスクトップ（eframe/wgpu）、単一インスタンス、OS 統合、コントロールサーバー
│  └─ pier-cli/    ヘッドレス CLI
├─ xtask/          layers | parity | ci | perf
└─ docs/
```

### 2.1 レイヤー規則

```text
L4  ui-egui  automation                     ← egui/eframe/winit/rfd はここから上だけ
L3  engine
L2  render  search  craft-bridge
L1  decode  catalog  fs
L0  core  meta（meta は standalone：ワークスペース内依存なし）
```

- 依存は下位レイヤーのみ。同レイヤー間の依存は禁止（必要なら下へ共通部分を切り出す）。
- `cargo xtask layers` が `cargo metadata` を読んで検査する（photocraft の `xtask/src/layers.rs` と同じ方式）。
- ファイルサイズ 2500 行（テスト除く）、関数 220 行超の新規追加を CI で検出する（Otspot と同じゲート）。

### 2.2 外部依存（予定）

| 用途 | クレート | 備考 |
|---|---|---|
| UI | `eframe` / `egui` 0.37（storytold フォーク rev）、`craft-ui` | `[patch.crates-io]` を craft-libs と同じ rev に |
| 設定 | `craft-settings` | config.toml、移行、ポータブルモード |
| ダイアログ | `craft-dialogs` | フレームを止めないネイティブダイアログ |
| 翻訳 | `craft-i18n` | `tl!` マクロ |
| ショートカット | `craft-keymap` | スコープ、衝突検出、ユーザー上書き |
| macOS メニュー | `craft-menu`、`craft-quit` | |
| 色管理 | `craft-cms` | ICC v2/v4、ディスプレイ変換 |
| アトミック書き込み | `craft-atomic` | |
| パス検証 | `craft-docpath` | コレクション等に保存したパスを開く前に検証 |
| PDF | `craft-pdf` | PDF サムネイル（ラスタライズ）、コンタクトシート生成（authoring） |
| 動画コンテナ | `craft-media` | MP4/MOV/MKV のメタデータ・キーフレーム位置 |
| フォント | `craft-font-catalog` | フォントプレビュー |
| 印刷 | `craft-print` | |
| 更新確認 | `craft-update` | オプトイン |
| 画像デコード | `image`（JPEG/PNG/GIF/WebP/TIFF/BMP、default-features off）、`zune-jpeg`、`craft-jpeg` | [file-formats.md](file-formats.md) |
| PSD / raw | photocraft の `photocraft-psd`、`photocraft-raw`（L0 standalone）を git rev 固定 | craft-libs への昇格を上流に提案（[decisions.md](decisions.md) ADR-004） |
| DB | `redb` | pure Rust の組み込み KV（ADR-003） |
| 監視 | `notify` | FSEvents / ReadDirectoryChangesW / inotify |
| ゴミ箱 | `trash` | OS のゴミ箱。復元は OS 別実装（ADR-008） |
| 並列 | `rayon`、`crossbeam-channel` | |
| 時刻 | `chrono` | |
| CLI | `clap` 4 | |

## 3. エンジンと UI の境界

```text
            ┌──────────── apps/pier ────────────┐
 winit ───▶ │ eframe  ─▶ ui-egui (state, panels) │ ◀── control_server (TCP, loopback)
            └───────────────┬───────────────────┘
                            │ Command{id, params} / Event
                     ┌──────▼──────┐
                     │   engine    │  Session（ウィンドウ毎の Browser 状態、選択、履歴）
                     └──┬──┬──┬──┬─┘
             ┌──────────┘  │  │  └───────────┐
          fs/catalog     render  search   craft-bridge
             │              │
         ファイルシステム   decode/meta
```

- `ui-egui` は `engine::Session` にコマンドを送り、`engine` からの `Event`（`FolderListed`、`ThumbReady`、`MetaUpdated`、
  `JobProgress`、`FsChanged` 等）をフレーム毎に drain して `UiState` を更新する。
- `UiState`（serde）はパネル配置、表示モード、サムネイルサイズ、ダイアログ状態を持つ。コントロールチャネルの `ui.inspect` / `ui.set` で読み書きできる。
- 表示だけの状態（スクロール位置、ホバー）以外のビジネスロジックを `ui-egui` に書かない。

### 3.1 コマンド

```rust
pub struct CommandSpec {
    pub id: &'static str,              // "label.rating.set"
    pub label: LocalizedKey,           // 表示名（craft-i18n キー）
    pub menu: Option<&'static str>,    // "Label/Rating/★★★★★"
    pub shortcut: Option<Shortcut>,    // 既定（craft-keymap で上書き可）
    pub params: ParamsDoc,             // JSON スキーマ相当の説明
    pub enabled: fn(&Session) -> bool,
    pub run: fn(&mut Session, Params) -> Result<Outcome, CommandError>,
    pub undo: UndoKind,                // None / FileOps / Metadata / Selection
}
```

- `run` はあらゆるパラメータ・状態で `Err` を返し、panic しない（`panic_hunt` テストで全コマンドを敵対的パラメータで叩く）。
- 時間のかかるコマンドは `Outcome::Job(JobId)` を返し、進捗は `Event::JobProgress` で通知する。
- コマンド ID と既定ショートカットの一覧は [command-catalog.md](command-catalog.md)。

### 3.2 Undo / Redo

| 種類 | 記録内容 | 取り消し方 |
|---|---|---|
| ファイル操作 | 操作ジャーナル（元パス、先パス、ゴミ箱内の識別子） | 逆操作。ゴミ箱は OS 別の復元 API／ゴミ箱フォルダからの移動 |
| メタデータ | 変更前の XMP プロパティ値（変更したプロパティのみ） | 変更前の値を書き戻す |
| コレクション・スタック | 変更前のユーザーデータ差分 | 差分を戻す |

- ジャーナルはディスクに残し（`catalog` のユーザーデータ）、アプリ再起動後も直近 N 件（既定 100）を取り消せる。
- 外部でファイルが変更されていたら（mtime・サイズ不一致）、取り消しを拒否してユーザーに理由を示す。

## 4. スレッドとジョブ

| スレッド／プール | 役割 | 優先度 |
|---|---|---|
| UI（メイン） | egui の描画、イベント drain、テクスチャのアップロード（1 フレームあたり上限付き） | 最高 |
| `fs` ワーカー | ディレクトリ列挙（バッチで UI に送る）、stat、監視イベントの集約（100ms デバウンス） | 高 |
| `render` プール（rayon、論理コア数−1） | サムネイル／プレビューのデコード・縮小・色変換 | 表示中 > 先読み > バックグラウンド構築 |
| `catalog` ライター（1 本） | DB 書き込みをバッチでまとめてコミット | 中 |
| `meta` ワーカー | メタデータの読み取り（一覧用の軽量読み取りと、メタデータパネル用の完全読み取り） | 高（表示中） |
| ジョブランナー | ファイル操作、一括リネーム、メタデータ一括書き込み、取り込み、書き出し、キャッシュ構築 | 中、キャンセル可 |
| 外部プロセス監視 | Craft App の CLI 実行、起動結果の待機 | 低 |

- **表示優先スケジューリング。** コンテンツパネルは表示範囲（とスクロール方向の先 2 画面）の AssetId 列を毎フレーム
  `render` に渡す。キューは世代番号付きで、表示外になった要求はデコード開始前に破棄する。
- **デコードの安全網。** デコードは `catch_unwind` で囲み、時間（既定 10s）とピクセル数（既定 600MP）の上限を持つ。
  失敗したファイルは「プレビューを生成できません」アイコンにし、失敗理由を DB に記録して再試行しない（mtime が変わるまで）。
- **GPU テクスチャ。** サムネイルは表示範囲＋前後 2 画面分だけテクスチャに保持し、LRU で解放する。

## 5. 主要データフロー

### 5.1 フォルダを開く

```text
1. NAV コマンド (browser.goTo {path})
2. fs: read_dir をストリーミング → 256 件ごとに FolderListed(batch) を送る
3. catalog: (path, mtime, size, file_id) で既知エントリを引き、キャッシュ済み軽量メタデータ（★・ラベル・寸法・撮影日）を返す
4. ui: プレースホルダー付きで即時表示 → 並べ替え・フィルター（search）
5. render: 表示範囲のサムネイルを要求。キャッシュヒットは即表示、ミスはデコード
6. meta: 未索引ファイルの軽量メタデータを読み、catalog に書く → フィルターパネルの集計を更新
7. fs: フォルダの監視を開始（離れたら停止。お気に入りは監視しない）
```

### 5.2 レーティングを付ける

```text
1. label.rating.set {rating: 5}（対象＝選択）
2. engine: 書き込み先を判定（埋め込み可 → 埋め込み、raw → サイドカー、不可 → DB のみ＋警告）
3. ジョブ: 1 件ずつ meta で XMP を更新 → craft-atomic で置換 → 再読込して画素領域ハッシュを照合
4. catalog: 索引を更新、Undo ジャーナルに旧値を記録
5. ui: 楽観的に即表示を更新し、失敗したものだけ元に戻してエラー表示
```

### 5.3 Craft App で開く

```text
1. file.open（対象＝選択）
2. craft-bridge: 拡張子 → 関連付け（環境設定）→ Craft App 検出結果
3a. 見つかった: 実行ファイルにパスを引数で渡して起動（既存インスタンスがあれば Craft App 側の single-instance がタブで開く）
3b. 見つからない: インストール誘導ダイアログ（[craft-integration.md](craft-integration.md) §4）
```

## 6. 永続化の場所

| 種類 | 場所（macOS / Windows / Linux） | 消してよいか |
|---|---|---|
| 設定 | `craft-settings` の標準フォルダ（`~/Library/Application Support/Pier/config.toml` 等） | リセット時のみ |
| ユーザーデータ（コレクション、スタック、手動並び順、キーワード辞書、Undo ジャーナル、テンプレート、プリセット） | 設定フォルダ内 `userdata/`（JSON、アトミック保存、世代バックアップ 5 つ） | いいえ |
| キャッシュ DB・サムネイル・プレビュー | `~/Library/Caches/Pier/` / `%LOCALAPPDATA%\Pier\Cache\` / `$XDG_CACHE_HOME/pier/` | はい（再構築される） |
| ログ | 設定フォルダ内 `logs/pier.log`（直近 3 回分をローテーション） | はい |
| クラッシュレポート | 設定フォルダ内 `crashes/` | はい |

## 7. 自動化インターフェイス

photocraft と同じ形状にそろえ、Craft Apps を横断するエージェントが同じ作法で扱えるようにする。

| 経路 | 内容 |
|---|---|
| `pier-cli` | `info <paths>`（メタデータ JSON）、`thumbs <dir>`（キャッシュ構築）、`run --cmd <id> --params <json> <paths>`、`commands`（レジストリ一覧）、`rename`、`meta get/set`、`mcp` |
| コントロールチャネル | `pier --control <port> --control-token-file <path>`。JSON Lines。最初のフレームで `auth`。メソッド：`engine.execute`、`engine.commands`、`ui.inspect`、`ui.set`、`ui.menu.invoke`、`ui.menu.list`、`ui.screenshot`、`ui.pointer`、`ui.key` |
| MCP | `pier-cli mcp`（ヘッドレス）／`pier-cli mcp --bridge 127.0.0.1:<port>`（稼働中アプリ）。ツール：`browse_list`、`asset_inspect`、`asset_set_rating`、`asset_set_label`、`asset_set_metadata`、`asset_add_keywords`、`collection_*`、`batch_rename_preview`、`batch_rename_apply`、`thumbnail_render`（PNG を返す）、`command_run`、`ui_*`（ブリッジ時のみ） |

- MCP / CLI は `--automation-read-root` / `--automation-write-root` の配下だけを読み書きできる。
- 削除系（ゴミ箱を含む）は MCP では既定で無効。`--allow-delete` を明示したときだけ有効。
