# テスト戦略・品質ゲート

> **最終更新:** 2026-11-03

## 1. 原則

- **テスト駆動。** 先にテストを書き、失敗を確認してから実装する。
- **既存テストのアサーション・ロジックを変更しない。** 挙動を変えるときは旧テストの削除＋新テストの追加として表す。
- **独立オラクル。** 実装を正とせず、仕様書の手計算、別ツール（exiftool、ImageMagick、Bridge の実機出力）で期待値を作る。
- **センチネルテスト。** バグ修正には、修正を revert すると失敗するテストを付ける（revert して失敗を確認済みであること）。
- **1 テスト 3 分以内。** nextest で実行する。

## 2. テストの種類

| 種類 | 対象 | 方法 |
|---|---|---|
| 単体 | 各クレートの関数 | `#[test]`、proptest（パス正規化、検索条件、一括リネームのトークン展開） |
| 往復 | `pier-meta` の XMP / EXIF / IPTC 書き込み | 書く → 読む → 一致。未知プロパティが残る。画素領域のハッシュが変わらない |
| 互換 | メタデータ書き込み結果 | exiftool で読み、期待値と比較（CI では exiftool をテスト依存として導入。無い環境では明示的に fail させ、黙ってスキップしない） |
| 合成ファイル | デコーダ | `pier-testkit` の生成器で全形式の最小・境界ファイルを作る |
| ファズ | `meta`、`decode` のパーサ | `cargo fuzz`（libFuzzer）。破損ファイル、巨大寸法、循環 IFD。panic・タイムアウト・過大確保を検出 |
| panic_hunt | 全コマンド | 全コマンドに敵対的パラメータ（空、範囲外、型違い、存在しないパス、巨大配列）を投げ、`Err` を返すこと（photocraft と同じ） |
| ファイル操作 | `fs`、Undo | 一時ディレクトリで移動・コピー・ゴミ箱・リネーム → Undo → 元通り（内容ハッシュ・mtime）。衝突・権限エラー・途中失敗 |
| 障害注入 | アトミック書き込み | `craft-atomic` の fault injection で書き込み途中の失敗を起こし、元ファイルが残ることを確認 |
| UI スナップショット | `ui-egui` | egui_kittest でワークスペース・ダイアログを描画し PNG 比較（許容差あり）。更新は明示的なフラグでのみ |
| UI 操作（シナリオ） | アプリ全体（オフスクリーン） | `crates/ui-driver/tests/scenarios.rs`：本物のマウス・キー入力で操作し、状態 JSON・パネル配置を検査（[debugging.md](debugging.md)） |
| 統合 | アプリ全体 | `pier --control` で起動し、コントロールチャネルからシナリオ実行（フォルダーを開く → ★ → フィルター → リネーム → Undo） |
| コーパス | 実ファイル | `cargo xtask corpus` で取得したファイルに対し、サムネイル生成成功率とメタデータ読み取りの一致率の下限（floor）を持つ。下限は上げるだけ |
| 性能 | NFR-P | `cargo xtask perf`。合成 1 万ファイルのフォルダーで NFR-P2〜P6 を計測し、`perf/budgets.toml` と比較 |

## 3. 実行コマンド（予定）

```bash
cargo nextest run --workspace                      # 通常
cargo nextest run --workspace --profile ci         # CI 設定（スレッド数を CI 用に）
cargo test --doc --workspace                       # doctest（nextest 非対応）
cargo test -p pier-engine --test panic_hunt -- --ignored
cargo xtask corpus && cargo nextest run --features corpus
cargo xtask perf --quick
```

`.config/nextest.toml` はローカル 6 スレッド、CI 3 スレッド（Otspot と同じ）。

**UI テストが要るフォント。** egui_kittest は描けない文字があると失敗する（アプリでは □ になる文字を、テストで見逃さないため）。
日本語 UI のテストには CJK フォント（macOS は標準の Hiragino、Linux は Noto Sans CJK：`fonts-noto-cjk` など）が要る。
ショートカットの記号（⌘⌥⇧⌫ など）は macOS の表記でだけ使い、macOS 標準の Apple Symbols で描く。Windows・Linux の表記は
`Ctrl+Backspace` のような文字なので、記号フォントは要らない。CI ではフォントを入れてから流し、黙ってスキップしない。

## 4. 機械ゲート（CI で red になるもの）

| ゲート | コマンド |
|---|---|
| フォーマット | `cargo fmt --all -- --check` |
| lint | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| ドキュメント | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` |
| レイヤー | `cargo xtask layers`（依存方向、egui の使用箇所） |
| サイズ | 本番ファイル 2500 行、220 行超の関数の新規追加（ベースライン登録制） |
| TODO 禁止 | 本番コードに `TODO` / `FIXME` / `XXX` / `HACK` / `todo!()` を置かない |
| 未使用依存 | `cargo machete` |
| パリティ | `cargo xtask parity`（command-catalog と menu_catalog の一致） |
| 移植性 | macOS / Windows / Linux で `cargo check --workspace --all-targets` |
| 依存監査 | `cargo audit` |

## 5. 品質特性マトリクス（ISO/IEC 25010）

認証ではなく「どのゲートがどの品質を守るか」の対応表。新しいゲートを足したらここを更新する。

| 品質特性 | 根拠（テスト・ゲート） | 残っている穴 |
|---|---|---|
| 機能適合性 | 単体、往復、互換（exiftool）、コーパス floor、command-catalog パリティ | Bridge との挙動差は観察依存（requirements の `要観察`） |
| 性能効率性 | `xtask perf` と予算、UI フレーム時間計測 | 実機（HDD、ネットワークボリューム）での計測が手動 |
| 互換性 | XMP 往復で未知プロパティ保持、Camera Raw / LightCraft と同じサイドカー命名 | 他 DAM（Lightroom Classic 等）との往復は手動確認 |
| 使用性 | UI スナップショット、キーボード操作テスト、ショートカット一致 | 実ユーザーによる評価 |
| 信頼性 | panic_hunt、ファズ、障害注入、Undo テスト、DB 破損からの復旧テスト | 電源断の実機試験 |
| セキュリティ | loopback + トークン、automation ルート制限、引数配列での起動、`cargo audit` | MCP 経由の操作の監査ログ |
| 保守性 | clippy、サイズゲート、レイヤーゲート、TODO 禁止 | 設計の妥当性は人のレビュー |
| 移植性 | 3 OS での check・test | Linux の Wayland / X11、各ファイルマネージャー連携は手動確認 |
