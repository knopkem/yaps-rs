//! Pattern string parser.
//!
//! Parses pattern strings like `"{year}/{month}-{month_long}"` into a sequence
//! of segments (literal text and tag references).

use super::tags::PatternTag;
use crate::error::YapsError;

/// A segment of a parsed pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternSegment {
    /// Literal text to include as-is.
    Literal(String),
    /// A tag reference to be resolved against EXIF metadata.
    Tag(PatternTag),
}

/// A parsed pattern consisting of a sequence of segments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPattern {
    /// The segments that make up this pattern.
    pub segments: Vec<PatternSegment>,
}

impl ParsedPattern {
    /// Returns `true` if the pattern contains no segments.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
}

/// A validation error in a pattern string with position information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternError {
    /// Human-readable error message.
    pub message: String,
    /// Byte offset of the start of the problematic region.
    pub start: usize,
    /// Byte offset past the end of the problematic region.
    pub end: usize,
}

impl std::fmt::Display for PatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (at {}..{})", self.message, self.start, self.end)
    }
}

/// Validate a pattern string and return all errors with their positions.
///
/// Unlike [`parse_pattern`], this function does not stop at the first error
/// but collects all problems so they can be highlighted in a UI.
///
/// Returns an empty `Vec` if the pattern is valid.
pub fn validate_pattern(input: &str) -> Vec<PatternError> {
    let mut errors = Vec::new();
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'{' {
            let open_pos = i;
            i += 1;

            // Escaped brace `{{`
            if i < len && bytes[i] == b'{' {
                i += 1;
                continue;
            }

            // Find closing brace
            let tag_start = i;
            let mut closed = false;
            while i < len {
                if bytes[i] == b'}' {
                    closed = true;
                    break;
                }
                i += 1;
            }

            if !closed {
                errors.push(PatternError {
                    message: "unclosed tag — missing '}'".to_string(),
                    start: open_pos,
                    end: len,
                });
                break;
            }

            let tag_name = &input[tag_start..i];
            let tag_name_trimmed = tag_name.trim();
            let close_pos = i + 1; // past the '}'
            i = close_pos;

            if tag_name_trimmed.is_empty() {
                errors.push(PatternError {
                    message: "empty tag name".to_string(),
                    start: open_pos,
                    end: close_pos,
                });
            } else if PatternTag::from_name(tag_name_trimmed).is_none() {
                errors.push(PatternError {
                    message: format!("unknown tag '{tag_name_trimmed}'"),
                    start: open_pos,
                    end: close_pos,
                });
            }
        } else if bytes[i] == b'}' {
            let pos = i;
            i += 1;
            if i < len && bytes[i] == b'}' {
                // Escaped brace `}}`
                i += 1;
            } else {
                errors.push(PatternError {
                    message: "unexpected '}' without matching '{'".to_string(),
                    start: pos,
                    end: i,
                });
            }
        } else {
            i += 1;
        }
    }

    errors
}

/// Parse a pattern string into a `ParsedPattern`.
///
/// Tags are delimited by `{` and `}`. Everything else is literal text.
/// Unknown tag names produce an error.
///
/// # Examples
/// ```
/// use yaps_core::pattern::parser::parse_pattern;
/// use yaps_core::pattern::tags::PatternTag;
/// use yaps_core::pattern::parser::PatternSegment;
///
/// let pattern = parse_pattern("{year}/{month}-{month_long}").unwrap();
/// assert_eq!(pattern.segments, vec![
///     PatternSegment::Tag(PatternTag::Year),
///     PatternSegment::Literal("/".to_string()),
///     PatternSegment::Tag(PatternTag::Month),
///     PatternSegment::Literal("-".to_string()),
///     PatternSegment::Tag(PatternTag::MonthLong),
/// ]);
/// ```
///
/// # Errors
/// Returns `YapsError::InvalidPattern` if:
/// - A `{` is not closed with `}`
/// - A tag name between `{}` is not recognized
/// - A `}` appears without a matching `{`
pub fn parse_pattern(input: &str) -> crate::Result<ParsedPattern> {
    let mut segments = Vec::new();
    let mut chars = input.chars().peekable();
    let mut literal_buf = String::new();

    while let Some(&ch) = chars.peek() {
        if ch == '{' {
            // Flush literal buffer
            if !literal_buf.is_empty() {
                segments.push(PatternSegment::Literal(literal_buf.clone()));
                literal_buf.clear();
            }

            chars.next(); // consume '{'

            // Check for escaped brace `{{`
            if chars.peek() == Some(&'{') {
                chars.next();
                literal_buf.push('{');
                continue;
            }

            // Read tag name until '}'
            let mut tag_name = String::new();
            let mut closed = false;
            for ch in chars.by_ref() {
                if ch == '}' {
                    closed = true;
                    break;
                }
                tag_name.push(ch);
            }

            if !closed {
                return Err(YapsError::InvalidPattern(format!(
                    "unclosed tag '{{{tag_name}' — missing closing '}}'"
                )));
            }

            let tag_name = tag_name.trim();
            if tag_name.is_empty() {
                return Err(YapsError::InvalidPattern("empty tag name '{}'".to_string()));
            }

            match PatternTag::from_name(tag_name) {
                Some(tag) => segments.push(PatternSegment::Tag(tag)),
                None => {
                    return Err(YapsError::InvalidPattern(format!(
                        "unknown tag '{{{tag_name}}}'"
                    )));
                }
            }
        } else if ch == '}' {
            chars.next(); // consume '}'

            // Check for escaped brace `}}`
            if chars.peek() == Some(&'}') {
                chars.next();
                literal_buf.push('}');
            } else {
                return Err(YapsError::InvalidPattern(
                    "unexpected '}' without matching '{'".to_string(),
                ));
            }
        } else {
            literal_buf.push(ch);
            chars.next();
        }
    }

    // Flush remaining literal
    if !literal_buf.is_empty() {
        segments.push(PatternSegment::Literal(literal_buf));
    }

    Ok(ParsedPattern { segments })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_tags() {
        let p = parse_pattern("{year}/{month}").unwrap();
        assert_eq!(
            p.segments,
            vec![
                PatternSegment::Tag(PatternTag::Year),
                PatternSegment::Literal("/".to_string()),
                PatternSegment::Tag(PatternTag::Month),
            ]
        );
    }

    #[test]
    fn test_parse_literal_only() {
        let p = parse_pattern("photos/sorted").unwrap();
        assert_eq!(
            p.segments,
            vec![PatternSegment::Literal("photos/sorted".to_string())]
        );
    }

    #[test]
    fn test_parse_tag_only() {
        let p = parse_pattern("{filename}").unwrap();
        assert_eq!(p.segments, vec![PatternSegment::Tag(PatternTag::Filename)]);
    }

    #[test]
    fn test_parse_complex_pattern() {
        let p =
            parse_pattern("{year}/{month}-{month_long}/{day}-{hour}{minute}{second}-{filename}")
                .unwrap();
        assert_eq!(p.segments.len(), 13);
    }

    #[test]
    fn test_parse_unknown_tag_returns_error() {
        let result = parse_pattern("{nonexistent}");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nonexistent"));
    }

    #[test]
    fn test_parse_unclosed_tag_returns_error() {
        let result = parse_pattern("{year");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unclosed"));
    }

    #[test]
    fn test_parse_unmatched_close_brace_returns_error() {
        let result = parse_pattern("foo}bar");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_tag_name_returns_error() {
        let result = parse_pattern("{}");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_escaped_braces() {
        let p = parse_pattern("{{literal}}").unwrap();
        assert_eq!(
            p.segments,
            vec![PatternSegment::Literal("{literal}".to_string())]
        );
    }

    #[test]
    fn test_parse_empty_string() {
        let p = parse_pattern("").unwrap();
        assert!(p.is_empty());
    }

    #[test]
    fn test_parse_whitespace_in_tag_name_is_trimmed() {
        let p = parse_pattern("{ year }").unwrap();
        assert_eq!(p.segments, vec![PatternSegment::Tag(PatternTag::Year)]);
    }

    #[test]
    fn test_parse_adjacent_tags() {
        let p = parse_pattern("{hour}{minute}{second}").unwrap();
        assert_eq!(p.segments.len(), 3);
        assert_eq!(p.segments[0], PatternSegment::Tag(PatternTag::Hour));
        assert_eq!(p.segments[1], PatternSegment::Tag(PatternTag::Minute));
        assert_eq!(p.segments[2], PatternSegment::Tag(PatternTag::Second));
    }

    #[test]
    fn test_parse_all_known_tags() {
        for tag in super::super::tags::ALL_TAGS {
            let input = format!("{{{}}}", tag.name());
            let p = parse_pattern(&input).unwrap();
            assert_eq!(p.segments, vec![PatternSegment::Tag(*tag)]);
        }
    }

    // -- validate_pattern tests --

    #[test]
    fn test_validate_valid_pattern() {
        assert!(validate_pattern("{year}/{month}").is_empty());
        assert!(validate_pattern("{year}/{month}-{month_long}").is_empty());
        assert!(validate_pattern("literal_only").is_empty());
        assert!(validate_pattern("").is_empty());
        assert!(validate_pattern("{{escaped}}").is_empty());
    }

    #[test]
    fn test_validate_unknown_tag() {
        let errs = validate_pattern("{foobar}");
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].start, 0);
        assert_eq!(errs[0].end, 8);
        assert!(errs[0].message.contains("unknown tag"));
        assert!(errs[0].message.contains("foobar"));
    }

    #[test]
    fn test_validate_unclosed_tag() {
        let errs = validate_pattern("{year");
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].start, 0);
        assert!(errs[0].message.contains("unclosed"));
    }

    #[test]
    fn test_validate_empty_tag() {
        let errs = validate_pattern("{}");
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].start, 0);
        assert_eq!(errs[0].end, 2);
        assert!(errs[0].message.contains("empty"));
    }

    #[test]
    fn test_validate_unmatched_close_brace() {
        let errs = validate_pattern("hello}world");
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].start, 5);
        assert_eq!(errs[0].end, 6);
    }

    #[test]
    fn test_validate_multiple_errors() {
        let errs = validate_pattern("{bad1}/{bad2}");
        assert_eq!(errs.len(), 2);
        assert_eq!(errs[0].start, 0);
        assert_eq!(errs[0].end, 6);
        assert_eq!(errs[1].start, 7);
        assert_eq!(errs[1].end, 13);
    }

    #[test]
    fn test_validate_mix_valid_and_invalid() {
        let errs = validate_pattern("{year}/{bogus}/{month}");
        assert_eq!(errs.len(), 1);
        assert!(errs[0].message.contains("bogus"));
        assert_eq!(errs[0].start, 7);
        assert_eq!(errs[0].end, 14);
    }

    #[test]
    fn test_validate_pattern_error_display() {
        let err = PatternError {
            message: "unknown tag 'foo'".to_string(),
            start: 5,
            end: 10,
        };
        assert_eq!(err.to_string(), "unknown tag 'foo' (at 5..10)");
    }
}
