//! Persistent hash store for per-folder duplicate detection.
//!
//! Each target folder gets a `hash.txt` file that records BLAKE3 hashes of
//! all files placed there. On subsequent runs, the store detects duplicates
//! without re-hashing files that haven't changed.

use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

/// Name of the hash store file within each target folder.
const HASH_STORE_FILENAME: &str = "hash.txt";
/// Current hash store format version.
const HASH_STORE_VERSION: u32 = 1;
/// Algorithm identifier stored in the hash file header.
const HASH_ALGORITHM: &str = "blake3";

/// In-memory hash store for a single folder.
///
/// Maps filename → BLAKE3 hash hex string.
#[derive(Debug, Clone)]
pub struct HashStore {
    /// The folder this store belongs to.
    folder: PathBuf,
    /// filename → hash mapping.
    entries: HashMap<String, String>,
    /// Whether the store has been modified since loading.
    dirty: bool,
}

impl HashStore {
    /// Create a new empty hash store for a folder.
    pub fn new(folder: impl Into<PathBuf>) -> Self {
        Self {
            folder: folder.into(),
            entries: HashMap::new(),
            dirty: false,
        }
    }

    /// Load an existing hash store from disk, or create a new one if none exists.
    ///
    /// If the file exists but is corrupt or incompatible, it is discarded and a fresh
    /// store is returned (existing files in the folder will be re-hashed on demand).
    pub fn load_or_new(folder: impl Into<PathBuf>) -> Self {
        let folder = folder.into();
        let path = folder.join(HASH_STORE_FILENAME);

        if path.exists() {
            match Self::load_from_file(&path, &folder) {
                Ok(store) => return store,
                Err(e) => {
                    tracing::warn!(
                        "Hash store at {} is invalid, will re-create: {}",
                        path.display(),
                        e
                    );
                }
            }
        }

        Self::new(folder)
    }

    /// Check if a hash already exists in the store (duplicate detection).
    pub fn contains_hash(&self, hash: &str) -> bool {
        self.entries.values().any(|h| h == hash)
    }

    /// Get the filename associated with a given hash, if any.
    pub fn filename_for_hash(&self, hash: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(_, h)| h.as_str() == hash)
            .map(|(name, _)| name.as_str())
    }

    /// Insert a new entry. Returns `true` if the hash was already present (duplicate).
    pub fn insert(&mut self, filename: String, hash: String) -> bool {
        let is_dup = self.contains_hash(&hash);
        self.entries.insert(filename, hash);
        self.dirty = true;
        is_dup
    }

    /// Get the hash for a filename.
    pub fn get(&self, filename: &str) -> Option<&str> {
        self.entries.get(filename).map(String::as_str)
    }

    /// Number of entries in the store.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Save the hash store to disk (only if modified).
    pub fn save(&mut self) -> crate::Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let path = self.folder.join(HASH_STORE_FILENAME);
        let file = std::fs::File::create(&path).map_err(|e| crate::YapsError::io(&path, e))?;
        let mut writer = std::io::BufWriter::new(file);

        // Write header
        writeln!(writer, "# yaps-rs hash store v{HASH_STORE_VERSION}")
            .map_err(|e| crate::YapsError::io(&path, e))?;
        writeln!(writer, "# algorithm: {HASH_ALGORITHM}")
            .map_err(|e| crate::YapsError::io(&path, e))?;
        writeln!(writer, "# file_count: {}", self.entries.len())
            .map_err(|e| crate::YapsError::io(&path, e))?;

        // Write entries sorted by filename for deterministic output
        let mut entries: Vec<_> = self.entries.iter().collect();
        entries.sort_by_key(|(name, _)| (*name).clone());

        for (filename, hash) in entries {
            writeln!(writer, "{filename},{hash}").map_err(|e| crate::YapsError::io(&path, e))?;
        }

        writer.flush().map_err(|e| crate::YapsError::io(&path, e))?;
        self.dirty = false;

        Ok(())
    }

    /// Load a hash store from a file.
    fn load_from_file(path: &Path, folder: &Path) -> crate::Result<Self> {
        let file = std::fs::File::open(path).map_err(|e| crate::YapsError::io(path, e))?;
        let reader = std::io::BufReader::new(file);
        let mut entries = HashMap::new();
        let mut found_version = false;

        for line in reader.lines() {
            let line = line.map_err(|e| crate::YapsError::io(path, e))?;
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            // Header comments
            if line.starts_with('#') {
                if line.contains("hash store v") {
                    found_version = true;
                }
                continue;
            }

            // Data line: filename,hash
            let Some((filename, hash)) = line.split_once(',') else {
                return Err(crate::YapsError::HashStore {
                    path: path.to_path_buf(),
                    message: format!("malformed line: {line}"),
                });
            };

            if hash.len() != 64 {
                return Err(crate::YapsError::HashStore {
                    path: path.to_path_buf(),
                    message: format!(
                        "invalid hash length for {filename}: expected 64, got {}",
                        hash.len()
                    ),
                });
            }

            entries.insert(filename.to_string(), hash.to_string());
        }

        if !found_version {
            return Err(crate::YapsError::HashStore {
                path: path.to_path_buf(),
                message: "missing version header".to_string(),
            });
        }

        Ok(Self {
            folder: folder.to_path_buf(),
            entries,
            dirty: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_store_is_empty() {
        let store = HashStore::new("/tmp/test");
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_insert_and_lookup() {
        let mut store = HashStore::new("/tmp/test");
        let hash = "a".repeat(64);

        let is_dup = store.insert("photo.jpg".to_string(), hash.clone());
        assert!(!is_dup);
        assert_eq!(store.len(), 1);
        assert_eq!(store.get("photo.jpg"), Some(hash.as_str()));
    }

    #[test]
    fn test_duplicate_detection() {
        let mut store = HashStore::new("/tmp/test");
        let hash = "b".repeat(64);

        store.insert("photo1.jpg".to_string(), hash.clone());
        let is_dup = store.insert("photo2.jpg".to_string(), hash.clone());
        assert!(is_dup);
    }

    #[test]
    fn test_contains_hash() {
        let mut store = HashStore::new("/tmp/test");
        let hash = "c".repeat(64);

        assert!(!store.contains_hash(&hash));
        store.insert("photo.jpg".to_string(), hash.clone());
        assert!(store.contains_hash(&hash));
    }

    #[test]
    fn test_filename_for_hash() {
        let mut store = HashStore::new("/tmp/test");
        let hash = "d".repeat(64);

        store.insert("photo.jpg".to_string(), hash.clone());
        assert_eq!(store.filename_for_hash(&hash), Some("photo.jpg"));
        assert_eq!(store.filename_for_hash(&"e".repeat(64)), None);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = HashStore::new(dir.path());

        store.insert("a.jpg".to_string(), "a".repeat(64));
        store.insert("b.jpg".to_string(), "b".repeat(64));
        store.save().unwrap();

        let loaded = HashStore::load_or_new(dir.path());
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.get("a.jpg"), Some("a".repeat(64).as_str()));
        assert_eq!(loaded.get("b.jpg"), Some("b".repeat(64).as_str()));
    }

    #[test]
    fn test_load_nonexistent_creates_new() {
        let dir = tempfile::tempdir().unwrap();
        let store = HashStore::load_or_new(dir.path().join("nonexistent"));
        assert!(store.is_empty());
    }

    #[test]
    fn test_load_corrupt_file_creates_new() {
        let dir = tempfile::tempdir().unwrap();
        let hash_file = dir.path().join(HASH_STORE_FILENAME);
        std::fs::write(&hash_file, "this is not a valid hash store").unwrap();

        let store = HashStore::load_or_new(dir.path());
        assert!(store.is_empty());
    }

    #[test]
    fn test_save_only_when_dirty() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = HashStore::new(dir.path());

        // Not dirty, no file created
        store.save().unwrap();
        assert!(!dir.path().join(HASH_STORE_FILENAME).exists());

        // Now dirty
        store.insert("a.jpg".to_string(), "a".repeat(64));
        store.save().unwrap();
        assert!(dir.path().join(HASH_STORE_FILENAME).exists());
    }

    #[test]
    fn test_hash_store_file_format() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = HashStore::new(dir.path());
        store.insert("photo.jpg".to_string(), "f".repeat(64));
        store.save().unwrap();

        let content = std::fs::read_to_string(dir.path().join(HASH_STORE_FILENAME)).unwrap();
        assert!(content.contains("# yaps-rs hash store v1"));
        assert!(content.contains("# algorithm: blake3"));
        assert!(content.contains("# file_count: 1"));
        assert!(content.contains(&format!("photo.jpg,{}", "f".repeat(64))));
    }
}
