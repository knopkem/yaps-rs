//! Benchmarks for directory scanning performance.

use std::fs;
use std::path::Path;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use tempfile::TempDir;
use yaps_core::ops::scanner::Scanner;

fn create_test_tree(dir: &Path, depth: usize, files_per_dir: usize) {
    for i in 0..files_per_dir {
        let ext = match i % 4 {
            0 => "jpg",
            1 => "png",
            2 => "cr2",
            _ => "mp4",
        };
        let name = format!("file_{i:04}.{ext}");
        fs::write(dir.join(name), b"fake media content").expect("write file");
    }

    if depth > 0 {
        for sub in 0..3 {
            let subdir = dir.join(format!("subdir_{sub}"));
            fs::create_dir_all(&subdir).expect("create subdir");
            create_test_tree(&subdir, depth - 1, files_per_dir);
        }
    }
}

fn bench_scan_flat(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_flat");

    for &file_count in &[10, 100, 500] {
        let dir = TempDir::new().unwrap();
        for i in 0..file_count {
            let name = format!("photo_{i:05}.jpg");
            fs::write(dir.path().join(name), b"x").unwrap();
        }

        group.bench_with_input(BenchmarkId::from_parameter(file_count), &dir, |b, d| {
            b.iter(|| Scanner::scan(d.path(), false).expect("scan"));
        });
    }

    group.finish();
}

fn bench_scan_recursive(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_recursive");

    // depth=2, 3 subdirs each, 10 files per dir → 10 + 3*10 + 9*10 = 130 files
    for &(depth, files) in &[(1, 10), (2, 10), (2, 50)] {
        let dir = TempDir::new().unwrap();
        create_test_tree(dir.path(), depth, files);

        let total: usize = (0..=depth)
            .map(|d| {
                let exp = u32::try_from(d).expect("depth fits in u32");
                3_usize.pow(exp) * files
            })
            .sum();
        let label = format!("d{depth}_f{files}_t{total}");

        group.bench_with_input(BenchmarkId::from_parameter(label), &dir, |b, d| {
            b.iter(|| Scanner::scan(d.path(), true).expect("scan"));
        });
    }

    group.finish();
}

fn bench_scan_mixed_extensions(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();

    // Create 200 files: 100 images, 50 videos, 50 non-media
    for i in 0..100 {
        fs::write(dir.path().join(format!("img_{i:04}.jpg")), b"x").unwrap();
    }
    for i in 0..50 {
        fs::write(dir.path().join(format!("vid_{i:04}.mp4")), b"x").unwrap();
    }
    for i in 0..50 {
        fs::write(dir.path().join(format!("doc_{i:04}.txt")), b"x").unwrap();
    }

    c.bench_function("scan_200_mixed_extensions", |b| {
        b.iter(|| {
            let result = Scanner::scan(dir.path(), false).expect("scan");
            assert_eq!(result.files.len(), 150, "should find 150 media files");
        });
    });
}

criterion_group!(
    benches,
    bench_scan_flat,
    bench_scan_recursive,
    bench_scan_mixed_extensions,
);
criterion_main!(benches);
