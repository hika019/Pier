---
name: ui-verify
description: UI (パネル、ワークスペース、ダイアログ、テーマ) を実装・変更したときに必ず読む。オフスクリーン描画で PNG を作り、Bridge の参照と並べて目視確認する手順。
---

# UI 検証手順

「ビルドが通った」「テストが通った」は UI の完了ではない。PNG を見て確認するまで完了にしない。

## 1. 描画
- オフスクリーン: `cargo run -p pier-ui-driver --bin pier-drive -- [--workspace <id>] [--theme <id>] [--lang en|ja] [--size 1440x900 --scale 2] --steps '[...,{"op":"screenshot","path":"x.png"}]'`
  (ウィンドウを出さない、フォーカスを奪わない。ダイアログはメニュー項目を `click {label}` で開く。手順一覧は docs/debugging.md)。
- 実アプリ: `pier --control <port> --control-token-file <path>` で起動し、`ui.set` で状態を作って `ui.screenshot`。
- デモ用フォルダーはパブリックドメイン画像のみ (`pier-testkit` のデモセット)。個人写真を使わない。

## 2. 確認項目
- 配置: docs/ui-design.md の図・表と順序・位置が一致するか。Bridge 参照 (`plan/bridge/`) と並べて比較する。
- 文言: ラベル・ツールチップ・メニュー項目が command-catalog / ui-design と一致するか (日本語・英語の両方)。
- トークン: 色・余白・角丸が `theme::Tokens` から来ているか (ハードコード禁止)。4 段階の明るさすべてで描画して確認。
- 状態: 空フォルダー、1 件、1 万件、選択なし／1 件／複数、読み込み中、エラー、無効状態。
- 幅: 最小ウィンドウ幅 (要観察) と 4K。折り畳み順が Bridge と同じか。
- CJK: 日本語 UI で文字切れ・重なりがないか。

## 3. スナップショットテスト
- 確認した操作は `crates/ui-driver/tests/scenarios.rs` のシナリオテストとして追加する (状態 JSON で検査できるものは JSON で)。
- 既存スナップショットの更新は意図した変更のときだけ。更新理由をコミットメッセージに書く。

## 4. 完了報告
①PNG のパス (確認した状態ごと) ②Bridge 参照との差分と、その差が意図的か ③追加したスナップショットテスト を報告する。
