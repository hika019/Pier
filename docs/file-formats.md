# 対応フォーマットとサムネイル／プレビュー生成

> **最終更新:** 2026-11-02

## 1. 生成戦略

サムネイル（長辺 256px）とプレビュー（モニターサイズ、既定は長辺 2048px と表示領域の物理ピクセルの大きい方）は、
次の順に**最初に成功した方法**で作ります。

1. **キャッシュ**（`(size, mtime, file_id)` が一致し、品質設定を満たすもの）
2. **埋め込み画像**（EXIF サムネイル、raw の埋め込みプレビュー JPEG、PSD のサムネイルリソース 1036 / 合成画像、HEIC のサムネイル、PDF/INDD のページプレビュー）
3. **縮小デコード**（JPEG の DCT スケーリング 1/2・1/4・1/8、raw の半分解像度デモザイク、PDF の低解像度ラスタライズ）
4. **フルデコード → 縮小**（Lanczos3、色変換は縮小後）
5. **Craft App の CLI に委譲**（インストール済みの場合のみ。§3）
6. **汎用アイコン**（形式ごとのアイコン。失敗理由は DB に記録）

- サムネイル品質の設定「埋め込みを優先（高速）」では 2 で止める。「常に高品質」では 2 をスキップして 3 以降。
- 埋め込み画像が実寸より明らかに小さい（例：160×120 の EXIF サムネイル）場合、プレビューには使わない。
- 向き（EXIF / XMP Orientation）は生成時に適用し、キャッシュは回転後の画像を保存する。回転を変えたらキャッシュを無効化する。
- 色：埋め込み ICC → sRGB（キャッシュ保存時）→ ディスプレイプロファイル（表示時、GPU シェーダー or `craft-cms`）。
  広色域ディスプレイではキャッシュを Display P3 で持つ選択肢を検討（ADR 候補）。

## 2. 形式別

| 種類 | 形式 | サムネイル／プレビュー | メタデータ読み | メタデータ書き | 優先 |
|---|---|---|---|---|---|
| 画像 | JPEG | `zune-jpeg`（DCT スケーリング）、12bit・算術符号は `craft-jpeg` | EXIF / XMP / IPTC / ICC | 埋め込み | P0 |
| 画像 | PNG、GIF、BMP、WebP、TIFF（マルチページ含む） | `image` クレート（default-features off、必要な形式のみ） | XMP / EXIF（TIFF・PNG eXIf・WebP） | 埋め込み（GIF・BMP はサイドカー不可 → CacheOnly） | P0 |
| 画像 | HEIC / HEIF / AVIF | 埋め込みサムネイル優先。デコードは photocraft の `heif` 相当を検討（pure Rust の HEVC は重い → Craft App 委譲も可） | EXIF / XMP | サイドカー | P1 |
| 画像 | OpenEXR、HDR（Radiance） | `image`（exr / hdr feature）。トーンマップして表示 | ヘッダー属性 | CacheOnly | P2 |
| 画像 | JPEG XL | 要調査（pure Rust デコーダ `jxl-oxide`） | EXIF / XMP | 埋め込み（要調査） | P2 |
| raw | DNG、CR2、CR3、CRW、NEF、ARW、ORF、RAF、RW2、PEF、SRW、3FR、IIQ ほか | 埋め込みプレビュー JPEG 抽出を第一。高品質時は `photocraft-raw`（standalone、rev 固定）で半分解像度デモザイク。CRW は `craft-crw` | EXIF / MakerNote（レンズ等の主要項目） / サイドカー XMP | サイドカー | P0（埋め込み）/ P1（デコード） |
| レイヤー | PSD / PSB | 合成画像（Image Data セクション）または サムネイルリソース 1036。`photocraft-psd`（standalone、rev 固定） | XMP（リソース 1060）、EXIF（1058）、IPTC（1028）、ICC（1039） | 埋め込み | P0 |
| レイヤー | TIFF（レイヤー付き） | 合成画像 | 同 TIFF | 埋め込み | P1 |
| Craft ネイティブ | `.pcraft`（PhotoCraft） | `craft-pcraft-format` のマニフェストからプレビュー／タイル合成（プレビュー画像エントリがあればそれ） | マニフェスト | 要調査 | P1 |
| Craft ネイティブ | `.vectorcraft`、`.ecproj` | `craft-vectorcraft-format`、`craft-ecproj-format` のエンベロープから埋め込みプレビュー（なければ Craft App CLI） | エンベロープ | 要調査 | P2 |
| Craft ネイティブ | LightCraft カタログ、DesignCraft・FilmCraft・SoundCraft 等の形式 | 各 Craft App の CLI に委譲、または埋め込みプレビューの仕様を上流で確認 | — | — | P2 |
| ベクター | SVG | 自前ラスタライズ（`resvg`、pure Rust） | `<metadata>` 内 XMP | サイドカー | P1 |
| ベクター | AI（PDF 互換）、EPS | AI は PDF として `craft-pdf` でラスタライズ。EPS は埋め込み TIFF/WMF プレビューのみ | XMP | サイドカー | P2 |
| レイアウト | INDD / INDT | 埋め込みページプレビュー（[storytold/indd-utils](https://github.com/storytold/indd-utils) の利用を検討） | XMP | CacheOnly | P2 |
| レイアウト | IDML | DesignCraft の CLI に委譲 | パッケージ内 XMP | CacheOnly | P2 |
| 文書 | PDF | `craft-pdf` で 1 ページ目をラスタライズ（ページ送り対応） | XMP メタデータストリーム、Info 辞書 | CacheOnly（初期） | P1 |
| 文書 | DOCX / XLSX / PPTX / ODF | ZIP 内の `docProps/thumbnail.*` | `docProps/core.xml` | CacheOnly | P2 |
| 動画 | MP4、MOV、M4V、MKV、WebM、AVI、MTS | `craft-media` で demux。サムネイルは埋め込みカバー or キーフレームのデコード（コーデック実装は FilmCraft 側にあるため CLI 委譲を第一候補、ADR-006） | 長さ、解像度、フレームレート、コーデック、XMP（`uuid` ボックス） | サイドカー | P1 |
| 音声 | WAV、AIFF、MP3、AAC/M4A、FLAC、OGG | 波形アイコン（波形描画は P2） | ID3 / RIFF INFO / Vorbis comment、長さ、サンプルレート | サイドカー | P1 |
| フォント | TTF、OTF、TTC、WOFF/WOFF2 | サンプル文字列のレンダリング（`skrifa` + 自前ラスタライズ、`craft-font-catalog`） | 名前テーブル | CacheOnly | P2 |
| 3D | glTF / GLB、OBJ、FBX、USDZ | GLB の埋め込みサムネイル、なければ汎用アイコン（3D 描画は P2 以降・要観察で範囲決定） | — | — | P2 |
| その他 | テキスト、Markdown、コード | 先頭数行のテキスト描画 | — | — | P2 |

## 3. Craft App CLI への委譲

Pier 単体でデコードできない、または品質が足りない形式は、インストール済み Craft App のヘッドレス CLI で PNG を作らせる。

| Craft App | 想定コマンド | 備考 |
|---|---|---|
| PhotoCraft | `photocraft-cli convert <in> <out.png>` | PSD/PSB/.pcraft/ORA/KRA/XCF/Affinity 等の正確な合成。配布物に CLI が同梱されるか、macOS .app 内のパスは要確認 |
| LightCraft | `lightcraft-cli`（サブコマンド要確認） | 現像設定を反映した raw プレビュー |
| VectorCraft / DesignCraft / PdfCraft / FilmCraft | 各 `*-cli`（有無・サブコマンド要確認） | |

- 実行は `std::process::Command` に引数配列で渡す（シェルを使わない）。タイムアウト（既定 30s）、同時実行数の上限（既定 2）。
- 出力先は Pier のキャッシュ内の一時ディレクトリ。終了コードと stderr をログに記録する。
- 委譲で作ったサムネイルは `source: CraftCli(app, version)` として DB に記録し、Craft App の更新時に作り直せるようにする。

## 4. テストデータ

- 合成データ：`pier-testkit` の生成器（全形式の最小ファイル、壊れたファイル、巨大寸法ヘッダー、ループする IFD）。
- 実ファイル：[photocraft-corpus](https://github.com/storytold/photocraft-corpus)（PSD）、PngSuite、raw は raw.pixls.us（ライセンス確認の上、ハッシュ固定でダウンロード）。
  `corpus/`（gitignore）に `cargo xtask corpus` で取得する（photocraft と同じ方式）。
