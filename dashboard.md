# ユーザー判断待ち

M0 着手前に決めてほしい事項。決まったら該当 ADR（docs/decisions.md）を `採用` にしてこの行を消す。

| # | 判断事項 | レコメンド | 関連 |
|---|---|---|---|
| 1 | 製品名とアプリ ID | 製品名 **Pier**、ID `io.github.hika019.pier`。storytold に合流する予定があるなら `{Function}Craft` 命名と `ai.storyteller.*` に今から合わせる（後から変えると設定フォルダーの移行が必要） | ADR-011 |
| 2 | JPEG・PNG など汎用画像をダブルクリックしたときの既定 | **OS の既定アプリ**（Bridge も OS の関連付けに従う）。PSD・raw などは Craft App を既定にする | craft-integration §1 |
| 3 | 「Camera Raw で開く」の既定の行き先 | **LightCraft**（非破壊 raw 現像。Bridge の Camera Raw の役割に最も近い） | ADR / CRF-05 |
| 4 | PSD・raw デコードのために photocraft リポジトリへ git 依存してよいか | **してよい**（rev 固定）。並行して craft-libs への昇格を上流に提案 | ADR-004 |
| 5 | キャッシュ DB | **redb**（pure Rust） | ADR-003 |
| 6 | Bridge 2026 の実機を観察に使えるか | 使える環境があれば M1 と並行して観察し `要観察` を潰す。使えなければ公開ヘルプ・公開スクリーンショットのみで進め、確度の低い箇所を一覧化する | ui-design.md |
