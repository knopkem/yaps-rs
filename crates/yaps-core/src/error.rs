//! Error types for yaps-core.

use std::path::PathBuf;

/// All errors that can occur during yaps-core operations.
#[derive(Debug, thiserror::Error)]
pub enum YapsError {
    /// An I/O error occurred while reading or writing files.
    #[error("I/O error at '{}': {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to read EXIF metadata from a file.
    #[error("EXIF error for '{}': {message}", path.display())]
    Exif { path: PathBuf, message: String },

    /// A pattern string is invalid or contains unknown tags.
    #[error("invalid pattern: {0}")]
    InvalidPattern(String),

    /// Hash store file is corrupted or has an incompatible version.
    #[error("hash store error in '{}': {message}", path.display())]
    HashStore { path: PathBuf, message: String },

    /// The source directory does not exist or is not accessible.
    #[error("source directory not found: '{}'", path.display())]
    SourceNotFound { path: PathBuf },

    /// The target directory could not be created.
    #[error("failed to create target directory '{}': {source}", path.display())]
    TargetCreation {
        path: PathBuf,
        source: std::io::Error,
    },

    /// A file conflict occurred and the chosen strategy cannot resolve it.
    #[error("file conflict: '{}' already exists at target", path.display())]
    FileConflict { path: PathBuf },

    /// Configuration file is invalid.
    #[error("configuration error: {0}")]
    Config(String),
}

impl YapsError {
    /// Create an I/O error with path context.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_display() {
        let err = YapsError::io(
            "/some/path.jpg",
            std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
        );
        let msg = err.to_string();
        assert!(msg.contains("/some/path.jpg"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn test_invalid_pattern_display() {
        let err = YapsError::InvalidPattern("unknown tag {foobar}".to_string());
        assert_eq!(err.to_string(), "invalid pattern: unknown tag {foobar}");
    }

    #[test]
    fn test_exif_error_display() {
        let err = YapsError::Exif {
            path: PathBuf::from("/photo.jpg"),
            message: "no DateTimeOriginal".to_string(),
        };
        assert!(err.to_string().contains("photo.jpg"));
        assert!(err.to_string().contains("no DateTimeOriginal"));
    }
}
