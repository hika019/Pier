//! A demo folder: original, procedurally drawn landscapes (no third-party images), a few other file
//! types and sub-folders.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use image::{Rgb, RgbImage};

/// `(file, sky top, sky bottom, ground, portrait)`.
type Scene = (&'static str, [u8; 3], [u8; 3], [u8; 3], bool);

const SCENES: [Scene; 14] = [
    ("aurora-03.jpg", [10, 20, 60], [40, 200, 140], [20, 30, 40], false),
    ("dunes-01.jpg", [250, 190, 120], [255, 230, 190], [200, 140, 70], false),
    ("harbor-02.jpg", [90, 140, 200], [200, 225, 245], [40, 70, 110], false),
    ("lake-04.jpg", [120, 170, 220], [230, 240, 250], [60, 110, 80], true),
    ("meadow-05.jpg", [140, 190, 240], [220, 235, 255], [90, 150, 60], false),
    ("ridge-06.jpg", [230, 120, 80], [250, 200, 150], [60, 50, 70], false),
    ("sunset-07.jpg", [200, 60, 90], [255, 170, 90], [40, 30, 50], false),
    ("valley-08.jpg", [100, 150, 210], [210, 230, 250], [70, 120, 70], true),
    ("glacier-09.jpg", [150, 200, 230], [240, 250, 255], [180, 210, 230], false),
    ("canyon-10.jpg", [210, 110, 60], [250, 210, 160], [150, 70, 40], false),
    ("forest-11.jpg", [90, 130, 110], [200, 220, 200], [30, 70, 40], true),
    ("coast-12.jpg", [70, 120, 180], [190, 220, 240], [210, 190, 150], false),
    ("night-13.jpg", [5, 10, 30], [30, 40, 90], [10, 10, 20], false),
    ("dawn-14.jpg", [240, 170, 150], [255, 230, 200], [90, 80, 110], false),
];

/// The other files: placeholders whose kind the extension decides.
const OTHER_FILES: [&str; 6] = ["poster.psd", "IMG_0412.CR3", "logo.svg", "brochure.pdf", "teaser.mp4", "theme.wav"];

/// Writes the demo into `dir` (created; existing files with the same names are replaced).
pub fn write(dir: &Path) -> Result<(), String> {
    for sub in ["Raw Exports", "Selects"] {
        std::fs::create_dir_all(dir.join(sub)).map_err(|e| e.to_string())?;
    }
    for (i, (name, top, bottom, ground, portrait)) in SCENES.iter().enumerate() {
        let (w, h) = if *portrait { (800, 1200) } else { (1200, 800) };
        landscape(w, h, *top, *bottom, *ground, i as u32).save(dir.join(name)).map_err(|e| e.to_string())?;
    }
    for name in OTHER_FILES {
        std::fs::write(dir.join(name), b"demo").map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// A fresh demo folder under the system temp folder, unique to this call (tests run in parallel).
pub fn temp(tag: &str) -> Result<PathBuf, String> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("pier-demo-{tag}-{}-{n}", std::process::id()));
    write(&dir)?;
    Ok(dir)
}

fn lerp(a: [u8; 3], b: [u8; 3], t: f32) -> Rgb<u8> {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round().clamp(0.0, 255.0) as u8;
    Rgb([m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2])])
}

fn landscape(w: u32, h: u32, top: [u8; 3], bottom: [u8; 3], ground: [u8; 3], seed: u32) -> RgbImage {
    let horizon = h as f32 * 0.62;
    let sun = (w as f32 * (0.25 + 0.5 * ((seed * 37 % 10) as f32 / 10.0)), horizon * 0.55, h as f32 * 0.07);
    RgbImage::from_fn(w, h, |x, y| {
        let (fx, fy) = (x as f32, y as f32);
        let ridge = horizon - (fx * 0.006 + seed as f32).sin() * h as f32 * 0.08 - (fx * 0.017 + seed as f32 * 2.0).sin() * h as f32 * 0.03;
        if fy > ridge {
            let t = ((fy - ridge) / (h as f32 - ridge)).clamp(0.0, 1.0);
            return lerp(ground, [ground[0] / 2, ground[1] / 2, ground[2] / 2], t);
        }
        let d = ((fx - sun.0).powi(2) + (fy - sun.1).powi(2)).sqrt();
        if d < sun.2 {
            return Rgb([255, 245, 215]);
        }
        let mut c = lerp(top, bottom, fy / ridge.max(1.0));
        let glow = (1.0 - (d - sun.2) / (w as f32 * 0.35)).clamp(0.0, 1.0) * 0.35;
        for ch in c.0.iter_mut() {
            *ch = (*ch as f32 + (255.0 - *ch as f32) * glow) as u8;
        }
        c
    })
}
