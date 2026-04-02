use yaps_core::exif::fields::ExifMetadata;
use yaps_core::pattern::formatter::format_pattern;
use yaps_core::pattern::parser::parse_pattern;

// ───────────────────── Default folder pattern ───────────────────────────

#[test]
fn default_folder_pattern_with_no_exif() {
    let pattern = parse_pattern("{year}/{month}-{month_long}").unwrap();
    let meta = ExifMetadata::default();

    let result = format_pattern(&pattern, &meta);

    // Without EXIF date, year/month should be "unknown".
    assert!(
        result.contains("unknown"),
        "pattern with no EXIF should contain 'unknown', got: {result}"
    );
}

// ───────────────────── Default file pattern ─────────────────────────────

#[test]
fn default_file_pattern_with_no_exif() {
    let pattern =
        parse_pattern("{day}-{month_short}-{hour}{minute}{second}-{filename}").unwrap();
    let meta = ExifMetadata {
        filename: Some("photo1".to_string()),
        extension: Some("jpg".to_string()),
        ..ExifMetadata::default()
    };

    let result = format_pattern(&pattern, &meta);

    assert!(
        result.contains("photo1"),
        "pattern should include the filename, got: {result}"
    );
    // Date parts should be "unknown" since no date_time_original.
    assert!(
        result.contains("unknown"),
        "date parts should be 'unknown' without EXIF, got: {result}"
    );
}

// ───────────────────── Complex pattern with multiple tags ───────────────

#[test]
fn complex_pattern_with_multiple_tags() {
    let pattern = parse_pattern("{make}_{model}/{iso}_{aperture}_{filename}").unwrap();
    let meta = ExifMetadata {
        camera_make: Some("Canon".to_string()),
        camera_model: Some("EOS R5".to_string()),
        iso: Some(400),
        aperture: Some(2.8),
        filename: Some("IMG_001".to_string()),
        ..ExifMetadata::default()
    };

    let result = format_pattern(&pattern, &meta);

    assert!(
        result.contains("Canon"),
        "should contain camera make, got: {result}"
    );
    assert!(
        result.contains("EOS R5"),
        "should contain camera model, got: {result}"
    );
    assert!(
        result.contains("400"),
        "should contain ISO value, got: {result}"
    );
    assert!(
        result.contains("IMG_001"),
        "should contain filename, got: {result}"
    );
}

// ───────────────────── Pattern with only literal text ───────────────────

#[test]
fn pattern_with_only_literal_text() {
    let pattern = parse_pattern("photos/my-album").unwrap();
    let meta = ExifMetadata::default();

    let result = format_pattern(&pattern, &meta);

    assert_eq!(
        result, "photos/my-album",
        "literal-only pattern should pass through unchanged"
    );
}

// ───────────────────── Pattern tags resolve to unknown when absent ──────

#[test]
fn missing_metadata_resolves_to_unknown() {
    let pattern = parse_pattern("{make}/{model}/{year}").unwrap();
    let meta = ExifMetadata::default();

    let result = format_pattern(&pattern, &meta);

    let parts: Vec<&str> = result.split('/').collect();
    assert_eq!(parts.len(), 3, "should have 3 segments, got: {result}");
    for part in &parts {
        assert_eq!(
            *part, "unknown",
            "all parts should be 'unknown', got: {result}"
        );
    }
}

// ───────────────────── Extension tag ────────────────────────────────────

#[test]
fn extension_tag_resolves() {
    let pattern = parse_pattern("{filename}.{ext}").unwrap();
    let meta = ExifMetadata {
        filename: Some("sunset".to_string()),
        extension: Some("jpg".to_string()),
        ..ExifMetadata::default()
    };

    let result = format_pattern(&pattern, &meta);

    assert_eq!(result, "sunset.jpg", "got: {result}");
}

// ───────────────────── Dimension and orientation tags ───────────────────

#[test]
fn dimension_tags_resolve() {
    let pattern = parse_pattern("{width}x{height}-{orientation}").unwrap();
    let meta = ExifMetadata {
        width: Some(4000),
        height: Some(3000),
        orientation: Some(1),
        ..ExifMetadata::default()
    };

    let result = format_pattern(&pattern, &meta);

    assert!(
        result.contains("4000"),
        "should contain width, got: {result}"
    );
    assert!(
        result.contains("3000"),
        "should contain height, got: {result}"
    );
}

// ───────────────────── Parsing invalid pattern ──────────────────────────

#[test]
fn invalid_pattern_returns_error() {
    let result = parse_pattern("{nonexistent_tag}");
    assert!(
        result.is_err(),
        "unknown tag should produce an error"
    );
}

// ───────────────────── Empty pattern ────────────────────────────────────

#[test]
fn empty_pattern_produces_empty_string() {
    let pattern = parse_pattern("").unwrap();
    assert!(pattern.is_empty());

    let meta = ExifMetadata::default();
    let result = format_pattern(&pattern, &meta);
    assert!(result.is_empty(), "empty pattern should produce empty output");
}

// ───────────────────── Media type tag ───────────────────────────────────

#[test]
fn media_type_tag_resolves() {
    let pattern = parse_pattern("{media_type}/{filename}").unwrap();
    let meta = ExifMetadata {
        filename: Some("clip".to_string()),
        extension: Some("mp4".to_string()),
        ..ExifMetadata::default()
    };

    let result = format_pattern(&pattern, &meta);

    // media_type depends on extension mapping; at minimum it should not be empty.
    assert!(!result.is_empty());
    assert!(
        result.contains("clip"),
        "should contain filename, got: {result}"
    );
}

// ───────────────────── Pattern roundtrip: parse → format deterministic ──

#[test]
fn pattern_roundtrip_is_deterministic() {
    let input = "{year}-{month}/{day}_{filename}";
    let pattern = parse_pattern(input).unwrap();
    let meta = ExifMetadata {
        filename: Some("test".to_string()),
        ..ExifMetadata::default()
    };

    let r1 = format_pattern(&pattern, &meta);
    let r2 = format_pattern(&pattern, &meta);

    assert_eq!(r1, r2, "formatting the same pattern twice should yield identical results");
}
