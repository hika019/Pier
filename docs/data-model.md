# データモデル

> **最終更新:** 2026-11-02

## 1. 正本の所在

| 情報 | 正本 | 書き込み先 | 備考 |
|---|---|---|---|
| ★レーティング、却下 | ファイル | `xmp:Rating`（-1 = 却下、0〜5） | |
| ラベル | ファイル | `xmp:Label`（ラベル名の文字列） | ラベル名を環境設定で変えると、既存ファイルの表示は「ラベル名が一致しない」扱いになる（Bridge と同じ。白いラベルで表示） |
| キーワード | ファイル | `dc:subject`（フラット）、`lr:hierarchicalSubject`（`親|子`） | |
| IPTC Core / Extension | ファイル | XMP（`photoshop:*`、`Iptc4xmpCore:*`、`Iptc4xmpExt:*`、`dc:*`、`xmpRights:*`）。JPEG/TIFF/PSD の既存 IPTC-IIM は同期更新（Metadata Working Group のガイドライン） | |
| 回転 | ファイル | 画像：`tiff:Orientation`（XMP、JPEG は EXIF も）。raw：`crs:` ではなく `tiff:Orientation` をサイドカーへ | 要観察：Bridge は raw を Camera Raw 設定で回すか |
| 保持されたファイル名 | ファイル | `xmpMM:PreservedFileName` | |
| コレクション、スマートコレクション | ユーザーデータ | `userdata/collections.json` | Bridge の `.filelist` / `.collection` 相当（形式は独自） |
| スタック | ユーザーデータ | `userdata/stacks/<folder-hash>.json` | フォルダ単位 |
| 手動並び順 | ユーザーデータ | `userdata/sort/<folder-hash>.json` | |
| キーワード辞書（ツリー） | ユーザーデータ | `userdata/keywords.json` | ファイルに付いているが辞書にないものは「その他のキーワード」に表示 |
| メタデータテンプレート | ユーザーデータ | `userdata/metadata-templates/<name>.xmp` | XMP パケットとして保存（他ツールと交換可能） |
| 一括リネーム・書き出し・出力プリセット | ユーザーデータ | `userdata/presets/{rename,export,output}/<name>.json` | |
| Undo ジャーナル | ユーザーデータ | `userdata/journal.jsonl`（直近 100 件、ローテーション） | |
| サムネイル・プレビュー・メタデータ索引 | キャッシュ | `cache/` | 消してよい |

## 2. ドメイン型

```rust
/// ファイル 1 つ（またはパッケージ）。パスが変わっても追跡できるよう、ID はファイル ID を優先する。
pub struct Asset {
    pub id: AssetId,              // 内部 ID（u64）。catalog が採番
    pub path: CanonicalPath,      // NFC 正規化した絶対パス（表示用に元の表記も保持）
    pub identity: FileIdentity,   // (volume_id, file_id) — inode / NTFS file index。取れない FS は None
    pub kind: AssetKind,          // Image / Raw / Layered(PSD等) / Vector / Layout / Pdf / Video / Audio / Font / Model3d / Document / Folder / Package / Other
    pub format: FormatId,         // "jpeg", "cr3", "psd", "pcraft" ...
    pub size: u64,
    pub mtime: DateTime<Utc>,
    pub ctime: Option<DateTime<Utc>>,
    pub related: Vec<RelatedFile>, // サイドカー .xmp、.THM、同名 JPEG（RAW+JPEG）
}

/// 一覧・フィルター・並べ替えに使う軽量メタデータ（DB に索引）。
pub struct QuickMeta {
    pub rating: Option<i8>,        // -1..=5
    pub label: Option<String>,
    pub dimensions: Option<(u32, u32)>, // 回転適用後
    pub orientation: Orientation,
    pub captured: Option<DateTimeWithOffset>,
    pub camera: Option<CameraSummary>,   // make, model, lens, iso, exposure, f-number, focal, wb
    pub color_profile: Option<String>,
    pub bit_depth: Option<u8>,
    pub color_mode: Option<ColorMode>,
    pub keywords: Vec<String>,           // 階層は "a|b|c"
    pub has_develop_settings: bool,      // crs: / LightCraft 設定の有無
    pub duration: Option<Duration>,      // 動画・音声
    pub page_count: Option<u32>,
    pub writable: Writability,           // Embedded / Sidecar / CacheOnly / ReadOnly
}
```

- **完全メタデータ**（メタデータパネル・ファイル情報用）は都度 `pier-meta` で読み、DB には入れない（`QuickMeta` のみ索引）。
- メタデータの読み書きは LightCraft の `lightcraft-meta` の API 形状（`extract`、`embedded`、`parse_xmp`、`write_xmp`）を参考にする。
  共有ライブラリ化できるなら craft-libs に `craft-meta` として切り出す提案を上流に出す（[decisions.md](decisions.md) ADR-005）。

## 3. メタデータ書き込み方針

| 形式 | 方式 | 検証 |
|---|---|---|
| JPEG | APP1 XMP（＋拡張 XMP）を差し替え。EXIF Orientation は APP1 Exif 内の該当タグのみ上書き | 書き込み後に SOS 以降のバイト列ハッシュが一致 |
| TIFF / DNG | 既存の XMP タグ（700）を差し替え。サイズが増えるならファイル末尾に追記して IFD のオフセット更新 | 画像ストリップ／タイルのハッシュが一致 |
| PSD / PSB | イメージリソース 1060（XMP）を差し替え、セクション長を更新 | レイヤー・画像データ部のハッシュが一致 |
| PNG | `iTXt` `XML:com.adobe.xmp` チャンクを差し替え（IDAT の前） | IDAT のハッシュ |
| WebP | `XMP ` チャンク（VP8X が必要なら追加） | VP8/VP8L チャンクのハッシュ |
| HEIC / AVIF | 初期はサイドカー（ISOBMFF 内の書き換えは M8 以降に検討） | — |
| raw（CR2/CR3/NEF/ARW/ORF/RAF/RW2 等） | サイドカー `<basename>.xmp`（Camera Raw / LightCraft と互換の命名） | — |
| PDF | 初期は対象外（CacheOnly）。`craft-pdf` の増分更新で XMP メタデータストリームを書く案を検討 | — |
| 動画・音声 | 初期はサイドカー（MP4 の `uuid` XMP ボックス書き込みは検討） | — |
| Craft App ネイティブ（`.pcraft` 等） | 各フォーマットのマニフェストに XMP フィールドがあれば書く。なければサイドカー | — |
| 書き込み不可（読み取り専用、権限なし） | DB のみに保持し、サムネイルに「メタデータを書き込めません」バッジ | — |

- 既存のサイドカーがあれば、埋め込み可能形式でもサイドカーを優先するかは環境設定（既定：Bridge と同じく埋め込み。要観察）。
- 未知の XMP 名前空間・プロパティは**そのまま保持**する（往復で消さない）。
- 書き込み前にファイルの mtime・サイズを再確認し、読み取り後に外部で変わっていたら中止して再読込を促す。

## 4. キャッシュ DB（redb）

テーブル（キー → 値）。値は `postcard` でシリアライズし、先頭にスキーマバージョンを持つ。

| テーブル | キー | 値 |
|---|---|---|
| `assets` | `AssetId` | `path`、`identity`、`size`、`mtime`、`format`、`kind` |
| `path_index` | `CanonicalPath` | `AssetId` |
| `identity_index` | `(volume_id, file_id)` | `AssetId`（外部でのリネーム・移動の追跡） |
| `quick_meta` | `AssetId` | `QuickMeta`＋読み取り時の `(size, mtime)` |
| `thumbs` | `(AssetId, Kind)` | `ThumbRecord { file: blob ファイル名, w, h, source: Embedded/Decoded, quality, icc_hash, created }` |
| `failures` | `AssetId` | `(size, mtime, reason)` — 再試行抑止 |
| `folders` | `CanonicalPath` | `last_listed`、`child_count`、`exported_cache: bool` |
| `keywords_idx` | `(keyword, AssetId)` | `()` — キーワード検索用 |
| `meta` | `"schema_version"` 等 | 値 |

- 画像本体は DB に入れず `cache/thumbs/<aa>/<AssetId>-<kind>.webp`（ロスレス WebP、プレビューは高画質ロッシー）として保存する。
- スキーマ変更時は移行関数を書く。移行できない場合はキャッシュなので作り直す（ユーザーデータは別ファイルなので失われない）。
- 起動時に DB を開けなければ `cache/broken-<timestamp>/` に退避して新規作成する。
- 容量上限（既定 10GB）を超えたら `thumbs` の最終アクセスが古いものから削除する。

## 5. フォルダ書き出しキャッシュ

Bridge の「キャッシュをフォルダーに書き出し」相当。書き込み不可メディアや共有フォルダで他ユーザーがキャッシュを流用できる。

- 形式：フォルダ内 `.pier-cache/`（`index.postcard` ＋ サムネイル WebP）。
- Bridge の `.BridgeCache` / `.BridgeCacheT` は読まない（非公開形式でありクリーンルーム方針に反するため）。

## 6. 検索条件（スマートコレクション・詳細検索）

```rust
pub enum Query {
    All(Vec<Query>),
    Any(Vec<Query>),
    Not(Box<Query>),
    Field { field: Field, op: Op, value: Value },
}
// Field: FileName, Kind, Format, Rating, Label, Keyword, Captured, Modified, Created, Size, Width, Height,
//        Orientation, AspectRatio, ColorProfile, Iso, Exposure, FNumber, FocalLength, Lens, Model,
//        AnyXmp(path), AllMetadata（全文）, Description, Copyright, Creator ...
// Op: Contains, NotContains, Is, IsNot, StartsWith, EndsWith, Lt, Le, Gt, Ge, Exists, NotExists
```

- 範囲（スマートコレクション）：フォルダ集合＋サブフォルダを含む＋非インデックスファイルを含む（＝その場でメタデータを読む）。
- 結果件数の上限（既定 10 万）を設け、超えたら打ち切ってユーザーに知らせる。
