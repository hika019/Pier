# 設計判断の記録（ADR）

> **最終更新:** 2026-11-02 · 状態: `採用` / `提案`（ユーザー判断待ちは [../dashboard.md](../dashboard.md) にも記載） / `却下` / `置換`

新しい判断は末尾に追加し、番号を振り直さない。覆すときは新しい ADR を書き、旧 ADR を `置換` にする。

| # | 判断 | 状態 |
|---|---|---|
| ADR-001 | UI は egui/eframe on wgpu（Webview なし） | 採用 |
| ADR-002 | Rust 1.95.0 に固定、edition 2024 | 採用 |
| ADR-003 | キャッシュ DB は redb | 提案 |
| ADR-004 | PSD / raw は photocraft の standalone クレートを git rev 固定で使う | 提案 |
| ADR-005 | メタデータは自前の `pier-meta`（standalone）。将来 craft-libs へ | 提案 |
| ADR-006 | 動画サムネイルは FilmCraft CLI 委譲を第一候補 | 提案 |
| ADR-007 | Web 版は作らない | 採用 |
| ADR-008 | 削除は OS のゴミ箱、Undo で復元 | 採用 |
| ADR-009 | Craft App 検出は Launcher と同じ順序を Pier で実装、将来共通化を提案 | 提案 |
| ADR-010 | AI 機能は初期対象外 | 採用 |
| ADR-011 | 製品名 Pier、アプリ ID `io.github.hika019.pier` | 提案 |
| ADR-012 | Bridge 独自のキャッシュ・コレクション形式は読まない | 採用 |

---

## ADR-001 UI は egui/eframe on wgpu

- **背景:** Craft Apps はすべて egui/eframe（storytold フォーク）で、`craft-ui`（ドッキング、アイコン、タブ、テーマ）を共有している。
- **決定:** egui 0.37 + eframe（wgpu）。`[patch.crates-io]` で craft-libs と同じ storytold/egui の rev に固定する。Tauri / Electron / Webview は使わない。
- **結果:** Craft Apps と見た目・操作感・ライブラリを共有できる。Bridge の再現に必要な仮想スクロール（1 万件のグリッド）は自前で書く（egui の `ScrollArea::show_rows` 相当を 2 次元グリッドに拡張）。

## ADR-002 Rust 1.95.0 固定

- **背景:** photocraft / craft-libs / craft-launcher は `rust-version = "1.95"`、`craft-ui` は 1.95 以上を要求。Otspot は `rust-toolchain.toml` で 1.95.0 に固定してエージェント間の差異をなくしている。
- **決定:** `rust-toolchain.toml` の `channel = "1.95.0"`（components: rustfmt, clippy）、ワークスペースの `rust-version = "1.95"`、edition 2024、resolver 3。
- **結果:** 上げるときは craft-libs の要求を確認し、この ADR を置換する。

## ADR-003 キャッシュ DB は redb

- **背景:** SQLite（rusqlite）は C 依存で、photocraft の「pure Rust 原則」に反する。キャッシュは KV で足りる（複雑な結合クエリは `search` がメモリ上で行う）。
- **候補:** redb（pure Rust、ACID、単一ファイル、MVCC 読み取り）、fjall（LSM）、sled（開発停滞）、自前ファイル群。
- **決定（提案）:** redb。フォルダー単位の読み込みが主で、1 ライター＋複数リーダーのモデルに合う。
- **結果:** 全文検索が必要になったら `tantivy` 等を別途検討。

## ADR-004 PSD / raw は photocraft の standalone クレート

- **背景:** photocraft の `photocraft-psd`、`photocraft-codecs`、`photocraft-raw` はワークスペース内依存のない L0 standalone で、公開可能と明記されている。craft-libs の README は「RAW 対応などは craft-libs や独立ライブラリに昇格し得る」としている。
- **決定（提案）:** 当面は photocraft リポジトリへの git 依存（不変 rev 固定）。同時に、craft-libs への昇格を上流に提案する。
- **リスク:** photocraft リポジトリ全体の取得が重い、ライセンスは MIT OR Apache-2.0 で Pier（MIT）と両立。

## ADR-005 メタデータは `pier-meta`

- **背景:** LightCraft の `lightcraft-meta` は読み書きとも揃っているが L1 で、LightCraft のワークスペースに属する。Pier は PSD・PNG・WebP・TIFF への**埋め込み書き込み**が必要で、LightCraft の用途（サイドカー中心）より広い。
- **決定（提案）:** `pier-meta` を standalone で作る。API 形状は `lightcraft-meta` に合わせ、将来 `craft-meta` として craft-libs で共通化できるようにする。コードはコピーせず（ライセンス上は可能だが、責務が違うため）、必要なら上流と相談して共通化する。

## ADR-006 動画サムネイル

- **背景:** pure Rust の H.264/HEVC デコーダは重く、FilmCraft がコーデック実装を持つ。
- **決定（提案）:** コンテナは `craft-media`、サムネイルは埋め込みカバー → FilmCraft CLI 委譲 → 汎用アイコン。FilmCraft のデコーダがクレートとして切り出されたら直接依存を検討。

## ADR-007 Web 版は作らない

- **背景:** Bridge の価値はローカルファイルシステムの閲覧・操作。ブラウザの File System Access API は対応ブラウザと権限が限られる。
- **決定:** ネイティブのみ。L0〜L2 の wasm 互換は要求しない（photocraft の「wasm を壊さない」規則は採用しない）。

## ADR-008 削除は OS のゴミ箱

- **決定:** `trash` クレートでゴミ箱へ。Undo のための復元は、macOS は Finder の「戻す」相当の情報（元パス）をジャーナルに持ち、ゴミ箱内の実体を移動して戻す。Windows は Shell の復元、Linux は FreeDesktop Trash 仕様（`.trashinfo`）。完全削除は別コマンドで確認ダイアログ付き。

## ADR-009 Craft App 検出

- **背景:** ArtCraft Launcher の `detect.rs` が全 OS の探索を実装済み。ただし `artcraft-launcher-native` はアプリ内部クレートで、HTTP 等の重い依存を持つ。
- **決定（提案）:** 同じ探索順序を `pier-craft-bridge` で実装。並行して、検出ロジックと「アプリを指定して Launcher を開く」起動引数（または URL スキーム）を上流に提案する。

## ADR-010 AI 機能は初期対象外

- **決定:** 自動キーワード、顔認識、類似検索などは 1.0 以降。実装するときは `craft-ai-core` の契約（同意、ローカル優先）に従う。

## ADR-011 製品名とアプリ ID

- **背景:** storytold の命名は `{Function}Craft`、ID は `ai.storyteller.<name>`。Pier は storytold 組織外の個人リポジトリ。
- **決定（提案）:** 製品名 Pier、ID `io.github.hika019.pier`。storytold に寄贈・合流する場合は命名規則に合わせて変更する（設定フォルダーの移行が必要になるため、1.0 前に確定する）。

## ADR-012 Bridge 独自形式は読まない

- **決定:** `.BridgeCache`、`.BridgeSort`、Bridge のコレクションファイルなど、Adobe 非公開の形式は読み書きしない。Bridge から移行するユーザー向けには、XMP（ファイル側に書かれた★・ラベル・キーワード）がそのまま引き継がれることを案内する。
