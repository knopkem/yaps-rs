//! Available pattern tags.
//!
//! Each tag maps to a piece of EXIF metadata or file information.

use serde::{Deserialize, Serialize};

/// A tag that can appear in a pattern string.
///
/// Pattern strings use `{tag_name}` syntax, e.g., `{year}/{month}-{month_long}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternTag {
    // -- Date/time --
    /// 4-digit year (e.g., "2024").
    Year,
    /// 2-digit month (e.g., "03").
    Month,
    /// Abbreviated month name (e.g., "Mar").
    MonthShort,
    /// Full month name (e.g., "March").
    MonthLong,
    /// 2-digit day of month (e.g., "15").
    Day,
    /// Abbreviated day name (e.g., "Fri").
    DayShort,
    /// Full day name (e.g., "Friday").
    DayLong,
    /// 2-digit hour in 24h format (e.g., "14").
    Hour,
    /// 2-digit minute (e.g., "30").
    Minute,
    /// 2-digit second (e.g., "45").
    Second,
    /// ISO week number (e.g., "12").
    Week,

    // -- Camera info --
    /// Camera manufacturer.
    Make,
    /// Camera model.
    Model,
    /// Lens model.
    Lens,

    // -- Exposure --
    /// ISO speed rating.
    Iso,
    /// Aperture f-number (e.g., "f2.8").
    Aperture,
    /// Shutter speed (e.g., "1-250").
    Shutter,
    /// Focal length (e.g., "50mm").
    Focal,

    // -- Dimensions --
    /// Image width in pixels.
    Width,
    /// Image height in pixels.
    Height,
    /// Orientation label ("landscape", "portrait", "square").
    Orientation,

    // -- GPS --
    /// GPS latitude in decimal degrees.
    GpsLat,
    /// GPS longitude in decimal degrees.
    GpsLon,

    // -- Media info --
    /// Media type derived from extension ("image", "video", "audio", "other").
    MediaType,

    // -- File info --
    /// Original filename without extension.
    Filename,
    /// File extension (lowercase).
    Ext,
}

impl PatternTag {
    /// The string name used in pattern placeholders (e.g., "year" for `{year}`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Year => "year",
            Self::Month => "month",
            Self::MonthShort => "month_short",
            Self::MonthLong => "month_long",
            Self::Day => "day",
            Self::DayShort => "day_short",
            Self::DayLong => "day_long",
            Self::Hour => "hour",
            Self::Minute => "minute",
            Self::Second => "second",
            Self::Week => "week",
            Self::Make => "make",
            Self::Model => "model",
            Self::Lens => "lens",
            Self::Iso => "iso",
            Self::Aperture => "aperture",
            Self::Shutter => "shutter",
            Self::Focal => "focal",
            Self::Width => "width",
            Self::Height => "height",
            Self::Orientation => "orientation",
            Self::GpsLat => "gps_lat",
            Self::GpsLon => "gps_lon",
            Self::MediaType => "media_type",
            Self::Filename => "filename",
            Self::Ext => "ext",
        }
    }

    /// Look up a tag by its string name.
    pub fn from_name(name: &str) -> Option<Self> {
        ALL_TAGS.iter().copied().find(|t| t.name() == name)
    }

    /// A short description for UI display.
    pub fn description(self) -> &'static str {
        match self {
            Self::Year => "4-digit year (2024)",
            Self::Month => "2-digit month (03)",
            Self::MonthShort => "Short month name (Mar)",
            Self::MonthLong => "Full month name (March)",
            Self::Day => "2-digit day (15)",
            Self::DayShort => "Short day name (Fri)",
            Self::DayLong => "Full day name (Friday)",
            Self::Hour => "2-digit hour, 24h (14)",
            Self::Minute => "2-digit minute (30)",
            Self::Second => "2-digit second (45)",
            Self::Week => "ISO week number (12)",
            Self::Make => "Camera manufacturer (Nikon)",
            Self::Model => "Camera model (D850)",
            Self::Lens => "Lens model (24-70mm f/2.8)",
            Self::Iso => "ISO speed (400)",
            Self::Aperture => "Aperture (f2.8)",
            Self::Shutter => "Shutter speed (1-250)",
            Self::Focal => "Focal length (50mm)",
            Self::Width => "Image width in pixels",
            Self::Height => "Image height in pixels",
            Self::Orientation => "Orientation (landscape/portrait/square)",
            Self::GpsLat => "GPS latitude",
            Self::GpsLon => "GPS longitude",
            Self::MediaType => "Media type (image/video/audio/other)",
            Self::Filename => "Original filename",
            Self::Ext => "File extension",
        }
    }
}

/// All available pattern tags.
pub const ALL_TAGS: &[PatternTag] = &[
    PatternTag::Year,
    PatternTag::Month,
    PatternTag::MonthShort,
    PatternTag::MonthLong,
    PatternTag::Day,
    PatternTag::DayShort,
    PatternTag::DayLong,
    PatternTag::Hour,
    PatternTag::Minute,
    PatternTag::Second,
    PatternTag::Week,
    PatternTag::Make,
    PatternTag::Model,
    PatternTag::Lens,
    PatternTag::Iso,
    PatternTag::Aperture,
    PatternTag::Shutter,
    PatternTag::Focal,
    PatternTag::Width,
    PatternTag::Height,
    PatternTag::Orientation,
    PatternTag::GpsLat,
    PatternTag::GpsLon,
    PatternTag::MediaType,
    PatternTag::Filename,
    PatternTag::Ext,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_tags_have_unique_names() {
        let mut names: Vec<&str> = ALL_TAGS.iter().map(|t| t.name()).collect();
        let len_before = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), len_before, "Duplicate tag names found");
    }

    #[test]
    fn test_from_name_roundtrip() {
        for tag in ALL_TAGS {
            let name = tag.name();
            let resolved = PatternTag::from_name(name);
            assert_eq!(resolved, Some(*tag), "Failed roundtrip for {name}");
        }
    }

    #[test]
    fn test_from_name_unknown_returns_none() {
        assert_eq!(PatternTag::from_name("nonexistent"), None);
    }

    #[test]
    fn test_all_tags_have_descriptions() {
        for tag in ALL_TAGS {
            assert!(!tag.description().is_empty(), "Missing description for {tag:?}");
        }
    }
}
