//! # yaps-core
//!
//! Core library for the YAPS-RS photo sorting tool.
//!
//! This crate provides all business logic for organizing photos based on EXIF metadata:
//! - **EXIF extraction** — Pure Rust metadata reading via `kamadak-exif`
//! - **Pattern system** — User-defined `{tag}` templates for folder/file naming
//! - **Duplicate detection** — BLAKE3-based content hashing with persistent stores
//! - **File operations** — Copy, move, hardlink, symlink with conflict resolution
//!
//! This crate is UI-agnostic and can be used by both the CLI and GUI frontends.

pub mod config;
pub mod error;
pub mod exif;
pub mod hash;
pub mod ops;
pub mod pattern;
pub mod report;

pub use config::Config;
pub use error::YapsError;
pub use report::Report;

/// Result type alias for yaps-core operations.
pub type Result<T> = std::result::Result<T, YapsError>;
