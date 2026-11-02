# デバッグ環境（AI・人の両方向け）

> **最終更新:** 2026-11-03

Pier の UI は、ウィンドウを出さずに**本物の入力イベント**（マウス移動・クリック・ドラッグ・スクロール・キー）で動かし、
結果を **PNG（見た目）** と **JSON（状態）** で受け取れます。AI エージェントはこれで「操作 → 観察 → 判断」を回します。

| 層 | 何か | 使いどころ |
|---|---|---|
| 1. シナリオテスト | `crates/ui-driver/tests/scenarios.rs`（Rust、`Driver` API） | 直したバグのセンチネル、CI で常時回す操作の回帰テスト |
| 2. スクリプト駆動 | `pier-drive`（JSON の手順を実行） | その場の再現・調査。Rust を書かずに試せる |
| 3. 実アプリの遠隔操作 | `pier --control`（予定、photocraft と同じプロトコル） | 実ウィンドウ・OS 固有の問題（ネイティブメニュー、IME、DPI） |

1・2 は同じ `pier-ui-driver` クレートの上にあり、アプリ本体（`PierShell`）をそのまま egui_kittest で動かします。
時間は実アプリと同じ 60Hz で進めます（kittest 既定の 0.25 秒/フレームではダブルクリックが成立しないため）。

## pier-drive

```sh
cargo run -p pier-ui-driver --bin pier-drive -- --steps '[
  {"op": "markLayout"},
  {"op": "wiggle", "from": [20, 500], "to": [1420, 500], "steps": 80},
  {"op": "assertLayoutStable"},
  {"op": "click", "label": "aurora-03.jpg"},
  {"op": "key", "key": "5", "mods": ["cmd"]},
  {"op": "screenshot", "path": "after.png"},
  {"op": "state", "path": "after.json"}
]'
```

- オプション：`--script <file>` / `--steps <json>`、`--dir <folder>`（省略時は自作の風景画で作るデモフォルダー）、
  `--out-dir <dir>`（既定 `target/pier-drive`）、`--workspace <id>`、`--lang en|ja`、`--theme darkest|dark|light|lightest`、
  `--size 1440x900`、`--scale 1`。
- 最後に状態 JSON を出力します。失敗した手順があれば `FAILED step N (...)` を出して終了コード 1。

### 手順（op）

| op | 引数 | 内容 |
|---|---|---|
| `move` | `to` | マウスを移動して 1 フレーム |
| `wiggle` | `from`, `to`, `steps` | マウスを `steps` フレームかけて移動（ホバー・再描画の連続） |
| `click` / `doubleClick` / `rightClick` | `at` か `label`（`button`, `mods`） | クリック。`label` はアクセシビリティツリーの名前（完全一致→部分一致） |
| `drag` | `from`, `to`, `steps` | 押してから移動して離す |
| `scroll` | `at`, `dy`, `dx` | ホイール |
| `key` | `key`, `mods` | キー（egui のキー名：`A`, `Space`, `ArrowDown`, `F5`, `5`）。`mods`：`cmd`（mac は ⌘、他は Ctrl）, `ctrl`, `alt`, `shift` |
| `type` | `text` | 文字入力 |
| `waitIdle` | — | フォルダー読み込み・デコードが終わるまで待つ |
| `frames` | `n` | フレームだけ進める |
| `resize` | `size` | ウィンドウサイズ変更 |
| `screenshot` | `path` | PNG を保存 |
| `state` | `path?` | 状態 JSON（下記） |
| `widgets` | `path?`, `contains?` | クリックできる名前付きウィジェットの一覧（役割・名前・矩形） |
| `markLayout` / `assertLayoutStable` | `tolerance?` | パネル配置を記録 / 変わっていないことを検査 |
| `assert` | `path`, `equals` | 状態 JSON の JSON Pointer（例 `/workspace`）が値と一致することを検査 |

### 状態 JSON

`workspace`、`view`、`folder`、`loading`、`visible`（表示中のファイル名）、`selection`、`dialog`、`fullScreen`、`toast`、
`thumbSize`、`panelsHidden`、`pointer`、`groups`（各パネルグループの `id`・`panels`・`active`・`rect [x, y, w, h]`）。

## AI が調査するときの流れ

1. `widgets` で押せるものと位置を知る（ラベルで押せば座標を推測しなくてよい）。
2. 症状を `pier-drive` の手順で再現し、`state` / `screenshot` / `assert*` で事実として確認する。
3. 再現手順を `tests/scenarios.rs` のテストに移してセンチネルにする（修正前に失敗、修正後に成功、修正を戻すと失敗）。
4. 直してから同じ手順を流し、PNG を目で確認する（skill ui-verify）。

## 新しい UI を作るときの約束

- **クリックできるものにはアクセシブル名を付ける**（`resp.widget_info(|| WidgetInfo::labeled(...))`）。名前がないと
  AI もスクリーンリーダーも押せない。kittest は名前のないウィジェットや、フォントにない文字を見つけると失敗する。
- **状態は `PierShell::probe()` / `state()` に出す**。目で見るしかない状態は AI が判定できない。
- 表示言語はスレッドごと（`i18n::set_lang`）。テストは並列に走るので、プロセス全体の状態を持たない。

## 実例：マウスを動かすとパネルが動く（2026-11-03 修正）

- 再現：`markLayout` → `wiggle` → `assertLayoutStable` で、左右の列の上段パネルの高さが縮んだ。
- 切り分け：マウスを動かさず `frames` だけでも 1 フレーム 1pt ずつ縮んだので、原因は毎フレームの描画だった（実アプリではマウスを動かすたびに再描画されるため、動いて見えた）。
- 真因：パネルグループが与えられた領域を 1pt 縮めてから確保していたため、サイズ可変パネルが毎フレーム小さい値を覚え直していた。
- センチネル：`panels_stay_put_while_the_mouse_moves`（全ワークスペースでマウスを動かしても配置が変わらない）。
