//! EXIF metadata extraction module.
//!
//! Provides pure-Rust EXIF reading using the `kamadak-exif` crate.
//! Supports JPEG, HEIF, PNG, WebP, and TIFF formats.

pub mod date;
pub mod fields;
pub mod reader;

pub use fields::ExifMetadata;
pub use reader::ExifReader;
