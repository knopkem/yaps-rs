//! Pattern formatter — resolves pattern tags against EXIF metadata.

use chrono::{Datelike, NaiveDateTime, Timelike};

use super::parser::{ParsedPattern, PatternSegment};
use super::tags::PatternTag;
use crate::exif::fields::ExifMetadata;

/// Format a parsed pattern into a concrete string using the given metadata.
///
/// Tags whose values are unavailable are replaced with `"unknown"`.
///
/// # Examples
/// ```
/// use yaps_core::pattern::{parse_pattern, format_pattern};
/// use yaps_core::exif::ExifMetadata;
/// use chrono::NaiveDateTime;
///
/// let mut meta = ExifMetadata::default();
/// meta.date_time_original = Some(
///     NaiveDateTime::parse_from_str("2024-03-15 14:30:45", "%Y-%m-%d %H:%M:%S").unwrap()
/// );
/// meta.filename = Some("DSC_0001".to_string());
///
/// let pattern = parse_pattern("{year}/{month}-{month_long}").unwrap();
/// let result = format_pattern(&pattern, &meta);
/// assert_eq!(result, "2024/03-March");
/// ```
pub fn format_pattern(pattern: &ParsedPattern, meta: &ExifMetadata) -> String {
    let mut result = String::new();
    for segment in &pattern.segments {
        match segment {
            PatternSegment::Literal(text) => result.push_str(text),
            PatternSegment::Tag(tag) => result.push_str(&resolve_tag(*tag, meta)),
        }
    }
    result
}

/// Resolve a single tag to its string value.
fn resolve_tag(tag: PatternTag, meta: &ExifMetadata) -> String {
    match tag {
        // Date/time tags
        PatternTag::Year => format_dt(meta, |dt| format!("{:04}", dt.year())),
        PatternTag::Month => format_dt(meta, |dt| format!("{:02}", dt.month())),
        PatternTag::MonthShort => format_dt(meta, |dt| short_month_name(dt.month())),
        PatternTag::MonthLong => format_dt(meta, |dt| long_month_name(dt.month())),
        PatternTag::Day => format_dt(meta, |dt| format!("{:02}", dt.day())),
        PatternTag::DayShort => format_dt(meta, short_day_name),
        PatternTag::DayLong => format_dt(meta, long_day_name),
        PatternTag::Hour => format_dt(meta, |dt| format!("{:02}", dt.hour())),
        PatternTag::Minute => format_dt(meta, |dt| format!("{:02}", dt.minute())),
        PatternTag::Second => format_dt(meta, |dt| format!("{:02}", dt.second())),
        PatternTag::Week => format_dt(meta, |dt| format!("{:02}", dt.iso_week().week())),

        // Camera info
        PatternTag::Make => meta.camera_make.clone().unwrap_or_else(|| "unknown".to_string()),
        PatternTag::Model => meta.camera_model.clone().unwrap_or_else(|| "unknown".to_string()),
        PatternTag::Lens => meta.lens_model.clone().unwrap_or_else(|| "unknown".to_string()),

        // Exposure
        PatternTag::Iso => meta
            .iso.map_or_else(|| "unknown".to_string(), |v| v.to_string()),
        PatternTag::Aperture => meta
            .aperture.map_or_else(|| "unknown".to_string(), |v| format!("f{v:.1}")),
        PatternTag::Shutter => meta
            .exposure_time_display
            .clone().map_or_else(|| "unknown".to_string(), |s| s.replace('/', "-")),
        PatternTag::Focal => meta
            .focal_length.map_or_else(|| "unknown".to_string(), |v| format!("{v:.0}mm")),

        // Dimensions
        PatternTag::Width => meta
            .width.map_or_else(|| "unknown".to_string(), |v| v.to_string()),
        PatternTag::Height => meta
            .height.map_or_else(|| "unknown".to_string(), |v| v.to_string()),
        PatternTag::Orientation => meta.orientation_label().to_string(),

        // GPS
        PatternTag::GpsLat => meta
            .gps_latitude.map_or_else(|| "unknown".to_string(), |v| format!("{v:.6}")),
        PatternTag::GpsLon => meta
            .gps_longitude.map_or_else(|| "unknown".to_string(), |v| format!("{v:.6}")),

        // Media info
        PatternTag::MediaType => meta.media_type().to_string(),

        // File info
        PatternTag::Filename => meta
            .filename
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        PatternTag::Ext => meta
            .extension
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
    }
}

/// Helper: format a datetime-dependent tag, falling back to "unknown".
fn format_dt(meta: &ExifMetadata, f: impl FnOnce(&NaiveDateTime) -> String) -> String {
    meta.date_time_original
        .as_ref().map_or_else(|| "unknown".to_string(), f)
}

fn short_month_name(month: u32) -> String {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    }
    .to_string()
}

fn long_month_name(month: u32) -> String {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
    .to_string()
}

fn short_day_name(dt: &NaiveDateTime) -> String {
    match dt.weekday() {
        chrono::Weekday::Mon => "Mon",
        chrono::Weekday::Tue => "Tue",
        chrono::Weekday::Wed => "Wed",
        chrono::Weekday::Thu => "Thu",
        chrono::Weekday::Fri => "Fri",
        chrono::Weekday::Sat => "Sat",
        chrono::Weekday::Sun => "Sun",
    }
    .to_string()
}

fn long_day_name(dt: &NaiveDateTime) -> String {
    match dt.weekday() {
        chrono::Weekday::Mon => "Monday",
        chrono::Weekday::Tue => "Tuesday",
        chrono::Weekday::Wed => "Wednesday",
        chrono::Weekday::Thu => "Thursday",
        chrono::Weekday::Fri => "Friday",
        chrono::Weekday::Sat => "Saturday",
        chrono::Weekday::Sun => "Sunday",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::parser::parse_pattern;

    fn make_meta() -> ExifMetadata {
        ExifMetadata {
            date_time_original: Some(
                NaiveDateTime::parse_from_str("2024-03-15 14:30:45", "%Y-%m-%d %H:%M:%S").unwrap(),
            ),
            camera_make: Some("Nikon".to_string()),
            camera_model: Some("D850".to_string()),
            lens_model: Some("24-70mm f/2.8".to_string()),
            iso: Some(400),
            aperture: Some(2.8),
            exposure_time: Some(0.004),
            exposure_time_display: Some("1/250".to_string()),
            focal_length: Some(50.0),
            width: Some(8256),
            height: Some(5504),
            orientation: Some(1),
            gps_latitude: Some(48.856_600),
            gps_longitude: Some(2.352_200),
            gps_altitude: Some(35.0),
            filename: Some("DSC_0001".to_string()),
            extension: Some("jpg".to_string()),
            source_path: None,
        }
    }

    #[test]
    fn test_format_folder_pattern() {
        let pattern = parse_pattern("{year}/{month}-{month_long}").unwrap();
        let result = format_pattern(&pattern, &make_meta());
        assert_eq!(result, "2024/03-March");
    }

    #[test]
    fn test_format_file_pattern() {
        let pattern =
            parse_pattern("{day}-{month_short}-{hour}{minute}{second}-{filename}").unwrap();
        let result = format_pattern(&pattern, &make_meta());
        assert_eq!(result, "15-Mar-143045-DSC_0001");
    }

    #[test]
    fn test_format_camera_tags() {
        let pattern = parse_pattern("{make}/{model}").unwrap();
        let result = format_pattern(&pattern, &make_meta());
        assert_eq!(result, "Nikon/D850");
    }

    #[test]
    fn test_format_exposure_tags() {
        let pattern = parse_pattern("ISO{iso}_f{aperture}_{focal}").unwrap();
        let result = format_pattern(&pattern, &make_meta());
        assert_eq!(result, "ISO400_ff2.8_50mm");
    }

    #[test]
    fn test_format_missing_fields_show_unknown() {
        let meta = ExifMetadata::default();
        let pattern = parse_pattern("{year}/{make}").unwrap();
        let result = format_pattern(&pattern, &meta);
        assert_eq!(result, "unknown/unknown");
    }

    #[test]
    fn test_format_gps_tags() {
        let pattern = parse_pattern("{gps_lat},{gps_lon}").unwrap();
        let result = format_pattern(&pattern, &make_meta());
        assert_eq!(result, "48.856600,2.352200");
    }

    #[test]
    fn test_format_day_names() {
        // 2024-03-15 is a Friday
        let pattern = parse_pattern("{day_short}-{day_long}").unwrap();
        let result = format_pattern(&pattern, &make_meta());
        assert_eq!(result, "Fri-Friday");
    }

    #[test]
    fn test_format_week_number() {
        let pattern = parse_pattern("W{week}").unwrap();
        let result = format_pattern(&pattern, &make_meta());
        assert_eq!(result, "W11");
    }

    #[test]
    fn test_format_media_type() {
        let pattern = parse_pattern("{media_type}").unwrap();
        let result = format_pattern(&pattern, &make_meta());
        assert_eq!(result, "image");
    }

    #[test]
    fn test_format_orientation() {
        let pattern = parse_pattern("{orientation}").unwrap();
        let result = format_pattern(&pattern, &make_meta());
        assert_eq!(result, "landscape");
    }

    #[test]
    fn test_format_literal_only() {
        let pattern = parse_pattern("photos/sorted").unwrap();
        let result = format_pattern(&pattern, &make_meta());
        assert_eq!(result, "photos/sorted");
    }

    #[test]
    fn test_format_empty_pattern() {
        let pattern = parse_pattern("").unwrap();
        let result = format_pattern(&pattern, &make_meta());
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_extension() {
        let pattern = parse_pattern("{filename}.{ext}").unwrap();
        let result = format_pattern(&pattern, &make_meta());
        assert_eq!(result, "DSC_0001.jpg");
    }

    #[test]
    fn test_format_dimensions() {
        let pattern = parse_pattern("{width}x{height}").unwrap();
        let result = format_pattern(&pattern, &make_meta());
        assert_eq!(result, "8256x5504");
    }
}
