//! The metadata the Metadata panel shows: pixel dimensions and an EXIF summary (read only).

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use exif::{In, Tag};

#[derive(Clone, Debug, Default)]
pub struct MetaSummary {
    pub dimensions: Option<(u32, u32)>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens: Option<String>,
    pub f_number: Option<String>,
    pub exposure: Option<String>,
    pub iso: Option<String>,
    pub focal_length: Option<String>,
    pub exposure_bias: Option<String>,
    pub white_balance: Option<String>,
    pub flash: Option<String>,
    pub date_time_original: Option<String>,
    pub orientation: Option<String>,
    pub resolution: Option<String>,
    pub color_space: Option<String>,
    pub software: Option<String>,
    pub artist: Option<String>,
    pub copyright: Option<String>,
    pub gps_latitude: Option<String>,
    pub gps_longitude: Option<String>,
}

/// Never fails: unreadable parts are left empty.
pub fn read_summary(path: &Path) -> MetaSummary {
    let mut out = MetaSummary { dimensions: image::image_dimensions(path).ok(), ..MetaSummary::default() };
    let Ok(file) = File::open(path) else { return out };
    let Ok(exif) = exif::Reader::new().read_from_container(&mut BufReader::new(file)) else { return out };
    let get = |tag: Tag| exif.get_field(tag, In::PRIMARY).map(|f| f.display_value().with_unit(&exif).to_string().trim_matches('"').trim().to_owned());
    out.make = get(Tag::Make);
    out.model = get(Tag::Model);
    out.lens = get(Tag::LensModel);
    out.f_number = exif.get_field(Tag::FNumber, In::PRIMARY).map(|f| format!("f/{}", f.display_value()));
    out.exposure = exif.get_field(Tag::ExposureTime, In::PRIMARY).map(|f| f.display_value().to_string());
    out.iso = exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY).map(|f| format!("ISO {}", f.display_value()));
    out.focal_length = get(Tag::FocalLength);
    out.exposure_bias = get(Tag::ExposureBiasValue);
    out.white_balance = get(Tag::WhiteBalance);
    out.flash = get(Tag::Flash);
    out.date_time_original = get(Tag::DateTimeOriginal);
    out.orientation = get(Tag::Orientation);
    out.resolution = get(Tag::XResolution);
    out.color_space = get(Tag::ColorSpace);
    out.software = get(Tag::Software);
    out.artist = get(Tag::Artist);
    out.copyright = get(Tag::Copyright);
    out.gps_latitude = get(Tag::GPSLatitude);
    out.gps_longitude = get(Tag::GPSLongitude);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_without_exif_still_reports_dimensions() {
        let dir = std::env::temp_dir().join(format!("pier-meta-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("x.png");
        image::RgbaImage::new(3, 2).save(&p).unwrap();
        let m = read_summary(&p);
        assert_eq!(m.dimensions, Some((3, 2)));
        assert!(m.model.is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_file_is_empty() {
        let m = read_summary(Path::new("/no/such/file.jpg"));
        assert!(m.dimensions.is_none());
    }
}
