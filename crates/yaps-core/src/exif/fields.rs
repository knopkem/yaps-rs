//! EXIF metadata fields and the `ExifMetadata` struct.

use chrono::NaiveDateTime;

/// All metadata extracted from an image file.
///
/// Fields that could not be read from EXIF are `None`.
#[derive(Debug, Clone, Default)]
pub struct ExifMetadata {
    // -- Date/time --
    /// The original date/time when the photo was taken.
    pub date_time_original: Option<NaiveDateTime>,

    // -- Camera info --
    /// Camera manufacturer (e.g., "Nikon", "Canon").
    pub camera_make: Option<String>,
    /// Camera model (e.g., "D850", "EOS R5").
    pub camera_model: Option<String>,
    /// Lens model description.
    pub lens_model: Option<String>,

    // -- Exposure --
    /// ISO speed rating.
    pub iso: Option<u32>,
    /// Aperture f-number (e.g., 2.8).
    pub aperture: Option<f64>,
    /// Exposure time in seconds (e.g., 1/250 = 0.004).
    pub exposure_time: Option<f64>,
    /// Exposure time as a display string (e.g., "1/250").
    pub exposure_time_display: Option<String>,
    /// Focal length in mm.
    pub focal_length: Option<f64>,

    // -- Image dimensions --
    /// Image width in pixels.
    pub width: Option<u32>,
    /// Image height in pixels.
    pub height: Option<u32>,
    /// EXIF Orientation tag value (1-8).
    pub orientation: Option<u16>,

    // -- GPS --
    /// GPS latitude in decimal degrees (positive = North).
    pub gps_latitude: Option<f64>,
    /// GPS longitude in decimal degrees (positive = East).
    pub gps_longitude: Option<f64>,
    /// GPS altitude in meters.
    pub gps_altitude: Option<f64>,

    // -- File info (not from EXIF, populated by the scanner) --
    /// Original filename without extension.
    pub filename: Option<String>,
    /// File extension (lowercase, without dot).
    pub extension: Option<String>,
    /// Full source file path.
    pub source_path: Option<std::path::PathBuf>,
}

impl ExifMetadata {
    /// Returns `true` if this metadata has a valid original date/time.
    pub fn has_date(&self) -> bool {
        self.date_time_original.is_some()
    }

    /// Returns the media type derived from the file extension.
    pub fn media_type(&self) -> &str {
        match self.extension.as_deref() {
            Some(
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp" | "heic" | "heif"
                | "avif",
            ) => "image",
            Some("mp4" | "mov" | "avi" | "mkv" | "wmv" | "flv" | "m4v" | "3gp") => "video",
            Some("mp3" | "wav" | "aac" | "flac" | "ogg" | "wma") => "audio",
            _ => "other",
        }
    }

    /// Returns a human-readable orientation string.
    pub fn orientation_label(&self) -> &str {
        match (self.width, self.height) {
            (Some(w), Some(h)) if w > h => "landscape",
            (Some(w), Some(h)) if h > w => "portrait",
            (Some(w), Some(h)) if w == h => "square",
            _ => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_metadata_has_no_date() {
        let meta = ExifMetadata::default();
        assert!(!meta.has_date());
    }

    #[test]
    fn test_media_type_image() {
        let meta = ExifMetadata {
            extension: Some("jpg".to_string()),
            ..Default::default()
        };
        assert_eq!(meta.media_type(), "image");
    }

    #[test]
    fn test_media_type_video() {
        let meta = ExifMetadata {
            extension: Some("mp4".to_string()),
            ..Default::default()
        };
        assert_eq!(meta.media_type(), "video");
    }

    #[test]
    fn test_media_type_unknown() {
        let meta = ExifMetadata {
            extension: Some("xyz".to_string()),
            ..Default::default()
        };
        assert_eq!(meta.media_type(), "other");
    }

    #[test]
    fn test_orientation_landscape() {
        let meta = ExifMetadata {
            width: Some(6000),
            height: Some(4000),
            ..Default::default()
        };
        assert_eq!(meta.orientation_label(), "landscape");
    }

    #[test]
    fn test_orientation_portrait() {
        let meta = ExifMetadata {
            width: Some(4000),
            height: Some(6000),
            ..Default::default()
        };
        assert_eq!(meta.orientation_label(), "portrait");
    }

    #[test]
    fn test_orientation_square() {
        let meta = ExifMetadata {
            width: Some(4000),
            height: Some(4000),
            ..Default::default()
        };
        assert_eq!(meta.orientation_label(), "square");
    }

    #[test]
    fn test_has_date_with_datetime() {
        let meta = ExifMetadata {
            date_time_original: Some(
                NaiveDateTime::parse_from_str("2024-03-15 14:30:45", "%Y-%m-%d %H:%M:%S").unwrap(),
            ),
            ..Default::default()
        };
        assert!(meta.has_date());
    }
}
