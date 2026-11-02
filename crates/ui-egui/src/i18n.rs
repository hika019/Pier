//! Interface strings in English and Japanese.
//!
//! First cut: a compile-checked table. It moves to craft-i18n catalogs (docs/architecture.md §2.2)
//! when more languages arrive. Wording follows Bridge's English and Japanese editions where known.

use std::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    En,
    Ja,
}

/// Preferences › Advanced › Language: Auto follows the system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LangSetting {
    #[default]
    Auto,
    Fixed(Lang),
}

impl LangSetting {
    pub fn resolve(self) -> Lang {
        match self {
            Self::Fixed(l) => l,
            Self::Auto => system_lang(),
        }
    }
}

pub fn system_lang() -> Lang {
    match std::env::var("PIER_LOCALE").ok().or_else(sys_locale::get_locale) {
        Some(l) if l.to_ascii_lowercase().starts_with("ja") => Lang::Ja,
        _ => Lang::En,
    }
}

thread_local! {
    /// Per thread, not per process: only the UI thread reads it, and windows driven on separate test
    /// threads in one process (scenario tests) may each show another language.
    static CURRENT: Cell<Lang> = const { Cell::new(Lang::En) };
}

pub fn set_lang(lang: Lang) {
    CURRENT.with(|c| c.set(lang));
}

pub fn lang() -> Lang {
    CURRENT.with(Cell::get)
}

/// Japanese for the English labels craft-menu's layouts write (macOS app menu, Window menu, …).
pub fn layout_label(english: &'static str) -> String {
    if lang() != Lang::Ja {
        return english.to_owned();
    }
    match english {
        "About {app}" => "{app} について",
        "Settings…" => "設定…",
        "Settings" => "設定",
        "Preferences…" => "環境設定…",
        "Preferences" => "環境設定",
        "Language" => "言語",
        "Appearance" => "外観",
        "Services" => "サービス",
        "Hide {app}" => "{app} を隠す",
        "Hide Others" => "ほかを隠す",
        "Show All" => "すべてを表示",
        "Quit {app}" => "{app} を終了",
        "Quit" => "終了",
        "Exit" => "終了",
        "Minimize" => "しまう",
        "Zoom" => "拡大／縮小",
        "Bring All to Front" => "すべてを手前に移動",
        "Close Window" => "ウィンドウを閉じる",
        "File" => "ファイル",
        "Edit" => "編集",
        "Window" => "ウィンドウ",
        "Help" => "ヘルプ",
        "Menu" => "メニュー",
        other => other,
    }
    .to_owned()
}

/// The string in the current language.
pub fn tr(s: S) -> &'static str {
    s.get(lang())
}

macro_rules! strings {
    ($($key:ident => $en:literal, $ja:literal;)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum S { $($key),* }
        impl S {
            pub const ALL: &'static [S] = &[$(S::$key),*];
            pub fn get(self, lang: Lang) -> &'static str {
                match (self, lang) {
                    $((S::$key, Lang::En) => $en, (S::$key, Lang::Ja) => $ja,)*
                }
            }
        }
    };
}

strings! {
    // Menus
    MenuFile => "File", "ファイル";
    MenuEdit => "Edit", "編集";
    MenuView => "View", "表示";
    MenuStacks => "Stacks", "スタック";
    MenuLabel => "Label", "ラベル";
    MenuTools => "Tools", "ツール";
    MenuWindow => "Window", "ウィンドウ";
    MenuHelp => "Help", "ヘルプ";
    NewWindow => "New Window", "新規ウィンドウ";
    NewFolder => "New Folder", "新規フォルダー";
    Open => "Open", "開く";
    OpenWith => "Open With", "指定アプリで開く";
    OpenRecent => "Open Recent", "最近使用したファイルを開く";
    OpenInCameraRaw => "Open in Camera Raw (LightCraft)", "Camera Raw で開く（LightCraft）";
    CloseWindow => "Close Window", "ウィンドウを閉じる";
    MoveToTrash => "Move to Trash", "ゴミ箱に入れる";
    RevealInFinder => "Reveal in Finder", "Finder で表示";
    RevealInExplorer => "Reveal in Explorer", "エクスプローラーで表示";
    RevealInFileManager => "Show in File Manager", "ファイルマネージャーで表示";
    GetPhotosFromCamera => "Get Photos from Camera…", "カメラから写真を取り込み…";
    MoveTo => "Move to", "移動";
    CopyTo => "Copy to", "コピー";
    Place => "Place", "配置";
    AddToFavorites => "Add to Favorites", "お気に入りに追加";
    FileInfo => "File Info…", "ファイル情報…";
    Quit => "Quit Pier", "Pier を終了";
    Undo => "Undo", "取り消し";
    Redo => "Redo", "やり直し";
    Cut => "Cut", "カット";
    Copy => "Copy", "コピー";
    Paste => "Paste", "ペースト";
    Duplicate => "Duplicate", "複製";
    SelectAll => "Select All", "すべてを選択";
    DeselectAll => "Deselect All", "すべてを選択解除";
    InvertSelection => "Invert Selection", "選択範囲を反転";
    SelectLabeled => "Select Labeled", "ラベル付きを選択";
    SelectUnlabeled => "Select Unlabeled", "ラベルなしを選択";
    Find => "Find…", "検索…";
    DevelopSettings => "Develop Settings", "現像設定";
    Rotate180 => "Rotate 180°", "180° 回転";
    RotateCw => "Rotate 90° Clockwise", "90° 回転（時計回り）";
    RotateCcw => "Rotate 90° Counterclockwise", "90° 回転（反時計回り）";
    Preferences => "Preferences…", "環境設定…";
    FullScreenPreview => "Full Screen Preview", "フルスクリーンプレビュー";
    Slideshow => "Slideshow", "スライドショー";
    SlideshowOptions => "Slideshow Options…", "スライドショーオプション…";
    ReviewMode => "Review Mode", "レビューモード";
    AsThumbnails => "As Thumbnails", "サムネイルとして表示";
    AsDetails => "As Details", "詳細として表示";
    AsList => "As List", "リストとして表示";
    ShowThumbnailOnly => "Show Thumbnail Only", "サムネイルのみを表示";
    GridLock => "Grid Lock", "グリッドロック";
    ShowRejected => "Show Reject Files", "却下ファイルを表示";
    ShowHidden => "Show Hidden Files", "隠しファイルを表示";
    ShowFolders => "Show Folders", "フォルダーを表示";
    ShowSubfolderItems => "Show Items from Subfolders", "サブフォルダーの項目を表示";
    Sort => "Sort", "並べ替え";
    Refresh => "Refresh", "最新の情報に更新";
    GroupAsStack => "Group as Stack", "スタックとしてグループ化";
    Ungroup => "Ungroup from Stack", "スタックを解除";
    OpenStack => "Open Stack", "スタックを開く";
    CloseStack => "Close Stack", "スタックを閉じる";
    PromoteToTop => "Promote to Top of Stack", "スタックの先頭へ";
    ExpandAllStacks => "Expand All Stacks", "すべてのスタックを展開";
    CollapseAllStacks => "Collapse All Stacks", "すべてのスタックを折りたたむ";
    AutoStack => "Auto-Stack Panorama/HDR", "パノラマ／HDR の自動スタック";
    Rating => "Rating", "レーティング";
    NoRating => "No Rating", "レーティングなし";
    Reject => "Reject", "却下";
    DecreaseRating => "Decrease Rating", "レーティングを下げる";
    IncreaseRating => "Increase Rating", "レーティングを上げる";
    Label => "Label", "ラベル";
    NoLabel => "No Label", "ラベルなし";
    LabelSelect => "Select", "セレクト";
    LabelSecond => "Second", "セカンド";
    LabelApproved => "Approved", "承認";
    LabelReview => "Review", "レビュー";
    LabelTodo => "To Do", "To Do";
    BatchRename => "Batch Rename…", "一括リネーム…";
    CreateMetadataTemplate => "Create Metadata Template…", "メタデータテンプレートを作成…";
    AppendMetadata => "Append Metadata", "メタデータを追加";
    ReplaceMetadata => "Replace Metadata", "メタデータを置換";
    Cache => "Cache", "キャッシュ";
    BuildAndExportCache => "Build and Export Cache…", "キャッシュの構築と書き出し…";
    PurgeCache => "Purge Cache for Folder", "フォルダーのキャッシュをパージ";
    PcBatch => "Batch…", "バッチ…";
    PcImageProcessor => "Image Processor…", "画像プロセッサー…";
    PcLoadAsLayers => "Load Files into PhotoCraft Layers…", "ファイルを PhotoCraft のレイヤーとして読み込み…";
    PcMergeHdr => "Merge to HDR Pro…", "HDR Pro に統合…";
    PcPhotomerge => "Photomerge…", "Photomerge…";
    LcImport => "Import into LightCraft", "LightCraft に読み込み";
    LcDng => "Convert to DNG…", "DNG に変換…";
    VcImageTrace => "Image Trace…", "画像トレース…";
    Workspace => "Workspace", "ワークスペース";
    NewWorkspace => "New Workspace…", "新規ワークスペース…";
    ResetWorkspace => "Reset Workspace", "ワークスペースを初期化";
    PathBar => "Path Bar", "パスバー";
    PierHelp => "Pier Help", "Pier ヘルプ";
    KeyboardShortcuts => "Keyboard Shortcuts", "キーボードショートカット";
    GetCraftApps => "Get Craft Apps…", "Craft Apps を入手…";
    CheckForUpdates => "Check for Updates…", "アップデートを確認…";
    AboutPier => "About Pier", "Pier について";
    ComingSoon => "Coming soon", "今後実装予定";
    SoonSuffix => " (coming soon)", "（今後実装予定）";
    ComingSoonToast => "Coming soon:", "今後実装予定です：";
    // Workspaces
    WsEssentials => "Essentials", "必須";
    WsFilmstrip => "Filmstrip", "フィルムストリップ";
    WsMetadata => "Metadata", "メタデータ";
    WsKeywords => "Keywords", "キーワード";
    WsPreview => "Preview", "プレビュー";
    WsLightTable => "Light Table", "ライトテーブル";
    WsFolders => "Folders", "フォルダー";
    WsOutput => "Output", "出力";
    // Panels
    PFavorites => "Favorites", "お気に入り";
    PFolders => "Folders", "フォルダー";
    PFilter => "Filter", "フィルター";
    PCollections => "Collections", "コレクション";
    PExport => "Export", "書き出し";
    PContent => "Content", "コンテンツ";
    PPreview => "Preview", "プレビュー";
    PMetadata => "Metadata", "メタデータ";
    PKeywords => "Keywords", "キーワード";
    POutput => "Output", "出力";
    POutputPreview => "Output Preview", "出力プレビュー";
    // App bar / path bar / status bar tooltips
    TipBack => "Go back", "戻る";
    TipForward => "Go forward", "進む";
    TipParent => "Go to parent or Favorites", "親フォルダーまたはお気に入りに移動";
    TipRecent => "Show recent files or go to recent folders", "最近使用したファイルを表示、または最近使用したフォルダーに移動";
    TipCamera => "Get Photos from Camera", "カメラから写真を取り込み";
    TipRefine => "Refine", "絞り込み";
    TipCameraRaw => "Open in Camera Raw (LightCraft)", "Camera Raw で開く（LightCraft）";
    TipRotateCcw => "Rotate 90° counterclockwise", "90° 回転（反時計回り）";
    TipRotateCw => "Rotate 90° clockwise", "90° 回転（時計回り）";
    TipQuality => "Options for thumbnail quality and preview generation", "サムネイルの品質とプレビュー生成のオプション";
    TipFilterRating => "Filter items by rating", "レーティングで項目をフィルター";
    TipSortOrder => "Ascending / descending", "昇順／降順";
    TipNewFolder => "Create a new folder", "新規フォルダーを作成";
    TipDelete => "Delete item", "項目を削除";
    TipGridLock => "Lock thumbnail grid", "サムネイルグリッドをロック";
    TipViewThumbs => "View content as thumbnails", "コンテンツをサムネイルとして表示";
    TipViewDetails => "View content as details", "コンテンツを詳細として表示";
    TipViewList => "View content as list", "コンテンツをリストとして表示";
    TipSmaller => "Smaller thumbnail size", "サムネイルを小さく";
    TipLarger => "Larger thumbnail size", "サムネイルを大きく";
    TipPanelMenu => "Panel menu", "パネルメニュー";
    TipSubfolders => "Show subfolders", "サブフォルダーを表示";
    QuickSearch => "Search", "検索";
    RecentFolders => "Recent Folders", "最近使用したフォルダー";
    NoRecent => "No recent folders", "最近使用したフォルダーはありません";
    QualityEmbedded => "Prefer Embedded (Faster)", "埋め込みを優先（高速）";
    QualityOnDemand => "High Quality On Demand", "必要に応じて高品質";
    QualityAlways => "Always High Quality", "常に高品質";
    Quality100 => "Generate 100% Previews", "100% プレビューを生成";
    ShowAllItems => "Show All Items", "すべての項目を表示";
    ShowUnrated => "Show Unrated Items Only", "レーティングなしの項目のみを表示";
    ShowRatedAtLeast => "Show {n} or More", "{n} 以上を表示";
    ShowLabeled => "Show Labeled Items Only", "ラベル付きの項目のみを表示";
    ShowUnlabeled => "Show Unlabeled Items Only", "ラベルなしの項目のみを表示";
    ShowRejectedOnly => "Show Rejected Items Only", "却下された項目のみを表示";
    SortBy => "Sort by {key}", "並べ替え：{key}";
    SortName => "Filename", "ファイル名";
    SortType => "Type", "ファイルの種類";
    SortCreated => "Date Created", "作成日";
    SortModified => "Date Modified", "変更日";
    SortSize => "File Size", "ファイルサイズ";
    SortRating => "Rating", "レーティング";
    SortLabel => "Label", "ラベル";
    Ascending => "Ascending Order", "昇順";
    ItemsStatus => "{n} items", "{n} 項目";
    ItemsSelected => "{n} items, {m} selected - {size}", "{n} 項目、{m} 項目を選択 - {size}";
    Loading => "Loading…", "読み込み中…";
    FolderEmpty => "There are no items to display.", "表示する項目がありません。";
    FolderError => "Unable to open the folder.", "フォルダーを開けません。";
    NoMatches => "No items match the current filter.", "現在のフィルターに一致する項目はありません。";
    // Favorites / folders
    Computer => "Computer", "コンピューター";
    Home => "Home", "ホーム";
    Desktop => "Desktop", "デスクトップ";
    Documents => "Documents", "書類";
    Pictures => "Pictures", "ピクチャ";
    Downloads => "Downloads", "ダウンロード";
    DragFavoritesHint => "Drag folders here to add Favorites (coming soon)", "フォルダーをここにドラッグしてお気に入りに追加（今後実装予定）";
    // Filter
    FLabels => "Labels", "ラベル";
    FRatings => "Ratings", "レーティング";
    FFileType => "File Type", "ファイルの種類";
    FDateModified => "Date Modified", "変更日";
    FKeepFilter => "Keep filter when browsing (coming soon)", "参照中のフィルターを保持（今後実装予定）";
    FClear => "Clear filter", "フィルターをクリア";
    KindFolder => "Folder", "フォルダー";
    KindImage => "Image", "画像";
    KindRaw => "Camera Raw", "Camera Raw";
    KindLayered => "Layered Image", "レイヤー画像";
    KindVector => "Vector", "ベクター";
    KindLayout => "Layout", "レイアウト";
    KindPdf => "PDF", "PDF";
    KindVideo => "Video", "ビデオ";
    KindAudio => "Audio", "オーディオ";
    KindFont => "Font", "フォント";
    KindDocument => "Document", "ドキュメント";
    KindOther => "Other", "その他";
    // Metadata
    MFileProperties => "File Properties", "ファイルプロパティ";
    MIptcCore => "IPTC Core", "IPTC Core";
    MCameraData => "Camera Data (Exif)", "カメラデータ（Exif）";
    MGps => "GPS", "GPS";
    MFilename => "Filename", "ファイル名";
    MDocumentType => "Document Type", "ドキュメントの種類";
    MCreated => "Date Created", "作成日";
    MModified => "Date File Modified", "ファイル変更日";
    MFileSize => "File Size", "ファイルサイズ";
    MDimensions => "Dimensions", "寸法";
    MLocation => "Location", "場所";
    MMake => "Make", "メーカー";
    MModel => "Model", "モデル";
    MLens => "Lens", "レンズ";
    MDateOriginal => "Date Time Original", "撮影日時";
    MExposure => "Exposure", "露出時間";
    MFNumber => "F-Stop", "絞り値";
    MIso => "ISO Speed Ratings", "ISO 感度";
    MFocal => "Focal Length", "焦点距離";
    MBias => "Exposure Bias Value", "露出補正量";
    MWhiteBalance => "White Balance", "ホワイトバランス";
    MFlash => "Flash", "フラッシュ";
    MOrientation => "Orientation", "画像の向き";
    MResolution => "Resolution", "解像度";
    MColorSpace => "Color Space", "色空間";
    MSoftware => "Software", "ソフトウェア";
    MCreator => "Creator", "作成者";
    MCopyright => "Copyright Notice", "著作権情報";
    MLatitude => "Latitude", "緯度";
    MLongitude => "Longitude", "経度";
    MEditSoon => "Editing metadata is coming soon", "メタデータの編集は今後実装予定です";
    MNoSelection => "Select an item to see its metadata.", "メタデータを表示するには項目を選択してください。";
    MMultiple => "{n} items selected", "{n} 項目を選択";
    MarksSessionOnly => "Ratings and labels are kept for this session only; writing them to files (XMP) is coming soon.", "レーティングとラベルはこのセッション中だけ保持されます。ファイル（XMP）への書き込みは今後実装予定です。";
    // Keywords / collections / export / output
    KAssigned => "Assigned Keywords:", "割り当てられたキーワード：";
    KEvents => "Events", "イベント";
    KPeople => "People", "人物";
    KPlaces => "Places", "場所";
    KOther => "Other Keywords", "その他のキーワード";
    KNew => "New Keyword", "新規キーワード";
    KNewSub => "New Sub Keyword", "新規サブキーワード";
    KDelete => "Delete Keyword", "キーワードを削除";
    PanelSoon => "This panel is coming soon. The layout is in place.", "このパネルは今後実装予定です（レイアウトのみ）。";
    CNewSmart => "New Smart Collection", "新規スマートコレクション";
    CEditSmart => "Edit Smart Collection", "スマートコレクションを編集";
    CNew => "New Collection", "新規コレクション";
    CNone => "No collections", "コレクションはありません";
    ESaveToDisk => "Save to Hard Drive", "ハードディスクに保存";
    EAddPreset => "Add New Preset", "新規プリセットを追加";
    ODocument => "Document", "ドキュメント";
    OLayout => "Layout", "レイアウト";
    OOverlays => "Overlays", "オーバーレイ";
    OHeader => "Header", "ヘッダー";
    OFooter => "Footer", "フッター";
    OWatermark => "Watermark", "透かし";
    OTemplate => "Template", "テンプレート";
    ORefresh => "Refresh Preview", "プレビューを更新";
    OExportPdf => "Export to PDF…", "PDF に書き出し…";
    OPaper => "Paper Size", "用紙サイズ";
    OColumns => "Columns", "列";
    ORows => "Rows", "行";
    // Dialogs
    Ok => "OK", "OK";
    Cancel => "Cancel", "キャンセル";
    Close => "Close", "閉じる";
    CraftNeededTitle => "{app} is needed to open this file", "このファイルを開くには {app} が必要です";
    CraftNeededBody => "{app} is a free, open-source Craft App. ArtCraft Launcher installs and updates every Craft App in one click.", "{app} は無料でオープンソースの Craft App です。ArtCraft Launcher を使うと、Craft Apps をワンクリックでインストール・更新できます。";
    OpenLauncher => "Open ArtCraft Launcher", "ArtCraft Launcher を開く";
    DownloadLauncher => "Download ArtCraft Launcher", "ArtCraft Launcher をダウンロード";
    OpenAppPage => "Open the {app} page", "{app} のページを開く";
    OpenWithDefault => "Open with the Default App", "OS の既定アプリで開く";
    DefaultApp => "Default App", "既定のアプリ";
    CraftAppsTitle => "Craft Apps", "Craft Apps";
    CraftAppsBody => "Pier hands files to the Craft Apps instead of Adobe apps. Install them with ArtCraft Launcher.", "Pier は Adobe 製品の代わりに Craft Apps にファイルを渡します。ArtCraft Launcher でインストールできます。";
    Installed => "Installed", "インストール済み";
    NotInstalled => "Not installed", "未インストール";
    AboutBody => "A digital asset browser for the Craft Apps, written in Rust.", "Craft Apps のためのデジタルアセットブラウザ（Rust 製）。";
    AboutTrademark => "Adobe and Adobe Bridge are trademarks of Adobe Inc. Pier is not affiliated with Adobe.", "Adobe、Adobe Bridge は Adobe Inc. の商標です。Pier は Adobe とは無関係です。";
    PrefGeneral => "General", "一般";
    PrefInterface => "Interface", "インターフェイス";
    PrefThumbnails => "Thumbnails", "サムネイル";
    PrefPlayback => "Playback", "再生";
    PrefMetadata => "Metadata", "メタデータ";
    PrefKeywords => "Keywords", "キーワード";
    PrefLabels => "Labels", "ラベル";
    PrefFileTypes => "File Type Associations", "ファイルの種類の関連付け";
    PrefCache => "Cache", "キャッシュ";
    PrefMediaCache => "Media Cache", "メディアキャッシュ";
    PrefAdvanced => "Advanced", "高度な設定";
    PrefOutput => "Output", "出力";
    PrefColorTheme => "Color Theme", "カラーテーマ";
    PrefLanguage => "Language", "言語";
    PrefLangAuto => "Automatic (system)", "自動（システム）";
    PrefSoon => "These settings are coming soon.", "この設定は今後実装予定です。";
    PrefFileTypesNote => "Changing associations is coming soon.", "関連付けの変更は今後実装予定です。";
    OpenFailed => "Could not open:", "開けませんでした：";
}
