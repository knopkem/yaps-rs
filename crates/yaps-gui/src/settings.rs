//! Persistent GUI settings saved to the platform config directory.
//!
//! Settings are saved as TOML to `~/.config/yaps-rs/gui-settings.toml` (Linux),
//! `~/Library/Application Support/yaps-rs/gui-settings.toml` (macOS),
//! or `%APPDATA%\yaps-rs\gui-settings.toml` (Windows).

use serde::{Deserialize, Serialize};

use crate::messages::{ConflictChoice, DuplicateChoice, OperationChoice};

const SETTINGS_FILENAME: &str = "gui-settings.toml";
const APP_DIR_NAME: &str = "yaps-rs";

/// All persisted GUI settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub source: String,
    pub target: String,
    pub folder_pattern: String,
    pub file_pattern: String,
    pub operation: OperationChoice,
    pub conflict: ConflictChoice,
    pub duplicate: DuplicateChoice,
    pub recursive: bool,
    pub dry_run: bool,
    pub detect_duplicates: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            source: String::new(),
            target: String::new(),
            folder_pattern: "{year}/{month}".to_string(),
            file_pattern: "{day}-{month_short}-{hour}{minute}{second}-{filename}".to_string(),
            operation: OperationChoice::default(),
            conflict: ConflictChoice::default(),
            duplicate: DuplicateChoice::default(),
            recursive: true,
            dry_run: false,
            detect_duplicates: true,
        }
    }
}

impl Settings {
    /// Load settings from the platform config directory.
    /// Returns defaults if the file doesn't exist or can't be parsed.
    pub fn load() -> Self {
        let Some(path) = Self::settings_path() else {
            return Self::default();
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save settings to the platform config directory.
    /// Silently ignores errors (best-effort persistence).
    pub fn save(&self) {
        let Some(path) = Self::settings_path() else {
            tracing::warn!("Could not determine config directory for settings");
            return;
        };

        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                tracing::warn!("Failed to create settings directory: {e}");
                return;
            }
        }

        match toml::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = std::fs::write(&path, content) {
                    tracing::warn!("Failed to write settings: {e}");
                }
            }
            Err(e) => tracing::warn!("Failed to serialize settings: {e}"),
        }
    }

    fn settings_path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|d| d.join(APP_DIR_NAME).join(SETTINGS_FILENAME))
    }
}
