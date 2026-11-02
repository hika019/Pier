# エージェント運用
- lead はオーケストレーションと fact 化に専念する。実装・調査・レビュー・Bridge 観察・コーパス取得は subagent へ委譲する (background 起動)。1 task = 1 subagent。完了した subagent は終了し、追加前に既存 subagent の要否を確認する。
- モデル選択: テスト実行/ダウンロード/集計 = haiku。実装/バグ調査/リファクタ/テスト修正/レビュー = sonnet。設計判断・難度の高い検証・重要レビュー・UI パリティ判定 = opus。
- subagent には次を明示して渡す: 作業ディレクトリ (git worktree)、完了条件、報告に必ず含める実測値 (テスト PASS/FAIL 数、コミット hash、性能計測値、スナップショット PNG のパス)。
- subagent の完了検知は lead の責務。ユーザに監視や通知をさせない。状況を聞かれたら推測で答えず、実状態 (タスク一覧/git) を確認してから答える。
- subagent の報告は推論として扱う。lead がテスト実走・grep・実測値・PNG の目視で fact 化してから確定する。
- テスト/リファクタ/docs/cleanup は最初から 1 つの統合 task で起票する。不要になった task は廃止し、pending は少なく保つ。
- ユーザの判断が必要な事項のみ `dashboard.md` に記載し、レコメンドを併記する。空ファイルが正常。それ以外は lead が自律決定する。

# 対話
- 質問だけのメッセージには回答のみ返す。コード変更は明示的な指示を受けてから行う。質問と指示が混在したメッセージは分解し、それぞれ独立に扱う。
- 状況報告は ①完了 ②進行中 (subagent と内容) ③判断待ち ④次アクション を数行で。
- 情報が不足・曖昧なとき、勝手に補完して進めない。実測・観察で埋められるなら埋め、埋められないなら「何が不足か」を報告する。
- Bridge の挙動が不明なときは推測で実装しない。`要観察` として docs に残し、skill parity-observation で確定させる。
- 手戻り最小化が作業順序の指標。共有根 (engine・catalog・meta) から先に、correctness をリファクタより先に。
- 引き継ぎメモ・docs・過去の報告は観測者の推論を含む。事実 (code / git / テスト / 実機観察) で再検証してから方針として採用する。
- セッション中断からの再開・引き継ぎ → skill session-recovery を読んでから着手。

# git
- main へのコミット/マージはユーザの明示許可があるときのみ。通常はブランチを切り、integrate 系ブランチへ `--no-ff` でマージする。
- push は毎回ユーザに確認する。push 済みコミットは amend/rebase/squash せず、新コミットで修正する。
- コミットメッセージは日本語 (conventional prefix 可)。コミット前に `git status` で stage 内容を確認し、意図した成果物だけを stage する (`git add -A` を無確認で使わない。target/・corpus/・plan/ を含めない)。
- 1 branch = 1-2 commit が理想。
- subagent は git worktree を使用し、`CARGO_TARGET_DIR=target/agent-<name>` を使う。worktree 削除前に `git status --untracked-files=all` で未コミット成果物を確認する。
- push 前ゲート: `cargo fmt --all -- --check` / `cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo xtask layers` / `cargo nextest run --workspace`。コマンド追加時は `cargo xtask parity` と panic_hunt も。

# 実装
- 判断に困ったら本来あるべき形を考えて選択する。ユーザの意見にも安易に迎合しない。ただし UI のレイアウト・文言・ショートカットは「Bridge と同じ」が本来あるべき形。
- 症状隠し・それっぽい挙動での糊塗は修正ではない — 真因が未特定なら手を入れず、調査を続ける (skill bug-frontier)。
- 後方互換のために旧実装を残さない (ただしユーザーデータ・設定ファイルの移行は必ず書く)。
- マジックナンバーは根拠つきの named const にする。
- コメントは OSS 水準で少なめ。メモ書き・作業ログ的コメントは書かない。
- 指摘を受けたら、同じパターンを repo 全体へ横展開する: grep で全該当箇所を列挙してから直す。
- 部分実装 + 残課題のドキュメント化は「未完了」。完了条件 (テスト・レビュー・実測報告) を満たすまで task は in_progress のまま。
- 1 ファイル 2500 行超 (テストコード除く) で責任分割を確認する。
- 規約の詳細 (落ちない・ファイルを壊さない・すべてはコマンド・クリーンルーム) は AGENTS.md。

# テスト
- テスト駆動開発を実施する。
- 既存テストのアサーション/ロジックは変更しない。挙動変更は旧テストの削除 + 新テストの追加として表現する。
- 実装を正とせず、独立オラクル (仕様書の手計算、exiftool 等の別ツール、Bridge 実機の出力) で期待値を作る。
- 修正には sentinel test を付ける: 修正を revert すると fail することを確認済みであること。
- 1 テストの実行時間は 3 分以内。nextest を使用する (ローカル 6 / CI 3 スレッド)。
- UI 変更はスナップショット PNG を目視確認する (skill ui-verify)。

# 性能
- 性能の主張には実測値が必須 (`cargo xtask perf` の数値、データ件数、機種)。数値のない「速くなった」は主張として受け付けない。
- 予算は docs/requirements.md の NFR-P。

# レビュー
- すべてのアクション (実装・docs・設定を含む) にレビューが必要。実施者とレビュアーは別 subagent。
- レビューはバグが有る前提・批判的観点で行う。見つからなければ「見つけられなかった」と明記し、何を検証したかを列挙する。
- 対象は実装箇所に限らない: 設計、前後処理との整合、テストの質、Bridge とのパリティ (レイアウト・文言・ショートカット) も見る。
- 重大度: P0=blocker (データ破損/クラッシュ/build 失敗)、P1=機能バグ (Bridge と挙動が違う/未検証主張)、P2=中程度 (sentinel 不足/CI 漏れ)、P3=polish。
- reviewer follow-up は同 branch 内 commit に集約する。P0/P1 のみ別 task 可。

# skills (該当作業の開始前に必ず読む)
- バグ調査・修正・クラッシュ → bug-frontier
- セッション中断からの復旧・引き継ぎ → session-recovery
- Bridge の挙動・見た目の観察と docs 反映 → parity-observation
- UI の実装・変更の検証 → ui-verify

# このファイル
- CLAUDE.md の編集はユーザの明示許可があるときのみ。
