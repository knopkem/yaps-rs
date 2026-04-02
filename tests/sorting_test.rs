mod common;

use std::fs;
use std::path::Path;

use tempfile::tempdir;
use yaps_core::config::{ConflictStrategy, FileOperation};
use yaps_core::ops::organizer::Organizer;
use yaps_core::Config;

use common::{count_files_recursive, create_test_dir, create_test_dir_with_structure};

fn base_config(source: &Path, target: &Path) -> Config {
    Config {
        source: source.to_path_buf(),
        target: target.to_path_buf(),
        ..Config::default()
    }
}

// ───────────────────────── Basic copy operation ─────────────────────────

#[test]
fn copy_places_non_exif_files_in_no_exif_folder() {
    let src = create_test_dir();
    let tgt = tempdir().unwrap();
    let config = base_config(src.path(), tgt.path());

    let report = Organizer::run(&config, None).unwrap();

    assert_eq!(report.files_total, 3);
    assert_eq!(report.files_processed, 3);
    assert_eq!(report.files_without_exif, 3);
    assert_eq!(report.files_with_exif, 0);
    assert_eq!(report.files_failed, 0);

    let no_exif = tgt.path().join("[NoExifData]");
    assert!(no_exif.exists(), "NoExifData folder should be created");
    assert!(count_files_recursive(&no_exif) >= 3);

    // Source files should still exist (copy, not move).
    assert!(src.path().join("photo1.jpg").exists());
    assert!(src.path().join("photo2.jpg").exists());
    assert!(src.path().join("photo3.png").exists());
}

// ───────────────────────── Move operation ────────────────────────────────

#[test]
fn move_removes_source_files() {
    let src = create_test_dir();
    let tgt = tempdir().unwrap();
    let config = Config {
        file_operation: FileOperation::Move,
        ..base_config(src.path(), tgt.path())
    };

    let report = Organizer::run(&config, None).unwrap();

    assert_eq!(report.files_processed, 3);

    // Source files should be gone.
    assert!(!src.path().join("photo1.jpg").exists());
    assert!(!src.path().join("photo2.jpg").exists());
    assert!(!src.path().join("photo3.png").exists());

    // Target should have the files.
    let no_exif = tgt.path().join("[NoExifData]");
    assert!(no_exif.exists());
    assert!(count_files_recursive(&no_exif) >= 3);
}

// ───────────────────────── Dry run ──────────────────────────────────────

#[test]
fn dry_run_does_not_create_files() {
    let src = create_test_dir();
    let tgt = tempdir().unwrap();
    let config = Config {
        dry_run: true,
        ..base_config(src.path(), tgt.path())
    };

    let report = Organizer::run(&config, None).unwrap();

    // Report should still reflect what *would* happen.
    assert_eq!(report.files_total, 3);

    // Target should be empty — no files written.
    assert_eq!(count_files_recursive(tgt.path()), 0);

    // Source files untouched.
    assert!(src.path().join("photo1.jpg").exists());
}

// ───────────────────────── Non-recursive mode ───────────────────────────

#[test]
fn non_recursive_ignores_subdirectories() {
    let src = create_test_dir_with_structure();
    let tgt = tempdir().unwrap();
    let config = Config {
        recursive: false,
        ..base_config(src.path(), tgt.path())
    };

    let report = Organizer::run(&config, None).unwrap();

    // Only top-level files (top1.jpg, top2.png) should be processed.
    assert_eq!(report.files_total, 2);
    assert_eq!(report.files_processed, 2);
}

// ───────────────────────── Recursive with nested dirs ───────────────────

#[test]
fn recursive_processes_nested_subdirectories() {
    let src = create_test_dir_with_structure();
    let tgt = tempdir().unwrap();
    let config = Config {
        recursive: true,
        ..base_config(src.path(), tgt.path())
    };

    let report = Organizer::run(&config, None).unwrap();

    // All 5 files: top1.jpg, top2.png, nested1.jpg, nested2.jpg, deep1.jpg
    assert_eq!(report.files_total, 5);
    assert_eq!(report.files_processed, 5);
}

// ───────────────────────── Custom no_exif_folder name ───────────────────

#[test]
fn custom_no_exif_folder_name() {
    let src = create_test_dir();
    let tgt = tempdir().unwrap();
    let config = Config {
        no_exif_folder: "unsorted".to_string(),
        ..base_config(src.path(), tgt.path())
    };

    let report = Organizer::run(&config, None).unwrap();

    assert_eq!(report.files_processed, 3);

    let custom_folder = tgt.path().join("unsorted");
    assert!(
        custom_folder.exists(),
        "Custom no-exif folder 'unsorted' should be created"
    );
    assert!(count_files_recursive(&custom_folder) >= 3);

    // Default name should NOT exist.
    assert!(!tgt.path().join("[NoExifData]").exists());
}

// ───────────────────────── Report elapsed time ──────────────────────────

#[test]
fn report_records_elapsed_time() {
    let src = create_test_dir();
    let tgt = tempdir().unwrap();
    let config = base_config(src.path(), tgt.path());

    let report = Organizer::run(&config, None).unwrap();

    assert!(!report.elapsed.is_zero(), "elapsed time should be non-zero");
}

// ───────────────────────── Empty source directory ───────────────────────

#[test]
fn empty_source_produces_zero_totals() {
    let src = tempdir().unwrap();
    let tgt = tempdir().unwrap();
    let config = base_config(src.path(), tgt.path());

    let report = Organizer::run(&config, None).unwrap();

    assert_eq!(report.files_total, 0);
    assert_eq!(report.files_processed, 0);
}

// ───────────────────────── Conflict strategy: skip ──────────────────────

#[test]
fn conflict_skip_does_not_overwrite() {
    let src = create_test_dir();
    let tgt = tempdir().unwrap();

    // Run once to populate target.
    let config = base_config(src.path(), tgt.path());
    Organizer::run(&config, None).unwrap();

    // Run again with skip — should produce conflicts or skips.
    let config2 = Config {
        conflict_strategy: ConflictStrategy::Skip,
        ..base_config(src.path(), tgt.path())
    };
    let report = Organizer::run(&config2, None).unwrap();

    // Files already exist: they should be skipped or conflicted.
    let handled = report.files_skipped + report.conflicts + report.duplicates;
    assert!(
        handled > 0,
        "second run should skip/conflict existing files, got report: {report:?}"
    );
}

// ───────────────────────── Conflict strategy: rename ────────────────────

#[test]
fn conflict_rename_creates_additional_copies() {
    let src = create_test_dir();
    let tgt = tempdir().unwrap();

    let config = base_config(src.path(), tgt.path());
    Organizer::run(&config, None).unwrap();

    let count_before = count_files_recursive(tgt.path());

    let config2 = Config {
        conflict_strategy: ConflictStrategy::Rename,
        detect_duplicates: false,
        ..base_config(src.path(), tgt.path())
    };
    let report = Organizer::run(&config2, None).unwrap();

    assert_eq!(report.files_processed, 3);

    let count_after = count_files_recursive(tgt.path());
    assert!(
        count_after > count_before,
        "rename strategy should create additional files: before={count_before}, after={count_after}"
    );
}

// ───────────────────────── Progress callback fires ──────────────────────

#[test]
fn progress_callback_is_invoked() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let src = create_test_dir();
    let tgt = tempdir().unwrap();
    let config = base_config(src.path(), tgt.path());

    let call_count = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&call_count);
    let cb: yaps_core::ops::organizer::ProgressCallback =
        Box::new(move |_current, _total, _msg| {
            counter.fetch_add(1, Ordering::SeqCst);
        });

    Organizer::run(&config, Some(&cb)).unwrap();

    assert!(
        call_count.load(Ordering::SeqCst) > 0,
        "progress callback should have been called at least once"
    );
}

// ───────────────────────── Source with non-image files ───────────────────

#[test]
fn non_image_files_are_ignored() {
    let src = tempdir().unwrap();
    // Create a mix of image and non-image files.
    fs::write(src.path().join("readme.txt"), b"hello").unwrap();
    fs::write(src.path().join("data.csv"), b"a,b,c").unwrap();
    fs::write(
        src.path().join("photo.jpg"),
        [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x02, 0x00, 0x00, 0xFF, 0xD9],
    )
    .unwrap();

    let tgt = tempdir().unwrap();
    let config = base_config(src.path(), tgt.path());

    let report = Organizer::run(&config, None).unwrap();

    // Only the jpg should be processed.
    assert_eq!(report.files_total, 1);
    assert_eq!(report.files_processed, 1);
}
