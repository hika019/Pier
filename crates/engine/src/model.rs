use std::path::PathBuf;
use std::time::SystemTime;

/// What kind of thing a file is, from its extension. Drives icons, filters and which Craft App opens it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AssetKind {
    Folder,
    /// Formats Pier decodes itself (JPEG, PNG, GIF, WebP, BMP, TIFF).
    Image,
    /// Camera raw: shown from the camera's embedded JPEG, developed when it has none (decode.rs).
    Raw,
    Layered,
    Vector,
    Layout,
    Pdf,
    Video,
    Audio,
    Font,
    Document,
    Other,
}

impl AssetKind {
    pub const FILTERABLE: [Self; 12] = [
        Self::Folder,
        Self::Image,
        Self::Raw,
        Self::Layered,
        Self::Vector,
        Self::Layout,
        Self::Pdf,
        Self::Video,
        Self::Audio,
        Self::Font,
        Self::Document,
        Self::Other,
    ];

    pub fn from_extension(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" | "jpe" | "png" | "gif" | "webp" | "bmp" | "tif" | "tiff" => Self::Image,
            "dng" | "cr2" | "cr3" | "crw" | "nef" | "nrw" | "arw" | "srf" | "sr2" | "orf" | "raf" | "rw2" | "pef" | "srw" | "3fr" | "iiq" | "erf" | "mos"
            | "mrw" | "x3f" => Self::Raw,
            "psd" | "psb" | "pcraft" | "ora" | "kra" | "xcf" | "afphoto" => Self::Layered,
            "ai" | "svg" | "eps" | "vectorcraft" | "afdesign" => Self::Vector,
            "indd" | "indt" | "idml" | "afpub" => Self::Layout,
            "pdf" => Self::Pdf,
            "mp4" | "mov" | "m4v" | "mkv" | "webm" | "avi" | "mts" | "m2ts" | "mxf" => Self::Video,
            "wav" | "aif" | "aiff" | "mp3" | "aac" | "m4a" | "flac" | "ogg" | "opus" => Self::Audio,
            "ttf" | "otf" | "ttc" | "woff" | "woff2" => Self::Font,
            "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp" | "txt" | "md" | "rtf" | "csv" => Self::Document,
            _ => Self::Other,
        }
    }

    pub fn has_thumbnail(self) -> bool {
        matches!(self, Self::Image | Self::Raw)
    }
}

/// One entry of a folder listing.
#[derive(Clone, Debug)]
pub struct Asset {
    pub path: PathBuf,
    pub name: String,
    /// Lower-case extension without the dot; empty for folders and extension-less files.
    pub ext: String,
    pub kind: AssetKind,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub created: Option<SystemTime>,
    pub hidden: bool,
}

impl Asset {
    pub fn is_folder(&self) -> bool {
        self.kind == AssetKind::Folder
    }
}
