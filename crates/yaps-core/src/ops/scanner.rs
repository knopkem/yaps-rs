//! Recursive directory scanner using `walkdir`.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

/// Known image/video extensions for filtering.
const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp", "heic", "heif", "avif", "raw",
    "cr2", "cr3", "nef", "arw", "orf", "rw2", "dng", "raf", "pef", "srw",
];

const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mov", "avi", "mkv", "wmv", "flv", "m4v", "3gp", "mts", "m2ts",
];

/// Scans directories for image and video files.
pub struct Scanner;

/// Result of a directory scan.
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// All discovered file paths.
    pub files: Vec<PathBuf>,
    /// Number of directories traversed.
    pub dirs_traversed: usize,
}

impl Scanner {
    /// Scan a directory for image and video files.
    ///
    /// # Arguments
    /// * `path` — The root directory to scan.
    /// * `recursive` — Whether to recurse into subdirectories.
    ///
    /// # Errors
    /// Returns `YapsError::SourceNotFound` if the path does not exist or is not a directory.
    pub fn scan(path: impl AsRef<Path>, recursive: bool) -> crate::Result<ScanResult> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(crate::YapsError::SourceNotFound {
                path: path.to_path_buf(),
            });
        }

        if !path.is_dir() {
            return Err(crate::YapsError::SourceNotFound {
                path: path.to_path_buf(),
            });
        }

        let max_depth = if recursive { usize::MAX } else { 1 };

        let mut files = Vec::new();
        let mut dirs_traversed = 0;

        for entry in WalkDir::new(path)
            .max_depth(max_depth)
            .follow_links(false)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if entry.file_type().is_dir() {
                dirs_traversed += 1;
                continue;
            }

            if entry.file_type().is_file() && is_supported_file(entry.path()) {
                files.push(entry.into_path());
            }
        }

        tracing::info!(
            "Scanned {} — found {} files in {} directories",
            path.display(),
            files.len(),
            dirs_traversed
        );

        Ok(ScanResult {
            files,
            dirs_traversed,
        })
    }
}

/// Check if a file has a supported image or video extension.
fn is_supported_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            let ext_lower = ext.to_lowercase();
            IMAGE_EXTENSIONS.contains(&ext_lower.as_str())
                || VIDEO_EXTENSIONS.contains(&ext_lower.as_str())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        let result = Scanner::scan(dir.path(), true).unwrap();
        assert!(result.files.is_empty());
    }

    #[test]
    fn test_scan_finds_images() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("photo.jpg"), b"fake jpg").unwrap();
        std::fs::write(dir.path().join("photo.png"), b"fake png").unwrap();
        std::fs::write(dir.path().join("readme.txt"), b"text file").unwrap();

        let result = Scanner::scan(dir.path(), true).unwrap();
        assert_eq!(result.files.len(), 2);
    }

    #[test]
    fn test_scan_finds_videos() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("clip.mp4"), b"fake mp4").unwrap();
        std::fs::write(dir.path().join("clip.mov"), b"fake mov").unwrap();

        let result = Scanner::scan(dir.path(), true).unwrap();
        assert_eq!(result.files.len(), 2);
    }

    #[test]
    fn test_scan_recursive() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("subdir");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(dir.path().join("top.jpg"), b"top").unwrap();
        std::fs::write(sub.join("nested.jpg"), b"nested").unwrap();

        let result = Scanner::scan(dir.path(), true).unwrap();
        assert_eq!(result.files.len(), 2);
    }

    #[test]
    fn test_scan_non_recursive() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("subdir");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(dir.path().join("top.jpg"), b"top").unwrap();
        std::fs::write(sub.join("nested.jpg"), b"nested").unwrap();

        let result = Scanner::scan(dir.path(), false).unwrap();
        assert_eq!(result.files.len(), 1);
    }

    #[test]
    fn test_scan_nonexistent_returns_error() {
        let result = Scanner::scan("/nonexistent/path", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_case_insensitive_extensions() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("photo.JPG"), b"upper").unwrap();
        std::fs::write(dir.path().join("photo.Jpeg"), b"mixed").unwrap();

        let result = Scanner::scan(dir.path(), true).unwrap();
        assert_eq!(result.files.len(), 2);
    }

    #[test]
    fn test_is_supported_image() {
        assert!(is_supported_file(Path::new("photo.jpg")));
        assert!(is_supported_file(Path::new("photo.HEIC")));
        assert!(is_supported_file(Path::new("raw.CR2")));
        assert!(!is_supported_file(Path::new("readme.txt")));
        assert!(!is_supported_file(Path::new("no_extension")));
    }

    #[test]
    fn test_is_supported_video() {
        assert!(is_supported_file(Path::new("clip.mp4")));
        assert!(is_supported_file(Path::new("clip.MOV")));
        assert!(!is_supported_file(Path::new("audio.mp3")));
    }

    #[test]
    fn test_scan_raw_formats() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("photo.cr2"), b"cr2").unwrap();
        std::fs::write(dir.path().join("photo.nef"), b"nef").unwrap();
        std::fs::write(dir.path().join("photo.dng"), b"dng").unwrap();

        let result = Scanner::scan(dir.path(), true).unwrap();
        assert_eq!(result.files.len(), 3);
    }
}
