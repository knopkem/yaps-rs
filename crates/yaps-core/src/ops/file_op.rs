//! File operation implementations (copy, move, hardlink, symlink).

use std::path::Path;

use crate::config::FileOperation;

/// Execute a file operation (copy, move, hardlink, or symlink).
///
/// Creates parent directories as needed.
///
/// # Errors
/// Returns `YapsError::Io` if the operation fails.
pub fn execute(
    source: impl AsRef<Path>,
    target: impl AsRef<Path>,
    operation: FileOperation,
) -> crate::Result<()> {
    let source = source.as_ref();
    let target = target.as_ref();

    // Ensure parent directory exists
    if let Some(parent) = target.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| crate::YapsError::TargetCreation {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
    }

    match operation {
        FileOperation::Copy => {
            std::fs::copy(source, target).map_err(|e| crate::YapsError::io(target, e))?;
        }
        FileOperation::Move => {
            // Try rename first (fast, same filesystem)
            if std::fs::rename(source, target).is_err() {
                // Fall back to copy + delete (cross-filesystem)
                std::fs::copy(source, target).map_err(|e| crate::YapsError::io(target, e))?;
                std::fs::remove_file(source).map_err(|e| crate::YapsError::io(source, e))?;
            }
        }
        FileOperation::Hardlink => {
            std::fs::hard_link(source, target).map_err(|e| crate::YapsError::io(target, e))?;
        }
        FileOperation::Symlink => {
            #[cfg(unix)]
            std::os::unix::fs::symlink(source, target)
                .map_err(|e| crate::YapsError::io(target, e))?;
            #[cfg(windows)]
            std::os::windows::fs::symlink_file(source, target)
                .map_err(|e| crate::YapsError::io(target, e))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_file() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("dest.txt");
        std::fs::write(&src, b"hello").unwrap();

        execute(&src, &dst, FileOperation::Copy).unwrap();

        assert!(src.exists(), "Source should still exist after copy");
        assert!(dst.exists());
        assert_eq!(std::fs::read(&dst).unwrap(), b"hello");
    }

    #[test]
    fn test_move_file() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("dest.txt");
        std::fs::write(&src, b"hello").unwrap();

        execute(&src, &dst, FileOperation::Move).unwrap();

        assert!(!src.exists(), "Source should be gone after move");
        assert!(dst.exists());
        assert_eq!(std::fs::read(&dst).unwrap(), b"hello");
    }

    #[test]
    fn test_hardlink_file() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("link.txt");
        std::fs::write(&src, b"hello").unwrap();

        execute(&src, &dst, FileOperation::Hardlink).unwrap();

        assert!(dst.exists());
        assert_eq!(std::fs::read(&dst).unwrap(), b"hello");
    }

    #[test]
    fn test_creates_parent_directories() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("a/b/c/dest.txt");
        std::fs::write(&src, b"hello").unwrap();

        execute(&src, &dst, FileOperation::Copy).unwrap();
        assert!(dst.exists());
    }

    #[test]
    fn test_copy_nonexistent_source_errors() {
        let dir = tempfile::tempdir().unwrap();
        let result = execute(
            dir.path().join("nope.txt"),
            dir.path().join("dest.txt"),
            FileOperation::Copy,
        );
        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn test_symlink_file() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("link.txt");
        std::fs::write(&src, b"hello").unwrap();

        execute(&src, &dst, FileOperation::Symlink).unwrap();

        assert!(dst.exists());
        assert!(dst.is_symlink());
        assert_eq!(std::fs::read(&dst).unwrap(), b"hello");
    }
}
