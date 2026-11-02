# Craft Apps との連携

> **最終更新:** 2026-11-03

Pier は Photoshop などの Adobe 製品の代わりに、[storytold の Craft Apps](https://github.com/storytold) が入っている前提で動きます。
入っていない場合は [ArtCraft Launcher](https://github.com/storytold/craft-launcher) でのインストールへ誘導します。
Craft Apps がなくても Pier の閲覧・整理・メタデータ機能はすべて使えます（連携機能だけが無効になる）。

## 1. 対象アプリ

ArtCraft Launcher のカタログ（`craft-launcher/crates/core/src/catalog.rs`）を正とし、Pier はそのうち次を扱います。

| アプリ（machine name） | 開く形式（既定の関連付け） | Pier からの連携 |
|---|---|---|
| PhotoCraft（`photocraft`） | JPEG、PNG、TIFF、WebP、GIF、BMP、raw 全般（DNG を含む）、PSD、PSB、`.pcraft`、ORA、KRA、XCF、Affinity Photo（EXR は PhotoCraft が開けるが、Pier はまだ種類を判定せず OS の既定アプリで開く） | 開く（ダブルクリック・⌘O）、ツール › PhotoCraft、CLI によるサムネイル・書き出し |
| LightCraft（`lightcraft`） | —（ダブルクリックの既定にはしない） | Camera Raw で開く（⌘R、dashboard #3）、カタログに読み込み、DNG 変換 |
| VectorCraft（`vectorcraft`） | AI、SVG、EPS、`.vectorcraft` | 開く、画像トレース |
| DesignCraft（`designcraft`） | INDD、IDML、`.designcraft`（要確認） | 開く、配置、コンタクトシート |
| PdfCraft（`pdfcraft`） | PDF | 開く |
| FilmCraft（`filmcraft`） | MP4、MOV、MKV ほか動画 | 開く、サムネイル（CLI） |
| EffectCraft（`effectcraft`） | `.ecproj`、AEP（読めるなら） | 開く |
| SoundCraft（`soundcraft`） | WAV、AIFF、MP3、FLAC ほか | 開く |
| TextureCraft / MaterialCraft | テクスチャ・マテリアル形式 | 開く |
| WordCraft / GridCraft / DeckCraft | DOCX / XLSX / PPTX ほか | 開く |
| AniCraft / FigCraft / CADCraft | 各形式 | 開く |

### 既定の関連付け（ダブルクリック・ファイル › 開く）

Bridge は OS の関連付けに従わず、環境設定 › ファイルの種類の関連付けに自前の表を持つ（ヘルプ記載、実機未確認）。
Pier も自前の表を持ち、Bridge の行き先を Craft App に置き換える。

| 形式 | Bridge | Pier | 根拠 |
|---|---|---|---|
| JPEG・PNG・TIFF など | Photoshop | PhotoCraft | ユーザーの実機観察（2026-11-03, Bridge 16.0.8 macOS）、ヘルプ「ネイティブアプリで開く」 |
| raw（CR3・NEF・DNG など） | Photoshop が起動し、Camera Raw ダイアログを経て開く | PhotoCraft。PhotoCraft も raw を開くと先に Camera Raw ダイアログを出す（`camera_raw_ui::open`、環境設定 `open_in_camera_raw`、センサーデータから現像できた raw のとき） | ユーザーの実機観察（同上）。Bridge 内の Camera Raw で開く設定（「ダブルクリックで Bridge の Camera Raw 設定を編集」、既定オフ）はヘルプ外の情報で要観察 |
| PSD・PSB | Photoshop | PhotoCraft | ヘルプ「ネイティブアプリで開く」（PSD のネイティブアプリは Photoshop）。実機未確認 |
| SVG・AI・EPS / INDD・IDML / PDF / 動画 / 音声 | 要観察（既定表の公式資料なし） | VectorCraft / DesignCraft / PdfCraft / FilmCraft / SoundCraft（暫定） | — |
| フォント・書類・その他 | 要観察 | OS の既定アプリ | — |

- 複数のアプリが未インストールのときは、アプリごとに §4 の誘導を順に出す（黙ってファイルを捨てない）。
- 実装は `pier_engine::craft::app_for`。テストは `crates/ui-driver/tests/scenarios.rs`（`double_clicking_a_raw_opens_it_in_photocraft` ほか）。
- 以前の版は「Bridge も OS の関連付けに従う」という誤った前提で、汎用画像を OS の既定アプリ、raw を LightCraft で開いていた。

## 2. 検出

ArtCraft Launcher の `crates/native/src/detect.rs` と**同じ探索順序**で探す（Launcher で入れたものも手で入れたものも見つかる）。

| OS | 探索場所 |
|---|---|
| macOS | `/Applications/<App>.app`、`~/Applications/<App>.app`（`Info.plist` の `CFBundleIdentifier` = `ai.storyteller.<app>` とバージョンを読む） |
| Windows | Windows Installer のアンインストール情報（発行元 `Learning Machines LLC`）、`C:\Program Files\ArtCraft\<App>`、`%LOCALAPPDATA%\Programs\ArtCraft\<App>`、Launcher のポータブル展開先 |
| Linux | `/usr/bin/<app>`（.deb / .rpm）、Flatpak（`ai.storyteller.<app>`）、AppImage（`~/Applications`、`~/AppImages`、`~/.local/bin`）、tarball 展開先、Launcher の Flatpak サンドボックス内 |
| FreeBSD | `/usr/local/bin/<app>`、`~/.local` |

- 検出は起動時にバックグラウンドで 1 回、以降は「最新の情報に更新」と、Craft App 起動に失敗したときに再実行する。
- 検出結果（アプリ、バージョン、実行ファイル、CLI の有無とパス、取得元）をキャッシュ DB に保存する。
- Launcher のコード（`artcraft-launcher-native`）を直接依存にするか、探索ロジックを craft-libs に切り出す提案をするかは ADR-009。
  当面は同じ順序を Pier 側で実装し、Launcher のテストケースを参考にテストを書く。
- 環境設定 › ファイルの種類の関連付けで、各アプリの実行ファイルを手動で指定することもできる。

## 3. 起動とファイルの受け渡し

| 動作 | 方法 |
|---|---|
| 開く | 実行ファイルにファイルパスを引数で渡して起動（macOS は `open -b ai.storyteller.<app> <files>` 相当を LaunchServices 経由で）。Craft App 側の単一インスタンス機構（photocraft の `single_instance.rs`）が既存ウィンドウのタブとして開く |
| 複数ファイル | 1 回の起動で全パスを渡す（引数長の上限を超える場合は分割） |
| Camera Raw で開く（⌘R） | Bridge は Bridge 内に Camera Raw を出す（ヘルプ記載、実機未確認）。Pier には内蔵の Camera Raw がないため LightCraft にパスを渡す（dashboard #3）。ダブルクリック・⌘O とは別のコマンド |
| ツール › PhotoCraft › バッチ | `photocraft-cli batch --actions <file> --in <dir> --out <dir>` を Pier のジョブとして実行。選択ファイルは一時フォルダーへのリンクで渡す（要検討：PhotoCraft にファイルリスト入力を追加する上流 PR） |
| ツール › PhotoCraft › 画像プロセッサー | `photocraft-cli convert` を各ファイルに実行（形式・サイズ・画質を Pier のダイアログで指定） |
| その他のツール | PhotoCraft 等にコマンドが存在する範囲で、CLI またはコントロールチャネル（`--control`）で呼ぶ。存在しないものはメニューを無効表示にし、ツールチップで理由を示す |

- 引数はシェルを介さず配列で渡す。パスは絶対パスに正規化する。
- 起動失敗（実行ファイルが消えた、権限がない）は検出をやり直し、それでも失敗したら §4 の誘導へ。

## 4. 未インストール時の誘導

Craft App が必要な操作で見つからないとき、次のダイアログを出します。

```text
┌────────────────────────────────────────────────────────┐
│ [icon] このファイルを開くには PhotoCraft が必要です          │
│                                                        │
│ PhotoCraft は PSD などのレイヤー画像を編集できる、無料で     │
│ オープンソースの画像編集アプリです。                         │
│                                                        │
│ ArtCraft Launcher を使うと、PhotoCraft などの Craft Apps    │
│ をワンクリックでインストール・更新できます。                  │
│                                                        │
│ [ArtCraft Launcher を開く]  ← Launcher 検出済みのとき        │
│ [ArtCraft Launcher をダウンロード]  ← 未検出のとき（ブラウザ）│
│ [PhotoCraft のページを開く]   [OS の既定アプリで開く]  [キャンセル] │
│ ☐ 今後この種類のファイルは OS の既定アプリで開く              │
└────────────────────────────────────────────────────────┘
```

- **ArtCraft Launcher を開く**：Launcher が入っていれば起動する。Launcher にアプリを指定してインストール画面を開く
  起動引数・URL スキームは現時点で確認できていない（要確認）。存在しなければ Launcher を起動するだけにし、上流に提案する（ADR-009）。
- **ダウンロード**：`https://github.com/storytold/craft-launcher/releases/latest` を既定ブラウザで開く。
- **アプリのページ**：`https://getartcraft.com/apps/<app>`（PhotoCraft の README にあるリンク形式）または GitHub Releases。
- Pier 自身はダウンロード・インストールを行わない（Launcher の責務。署名検証・パッケージ形式の扱いを重複実装しない）。
- ネットワークにアクセスするのはユーザーがボタンを押したときだけ。
- ヘルプ › Craft Apps を入手 からも同じ画面（全アプリの検出状況一覧付き）を開ける。

## 5. ブーメラン（Craft App → Pier）

Bridge の「Photoshop に戻る」と、Photoshop 側の「Bridge で参照」に相当する往復。

- **Pier → 起動元**：Craft App から `pier --from <app> <path>` で起動された場合、アプリケーションバーに「↩ <App> に戻る」を表示し、
  押したらその Craft App を前面に出す。
- **Craft App → Pier**：各 Craft App の「ファイル › Pier で参照」を追加する上流 PR が必要（P2）。Pier は単一インスタンスで
  パスを受け取り、そのフォルダーを開いて該当ファイルを選択する。
- 上流への PR は各リポジトリの AGENTS.md（クリーンルーム・テスト・CI 規則）に従う。

## 6. 将来の連携候補

| 候補 | 内容 |
|---|---|
| AI 連携 | `craft-ai-integrations` / `craft-ai-core` による自動キーワード付け・類似画像検索（ローカル優先、同意制） |
| 色設定の同期 | Craft Apps 間でカラー設定を共有（Bridge の「カラー設定」同期に相当） |
| フォント | Craft Font Manager（`craft-font-manager`）との連携 |
| Launcher への掲載 | Pier を ArtCraft Launcher のカタログ（コミュニティアプリ枠：SolidCraft と同様）に載せる |
