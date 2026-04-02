//! Main orchestrator — ties together scanning, EXIF parsing, hashing, and file operations.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;

use crate::config::Config;
use crate::exif::date::parse_date_from_filename;
use crate::exif::fields::ExifMetadata;
use crate::exif::reader::ExifReader;
use crate::hash::hasher::hash_file;
use crate::hash::store::HashStore;
use crate::ops::conflict::ConflictResolver;
use crate::ops::file_op;
use crate::ops::scanner::Scanner;
use crate::pattern::formatter::format_pattern;
use crate::pattern::parser::parse_pattern;
use crate::report::Report;

/// The main orchestrator for photo sorting operations.
pub struct Organizer;

/// Callback for progress updates.
pub type ProgressCallback = Box<dyn Fn(usize, usize, &str) + Send + Sync>;

impl Organizer {
    /// Execute a full sorting operation.
    ///
    /// This is the main entry point that performs:
    /// 1. Validate and parse patterns
    /// 2. Scan source directory for files
    /// 3. Extract EXIF metadata (in parallel)
    /// 4. Organize files into target structure
    /// 5. Save hash stores
    ///
    /// # Arguments
    /// * `config` — Full configuration for the operation.
    /// * `progress` — Optional callback for progress updates `(current, total, message)`.
    ///
    /// # Errors
    /// Returns errors for invalid configuration, I/O failures, etc.
    pub fn run(config: &Config, progress: Option<&ProgressCallback>) -> crate::Result<Report> {
        let start = Instant::now();
        let mut report = Report::new();

        let folder_pattern = parse_pattern(&config.folder_pattern)?;
        let file_pattern = parse_pattern(&config.file_pattern)?;

        let files = Self::scan_files(config, progress, &mut report)?;
        if files.is_empty() {
            report.elapsed = start.elapsed();
            return Ok(report);
        }

        let metadata = Self::extract_metadata(&files, progress, report.files_total);
        Self::count_exif_stats(&metadata, &mut report);
        Self::organize_files(config, &metadata, &folder_pattern, &file_pattern, progress, &mut report)?;

        report.elapsed = start.elapsed();
        Ok(report)
    }

    fn scan_files(
        config: &Config,
        progress: Option<&ProgressCallback>,
        report: &mut Report,
    ) -> crate::Result<Vec<PathBuf>> {
        if let Some(cb) = progress {
            cb(0, 0, "Scanning for files...");
        }
        let scan_result = Scanner::scan(&config.source, config.recursive)?;
        report.files_total = scan_result.files.len();
        Ok(scan_result.files)
    }

    fn extract_metadata(
        files: &[PathBuf],
        progress: Option<&ProgressCallback>,
        total: usize,
    ) -> Vec<ExifMetadata> {
        if let Some(cb) = progress {
            cb(0, total, "Extracting metadata...");
        }

        files
            .par_iter()
            .enumerate()
            .map(|(i, path)| {
                if let Some(cb) = progress {
                    if i % 100 == 0 {
                        cb(i, total, "Extracting metadata...");
                    }
                }

                let mut meta = ExifReader::read(path).unwrap_or_else(|e| {
                    tracing::warn!("Failed to read EXIF from {}: {}", path.display(), e);
                    ExifMetadata {
                        source_path: Some(path.clone()),
                        filename: path.file_stem().and_then(|s| s.to_str()).map(String::from),
                        extension: path
                            .extension()
                            .and_then(|s| s.to_str())
                            .map(str::to_lowercase),
                        ..ExifMetadata::default()
                    }
                });

                // Fallback: try to parse date from filename
                if meta.date_time_original.is_none() {
                    if let Some(stem) = meta.filename.as_deref() {
                        meta.date_time_original = parse_date_from_filename(stem);
                    }
                }

                meta
            })
            .collect()
    }

    fn count_exif_stats(metadata: &[ExifMetadata], report: &mut Report) {
        for meta in metadata {
            if meta.has_date() {
                report.files_with_exif += 1;
            } else {
                report.files_without_exif += 1;
            }
        }
    }

    fn organize_files(
        config: &Config,
        metadata: &[ExifMetadata],
        folder_pattern: &crate::pattern::parser::ParsedPattern,
        file_pattern: &crate::pattern::parser::ParsedPattern,
        progress: Option<&ProgressCallback>,
        report: &mut Report,
    ) -> crate::Result<()> {
        if let Some(cb) = progress {
            cb(0, metadata.len(), "Organizing files...");
        }

        let mut hash_stores: HashMap<PathBuf, HashStore> = HashMap::new();

        for (i, meta) in metadata.iter().enumerate() {
            if let Some(cb) = progress {
                if i % 50 == 0 {
                    cb(i, metadata.len(), "Organizing files...");
                }
            }

            let Some(source_path) = &meta.source_path else {
                report.files_failed += 1;
                continue;
            };

            let (target_folder, target_filename) = Self::compute_target(
                meta, source_path, config, folder_pattern, file_pattern,
            );

            let full_target_dir = config.target.join(&target_folder);
            let full_target = full_target_dir.join(&target_filename);

            // Duplicate detection
            if config.detect_duplicates && meta.has_date() {
                let should_continue = Self::handle_duplicates(
                    config, source_path, &full_target_dir, &target_folder,
                    &target_filename, &mut hash_stores, report,
                );
                if should_continue {
                    continue;
                }
            }

            // Conflict resolution
            let Some(final_target) = ConflictResolver::resolve(
                &full_target, config.conflict_strategy,
            )? else {
                report.conflicts += 1;
                report.files_skipped += 1;
                continue;
            };

            Self::execute_operation(config, source_path, &final_target, report);
        }

        // Save hash stores
        for store in hash_stores.values_mut() {
            if let Err(e) = store.save() {
                tracing::error!("Failed to save hash store: {e}");
            }
        }

        Ok(())
    }

    fn compute_target(
        meta: &ExifMetadata,
        source_path: &Path,
        config: &Config,
        folder_pattern: &crate::pattern::parser::ParsedPattern,
        file_pattern: &crate::pattern::parser::ParsedPattern,
    ) -> (String, String) {
        if meta.has_date() {
            let folder = format_pattern(folder_pattern, meta);
            let filename = format_pattern(file_pattern, meta);
            let ext = meta.extension.as_deref().unwrap_or("unknown");
            (folder, format!("{filename}.{ext}"))
        } else {
            let rel = compute_relative_path(source_path, &config.source);
            let folder = format!("{}/{rel}", config.no_exif_folder);
            let filename = source_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            (folder, filename)
        }
    }

    /// Returns `true` if the caller should `continue` (skip this file).
    fn handle_duplicates(
        config: &Config,
        source_path: &Path,
        full_target_dir: &Path,
        target_folder: &str,
        target_filename: &str,
        hash_stores: &mut HashMap<PathBuf, HashStore>,
        report: &mut Report,
    ) -> bool {
        let store = hash_stores
            .entry(full_target_dir.to_path_buf())
            .or_insert_with(|| HashStore::load_or_new(full_target_dir));

        match hash_file(source_path) {
            Ok(hash) => {
                if store.contains_hash(&hash) {
                    report.duplicates += 1;
                    match config.duplicate_strategy {
                        crate::config::DuplicateStrategy::Skip => {
                            report.files_skipped += 1;
                            tracing::debug!("Skipping duplicate: {}", source_path.display());
                            return true;
                        }
                        crate::config::DuplicateStrategy::CopyToFolder => {
                            let dup_folder = config
                                .target
                                .join(&config.duplicates_folder)
                                .join(target_folder);
                            let dup_target = dup_folder.join(target_filename);
                            Self::execute_operation(config, source_path, &dup_target, report);
                            return true;
                        }
                    }
                }
                // Not a duplicate — record the hash
                store.insert(target_filename.to_owned(), hash);
            }
            Err(e) => {
                tracing::warn!("Failed to hash {}: {e}", source_path.display());
            }
        }
        false
    }

    fn execute_operation(config: &Config, source: &Path, target: &Path, report: &mut Report) {
        if config.dry_run {
            tracing::info!("[DRY RUN] {} -> {}", source.display(), target.display());
            report.files_processed += 1;
        } else {
            match file_op::execute(source, target, config.file_operation) {
                Ok(()) => {
                    report.files_processed += 1;
                    tracing::debug!("{} -> {}", source.display(), target.display());
                }
                Err(e) => {
                    tracing::error!("Failed: {} -> {}: {e}", source.display(), target.display());
                    report.files_failed += 1;
                }
            }
        }
    }
}

/// Compute the relative path of a file within a source directory.
fn compute_relative_path(file: &Path, source_root: &Path) -> String {
    file.parent()
        .and_then(|p| p.strip_prefix(source_root).ok())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ConflictStrategy, FileOperation};

    #[test]
    fn test_compute_relative_path() {
        let file = Path::new("/photos/2024/march/photo.jpg");
        let root = Path::new("/photos");
        assert_eq!(compute_relative_path(file, root), "2024/march");
    }

    #[test]
    fn test_compute_relative_path_same_dir() {
        let file = Path::new("/photos/photo.jpg");
        let root = Path::new("/photos");
        assert_eq!(compute_relative_path(file, root), "");
    }

    #[test]
    fn test_organizer_empty_source() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();

        let config = Config {
            source: source.path().to_path_buf(),
            target: target.path().to_path_buf(),
            ..Default::default()
        };

        let report = Organizer::run(&config, None).unwrap();
        assert_eq!(report.files_total, 0);
    }

    #[test]
    fn test_organizer_dry_run() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();

        // Create a fake jpg (no real EXIF, will go to NoExifData)
        std::fs::write(source.path().join("photo.jpg"), b"fake jpg content").unwrap();

        let config = Config {
            source: source.path().to_path_buf(),
            target: target.path().to_path_buf(),
            dry_run: true,
            detect_duplicates: false,
            ..Default::default()
        };

        let report = Organizer::run(&config, None).unwrap();
        assert_eq!(report.files_total, 1);
        assert_eq!(report.files_processed, 1);

        // In dry run, no files should actually be created
        let target_entries: Vec<_> = std::fs::read_dir(target.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(target_entries.is_empty(), "Dry run should not create files");
    }

    #[test]
    fn test_organizer_copies_no_exif_files() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();

        std::fs::write(source.path().join("photo.jpg"), b"no exif here").unwrap();

        let config = Config {
            source: source.path().to_path_buf(),
            target: target.path().to_path_buf(),
            file_operation: FileOperation::Copy,
            detect_duplicates: false,
            conflict_strategy: ConflictStrategy::Rename,
            ..Default::default()
        };

        let report = Organizer::run(&config, None).unwrap();
        assert_eq!(report.files_total, 1);
        assert_eq!(report.files_without_exif, 1);
        assert_eq!(report.files_processed, 1);

        // File should be in [NoExifData] folder
        let no_exif_dir = target.path().join("[NoExifData]");
        assert!(no_exif_dir.exists(), "[NoExifData] folder should be created");
    }

    #[test]
    fn test_organizer_nonexistent_source() {
        let config = Config {
            source: PathBuf::from("/nonexistent/source"),
            target: PathBuf::from("/tmp/target"),
            ..Default::default()
        };

        let result = Organizer::run(&config, None);
        assert!(result.is_err());
    }
}
