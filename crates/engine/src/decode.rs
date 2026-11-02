//! Decoding images into small RGBA buffers for thumbnails and previews.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use image::{DynamicImage, ImageDecoder, ImageReader, Limits};

/// Refuses images larger than this many pixels (docs/architecture.md §4: 600 MP).
const MAX_PIXELS: u64 = 600_000_000;
/// Upper bound for one decode's allocations.
const MAX_ALLOC_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Decodes `path`, applies its EXIF orientation and shrinks it to fit `max_side` (aspect kept).
/// A decoder panic is reported as an error.
pub fn decode_scaled(path: &Path, max_side: u32) -> Result<Rgba, String> {
    catch_unwind(AssertUnwindSafe(|| decode_inner(path, max_side))).unwrap_or_else(|_| Err("decoder panicked".to_owned()))
}

fn decode_inner(path: &Path, max_side: u32) -> Result<Rgba, String> {
    let (w, h) = image::image_dimensions(path).map_err(|e| e.to_string())?;
    if u64::from(w).saturating_mul(u64::from(h)) > MAX_PIXELS {
        return Err(format!("{w}×{h} is larger than the {MAX_PIXELS} pixel limit"));
    }
    let mut reader = ImageReader::open(path).map_err(|e| e.to_string())?.with_guessed_format().map_err(|e| e.to_string())?;
    let mut limits = Limits::default();
    limits.max_alloc = Some(MAX_ALLOC_BYTES);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(|e| e.to_string())?;
    let orientation = decoder.orientation().map_err(|e| e.to_string())?;
    let mut img = DynamicImage::from_decoder(decoder).map_err(|e| e.to_string())?;
    img.apply_orientation(orientation);
    let max_side = max_side.max(1);
    let img = if img.width() > max_side || img.height() > max_side { img.thumbnail(max_side, max_side) } else { img };
    let rgba = img.into_rgba8();
    Ok(Rgba { width: rgba.width(), height: rgba.height(), pixels: rgba.into_raw() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrinks_to_fit_and_keeps_aspect() {
        let dir = std::env::temp_dir().join(format!("pier-decode-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("wide.png");
        image::RgbaImage::from_pixel(400, 100, image::Rgba([10, 20, 30, 255])).save(&p).unwrap();
        let out = decode_scaled(&p, 200).unwrap();
        assert_eq!((out.width, out.height), (200, 50));
        assert_eq!(out.pixels.len(), 200 * 50 * 4);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn garbage_is_an_error() {
        let dir = std::env::temp_dir().join(format!("pier-decode-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("bad.jpg");
        std::fs::write(&p, [0xff, 0xd8, 0xff, 0x00, 0x01, 0x02]).unwrap();
        assert!(decode_scaled(&p, 64).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
