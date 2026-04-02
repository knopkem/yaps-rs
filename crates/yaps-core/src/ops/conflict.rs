//! File conflict resolution strategies.

use std::path::{Path, PathBuf};

use crate::config::ConflictStrategy;

/// Resolves filename conflicts at the target directory.
pub struct ConflictResolver;

impl ConflictResolver {
    /// Resolve a file conflict according to the chosen strategy.
    ///
    /// Returns the final target path to use, or `None` if the file should be skipped.
    pub fn resolve(target: &Path, strategy: ConflictStrategy) -> crate::Result<Option<PathBuf>> {
        if !target.exists() {
            return Ok(Some(target.to_path_buf()));
        }

        match strategy {
            ConflictStrategy::Skip => {
                tracing::info!("Skipping (conflict): {}", target.display());
                Ok(None)
            }
            ConflictStrategy::Overwrite => {
                tracing::info!("Overwriting: {}", target.display());
                Ok(Some(target.to_path_buf()))
            }
            ConflictStrategy::Rename => {
                let renamed = propose_new_filename(target);
                tracing::info!(
                    "Renaming to avoid conflict: {} -> {}",
                    target.display(),
                    renamed.display()
                );
                Ok(Some(renamed))
            }
        }
    }
}

/// Generate a new filename with an incrementing suffix to avoid conflicts.
///
/// Given `photo.jpg`, tries `photo(1).jpg`, `photo(2).jpg`, etc.
fn propose_new_filename(path: &Path) -> PathBuf {
    let parent = path.parent().unwrap_or(Path::new(""));
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let ext = path.extension().and_then(|s| s.to_str());

    for counter in 1..=10_000 {
        let new_name = match ext {
            Some(ext) => format!("{stem}({counter}).{ext}"),
            None => format!("{stem}({counter})"),
        };
        let candidate = parent.join(&new_name);
        if !candidate.exists() {
            return candidate;
        }
    }

    // Fallback: very unlikely to reach here
    let new_name = match ext {
        Some(ext) => format!("{stem}(renamed).{ext}"),
        None => format!("{stem}(renamed)"),
    };
    parent.join(new_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_no_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("new_file.jpg");

        let result = ConflictResolver::resolve(&target, ConflictStrategy::Skip).unwrap();
        assert_eq!(result, Some(target));
    }

    #[test]
    fn test_resolve_skip_on_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("existing.jpg");
        std::fs::write(&target, b"exists").unwrap();

        let result = ConflictResolver::resolve(&target, ConflictStrategy::Skip).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_overwrite_on_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("existing.jpg");
        std::fs::write(&target, b"exists").unwrap();

        let result = ConflictResolver::resolve(&target, ConflictStrategy::Overwrite).unwrap();
        assert_eq!(result, Some(target));
    }

    #[test]
    fn test_resolve_rename_on_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("photo.jpg");
        std::fs::write(&target, b"exists").unwrap();

        let result = ConflictResolver::resolve(&target, ConflictStrategy::Rename).unwrap();
        let resolved = result.unwrap();
        assert_eq!(resolved, dir.path().join("photo(1).jpg"));
    }

    #[test]
    fn test_rename_increments_past_existing() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("photo.jpg"), b"original").unwrap();
        std::fs::write(dir.path().join("photo(1).jpg"), b"first").unwrap();
        std::fs::write(dir.path().join("photo(2).jpg"), b"second").unwrap();

        let target = dir.path().join("photo.jpg");
        let result = ConflictResolver::resolve(&target, ConflictStrategy::Rename).unwrap();
        let resolved = result.unwrap();
        assert_eq!(resolved, dir.path().join("photo(3).jpg"));
    }

    #[test]
    fn test_propose_filename_without_extension() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("file_no_ext");
        std::fs::write(&target, b"exists").unwrap();

        let proposed = propose_new_filename(&target);
        assert_eq!(proposed, dir.path().join("file_no_ext(1)"));
    }
}
