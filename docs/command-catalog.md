# コマンドカタログ（メニュー・ショートカット）

> **最終更新:** 2026-11-03 · **対象:** Adobe Bridge 2026 · 状態: `未` / `部分` / `済` / `—`（対象外）

- **この表がメニュー構成・コマンド ID・既定ショートカットの正本**です。実装時は `crates/ui-egui/src/menu_catalog.rs` をこの表から作り、
  `cargo xtask parity` で表と実装の差分（未実装のメニュー項目、ID の不一致）を検出します。
- ショートカットは `Mod` = macOS では ⌘、Windows/Linux では Ctrl。`Alt` = macOS では ⌥。
- `部分` は最初の試作で動くもの（★・ラベルはセッション内のみ、ファイルへは未書き込み）。メニューにある `未` の項目は「今後実装予定」と表示されます。
- 項目名・順序・ショートカットは記憶と公開資料に基づく初版です。`要観察` の項目は実機で確認して更新してください。
- メニュー名の日本語は Bridge 日本語版に合わせる（要観察で確定）。Photoshop 等の製品名は Craft App 名に置き換える。

## ファイル（File）

| メニュー項目 | コマンド ID | ショートカット | 状態 |
|---|---|---|---|
| 新規ウィンドウ | `window.new` | Mod+N | 未 |
| 新規フォルダー | `file.newFolder` | Mod+Shift+N | 未 |
| 開く | `file.open` | Mod+O | 部分 |
| 指定アプリで開く ▸ | `file.openWith` | — | 部分 |
| 最近使用したファイルを開く ▸ | `file.openRecent` | — | 部分 |
| Camera Raw で開く（LightCraft） | `file.openInCameraRaw` | Mod+R | 部分 |
| ウィンドウを閉じる | `window.close` | Mod+W | 部分 |
| 削除（ゴミ箱へ） | `file.trash` | Mod+Delete（macOS）/ Delete（Windows） | 未 |
| 却下ファイルを削除 | `file.trashRejected` | — | 未（要観察） |
| PhotoCraft に戻る（ブーメラン） | `app.returnToCraftApp` | — | 未 |
| Finder で表示 / エクスプローラーで表示 | `file.reveal` | — | 部分 |
| カメラから写真を取り込み… | `file.getPhotosFromCamera` | — | 未 |
| 移動 ▸ | `file.moveTo` | — | 未 |
| コピー ▸ | `file.copyTo` | — | 未 |
| 配置 ▸（DesignCraft / VectorCraft / PhotoCraft） | `file.place` | — | 未（Craft App 側の受け口が必要） |
| お気に入りに追加 | `favorites.add` | — | 未 |
| お気に入りから削除 | `favorites.remove` | — | 未 |
| ファイル情報… | `file.info` | Mod+Alt+Shift+I（要観察：Bridge は Mod+I の可能性） | 未 |
| 終了（Windows） / Pier を終了（macOS アプリメニュー） | `app.quit` | Mod+Q | 部分 |

## 編集（Edit）

| メニュー項目 | コマンド ID | ショートカット | 状態 |
|---|---|---|---|
| 取り消し | `edit.undo` | Mod+Z | 未 |
| やり直し | `edit.redo` | Mod+Shift+Z（Windows は Mod+Y も） | 未 |
| カット | `edit.cut` | Mod+X | 未 |
| コピー | `edit.copy` | Mod+C | 未 |
| ペースト | `edit.paste` | Mod+V | 未 |
| 複製 | `edit.duplicate` | Mod+D | 未 |
| すべてを選択 | `select.all` | Mod+A | 部分 |
| すべてを選択解除 | `select.none` | Mod+Shift+A | 部分 |
| 選択範囲を反転 | `select.invert` | Mod+Shift+I | 部分 |
| ラベル付きを選択 | `select.labeled` | Mod+Alt+L | 部分 |
| ラベルなしを選択 | `select.unlabeled` | Mod+Alt+Shift+L | 部分 |
| 検索… | `search.find` | Mod+F | 未 |
| 現像設定 ▸（LightCraft の設定をコピー／ペースト／クリア） | `edit.developSettings.*` | Mod+Alt+C / Mod+Alt+V（要観察） | 未（LightCraft 側の CLI が必要） |
| 180° 回転 | `edit.rotate.180` | — | 未 |
| 90° 回転（時計回り） | `edit.rotate.cw90` | Mod+] | 未 |
| 90° 回転（反時計回り） | `edit.rotate.ccw90` | Mod+[ | 未 |
| 環境設定…（Windows。macOS はアプリメニュー） | `app.preferences` | Mod+K | 部分 |
| キーボードショートカット… | `app.keyboardShortcuts` | — | 未（P2） |
| カラー設定… | `app.colorSettings` | Mod+Shift+K（要観察） | 未（Craft Apps 間の色設定同期は将来） |

## 表示（View）

| メニュー項目 | コマンド ID | ショートカット | 状態 |
|---|---|---|---|
| フルスクリーンプレビュー | `view.fullScreenPreview` | Space | 部分 |
| スライドショー | `view.slideshow` | Mod+L | 未 |
| スライドショーオプション… | `view.slideshowOptions` | Mod+Shift+L | 未 |
| レビューモード | `view.reviewMode` | Mod+B | 未 |
| サムネイルとして表示 | `view.asThumbnails` | — | 部分 |
| 詳細として表示 | `view.asDetails` | — | 部分 |
| リストとして表示 | `view.asList` | — | 部分 |
| サムネイルのみを表示 | `view.thumbnailOnly` | Mod+T | 未 |
| グリッドロック | `view.gridLock` | — | 未 |
| 却下ファイルを表示 | `view.showRejected` | — | 部分 |
| 隠しファイルを表示 | `view.showHidden` | — | 部分 |
| フォルダーを表示 | `view.showFolders` | — | 部分 |
| サブフォルダーの項目を表示 | `view.showSubfolderItems` | — | 未 |
| 並べ替え ▸ | `view.sort` | — | 部分 |
| 最新の情報に更新 | `view.refresh` | F5 | 部分 |
| サムネイルを大きく / 小さく | `view.thumbSize.inc` / `.dec` | Mod+= / Mod+- | 部分 |

## スタック（Stacks）

| メニュー項目 | コマンド ID | ショートカット | 状態 |
|---|---|---|---|
| スタックとしてグループ化 | `stack.group` | Mod+G | 未 |
| スタックを解除 | `stack.ungroup` | Mod+Shift+G | 未 |
| スタックを開く | `stack.open` | Mod+→ | 未 |
| スタックを閉じる | `stack.close` | Mod+← | 未 |
| スタックの先頭へ | `stack.promoteToTop` | — | 未 |
| すべてのスタックを展開 | `stack.expandAll` | Mod+Alt+→ | 未 |
| すべてのスタックを折りたたむ | `stack.collapseAll` | Mod+Alt+← | 未 |
| フレームレート ▸ | `stack.frameRate` | — | 未 |
| パノラマ／HDR の自動スタック | `stack.autoStack` | — | 未 |

## ラベル（Label）

| メニュー項目 | コマンド ID | ショートカット | 状態 |
|---|---|---|---|
| レーティングなし | `label.rating.set {rating:0}` | Mod+0 | 部分 |
| 却下 | `label.rating.set {rating:-1}` | Alt+Delete | 部分 |
| ★ 〜 ★★★★★ | `label.rating.set {rating:1..5}` | Mod+1 〜 Mod+5 | 部分 |
| レーティングを下げる | `label.rating.dec` | Mod+, | 部分 |
| レーティングを上げる | `label.rating.inc` | Mod+. | 部分 |
| ラベルなし | `label.color.set {label:null}` | — | 部分 |
| セレクト | `label.color.set {label:"select"}` | Mod+6 | 部分 |
| セカンド | `label.color.set {label:"second"}` | Mod+7 | 部分 |
| 承認 | `label.color.set {label:"approved"}` | Mod+8 | 部分 |
| レビュー | `label.color.set {label:"review"}` | Mod+9 | 部分 |
| To Do | `label.color.set {label:"todo"}` | — | 部分 |

- 環境設定 › ラベルで「ラベルの適用に Ctrl/⌘ キーを必要とする」をオフにすると、修飾キーなしの数字キーでも動作（RAT-04）。
- フルスクリーンプレビュー・レビューモード中は修飾キーなしの数字キーで★（要観察）。

## ツール（Tools）

| メニュー項目 | コマンド ID | ショートカット | 状態 |
|---|---|---|---|
| 一括リネーム… | `tools.batchRename` | Mod+Shift+R | 未 |
| メタデータテンプレートを作成… | `meta.template.create` | — | 未 |
| メタデータテンプレートを編集 ▸ | `meta.template.edit` | — | 未 |
| メタデータを追加 ▸ | `meta.template.append` | — | 未 |
| メタデータを置換 ▸ | `meta.template.replace` | — | 未 |
| キャッシュ ▸ キャッシュの構築と書き出し… | `cache.buildAndExport` | — | 未 |
| キャッシュ ▸ フォルダー「…」のキャッシュをパージ | `cache.purgeFolder` | — | 未 |
| キャッシュ ▸ キャッシュを管理…（要観察） | `cache.manage` | — | 未 |
| PhotoCraft ▸ バッチ… | `craft.photocraft.batch` | — | 未 |
| PhotoCraft ▸ 画像プロセッサー… | `craft.photocraft.imageProcessor` | — | 未 |
| PhotoCraft ▸ ファイルを PhotoCraft のレイヤーとして読み込み | `craft.photocraft.loadAsLayers` | — | 未（PhotoCraft 側要確認） |
| PhotoCraft ▸ HDR Pro に統合… | `craft.photocraft.mergeHdr` | — | 未（PhotoCraft 側要確認） |
| PhotoCraft ▸ Photomerge… | `craft.photocraft.photomerge` | — | 未（PhotoCraft 側要確認） |
| PhotoCraft ▸ レンズ補正… | `craft.photocraft.lensCorrection` | — | 未（PhotoCraft 側要確認） |
| PhotoCraft ▸ コレクションを PhotoCraft で処理 | `craft.photocraft.processCollections` | — | 未 |
| VectorCraft ▸ 画像トレース… | `craft.vectorcraft.imageTrace` | — | 未（VectorCraft 側要確認） |
| DesignCraft ▸ コンタクトシートを作成… | `craft.designcraft.contactSheet` | — | 未（DesignCraft 側要確認） |
| LightCraft ▸ LightCraft に読み込み | `craft.lightcraft.import` | — | 未 |
| LightCraft ▸ DNG に変換… | `craft.lightcraft.convertToDng` | — | 未（LightCraft 側要確認） |

## ウィンドウ（Window）

| メニュー項目 | コマンド ID | ショートカット | 状態 |
|---|---|---|---|
| ワークスペース ▸ 必須 … 出力（上位 6 つ） | `workspace.select {id}` | Mod+F1 〜 Mod+F6 | 部分 |
| ワークスペース ▸ 新規ワークスペース… | `workspace.save` | — | 未 |
| ワークスペース ▸ ワークスペースを削除… | `workspace.delete` | — | 未 |
| ワークスペース ▸ ワークスペースを初期化 | `workspace.reset` | — | 部分 |
| ワークスペース ▸ 標準ワークスペースをリセット | `workspace.resetAll` | — | 部分 |
| お気に入りパネル / フォルダーパネル / コンテンツパネル / プレビューパネル / メタデータパネル / キーワードパネル / フィルターパネル / コレクションパネル / 書き出しパネル / 出力パネル / 出力プレビューパネル | `panel.toggle {id}` | — | 未 |
| パスバー | `panel.toggle {id:"pathBar"}` | — | 未 |
| 最小化（macOS） | `window.minimize` | Mod+M | 未 |
| パネルをすべて隠す／表示 | `panel.toggleAll` | Tab | 未 |

## ヘルプ（Help）

| メニュー項目 | コマンド ID | ショートカット | 状態 |
|---|---|---|---|
| Pier ヘルプ | `help.open` | F1 | 未 |
| キーボードショートカット一覧 | `help.shortcuts` | — | 未 |
| Craft Apps を入手（ArtCraft Launcher） | `help.getCraftApps` | — | 未（Pier 独自） |
| アップデートを確認… | `help.checkForUpdates` | — | 未 |
| システム情報 | `help.systemInfo` | — | 未 |
| Pier について（macOS はアプリメニュー） | `help.about` | — | 部分 |

## ナビゲーション・選択（メニューなし、ショートカットのみ）

| 操作 | コマンド ID | ショートカット |
|---|---|---|
| 戻る / 進む | `browser.back` / `browser.forward` | 要観察（Mod+[ / Mod+] は回転に割り当て済みのため使わない） |
| 親フォルダーへ | `browser.up` | Mod+↑ |
| フォルダーに入る／開く | `file.open` | Mod+↓ / Enter |
| 選択を移動 | `select.move {dir}` | ←↑→↓、Shift で範囲拡張 |
| 先頭／末尾 | `select.first` / `select.last` | Home / End |
| 名前の変更 | `file.rename` | F2（Windows）/ Enter（macOS） |
| Craft Apps のインストール誘導を表示 | `craft.showInstallGuide {app}` | — |

## パリティ集計

| 区分 | 項目数 | 済 | 部分 | 未 | 対象外 |
|---|---|---|---|---|---|
| メニュー項目 | （`cargo xtask parity` が生成） | 0 | 0 | — | — |
