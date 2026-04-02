//! BLAKE3-based file hashing and persistent hash stores for duplicate detection.

pub mod hasher;
pub mod store;

pub use hasher::hash_file;
pub use store::HashStore;
