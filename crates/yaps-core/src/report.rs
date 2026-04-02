//! Operation report and statistics.

use std::time::Duration;

/// Statistics collected during a sorting operation.
#[derive(Debug, Clone, Default)]
pub struct Report {
    /// Total number of files discovered in the source.
    pub files_total: usize,
    /// Number of files with valid EXIF metadata.
    pub files_with_exif: usize,
    /// Number of files without EXIF metadata.
    pub files_without_exif: usize,
    /// Number of files successfully processed (copied/moved/linked).
    pub files_processed: usize,
    /// Number of files that failed during processing.
    pub files_failed: usize,
    /// Number of duplicate files detected.
    pub duplicates: usize,
    /// Number of filename conflicts encountered.
    pub conflicts: usize,
    /// Number of files skipped (duplicates + conflicts with Skip strategy).
    pub files_skipped: usize,
    /// Total elapsed wall-clock time.
    pub elapsed: Duration,
    /// Path to the log file, if file logging was enabled.
    pub log_path: Option<std::path::PathBuf>,
}

impl Report {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self::default()
    }
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "─── Operation Report ───")?;
        writeln!(f, "  Time elapsed:    {:?}", self.elapsed)?;
        writeln!(f, "  Files found:     {}", self.files_total)?;
        writeln!(f, "  With EXIF:       {}", self.files_with_exif)?;
        writeln!(f, "  Without EXIF:    {}", self.files_without_exif)?;
        writeln!(f, "  Processed:       {}", self.files_processed)?;
        writeln!(f, "  Duplicates:      {}", self.duplicates)?;
        writeln!(f, "  Conflicts:       {}", self.conflicts)?;
        writeln!(f, "  Skipped:         {}", self.files_skipped)?;
        writeln!(f, "  Failed:          {}", self.files_failed)?;
        if let Some(ref path) = self.log_path {
            writeln!(f, "  Log file:        {}", path.display())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_report_is_zeroed() {
        let report = Report::new();
        assert_eq!(report.files_total, 0);
        assert_eq!(report.files_processed, 0);
        assert_eq!(report.duplicates, 0);
        assert_eq!(report.elapsed, Duration::ZERO);
        assert!(report.log_path.is_none());
    }

    #[test]
    fn test_report_display() {
        let report = Report {
            files_total: 100,
            files_with_exif: 90,
            files_without_exif: 10,
            files_processed: 85,
            files_failed: 2,
            duplicates: 3,
            conflicts: 1,
            files_skipped: 13,
            elapsed: Duration::from_secs(5),
            log_path: None,
        };

        let output = report.to_string();
        assert!(output.contains("Files found:     100"));
        assert!(output.contains("Processed:       85"));
        assert!(output.contains("Duplicates:      3"));
    }

    #[test]
    fn test_report_display_with_log_path() {
        let report = Report {
            files_total: 10,
            log_path: Some(std::path::PathBuf::from("/tmp/yaps.log")),
            ..Report::default()
        };

        let output = report.to_string();
        assert!(output.contains("/tmp/yaps.log"));
        assert!(output.contains("Log file:"));
    }

    #[test]
    fn test_report_clone() {
        let original = Report {
            files_total: 42,
            duplicates: 5,
            ..Report::default()
        };
        let cloned = original.clone();
        assert_eq!(cloned.files_total, 42);
        assert_eq!(cloned.duplicates, 5);
    }
}
