# AGENTS.md

エージェント・実装者向けメモ。運用規約の一次ソースは `CLAUDE.md` (編集にはユーザの明示許可が必要)。
バグ調査・セッション復旧・Bridge 観察・UI 検証の着手前に必ず `.claude/skills/{bug-frontier,session-recovery,parity-observation,ui-verify}/SKILL.md` を読む。

## 何を作るか

Adobe Bridge を 1:1 で再現するデジタルアセットブラウザ。pure Rust、ネイティブ (egui/eframe on wgpu)。
構成・規約は [storytold/photocraft](https://github.com/storytold/photocraft) に揃える。迷ったら photocraft の AGENTS.md に倣う。

- **Tauri / Electron / Webview・JavaScript は使わない。**
- **画面のレイアウト・パネル構成・メニュー・ショートカット・文言は Bridge を可能な限りそのまま再現する。** 正本は
  [docs/ui-design.md](docs/ui-design.md) と [docs/command-catalog.md](docs/command-catalog.md)。「より良い配置」は既定にしない。
- **編集は Craft Apps に委譲する。** Photoshop → PhotoCraft、Camera Raw → LightCraft 等。未インストールなら ArtCraft Launcher へ誘導。
  [docs/craft-integration.md](docs/craft-integration.md)。
- 最初に [docs/overview.md](docs/overview.md)、[docs/requirements.md](docs/requirements.md)、[docs/architecture.md](docs/architecture.md) を読む。

## ツールチェーン

| 項目 | 値 |
|---|---|
| Rust | `rust-toolchain.toml` で **1.95.0** に固定 (components: rustfmt, clippy)。`rust-version = "1.95"`。photocraft・craft-libs と同じ下限 |
| edition / resolver | 2024 / "3" |
| egui / eframe | 0.37。`[patch.crates-io]` で craft-libs と同じ storytold/egui・winit の rev に固定済み (ずれると egui が二重リンクされる) |
| craft-libs | git 依存、不変の commit rev で固定 |
| rustfmt | `max_width = 160`, `use_small_heuristics = "Max"` (Craft Apps 共通) |
| clippy.toml | テストでのみ unwrap/expect/panic/添字を許可。`egui::ComboBox` 禁止 (`craft_ui::ComboBox` を使う) |
| lints | `unsafe_code = "forbid"`、各クレートに `[lints] workspace = true`。本番クレートルートに `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]` |

Rust を上げるときは craft-libs の要求以上であることを確認し、[docs/decisions.md](docs/decisions.md) ADR-002 を置換する。

## 構成 (予定)

> 現状 (2026-11-03 試作): `crates/engine` (`pier-engine`: 一覧・デコード・EXIF・Craft App 起動をまとめた暫定クレート、egui 非依存)、
> `crates/ui-egui`、`crates/ui-driver` (`pier-ui-driver`: オフスクリーン操作のドライバーと `pier-drive`、シナリオテスト)、`apps/pier`。`pier-engine` は下表の core / fs / decode / render / craft-bridge へ順次分割する。
> 文字列は `ui-egui/src/i18n.rs` の英日テーブル (後で craft-i18n へ)。メニューは `ui-egui/src/menu.rs` の 1 つの定義 (craft-menu) から、mac はネイティブ、他は egui で描く。ショートカットもここが唯一の表。UI のデバッグは [docs/debugging.md](docs/debugging.md)。

| クレート | レイヤー | 役割 |
|---|---|---|
| `pier-core` | L0 | AssetId、パス正規化、ファイル種別判定、エラー型、時刻 |
| `pier-meta` | L0 standalone | EXIF / IPTC / XMP の読み書き。ワークスペース内依存禁止 |
| `pier-decode` / `pier-catalog` / `pier-fs` | L1 | デコーダ集約 / キャッシュ DB (redb)・ユーザーデータ / ファイル操作・監視・Undo ジャーナル |
| `pier-render` / `pier-search` / `pier-craft-bridge` | L2 | サムネイル生成ジョブ / フィルター・検索 / Craft Apps 検出・起動 |
| `pier-engine` | L3 | Session + コマンドレジストリ |
| `pier-ui-egui` / `pier-automation` | L4 | egui シェル (薄く) / コントロールチャネル・MCP |
| `apps/pier`、`apps/pier-cli` | — | デスクトップ / ヘッドレス CLI |

- 依存は下位レイヤーのみ。`ui-egui` より下で egui / eframe / winit / rfd を使わない。`cargo xtask layers` が強制。新クレートは `xtask/src/layers.rs` に登録。

## 規約 (要点)

### 落ちない (機能追加より優先)
- 本番コードは panic しない。`unwrap` / `expect` / `panic!` / `unreachable!` / `todo!` / `unimplemented!` 禁止。エラー型 + `?`。
- `unsafe` 禁止。OS 連携でどうしても必要なら隔離クレート + ADR。
- 入力由来の数値 (ファイル内オフセット・長さ・個数) は `get()`・`checked_*`・上限付き確保で扱う。再帰は深さ上限か visited 集合。
- コマンドの `run` はどんなパラメータ・状態でも `Err` を返す。`panic_hunt` テストが全コマンドを敵対的入力で叩く。
- デコードは `catch_unwind` + 時間・ピクセル数の上限付き。

### ユーザーのファイルを壊さない
- ファイル書き込みはすべて `craft-atomic` 経由。削除は OS のゴミ箱。ファイル操作・メタデータ変更は Undo できる。
- メタデータ書き込みは画素領域に触れない。書き込み後に再読込して画素領域のハッシュ一致を確認。確認できない形式はサイドカー。
- 一括操作は書き込み開始前に全件検証 (衝突・不正文字・長さ)。途中失敗は成功分と失敗分を報告。
- 書き込み前に mtime・サイズを再確認し、外部変更があれば中止。

### その他
- **すべてはコマンド。** ユーザー操作は engine のコマンド (id・ラベル・メニューパス・ショートカット・params・enabled・run)。id は command-catalog に従う。新コマンドは `engine/src/<area>_cmds.rs` の新モジュールに `specs()` で置く。
- **UI スレッドを止めない。** FS 走査・デコード・DB・外部プロセスはワーカー。UI はフレーム毎にイベントを drain。
- **クリーンルーム。** Bridge は挙動と見た目だけを観察する。Adobe のコード・アイコン・プリセット・テンプレート・非公開ファイル形式をコピー・解析しない。Bridge のスクリーンショットはコミットしない (`plan/bridge/` は gitignore)。サードパーティアセットは寛容ライセンスのみ、`ATTRIBUTION.md` に記載。フォントはコミットしない (`CRAFT_FONTS_DIR`)。
- **色を仮定しない。** 埋め込み ICC を尊重し `craft-cms` で変換。
- **時刻は `chrono`。** EXIF 撮影日時はオフセットを保持。
- **デモ画像はパブリックドメインのみ。** 個人写真を使わない。外部取得時のリクエストに個人情報を入れない。

## テスト・検証コマンド (予定)

```bash
cargo nextest run --workspace                         # suite (ローカル 6 スレッド)
cargo test --doc --workspace                          # doctest
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo xtask layers
cargo xtask parity                                    # コマンド追加時。command-catalog と menu_catalog の突き合わせ
cargo test -p pier-engine --test panic_hunt -- --ignored   # コマンド追加・変更時
cargo xtask perf --quick                              # ホットパス変更時
cargo test -p pier-ui-driver                         # シナリオテスト (本物のマウス・キー入力)
cargo run -p pier-ui-driver --bin pier-drive -- --steps '[{"op":"screenshot","path":"x.png"}]'   # UI 変更時 (docs/debugging.md)
```

テスト方針・CI ゲート・品質マトリクスは [docs/testing.md](docs/testing.md)。

## 作業の選び方

1. [docs/roadmap.md](docs/roadmap.md) の「現在のフォーカス」。
2. [docs/requirements.md](docs/requirements.md) の `未` 行を P0 → P1 → P2 の順で。
3. 機能を入れたら requirements と command-catalog の状態列を同じコミットで更新する。`要観察` を解消したら観察結果を書き込む。

## ディレクトリ規律

- `plan/` (gitignore): Bridge 観察メモとスクリーンショット (`plan/bridge/`)、調査メモ。
- `corpus/` (gitignore): 実ファイルのテストコーパス。`cargo xtask corpus` で取得、sha256 固定。
- `target/agent-*`: 並列エージェントのビルド先。終わったら削除 (1 つ約 10GB)。

## その他

- `dashboard.md` は空が正常。ユーザ判断が必要な事項とレコメンドのみ記載する。
- 性能・修正の主張には実測値が必須。
- CI は storytold/craft-common-actions の共有ワークフローを第一候補とする (組織外から使えるか要確認。不可なら同等ジョブを自前で: fmt / clippy / nextest / layers / doc を macOS・Windows・Linux で)。
