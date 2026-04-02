#![allow(dead_code)]

use std::fs;
use std::path::Path;

use tempfile::{tempdir, TempDir};

/// Minimal JPEG file header (smallest valid JPEG).
const JPEG_HEADER: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x02, 0x00, 0x00, 0xFF, 0xD9];

/// Minimal PNG file header.
const PNG_HEADER: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// Create a temporary directory with a few fake image files at the top level.
///
/// Layout:
///   photo1.jpg
///   photo2.jpg
///   photo3.png
pub fn create_test_dir() -> TempDir {
    let dir = tempdir().expect("failed to create temp dir");
    fs::write(dir.path().join("photo1.jpg"), JPEG_HEADER).unwrap();
    fs::write(dir.path().join("photo2.jpg"), JPEG_HEADER).unwrap();
    fs::write(dir.path().join("photo3.png"), PNG_HEADER).unwrap();
    dir
}

/// Create a temporary directory with nested subdirectories containing media files.
///
/// Layout:
///   top1.jpg
///   top2.png
///   subdir_a/
///     nested1.jpg
///     nested2.jpg
///   subdir_b/
///     deep/
///       deep1.jpg
pub fn create_test_dir_with_structure() -> TempDir {
    let dir = tempdir().expect("failed to create temp dir");

    fs::write(dir.path().join("top1.jpg"), JPEG_HEADER).unwrap();
    fs::write(dir.path().join("top2.png"), PNG_HEADER).unwrap();

    let sub_a = dir.path().join("subdir_a");
    fs::create_dir(&sub_a).unwrap();
    fs::write(sub_a.join("nested1.jpg"), JPEG_HEADER).unwrap();
    fs::write(sub_a.join("nested2.jpg"), JPEG_HEADER).unwrap();

    let deep = dir.path().join("subdir_b").join("deep");
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("deep1.jpg"), JPEG_HEADER).unwrap();

    dir
}

/// Recursively count all **files** (not directories) under `dir`.
pub fn count_files_recursive(dir: &Path) -> usize {
    let mut count = 0;
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                count += count_files_recursive(&path);
            } else {
                count += 1;
            }
        }
    }
    count
}
