# ユーザー判断待ち

M0 着手前に決めてほしい事項。決まったら該当 ADR（docs/decisions.md）を `採用` にしてこの行を消す。

| # | 判断事項 | レコメンド | 関連 |
|---|---|---|---|
| 1 | 製品名とアプリ ID | 製品名 **Pier**、ID `io.github.hika019.pier`。storytold に合流する予定があるなら `{Function}Craft` 命名と `ai.storyteller.*` に今から合わせる（後から変えると設定フォルダーの移行が必要） | ADR-011 |
| 3 | 「Camera Raw で開く」（⌘R）の行き先。ダブルクリック・⌘O は Bridge と同じく PhotoCraft（Photoshop 相当）で確定済みで、ここは ⌘R だけの話 | **LightCraft**（現状）。Bridge の ⌘R は Bridge 内に Camera Raw を出す（ヘルプ記載）が、Pier には内蔵の Camera Raw がないため、raw 現像専用の LightCraft が役割として最も近い。PhotoCraft の Camera Raw に揃える案もある | craft-integration §3 / CRF-05 |
| 4 | PSD・raw デコードのために photocraft リポジトリへ git 依存してよいか | **してよい**（rev 固定）。並行して craft-libs への昇格を上流に提案 | ADR-004 |
| 5 | キャッシュ DB | **redb**（pure Rust） | ADR-003 |
| 6 | Bridge 2026（16.0.8）の実機観察のための権限 | このマシンには Bridge 2026 が入っているが、AI からは画面を撮れず（画面収録の権限なし）、操作もできない（アクセシビリティ／オートメーションの Apple event がタイムアウト）。システム設定 › プライバシーとセキュリティで、Claude を動かしているアプリ（VS Code / ターミナル）に**画面収録**と**アクセシビリティ**を許可し、Bridge と System Events への**オートメーション**を許可すると、`要観察` を実機で潰せる。許可しない場合は公開ヘルプとユーザーの観察で進める | ui-design.md / skill parity-observation |
