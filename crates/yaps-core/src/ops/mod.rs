//! File operations module.
//!
//! Provides directory scanning, file copy/move/link operations,
//! conflict resolution, and the main orchestrator.

pub mod conflict;
pub mod file_op;
pub mod organizer;
pub mod scanner;

pub use conflict::ConflictResolver;
pub use organizer::Organizer;
pub use scanner::Scanner;
