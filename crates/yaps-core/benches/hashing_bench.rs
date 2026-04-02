//! Benchmarks for BLAKE3 file hashing throughput.

use std::io::Write;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use tempfile::NamedTempFile;
use yaps_core::hash::hasher::hash_file;

fn create_temp_file(size_bytes: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create temp file");
    // Write deterministic data in chunks
    let chunk: Vec<u8> = (0..=255u8).cycle().take(4096).collect();
    let mut remaining = size_bytes;
    while remaining > 0 {
        let to_write = remaining.min(chunk.len());
        file.write_all(&chunk[..to_write])
            .expect("write temp data");
        remaining -= to_write;
    }
    file.flush().expect("flush temp file");
    file
}

fn bench_hash_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3_hash_file");

    for &size in &[
        1024,            // 1 KB
        64 * 1024,       // 64 KB
        1024 * 1024,     // 1 MB
        10 * 1024 * 1024, // 10 MB
    ] {
        let file = create_temp_file(size);
        let path = file.path().to_path_buf();

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(format_size(size)), &path, |b, p| {
            b.iter(|| hash_file(p).expect("hash should succeed"));
        });
    }

    group.finish();
}

fn bench_hash_consistency(c: &mut Criterion) {
    let file = create_temp_file(1024 * 1024); // 1 MB
    let path = file.path().to_path_buf();

    c.bench_function("blake3_1mb_consistency", |b| {
        let expected = hash_file(&path).unwrap();
        b.iter(|| {
            let hash = hash_file(&path).unwrap();
            assert_eq!(hash, expected);
        });
    });
}

fn format_size(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{}MB", bytes / (1024 * 1024))
    } else if bytes >= 1024 {
        format!("{}KB", bytes / 1024)
    } else {
        format!("{bytes}B")
    }
}

criterion_group!(benches, bench_hash_file, bench_hash_consistency);
criterion_main!(benches);
