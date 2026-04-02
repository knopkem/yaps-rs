//! All message types for the iced GUI application.

use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};

/// Messages that drive the application state machine.
#[derive(Debug, Clone)]
pub enum Message {
    // Folder paths
    SourceChanged(String),
    TargetChanged(String),
    BrowseSource,
    BrowseTarget,

    // Pattern editing
    FolderPatternChanged(String),
    FilePatternChanged(String),

    // Options
    OperationSelected(OperationChoice),
    ConflictSelected(ConflictChoice),
    DuplicateSelected(DuplicateChoice),
    ToggleRecursive(bool),
    ToggleDryRun(bool),
    ToggleDedup(bool),

    // Actions
    StartSorting,
    SortingComplete(SortingResult),
    ProgressUpdate(ProgressInfo),
    Reset,

    // Folder dialog result
    FolderSelected(FolderTarget, Option<PathBuf>),
}

/// Progress update from the sorting thread.
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub current: usize,
    pub total: usize,
    pub message: String,
}

/// Thread-safe receiver wrapped for sharing with the subscription.
pub type SharedReceiver = Arc<Mutex<mpsc::Receiver<ProgressInfo>>>;

/// Which folder dialog is open.
#[derive(Debug, Clone, Copy)]
pub enum FolderTarget {
    Source,
    Target,
}

/// Wrapper for sorting result to pass through iced messages.
#[derive(Debug, Clone)]
pub enum SortingResult {
    Success(ReportData),
    Error(String),
}

/// Serializable report data (iced messages must be Clone).
#[derive(Debug, Clone)]
pub struct ReportData {
    pub files_total: usize,
    pub files_with_exif: usize,
    pub files_without_exif: usize,
    pub files_processed: usize,
    pub files_failed: usize,
    pub duplicates: usize,
    pub conflicts: usize,
    pub files_skipped: usize,
    pub elapsed_secs: f64,
}

impl From<&yaps_core::Report> for ReportData {
    fn from(r: &yaps_core::Report) -> Self {
        Self {
            files_total: r.files_total,
            files_with_exif: r.files_with_exif,
            files_without_exif: r.files_without_exif,
            files_processed: r.files_processed,
            files_failed: r.files_failed,
            duplicates: r.duplicates,
            conflicts: r.conflicts,
            files_skipped: r.files_skipped,
            elapsed_secs: r.elapsed.as_secs_f64(),
        }
    }
}

/// File operation choices for the dropdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationChoice {
    #[default]
    Copy,
    Move,
    Hardlink,
    Symlink,
}

impl std::fmt::Display for OperationChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Copy => write!(f, "Copy"),
            Self::Move => write!(f, "Move"),
            Self::Hardlink => write!(f, "Hard Link"),
            Self::Symlink => write!(f, "Symbolic Link"),
        }
    }
}

impl OperationChoice {
    pub const ALL: &'static [Self] = &[Self::Copy, Self::Move, Self::Hardlink, Self::Symlink];

    pub fn to_file_operation(self) -> yaps_core::config::FileOperation {
        match self {
            Self::Copy => yaps_core::config::FileOperation::Copy,
            Self::Move => yaps_core::config::FileOperation::Move,
            Self::Hardlink => yaps_core::config::FileOperation::Hardlink,
            Self::Symlink => yaps_core::config::FileOperation::Symlink,
        }
    }
}

/// Conflict strategy choices for the dropdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictChoice {
    #[default]
    Skip,
    Rename,
    Overwrite,
}

impl std::fmt::Display for ConflictChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Skip => write!(f, "Skip"),
            Self::Rename => write!(f, "Rename (auto)"),
            Self::Overwrite => write!(f, "Overwrite"),
        }
    }
}

impl ConflictChoice {
    pub const ALL: &'static [Self] = &[Self::Skip, Self::Rename, Self::Overwrite];

    pub fn to_strategy(self) -> yaps_core::config::ConflictStrategy {
        match self {
            Self::Skip => yaps_core::config::ConflictStrategy::Skip,
            Self::Rename => yaps_core::config::ConflictStrategy::Rename,
            Self::Overwrite => yaps_core::config::ConflictStrategy::Overwrite,
        }
    }
}

/// Duplicate strategy choices for the dropdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateChoice {
    #[default]
    Skip,
    CopyToFolder,
}

impl std::fmt::Display for DuplicateChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Skip => write!(f, "Skip duplicates"),
            Self::CopyToFolder => write!(f, "Copy to [Duplicates]"),
        }
    }
}

impl DuplicateChoice {
    pub const ALL: &'static [Self] = &[Self::Skip, Self::CopyToFolder];

    pub fn to_strategy(self) -> yaps_core::config::DuplicateStrategy {
        match self {
            Self::Skip => yaps_core::config::DuplicateStrategy::Skip,
            Self::CopyToFolder => yaps_core::config::DuplicateStrategy::CopyToFolder,
        }
    }
}
