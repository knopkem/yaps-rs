//! Configuration types for yaps-rs.
//!
//! Settings are persisted as TOML files at the platform-standard config directory.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// How to handle file operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FileOperation {
    /// Copy files to the target (source remains untouched).
    #[default]
    Copy,
    /// Move files to the target (source is removed on success).
    Move,
    /// Create hard links at the target (same inode, no extra disk space).
    Hardlink,
    /// Create symbolic links at the target pointing to the source.
    Symlink,
}

/// How to resolve filename conflicts at the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ConflictStrategy {
    /// Skip the file and log a warning.
    #[default]
    Skip,
    /// Auto-rename with an incrementing suffix: `photo(1).jpg`, `photo(2).jpg`, etc.
    Rename,
    /// Overwrite the existing file at the target.
    Overwrite,
}

/// How to handle duplicate files (same content, detected by hash).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DuplicateStrategy {
    /// Skip duplicates silently.
    #[default]
    Skip,
    /// Copy duplicates into a special `[Duplicates]` subfolder.
    CopyToFolder,
}

/// Complete configuration for a yaps-rs sorting operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Source directory to scan for photos.
    pub source: PathBuf,
    /// Target root directory for organized output.
    pub target: PathBuf,
    /// Whether to recurse into subdirectories of the source.
    pub recursive: bool,
    /// File operation mode.
    pub file_operation: FileOperation,
    /// How to resolve filename conflicts.
    pub conflict_strategy: ConflictStrategy,
    /// Whether to detect and handle duplicate files.
    pub detect_duplicates: bool,
    /// How to handle detected duplicates.
    pub duplicate_strategy: DuplicateStrategy,
    /// Pattern for the folder structure (e.g., `{year}/{month}-{month_long}`).
    pub folder_pattern: String,
    /// Pattern for the filename (e.g., `{day}-{hour}{minute}{second}-{filename}`).
    pub file_pattern: String,
    /// If true, only preview operations without executing them.
    pub dry_run: bool,
    /// Name of the special folder for files without EXIF data.
    pub no_exif_folder: String,
    /// Name of the special folder for duplicate files.
    pub duplicates_folder: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            source: PathBuf::new(),
            target: PathBuf::new(),
            recursive: true,
            file_operation: FileOperation::default(),
            conflict_strategy: ConflictStrategy::default(),
            detect_duplicates: true,
            duplicate_strategy: DuplicateStrategy::default(),
            folder_pattern: "{year}/{month}-{month_long}".to_string(),
            file_pattern: "{day}-{month_short}-{hour}{minute}{second}-{filename}".to_string(),
            dry_run: false,
            no_exif_folder: "[NoExifData]".to_string(),
            duplicates_folder: "[Duplicates]".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn load(path: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| crate::YapsError::io(path.as_ref(), e))?;
        toml::from_str(&content).map_err(|e| crate::YapsError::Config(e.to_string()))
    }

    /// Save configuration to a TOML file.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> crate::Result<()> {
        let content =
            toml::to_string_pretty(self).map_err(|e| crate::YapsError::Config(e.to_string()))?;
        std::fs::write(path.as_ref(), content)
            .map_err(|e| crate::YapsError::io(path.as_ref(), e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.recursive);
        assert!(config.detect_duplicates);
        assert!(!config.dry_run);
        assert_eq!(config.file_operation, FileOperation::Copy);
        assert_eq!(config.conflict_strategy, ConflictStrategy::Skip);
        assert_eq!(config.duplicate_strategy, DuplicateStrategy::Skip);
        assert_eq!(config.folder_pattern, "{year}/{month}-{month_long}");
        assert_eq!(config.no_exif_folder, "[NoExifData]");
        assert_eq!(config.duplicates_folder, "[Duplicates]");
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = Config {
            source: PathBuf::from("/photos/raw"),
            target: PathBuf::from("/photos/sorted"),
            recursive: false,
            file_operation: FileOperation::Move,
            conflict_strategy: ConflictStrategy::Rename,
            detect_duplicates: true,
            duplicate_strategy: DuplicateStrategy::CopyToFolder,
            folder_pattern: "{year}/{month}".to_string(),
            file_pattern: "{filename}".to_string(),
            dry_run: true,
            no_exif_folder: "_no_exif".to_string(),
            duplicates_folder: "_dupes".to_string(),
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.source, config.source);
        assert_eq!(deserialized.target, config.target);
        assert_eq!(deserialized.recursive, config.recursive);
        assert_eq!(deserialized.file_operation, config.file_operation);
        assert_eq!(deserialized.conflict_strategy, config.conflict_strategy);
        assert_eq!(deserialized.duplicate_strategy, config.duplicate_strategy);
        assert_eq!(deserialized.folder_pattern, config.folder_pattern);
        assert_eq!(deserialized.file_pattern, config.file_pattern);
        assert!(deserialized.dry_run);
    }

    #[test]
    fn test_config_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let config = Config {
            source: PathBuf::from("/test/source"),
            target: PathBuf::from("/test/target"),
            ..Default::default()
        };

        config.save(&path).unwrap();
        let loaded = Config::load(&path).unwrap();

        assert_eq!(loaded.source, config.source);
        assert_eq!(loaded.target, config.target);
        assert_eq!(loaded.folder_pattern, config.folder_pattern);
    }

    #[test]
    fn test_config_load_nonexistent_returns_error() {
        let result = Config::load("/nonexistent/config.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_load_invalid_toml_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, "this is not valid toml {{{{").unwrap();

        let result = Config::load(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_operation_serde() {
        #[derive(Serialize, Deserialize)]
        struct Wrapper {
            op: FileOperation,
        }

        let w = Wrapper {
            op: FileOperation::Hardlink,
        };
        let s = toml::to_string(&w).unwrap();
        assert!(s.contains("hardlink"));

        let parsed: Wrapper = toml::from_str(&s).unwrap();
        assert_eq!(parsed.op, FileOperation::Hardlink);
    }
}
