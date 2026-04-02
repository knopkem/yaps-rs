//! EXIF metadata reader using the `kamadak-exif` crate.
//!
//! Provides pure-Rust EXIF extraction with no external binary dependencies.

use std::io::BufReader;
use std::path::Path;

use super::date::parse_exif_datetime;
use super::fields::ExifMetadata;

/// Reads EXIF metadata from image files.
pub struct ExifReader;

impl ExifReader {
    /// Read EXIF metadata from a file.
    ///
    /// Returns metadata with all extractable fields populated. Fields that
    /// cannot be read are left as `None`.
    ///
    /// # Errors
    /// Returns `YapsError::Exif` if the file cannot be opened.
    /// Note: missing EXIF data is not an error — the metadata will simply have `None` fields.
    pub fn read(path: impl AsRef<Path>) -> crate::Result<ExifMetadata> {
        let path = path.as_ref();
        let mut meta = ExifMetadata::default();

        // Populate file info
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            meta.filename = Some(stem.to_string());
        }
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            meta.extension = Some(ext.to_lowercase());
        }
        meta.source_path = Some(path.to_path_buf());

        // Open and parse EXIF
        let file = std::fs::File::open(path).map_err(|e| crate::YapsError::io(path, e))?;
        let mut reader = BufReader::new(file);

        let Ok(exif) = exif::Reader::new().read_from_container(&mut reader) else {
            // No EXIF data — return metadata with file info only
            tracing::debug!("No EXIF data in {}", path.display());
            return Ok(meta);
        };

        // Extract date/time
        if let Some(field) = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
            let value = field.display_value().to_string();
            meta.date_time_original = parse_exif_datetime(&value);
        }

        // Camera info
        meta.camera_make = get_string_field(&exif, exif::Tag::Make);
        meta.camera_model = get_string_field(&exif, exif::Tag::Model);
        meta.lens_model = get_string_field(&exif, exif::Tag::LensModel);

        // Exposure settings
        meta.iso = get_iso(&exif);
        meta.aperture = get_rational_f64(&exif, exif::Tag::FNumber);
        meta.exposure_time = get_rational_f64(&exif, exif::Tag::ExposureTime);
        meta.exposure_time_display = get_exposure_display(&exif);
        meta.focal_length = get_rational_f64(&exif, exif::Tag::FocalLength);

        // Dimensions
        meta.width = get_u32_field(&exif, exif::Tag::PixelXDimension);
        meta.height = get_u32_field(&exif, exif::Tag::PixelYDimension);
        meta.orientation = get_u16_field(&exif, exif::Tag::Orientation);

        // GPS
        meta.gps_latitude =
            get_gps_coordinate(&exif, exif::Tag::GPSLatitude, exif::Tag::GPSLatitudeRef);
        meta.gps_longitude =
            get_gps_coordinate(&exif, exif::Tag::GPSLongitude, exif::Tag::GPSLongitudeRef);
        if let Some(field) = exif.get_field(exif::Tag::GPSAltitude, exif::In::PRIMARY) {
            meta.gps_altitude = rational_to_f64(&field.value);
        }

        Ok(meta)
    }
}

/// Extract a string field from EXIF, trimming whitespace and null bytes.
fn get_string_field(exif: &exif::Exif, tag: exif::Tag) -> Option<String> {
    exif.get_field(tag, exif::In::PRIMARY).map(|f| {
        f.display_value()
            .to_string()
            .trim()
            .trim_matches('"')
            .to_string()
    })
}

/// Extract a u32 field.
fn get_u32_field(exif: &exif::Exif, tag: exif::Tag) -> Option<u32> {
    exif.get_field(tag, exif::In::PRIMARY)
        .and_then(|f| match &f.value {
            exif::Value::Long(v) if !v.is_empty() => Some(v[0]),
            exif::Value::Short(v) if !v.is_empty() => Some(u32::from(v[0])),
            _ => None,
        })
}

/// Extract a u16 field.
fn get_u16_field(exif: &exif::Exif, tag: exif::Tag) -> Option<u16> {
    exif.get_field(tag, exif::In::PRIMARY)
        .and_then(|f| match &f.value {
            exif::Value::Short(v) if !v.is_empty() => Some(v[0]),
            _ => None,
        })
}

/// Extract ISO speed value.
fn get_iso(exif: &exif::Exif) -> Option<u32> {
    exif.get_field(exif::Tag::PhotographicSensitivity, exif::In::PRIMARY)
        .and_then(|f| match &f.value {
            exif::Value::Short(v) if !v.is_empty() => Some(u32::from(v[0])),
            exif::Value::Long(v) if !v.is_empty() => Some(v[0]),
            _ => None,
        })
}

/// Extract a rational value as f64.
fn get_rational_f64(exif: &exif::Exif, tag: exif::Tag) -> Option<f64> {
    exif.get_field(tag, exif::In::PRIMARY)
        .and_then(|f| rational_to_f64(&f.value))
}

/// Convert an EXIF Value containing a rational to f64.
fn rational_to_f64(value: &exif::Value) -> Option<f64> {
    match value {
        exif::Value::Rational(v) if !v.is_empty() => {
            if v[0].denom == 0 {
                None
            } else {
                Some(f64::from(v[0].num) / f64::from(v[0].denom))
            }
        }
        _ => None,
    }
}

/// Get a human-readable exposure time string (e.g., "1/250").
fn get_exposure_display(exif: &exif::Exif) -> Option<String> {
    exif.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY)
        .map(|f| f.display_value().to_string())
}

/// Parse GPS coordinates from EXIF rational values + reference direction.
fn get_gps_coordinate(exif: &exif::Exif, coord_tag: exif::Tag, ref_tag: exif::Tag) -> Option<f64> {
    let field = exif.get_field(coord_tag, exif::In::PRIMARY)?;
    let rationals = match &field.value {
        exif::Value::Rational(v) if v.len() >= 3 => v,
        _ => return None,
    };

    let degrees = f64::from(rationals[0].num) / f64::from(rationals[0].denom);
    let minutes = f64::from(rationals[1].num) / f64::from(rationals[1].denom);
    let seconds = f64::from(rationals[2].num) / f64::from(rationals[2].denom);

    let mut decimal = degrees + minutes / 60.0 + seconds / 3600.0;

    // Check reference for sign (S/W are negative)
    if let Some(ref_field) = exif.get_field(ref_tag, exif::In::PRIMARY) {
        let ref_str = ref_field.display_value().to_string();
        if ref_str.contains('S') || ref_str.contains('W') {
            decimal = -decimal;
        }
    }

    Some(decimal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_nonexistent_file_returns_error() {
        let result = ExifReader::read("/nonexistent/photo.jpg");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_non_image_file_returns_metadata_without_exif() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "not an image").unwrap();

        let meta = ExifReader::read(&path).unwrap();
        assert!(!meta.has_date());
        assert_eq!(meta.filename.as_deref(), Some("test"));
        assert_eq!(meta.extension.as_deref(), Some("txt"));
    }

    #[test]
    fn test_file_info_populated_even_without_exif() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("DSC_0001.jpg");
        // Write minimal bytes — not a valid JPEG but tests file info extraction
        std::fs::write(&path, b"\xff\xd8\xff\xe0").unwrap();

        let meta = ExifReader::read(&path).unwrap();
        assert_eq!(meta.filename.as_deref(), Some("DSC_0001"));
        assert_eq!(meta.extension.as_deref(), Some("jpg"));
        assert!(meta.source_path.is_some());
    }
}
