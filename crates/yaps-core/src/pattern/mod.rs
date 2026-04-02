//! Pattern parsing and formatting module.
//!
//! Users define patterns using `{tag_name}` placeholders that are resolved
//! against EXIF metadata to produce concrete folder paths and filenames.

pub mod formatter;
pub mod parser;
pub mod tags;

pub use formatter::format_pattern;
pub use parser::{parse_pattern, validate_pattern, PatternError};
pub use tags::PatternTag;
