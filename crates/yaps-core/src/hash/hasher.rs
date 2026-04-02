//! Streaming BLAKE3 file hasher.
//!
//! Hashes files in fixed-size chunks without loading the entire file into memory.
//! This is critical for large video files that could be multiple gigabytes.

use std::io::Read;
use std::path::Path;

/// Default buffer size for streaming hash computation (8 KB).
const HASH_BUFFER_SIZE: usize = 8 * 1024;

/// Compute the BLAKE3 hash of a file using streaming reads.
///
/// The file is read in 8KB chunks, so memory usage is constant regardless of file size.
///
/// # Errors
/// Returns `YapsError::Io` if the file cannot be opened or read.
///
/// # Examples
/// ```no_run
/// let hash = yaps_core::hash::hash_file("/path/to/photo.jpg").unwrap();
/// println!("BLAKE3: {hash}");
/// ```
pub fn hash_file(path: impl AsRef<Path>) -> crate::Result<String> {
    let path = path.as_ref();
    let file = std::fs::File::open(path).map_err(|e| crate::YapsError::io(path, e))?;
    let mut reader = std::io::BufReader::with_capacity(HASH_BUFFER_SIZE, file);
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; HASH_BUFFER_SIZE];

    loop {
        let bytes_read = reader
            .read(&mut buf)
            .map_err(|e| crate::YapsError::io(path, e))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buf[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_file_produces_consistent_result() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, b"hello world").unwrap();

        let hash1 = hash_file(&path).unwrap();
        let hash2 = hash_file(&path).unwrap();
        assert_eq!(hash1, hash2);
        // BLAKE3 hash is 64 hex characters
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_hash_different_content_produces_different_hash() {
        let dir = tempfile::tempdir().unwrap();

        let path_a = dir.path().join("a.txt");
        let path_b = dir.path().join("b.txt");
        std::fs::write(&path_a, b"content A").unwrap();
        std::fs::write(&path_b, b"content B").unwrap();

        let hash_a = hash_file(&path_a).unwrap();
        let hash_b = hash_file(&path_b).unwrap();
        assert_ne!(hash_a, hash_b);
    }

    #[test]
    fn test_hash_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.txt");
        std::fs::write(&path, b"").unwrap();

        let hash = hash_file(&path).unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_nonexistent_file_returns_error() {
        let result = hash_file("/nonexistent/file.jpg");
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_identical_content_in_different_files() {
        let dir = tempfile::tempdir().unwrap();
        let path_a = dir.path().join("copy1.txt");
        let path_b = dir.path().join("copy2.txt");
        let content = b"identical content for both files";
        std::fs::write(&path_a, content).unwrap();
        std::fs::write(&path_b, content).unwrap();

        let hash_a = hash_file(&path_a).unwrap();
        let hash_b = hash_file(&path_b).unwrap();
        assert_eq!(hash_a, hash_b);
    }
}
