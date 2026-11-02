//! Adds a system CJK face behind egui's default fonts, so the Japanese interface draws.
//!
//! Same search order idea as ArtCraft Launcher: Hiragino on macOS, Yu Gothic / Meiryo on Windows,
//! Noto Sans CJK on Linux and FreeBSD. Without one, Japanese text shows as boxes but nothing fails.

use std::sync::Arc;

/// `(path, face index inside a collection)`, best first.
const CJK_CANDIDATES: &[(&str, u32)] = &[
    ("/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc", 0),
    ("/System/Library/Fonts/Hiragino Sans GB.ttc", 0),
    ("/System/Library/Fonts/Supplemental/Arial Unicode.ttf", 0),
    ("C:\\Windows\\Fonts\\YuGothM.ttc", 0),
    ("C:\\Windows\\Fonts\\meiryo.ttc", 0),
    ("C:\\Windows\\Fonts\\msgothic.ttc", 0),
    ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 0),
    ("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 0),
    ("/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc", 0),
    ("/usr/local/share/fonts/noto/NotoSansCJK-Regular.ttc", 0),
];

const CJK_FONT_NAME: &str = "pier-system-cjk";

/// Returns the path of the face it installed, if any.
pub fn install_system_cjk(ctx: &egui::Context) -> Option<&'static str> {
    let (path, index, bytes) = CJK_CANDIDATES.iter().find_map(|&(path, index)| std::fs::read(path).ok().map(|bytes| (path, index, bytes)))?;
    let mut fonts = egui::FontDefinitions::default();
    let mut data = egui::FontData::from_owned(bytes);
    data.index = index;
    fonts.font_data.insert(CJK_FONT_NAME.to_owned(), Arc::new(data));
    // First for proportional text, so Latin and Japanese share one system face (closer to a native app than
    // egui's Ubuntu); egui's fonts stay behind it for the symbols the system face lacks.
    fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, CJK_FONT_NAME.to_owned());
    fonts.families.entry(egui::FontFamily::Monospace).or_default().push(CJK_FONT_NAME.to_owned());
    ctx.set_fonts(fonts);
    Some(path)
}
