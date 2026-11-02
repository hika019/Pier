//! Decoding images into small RGBA buffers for thumbnails and previews.

use std::io::Cursor;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageReader, Limits};

use crate::model::AssetKind;

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
    if path.extension().is_some_and(|e| AssetKind::from_extension(&e.to_string_lossy()) == AssetKind::Raw) {
        return decode_raw(path, max_side);
    }
    let (w, h) = image::image_dimensions(path).map_err(|e| e.to_string())?;
    if u64::from(w).saturating_mul(u64::from(h)) > MAX_PIXELS {
        return Err(format!("{w}×{h} is larger than the {MAX_PIXELS} pixel limit"));
    }
    let mut reader = ImageReader::open(path).map_err(|e| e.to_string())?.with_guessed_format().map_err(|e| e.to_string())?;
    reader.limits(limits());
    let mut decoder = reader.into_decoder().map_err(|e| e.to_string())?;
    let orientation = decoder.orientation().map_err(|e| e.to_string())?;
    let mut img = DynamicImage::from_decoder(decoder).map_err(|e| e.to_string())?;
    img.apply_orientation(orientation);
    Ok(scaled(img, max_side))
}

fn limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_alloc = Some(MAX_ALLOC_BYTES);
    limits
}

/// Shrink to fit `max_side` (aspect kept) and flatten to RGBA8.
fn scaled(mut img: DynamicImage, max_side: u32) -> Rgba {
    let max_side = max_side.max(1);
    if img.width() > max_side || img.height() > max_side {
        img = img.thumbnail(max_side, max_side);
    }
    let rgba = img.into_rgba8();
    Rgba { width: rgba.width(), height: rgba.height(), pixels: rgba.into_raw() }
}

/// Camera raw files. Nearly every body carries a JPEG it rendered from the same sensor data,
/// so that is what we show (what Bridge draws from its cache previews; the default for CNT-10's
/// 埋め込み優先): fast, and the colour the camera intended. CR3 holds nothing else at all
/// (photocraft-raw reads no CRX sensor data either). Only a raw with no usable preview is
/// developed from the sensor data instead.
fn decode_raw(path: &Path, max_side: u32) -> Result<Rgba, String> {
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > MAX_ALLOC_BYTES {
        return Err(format!("raw file is larger than the {} GiB limit", MAX_ALLOC_BYTES >> 30));
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    // The smallest preview that still covers the request (cameras often store several);
    // fall back to the largest when none does, and to developing when there is none at all.
    let previews = photocraft_raw::embedded_previews(&bytes);
    let pixels = |p: &photocraft_raw::Preview| u64::from(p.width).saturating_mul(u64::from(p.height));
    let want = max_side.max(1);
    let usable = previews.iter().filter(|p| pixels(p) <= MAX_PIXELS);
    let preview =
        usable.clone().filter(|p| p.width.max(p.height) >= want).min_by_key(|p| u64::from(p.width.max(p.height))).or_else(|| usable.max_by_key(|p| pixels(p)));
    if let Some(preview) = preview
        && let Ok((mut img, orientation)) = decode_jpeg(preview.jpeg)
    {
        let orientation = if orientation == Orientation::NoTransforms {
            // The container's orientation for a preview that carries none of its own (CR3 CMT1).
            let container = photocraft_raw::orientation(&bytes).unwrap_or(1).min(8) as u8;
            Orientation::from_exif(container).unwrap_or(Orientation::NoTransforms)
        } else {
            orientation
        };
        img.apply_orientation(orientation);
        return Ok(scaled(img, max_side));
    }
    let dev = photocraft_raw::develop(&bytes, &photocraft_raw::DevelopOptions::default()).map_err(|e| e.to_string())?;
    // dev.rgb: interleaved 16-bit RGB, already oriented (DevelopOptions::orient defaults on).
    let buf = image::ImageBuffer::<image::Rgb<u16>, Vec<u16>>::from_raw(dev.width, dev.height, dev.rgb).ok_or("developed size disagrees with the buffer")?;
    Ok(scaled(DynamicImage::ImageRgb16(buf), max_side))
}

fn decode_jpeg(jpeg: &[u8]) -> Result<(DynamicImage, Orientation), String> {
    let mut reader = ImageReader::new(Cursor::new(jpeg)).with_guessed_format().map_err(|e| e.to_string())?;
    reader.limits(limits());
    let mut decoder = reader.into_decoder().map_err(|e| e.to_string())?;
    let orientation = decoder.orientation().map_err(|e| e.to_string())?;
    let img = DynamicImage::from_decoder(decoder).map_err(|e| e.to_string())?;
    Ok((img, orientation))
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
