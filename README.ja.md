# Pier

[English](README.md) | **日本語**

[![CI](https://github.com/hika019/Pier/actions/workflows/ci.yml/badge.svg)](https://github.com/hika019/Pier/actions/workflows/ci.yml)

**Pier は Adobe Bridge 相当のデジタルアセットブラウザを、pure Rust でクリーンルーム再実装するプロジェクトです。**
フォルダを辿ってサムネイルで素材を一覧し、プレビュー・レーティング・ラベル・キーワード・メタデータ編集・
一括リネーム・コレクション・スタックで整理して、編集は [storytold の Craft Apps](https://github.com/storytold)
（PhotoCraft、LightCraft、VectorCraft、DesignCraft、PdfCraft、FilmCraft など）へ渡します。

- Photoshop / Camera Raw / Illustrator / InDesign の代わりに **Craft Apps が入っている前提** で連携します。
  入っていなければ [ArtCraft Launcher](https://github.com/storytold/craft-launcher) でのインストールへ誘導します。
- 構成・規約は [storytold/photocraft](https://github.com/storytold/photocraft) に揃えます
  （egui/eframe + wgpu のネイティブ UI、Webview なし、Rust 1.95.0 固定（photocraft の下限 1.95 と同じ）、edition 2024、[craft-libs](https://github.com/storytold/craft-libs) を rev 固定で利用）。

> ステータス: **最初の試作**。Bridge と同じ画面構成で、フォルダー閲覧・サムネイル・プレビュー・メタデータ表示・並べ替え・
> フィルター・★／ラベル（セッション内のみ）・Craft App で開く（画像と raw は Bridge と同じく PhotoCraft で、raw は Camera Raw を経由）、が動きます。それ以外のメニューやパネルは配置だけで、
> 「今後実装予定」と表示されます。mac ではメニューは OS のメニューバーに出ます。英語・日本語に対応（環境設定 › 詳細設定 › 言語、または `PIER_LOCALE=ja`）。

## 動かし方

```sh
cargo run -p pier                 # ピクチャフォルダーを開く
cargo run -p pier -- ~/Photos     # フォルダーを指定
cargo run -p pier-ui-driver --bin pier-drive -- --lang ja --steps '[{"op":"screenshot","path":"shot.png"}]'   # オフスクリーンで操作・撮影（docs/debugging.md）
cargo nextest run --workspace     # テスト（CI と同じ。doctest は cargo test --doc --workspace）
```

テストは GitHub Actions で push ごとに流れます（macOS・Ubuntu でテスト、Ubuntu で fmt・clippy、Windows でビルド確認。[.github/workflows/ci.yml](.github/workflows/ci.yml)）。

Rust は `rust-toolchain.toml` により 1.95.0 が使われます（rustup が自動で入れます）。

## ドキュメント

| ドキュメント | 内容 |
|---|---|
| [AGENTS.md](AGENTS.md) | 実装者・AI エージェント向けの規約（最初に読む） |
| [CLAUDE.md](CLAUDE.md) | エージェント運用の一次ソース（lead / subagent、git、レビュー、テスト） |
| [dashboard.md](dashboard.md) | ユーザー判断待ちの事項（空が正常） |
| [.claude/skills/](.claude/skills/) | 作業別手順：bug-frontier、session-recovery、parity-observation（Bridge 観察）、ui-verify |
| [docs/overview.md](docs/overview.md) | 製品概要・目的・スコープ・用語 |
| [docs/requirements.md](docs/requirements.md) | 機能要件・非機能要件（ID 付き、優先度付き） |
| [docs/architecture.md](docs/architecture.md) | 概要設計：ワークスペース構成、レイヤー、スレッド、データフロー |
| [docs/data-model.md](docs/data-model.md) | データモデル、キャッシュ DB、メタデータの書き込み方針 |
| [docs/ui-design.md](docs/ui-design.md) | 画面設計：ワークスペース、パネル、ダイアログ |
| [docs/command-catalog.md](docs/command-catalog.md) | メニュー・コマンド ID・ショートカットの一覧 |
| [docs/file-formats.md](docs/file-formats.md) | 対応フォーマットとサムネイル／プレビュー生成方針 |
| [docs/craft-integration.md](docs/craft-integration.md) | Craft Apps との連携・インストール誘導 |
| [docs/testing.md](docs/testing.md) | テスト戦略・性能予算 |
| [docs/debugging.md](docs/debugging.md) | AI も使えるデバッグ環境（オフスクリーン操作・PNG・状態 JSON） |
| [docs/roadmap.md](docs/roadmap.md) | マイルストーンと完了条件 |
| [docs/decisions.md](docs/decisions.md) | 設計判断の記録（ADR） |

## ライセンス

[MIT](LICENSE)。Adobe、Adobe Bridge、Photoshop は Adobe Inc. の商標です。Pier は Adobe とは無関係で、
Adobe のコード・アイコン・アセットを一切含みません（[AGENTS.md](AGENTS.md) のクリーンルーム規則を参照）。
