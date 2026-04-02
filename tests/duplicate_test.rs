mod common;

use std::fs;
use std::path::Path;

use tempfile::tempdir;
use yaps_core::config::{ConflictStrategy, DuplicateStrategy, FileOperation};
use yaps_core::hash::hasher::hash_file;
use yaps_core::hash::store::HashStore;
use yaps_core::ops::organizer::Organizer;
use yaps_core::Config;

use common::count_files_recursive;

/// Minimal JPEG bytes used throughout these tests.
const JPEG_BYTES_A: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x02, 0x00, 0x00, 0xFF, 0xD9];

/// Different JPEG bytes (distinct content).
const JPEG_BYTES_B: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x02, 0x01, 0x01, 0xFF, 0xD9];

fn base_config(source: &Path, target: &Path) -> Config {
    Config {
        source: source.to_path_buf(),
        target: target.to_path_buf(),
        detect_duplicates: true,
        ..Config::default()
    }
}

// ────────────────── hash_file produces consistent hashes ─────────────────

#[test]
fn hash_file_identical_content_same_hash() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.jpg"), JPEG_BYTES_A).unwrap();
    fs::write(dir.path().join("b.jpg"), JPEG_BYTES_A).unwrap();

    let h1 = hash_file(dir.path().join("a.jpg")).unwrap();
    let h2 = hash_file(dir.path().join("b.jpg")).unwrap();

    assert_eq!(h1, h2, "identical content should produce the same hash");
}

#[test]
fn hash_file_different_content_different_hash() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.jpg"), JPEG_BYTES_A).unwrap();
    fs::write(dir.path().join("b.jpg"), JPEG_BYTES_B).unwrap();

    let h1 = hash_file(dir.path().join("a.jpg")).unwrap();
    let h2 = hash_file(dir.path().join("b.jpg")).unwrap();

    assert_ne!(h1, h2, "different content should produce different hashes");
}

// ────────────────── HashStore insert detects duplicates ──────────────────

#[test]
fn hash_store_detects_duplicate_on_insert() {
    let dir = tempdir().unwrap();
    let mut store = HashStore::new(dir.path());

    let is_dup_first = store.insert("a.jpg".into(), "abc123".into());
    assert!(!is_dup_first, "first insert should not be a duplicate");

    let is_dup_second = store.insert("b.jpg".into(), "abc123".into());
    assert!(
        is_dup_second,
        "second insert with same hash should be a duplicate"
    );
}

#[test]
fn hash_store_contains_hash_after_insert() {
    let dir = tempdir().unwrap();
    let mut store = HashStore::new(dir.path());

    assert!(!store.contains_hash("abc123"));
    store.insert("a.jpg".into(), "abc123".into());
    assert!(store.contains_hash("abc123"));
}

#[test]
fn hash_store_different_hashes_not_duplicate() {
    let dir = tempdir().unwrap();
    let mut store = HashStore::new(dir.path());

    store.insert("a.jpg".into(), "hash_aaa".into());
    let is_dup = store.insert("b.jpg".into(), "hash_bbb".into());

    assert!(
        !is_dup,
        "different hashes should not be detected as duplicate"
    );
}

// ──────────────── HashStore persistence via save/load ────────────────────

#[test]
fn hash_store_persists_across_save_load() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("dummy.jpg");
    fs::write(&file, JPEG_BYTES_A).unwrap();

    // Use a real hash so the store file passes validation on load.
    let real_hash = hash_file(&file).unwrap();

    {
        let mut store = HashStore::new(dir.path());
        store.insert("photo.jpg".into(), real_hash.clone());
        store.save().unwrap();
    }

    let loaded = HashStore::load_or_new(dir.path());
    assert!(
        loaded.contains_hash(&real_hash),
        "hash should persist after save and load"
    );
    assert_eq!(loaded.len(), 1);
}

// ──── No-EXIF files: dedup is skipped (organizer requires has_date) ─────

#[test]
fn no_exif_files_bypass_duplicate_detection() {
    // The organizer only deduplicates files that have EXIF date metadata.
    // Files without EXIF go to NoExifData without hash checking.
    let src = tempdir().unwrap();
    fs::write(src.path().join("a.jpg"), JPEG_BYTES_A).unwrap();
    fs::write(src.path().join("b.jpg"), JPEG_BYTES_A).unwrap();

    let tgt = tempdir().unwrap();
    let config = Config {
        conflict_strategy: ConflictStrategy::Rename,
        ..base_config(src.path(), tgt.path())
    };

    let report = Organizer::run(&config, None).unwrap();

    assert_eq!(report.files_total, 2);
    assert_eq!(report.files_without_exif, 2);
    // No duplicates reported because dedup requires EXIF date.
    assert_eq!(report.duplicates, 0);
    // Both files are processed since dedup is bypassed.
    assert_eq!(report.files_processed, 2);
}

// ───────────────── Different files with same name NOT duplicates ─────────

#[test]
fn different_content_same_name_not_flagged_as_duplicate() {
    let src = tempdir().unwrap();

    let sub_a = src.path().join("a");
    let sub_b = src.path().join("b");
    fs::create_dir_all(&sub_a).unwrap();
    fs::create_dir_all(&sub_b).unwrap();

    fs::write(sub_a.join("photo.jpg"), JPEG_BYTES_A).unwrap();
    fs::write(sub_b.join("photo.jpg"), JPEG_BYTES_B).unwrap();

    let tgt = tempdir().unwrap();
    let config = Config {
        recursive: true,
        conflict_strategy: ConflictStrategy::Rename,
        ..base_config(src.path(), tgt.path())
    };

    let report = Organizer::run(&config, None).unwrap();

    assert_eq!(report.files_total, 2);
    assert_eq!(
        report.duplicates, 0,
        "files with different content should not be flagged as duplicates"
    );
    assert_eq!(report.files_processed, 2);
}

// ─────────────────────── Dedup disabled processes all ────────────────────

#[test]
fn dedup_disabled_processes_all_identical_files() {
    let src = tempdir().unwrap();
    fs::write(src.path().join("a.jpg"), JPEG_BYTES_A).unwrap();
    fs::write(src.path().join("b.jpg"), JPEG_BYTES_A).unwrap();

    let tgt = tempdir().unwrap();
    let config = Config {
        detect_duplicates: false,
        conflict_strategy: ConflictStrategy::Rename,
        ..Config {
            source: src.path().to_path_buf(),
            target: tgt.path().to_path_buf(),
            ..Config::default()
        }
    };

    let report = Organizer::run(&config, None).unwrap();

    assert_eq!(report.files_total, 2);
    assert_eq!(report.files_processed, 2);
    assert_eq!(
        report.duplicates, 0,
        "with dedup disabled, no duplicates should be reported"
    );

    let no_exif = tgt.path().join("[NoExifData]");
    assert!(
        count_files_recursive(&no_exif) >= 2,
        "both files should be written when dedup is disabled"
    );
}

// ─────────────── Custom duplicates folder config ────────────────────────

#[test]
fn custom_duplicates_folder_name_is_respected() {
    // Verify Config carries the custom duplicates folder name.
    let config = Config {
        duplicates_folder: "my_dups".to_string(),
        ..Config::default()
    };
    assert_eq!(config.duplicates_folder, "my_dups");
    assert_ne!(config.duplicates_folder, "[Duplicates]");
}

// ───────────────── Conflict on second run detected ──────────────────────

#[test]
fn second_run_same_files_produces_conflicts_or_skips() {
    let src = tempdir().unwrap();
    fs::write(src.path().join("orig.jpg"), JPEG_BYTES_A).unwrap();

    let tgt = tempdir().unwrap();

    // First run: file is copied to target.
    let config = Config {
        conflict_strategy: ConflictStrategy::Skip,
        ..base_config(src.path(), tgt.path())
    };
    let r1 = Organizer::run(&config, None).unwrap();
    assert_eq!(r1.files_processed, 1);

    // Second run: same file, same target — should conflict/skip.
    let r2 = Organizer::run(&config, None).unwrap();
    let handled = r2.files_skipped + r2.conflicts + r2.duplicates;
    assert!(
        handled > 0,
        "second run should skip or conflict on existing file, report: {r2:?}"
    );
}

// ──────────────── Move + conflict skip is consistent ────────────────────

#[test]
fn move_with_conflict_skip_report_is_consistent() {
    let src = tempdir().unwrap();
    fs::write(src.path().join("a.jpg"), JPEG_BYTES_A).unwrap();
    fs::write(src.path().join("b.jpg"), JPEG_BYTES_B).unwrap();

    let tgt = tempdir().unwrap();
    let config = Config {
        file_operation: FileOperation::Move,
        conflict_strategy: ConflictStrategy::Skip,
        ..base_config(src.path(), tgt.path())
    };

    let report = Organizer::run(&config, None).unwrap();

    assert_eq!(report.files_total, 2);

    let total_accounted =
        report.files_processed + report.files_skipped + report.duplicates + report.files_failed;
    assert!(
        total_accounted >= report.files_total,
        "all files should be accounted for in the report"
    );
}

// ──────────── DuplicateStrategy enum variants exist ─────────────────────

#[test]
fn duplicate_strategy_defaults_to_skip() {
    let config = Config::default();
    assert_eq!(config.duplicate_strategy, DuplicateStrategy::Skip);
}

#[test]
fn copy_to_folder_strategy_variant_exists() {
    let config = Config {
        duplicate_strategy: DuplicateStrategy::CopyToFolder,
        ..Config::default()
    };
    assert_eq!(config.duplicate_strategy, DuplicateStrategy::CopyToFolder);
}
