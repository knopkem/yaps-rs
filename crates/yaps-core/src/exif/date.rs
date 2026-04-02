//! EXIF date/time parsing utilities.
//!
//! EXIF dates come in the format `"YYYY:MM:DD HH:MM:SS"`. This module provides
//! robust parsing that handles malformed dates gracefully.

use chrono::NaiveDateTime;

/// The standard EXIF date/time format.
const EXIF_DATETIME_FORMAT: &str = "%Y:%m:%d %H:%M:%S";

/// Alternative formats that some cameras/tools produce.
const ALTERNATIVE_FORMATS: &[&str] = &[
    "%Y-%m-%d %H:%M:%S",
    "%Y/%m/%d %H:%M:%S",
    "%Y:%m:%d %H:%M",
    "%Y-%m-%dT%H:%M:%S",
    "%Y-%m-%dT%H:%M:%S%.f",
];

/// Parse an EXIF date/time string into a `NaiveDateTime`.
///
/// Tries the standard EXIF format first, then falls back to alternatives.
/// Returns `None` if the string cannot be parsed.
///
/// # Examples
/// ```
/// use yaps_core::exif::date::parse_exif_datetime;
///
/// let dt = parse_exif_datetime("2024:03:15 14:30:45").unwrap();
/// assert_eq!(dt.to_string(), "2024-03-15 14:30:45");
///
/// // Also handles alternative formats
/// let dt = parse_exif_datetime("2024-03-15T14:30:45").unwrap();
/// assert_eq!(dt.to_string(), "2024-03-15 14:30:45");
/// ```
pub fn parse_exif_datetime(input: &str) -> Option<NaiveDateTime> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try standard EXIF format first
    if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, EXIF_DATETIME_FORMAT) {
        return Some(dt);
    }

    // Try alternative formats
    for fmt in ALTERNATIVE_FORMATS {
        if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, fmt) {
            return Some(dt);
        }
    }

    None
}

/// Attempt to extract a date/time from a filename.
///
/// Looks for patterns like `20240315_143045`, `2024-03-15_14-30-45`, etc.
pub fn parse_date_from_filename(filename: &str) -> Option<NaiveDateTime> {
    // Pattern: YYYYMMDD_HHMMSS or YYYYMMDD-HHMMSS
    let filename_formats = &[
        ("%Y%m%d_%H%M%S", 15),
        ("%Y%m%d-%H%M%S", 15),
        ("%Y-%m-%d_%H-%M-%S", 19),
        ("%Y%m%d", 8),
    ];

    for &(fmt, len) in filename_formats {
        // Try to find a date-like substring in the filename
        for start in 0..filename.len().saturating_sub(len - 1) {
            let end = (start + len).min(filename.len());
            let candidate = &filename[start..end];
            if let Ok(dt) = NaiveDateTime::parse_from_str(candidate, fmt) {
                return Some(dt);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_exif_format() {
        let dt = parse_exif_datetime("2024:03:15 14:30:45").unwrap();
        assert_eq!(dt.to_string(), "2024-03-15 14:30:45");
    }

    #[test]
    fn test_parse_dash_format() {
        let dt = parse_exif_datetime("2024-03-15 14:30:45").unwrap();
        assert_eq!(dt.to_string(), "2024-03-15 14:30:45");
    }

    #[test]
    fn test_parse_iso_format() {
        let dt = parse_exif_datetime("2024-03-15T14:30:45").unwrap();
        assert_eq!(dt.to_string(), "2024-03-15 14:30:45");
    }

    #[test]
    fn test_parse_empty_returns_none() {
        assert!(parse_exif_datetime("").is_none());
    }

    #[test]
    fn test_parse_garbage_returns_none() {
        assert!(parse_exif_datetime("not a date").is_none());
    }

    #[test]
    fn test_parse_whitespace_only_returns_none() {
        assert!(parse_exif_datetime("   ").is_none());
    }

    #[test]
    fn test_parse_with_leading_trailing_whitespace() {
        let dt = parse_exif_datetime("  2024:03:15 14:30:45  ").unwrap();
        assert_eq!(dt.to_string(), "2024-03-15 14:30:45");
    }

    #[test]
    fn test_parse_date_from_filename_yyyymmdd_hhmmss() {
        let dt = parse_date_from_filename("IMG_20240315_143045.jpg").unwrap();
        assert_eq!(dt.to_string(), "2024-03-15 14:30:45");
    }

    #[test]
    fn test_parse_date_from_filename_no_date() {
        assert!(parse_date_from_filename("vacation_photo.jpg").is_none());
    }
}
