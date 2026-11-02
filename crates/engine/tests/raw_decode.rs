//! Sentinel for raw previews: a `.cr3` / `.raf` / preview-less `.dng` shown in the grid must
//! decode to pixels, not to an error. Files are synthetic (`photocraft-raw` testgen plus a
//! procedurally encoded JPEG); no camera photos are needed (AGENTS.md: no personal photos).

use std::io::Cursor;
use std::path::{Path, PathBuf};

use photocraft_raw::testgen::{DngSpec, cr3_preview};

/// A real baseline JPEG (the `image` encoder), so the preview decode path is exercised end to end.
fn jpeg_bytes(width: u32, height: u32) -> Vec<u8> {
    let img = image::RgbImage::from_fn(width, height, |x, y| image::Rgb([(x * 3) as u8, (y * 5) as u8, ((x + y) * 2) as u8]));
    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), image::ImageFormat::Jpeg).expect("encode");
    out
}

fn temp_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pier-raw-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir.join(name)
}

fn write(name: &str, bytes: &[u8]) -> PathBuf {
    let p = temp_path(name);
    std::fs::write(&p, bytes).expect("write");
    p
}

fn cleanup(p: &Path) {
    let _ = std::fs::remove_file(p);
    let _ = std::fs::remove_dir(p.parent().expect("parent"));
}

#[test]
fn cr3_preview_decodes() {
    let p = write("shot.cr3", &cr3_preview(&jpeg_bytes(400, 200), 1));
    let out = pier_engine::decode::decode_scaled(&p, 512);
    cleanup(&p);
    let out = out.expect("the CR3 camera preview must decode");
    assert_eq!((out.width, out.height), (400, 200));
    assert_eq!(out.pixels.len(), 400 * 200 * 4);
}

#[test]
fn cr3_cmt1_orientation_is_applied() {
    // Orientation 6: the camera rendered the frame rotated 90°, so the decoded buffer is upright.
    let p = write("rot.cr3", &cr3_preview(&jpeg_bytes(400, 200), 6));
    let out = pier_engine::decode::decode_scaled(&p, 512);
    cleanup(&p);
    let out = out.expect("the CR3 camera preview must decode");
    assert_eq!((out.width, out.height), (200, 400));
}

#[test]
fn raf_preview_decodes() {
    // Minimal RAF container: header magic, big-endian JPEG offset / length at bytes 84 and 88.
    let jpeg = jpeg_bytes(320, 240);
    let mut file = vec![0u8; 92];
    file[..15].copy_from_slice(b"FUJIFILMCCD-RAW");
    file[84..88].copy_from_slice(&92u32.to_be_bytes());
    file[88..92].copy_from_slice(&(jpeg.len() as u32).to_be_bytes());
    file.extend_from_slice(&jpeg);
    let p = write("shot.raf", &file);
    let out = pier_engine::decode::decode_scaled(&p, 512);
    cleanup(&p);
    let out = out.expect("the RAF camera preview must decode");
    assert_eq!((out.width, out.height), (320, 240));
}

#[test]
fn preview_less_dng_develops() {
    let data = vec![1000u16; 24 * 16];
    let file = DngSpec::cfa(24, 16, data).build();
    assert!(photocraft_raw::embedded_previews(&file).is_empty(), "the fixture must have no embedded preview");
    let p = write("synthetic.dng", &file);
    let out = pier_engine::decode::decode_scaled(&p, 512);
    cleanup(&p);
    let out = out.expect("a raw without a preview must develop from the sensor data");
    assert_eq!((out.width, out.height), (24, 16));
}
