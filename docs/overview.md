# 製品概要

> **最終更新:** 2026-11-02 · **対象:** Adobe Bridge 2026（観察による参照） · **状態:** 設計段階

## 1. Pier とは

Pier はクリエイティブ素材（写真、raw、PSD、ベクター、レイアウト、PDF、動画、音声、フォント）を
**フォルダ単位で閲覧・選別・整理・受け渡し**するためのデスクトップアプリです。Adobe Bridge が Creative Cloud の
ハブとして果たしている役割を、storytold の Craft Apps に対して果たします。

```text
              ┌───────────────── Pier ─────────────────┐
  カメラ/カード ─▶│ 取り込み → 閲覧 → 選別 → メタデータ → 出力 │──▶ PDF コンタクトシート / 書き出し
              └──────┬──────────┬──────────┬───────────┘
                     │ 開く/渡す │          │
           PhotoCraft  LightCraft  VectorCraft  DesignCraft  PdfCraft  FilmCraft …
```

## 2. 目的

| # | 目的 | 測り方 |
|---|---|---|
| G1 | Adobe Bridge のユーザーが学び直しなしで使える | メニュー・ショートカット・パネルの一致率（[command-catalog.md](command-catalog.md) のパリティ率） |
| G2 | Bridge より速い | 1 万ファイルのフォルダ表示・スクロール・フィルターの性能予算（[requirements.md](requirements.md) NFR-P） |
| G3 | ユーザーのファイルを絶対に壊さない | 書き込み系操作のアトミック性・Undo 可能性、破損ファイル投入時の非クラッシュ |
| G4 | Craft Apps との往復が自然 | 「開く」「Craft App で処理」「戻る」が 1 操作で完結 |
| G5 | エージェントから全機能を操作できる | 全コマンドが CLI / MCP から実行可能 |

## 3. 想定ユーザー

- **写真家**：カードから取り込み、レーティングで選別、キーワード・IPTC を付けて納品。
- **デザイナー**：PSD・ベクター・レイアウト・フォントを横断して素材を探し、Craft App で開く。
- **映像制作者**：動画・音声クリップのプレビュー、メタデータ確認、コレクション化。
- **エージェント / 自動化**：CLI・MCP で一括リネーム、メタデータ付与、キャッシュ生成。

## 4. スコープ

### 4.1 対象（Bridge の機能を再現する）

- フォルダ・お気に入り・パスバーによるナビゲーション
- コンテンツパネル（サムネイル／詳細／リスト表示、並べ替え、グリッドロック）
- プレビューパネル（ルーペ、複数比較、動画・音声再生）
- フルスクリーンプレビュー、レビューモード、スライドショー
- レーティング（★0〜5、却下）、ラベル（5 色＋なし）
- メタデータパネル（ファイルプロパティ、IPTC Core / Extension、カメラデータ EXIF、GPS、音声、動画、Camera Raw 設定、編集履歴）
- ファイル情報ダイアログ、メタデータテンプレート（作成・追加・置換）
- キーワードパネル（階層キーワード、検索、読み込み／書き出し）
- フィルターパネル、検索（クイック検索・詳細検索）
- コレクション、スマートコレクション
- スタック（手動、パノラマ／HDR の自動スタック）
- 一括リネーム（プリセット、プレビュー、元ファイル名の保持）
- 写真のダウンロード（カメラ・カードからの取り込み、リネーム、DNG 変換は Craft App に委譲）
- 書き出しパネル（JPEG / PNG / TIFF などへのサイズ指定書き出し）
- 出力（PDF コンタクトシート）
- キャッシュ管理（構築・書き出し・パージ）
- ワークスペース（Essentials、Filmstrip、Metadata、Keywords、Preview、Light Table、Folders、Output）
- ツールメニュー経由の Craft App 処理（Bridge の「ツール › Photoshop」相当を PhotoCraft で）

### 4.2 対象外

| 対象外 | 理由 / 代替 |
|---|---|
| 画素編集・現像 | Craft Apps（PhotoCraft、LightCraft）に委譲 |
| Bridge 内蔵の Camera Raw ホスティング | LightCraft（または PhotoCraft の Camera Raw フィルター）で開く |
| Creative Cloud ライブラリ、Adobe Stock への公開、Adobe アカウント連携 | Adobe のサービスに依存するため。将来の拡張点として残す |
| Adobe Firefly / Sensei などの AI クイックアクション | 初期対象外。将来 `craft-ai-integrations` 経由で検討（[decisions.md](decisions.md) ADR-010） |
| ExtendScript（JavaScript）による Bridge スクリプト | JS を使わない方針。代わりに JSON コントロールチャネル・MCP・CLI |
| Web 版 | ローカルファイルシステムが前提のアプリのため（[decisions.md](decisions.md) ADR-007） |
| Craft Apps 自体のインストーラー | ArtCraft Launcher へ誘導する（[craft-integration.md](craft-integration.md)） |

## 5. Adobe 製品と Craft Apps の対応

Bridge が Adobe 製品へ受け渡す箇所は、次の Craft App に置き換えます。

| Bridge での連携先 | Pier での連携先 | 主な用途 |
|---|---|---|
| Photoshop | **PhotoCraft** | PSD・画像を開く、バッチ、画像プロセッサー、レイヤーとして読み込み |
| Camera Raw | **LightCraft**（既定）／PhotoCraft の Camera Raw | raw 現像 |
| Lightroom | **LightCraft** | カタログへの読み込み |
| Illustrator | **VectorCraft** | AI / SVG / EPS を開く、画像トレース |
| InDesign | **DesignCraft** | INDD / IDML を開く、コンタクトシート |
| Acrobat | **PdfCraft** | PDF を開く |
| Premiere Pro | **FilmCraft** | 動画を開く |
| After Effects | **EffectCraft** | コンポジション素材を開く |
| Audition | **SoundCraft** | 音声を開く |
| Substance | **TextureCraft / MaterialCraft** | テクスチャ・マテリアルを開く |
| Word / Excel / PowerPoint（OS 既定） | **WordCraft / GridCraft / DeckCraft** | 文書を開く |
| Animate | **AniCraft** | アニメーションを開く |
| XD / Figma | **FigCraft** | デザインファイルを開く |

## 6. 製品名と識別子

| 項目 | 値 | 備考 |
|---|---|---|
| 製品名（表示） | **Pier** | UI、ウィンドウタイトル、About、インストーラー |
| クレート / バイナリ | `pier-*` / `pier`、`pier-cli` | 小文字 |
| アプリ ID | `io.github.hika019.pier`（案） | 逆 DNS。storytold 組織に寄贈する場合は `ai.storyteller.<name>` と `{Function}Craft` 命名に合わせ直す（[decisions.md](decisions.md) ADR-011） |
| 対応 OS | macOS 13+、Windows 10/11、Linux（X11 / Wayland）、FreeBSD（ベストエフォート） | Craft Apps と同じ |

## 7. 用語

| 用語 | 意味 |
|---|---|
| アセット | Pier が扱う 1 ファイル（またはパッケージ：`.app`、`.pcraft` バンドル等はフォルダでも 1 アセット） |
| サムネイル | コンテンツパネル用の縮小画像（長辺 256px 基準） |
| プレビュー | プレビューパネル・フルスクリーン用の画面サイズ画像（モニターサイズ） |
| 100% プレビュー | ルーペ・等倍表示用のフル解像度画像 |
| キャッシュ | サムネイル・プレビュー・メタデータ索引の保存先。集中キャッシュとフォルダ書き出しキャッシュがある |
| サイドカー | raw などに対して元ファイルの隣に置く `.xmp` ファイル |
| スタック | 複数アセットを 1 つのサムネイルにまとめたグループ |
| コレクション | フォルダを横断したアセットの仮想的な集合。スマートコレクションは条件で自動更新される |
| ワークスペース | パネル配置の保存単位 |
| Craft App | storytold の Craft Apps の各アプリ |
| ブーメラン | Craft App から Pier に戻る操作（Bridge の「Photoshop に戻る」相当） |
