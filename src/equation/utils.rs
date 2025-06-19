//! # XMILE Unicode and String Processing Utilities
//!
//! This module provides Unicode-compliant string processing functions required
//! for XMILE identifier handling. It implements the Unicode normalization,
//! case folding, and comparison requirements specified in XMILE section 3.2.2.2.
//!
//! ## Key Features
//!
//! - **Unicode Normalization**: NFKC normalization for compatibility characters
//! - **UCA Compliance**: Unicode Collation Algorithm for case-insensitive comparison
//! - **Full-width Conversion**: Mapping full-width characters to half-width equivalents
//! - **XMILE Whitespace Rules**: Normalization of spaces, underscores, and newlines
//! - **Escape Sequence Processing**: Handling of quoted identifier escape sequences
//!
//! ## Unicode Processing Pipeline
//!
//! XMILE identifier processing follows this pipeline:
//!
//! 1. **Full-width conversion**: `full_to_half_width()`
//! 2. **NFKC normalization**: `nfkc_normalize()`
//! 3. **Whitespace normalization**: `xmile_normalize()`
//! 4. **Case folding for comparison**: `uca_case_fold()`
//! 5. **Comparison key generation**: `uca_compare_key()`
//!
//! ## Examples
//!
//! ```rust
//! use xmile::equation::utils;
//!
//! // Full-width character conversion
//! assert_eq!(utils::full_to_half_width('Ａ'), 'A');
//! assert_eq!(utils::full_to_half_width('１'), '1');
//!
//! // XMILE whitespace normalization
//! let (normalized, warnings) = utils::xmile_normalize("test___variable").into();
//! assert_eq!(normalized, "test variable");
//!
//! // UCA comparison key for case-insensitive comparison
//! let key1 = utils::uca_compare_key("Cash_Balance").unwrap();
//! let key2 = utils::uca_compare_key("CASH BALANCE").unwrap();
//! assert_eq!(key1, key2);
//! ```

use icu_collator::{CaseLevel, Collator, CollatorOptions, Strength};
use thiserror::Error;

use crate::types::WithWarnings;

/// Errors that can occur during string processing operations.
#[derive(Debug, Error)]
pub enum ProcessingError {
    /// An invalid escape sequence was encountered in a quoted identifier
    #[error("Invalid escape sequence: {0}")]
    InvalidEscapeSequence(char),

    /// An escape sequence was started but not completed
    #[error("Unterminated escape sequence")]
    UnterminatedEscapeSequence,

    /// An error occurred in ICU Unicode processing libraries
    #[error("ICU error during Unicode processing: {0}")]
    IcuError(String),
}

/// Converts a single full-width character to its half-width equivalent.
///
/// According to XMILE specification section 3.2.2.2, full-width Roman characters
/// (U+FF00 to U+FF5E) should be mapped to their normal Roman equivalents.
/// This function performs that mapping for individual characters.
///
/// # Arguments
///
/// * `ch` - The character to convert
///
/// # Returns
///
/// The half-width equivalent if the character is full-width, otherwise
/// returns the original character unchanged.
///
/// # Examples
///
/// ```rust
/// use xmile::equation::utils;
///
/// assert_eq!(utils::full_to_half_width('Ａ'), 'A'); // Full-width A to normal A
/// assert_eq!(utils::full_to_half_width('１'), '1'); // Full-width 1 to normal 1
/// assert_eq!(utils::full_to_half_width('！'), '!'); // Full-width exclamation
/// assert_eq!(utils::full_to_half_width('A'), 'A');  // Normal A unchanged
/// ```
///
/// # Unicode Range
///
/// This function converts characters in the range U+FF01 to U+FF5E by
/// subtracting the full-width offset (0xFF00 - 0x0020 = 0xFEE0).
pub fn full_to_half_width(ch: char) -> char {
    const FULL_WIDTH_OFFSET: u32 = 0xFF00 - 0x0020;
    match ch as u32 {
        code if code >= 0xFF01 && code <= 0xFF5E => {
            char::from_u32(code - FULL_WIDTH_OFFSET).unwrap_or(ch)
        }
        _ => ch,
    }
}

/// Normalizes a string using ICU NFKC normalization.
///
/// NFKC (Normalization Form Canonical Composition) ensures that equivalent
/// Unicode characters are represented consistently. This includes decomposing
/// and recomposing characters, and mapping compatibility characters to their
/// canonical equivalents.
///
/// # Arguments
///
/// * `input` - The string to normalize
///
/// # Returns
///
/// Returns the NFKC-normalized string, or a `ProcessingError` if normalization fails.
///
/// # Examples
///
/// ```rust
/// use xmile::equation::utils;
///
/// // Compatibility ligature fi (U+FB01) becomes "fi"
/// let normalized = utils::nfkc_normalize("ﬁle").unwrap();
/// assert_eq!(normalized, "file");
/// ```
///
/// # XMILE Compliance
///
/// This function is part of the Unicode processing pipeline required for
/// XMILE identifier equivalence rules as specified in section 3.2.2.2.
pub fn nfkc_normalize(input: &str) -> Result<String, ProcessingError> {
    let normalizer = icu_normalizer::ComposingNormalizer::new_nfkc();
    Ok(normalizer.normalize(input))
}

/// Performs UCA-compliant case folding for identifiers.
///
/// Case folding converts strings to a canonical lowercase form using the
/// Unicode Collation Algorithm (UCA). This is more comprehensive than
/// simple ASCII lowercasing and handles complex Unicode case mappings.
///
/// # Arguments
///
/// * `input` - The string to case-fold
///
/// # Returns
///
/// Returns the case-folded string, or a `ProcessingError` if folding fails.
///
/// # Examples
///
/// ```rust
/// use xmile::equation::utils;
///
/// // Basic ASCII case folding
/// assert_eq!(utils::uca_case_fold("HELLO").unwrap(), "hello");
///
/// // Unicode case folding - German ß
/// assert_eq!(utils::uca_case_fold("STRAßE").unwrap(), "strasse");
///
/// // Turkish dotted/dotless i
/// assert_eq!(utils::uca_case_fold("İstanbul").unwrap(), "i̇stanbul");
/// ```
///
/// # XMILE Compliance
///
/// The XMILE specification requires UCA-compliant case-insensitive comparison
/// as defined in Unicode Technical Report #10 and ISO 14651.
pub fn uca_case_fold(input: &str) -> Result<String, ProcessingError> {
    let case_mapper = icu_casemap::CaseMapper::new();
    Ok(case_mapper.fold_string(input))
}

/// Creates a UCA-compliant comparison key for identifiers.
///
/// This function applies the complete XMILE identifier normalization pipeline:
/// 1. NFKC normalization
/// 2. UCA case folding
/// 3. XMILE whitespace normalization
///
/// The resulting key can be used for efficient case-insensitive and
/// whitespace-insensitive comparison of identifiers.
///
/// # Arguments
///
/// * `input` - The identifier string to process
///
/// # Returns
///
/// Returns a normalized comparison key, or a `ProcessingError` if processing fails.
///
/// # Examples
///
/// ```rust
/// use xmile::equation::utils;
///
/// let key1 = utils::uca_compare_key("Cash_Balance").unwrap();
/// let key2 = utils::uca_compare_key("CASH BALANCE").unwrap();
/// let key3 = utils::uca_compare_key("cash_balance").unwrap();
///
/// assert_eq!(key1, key2);
/// assert_eq!(key2, key3);
/// ```
///
/// # Warning Handling
///
/// Any warnings from whitespace normalization (e.g., control characters
/// found) are logged using the `log` crate's warning facility.
pub fn uca_compare_key(input: &str) -> Result<String, ProcessingError> {
    // First normalize to NFKC
    let normalized = nfkc_normalize(input)?;

    // Apply case folding using UCA
    let case_folded = uca_case_fold(&normalized)?;

    // Apply XMILE-specific whitespace normalization
    let (whitespace_normalized, warnings) = xmile_normalize(&case_folded).into();

    // Log any warnings about control characters or whitespace issues
    warnings.into_iter().for_each(|w| log::warn!("{}", w));

    Ok(whitespace_normalized)
}

/// Compares two strings using UCA for case-insensitive comparison.
///
/// Creates a UCA-compliant collator configured for case-insensitive comparison
/// and uses it to compare the two input strings. This provides proper Unicode
/// ordering according to the Unicode Collation Algorithm.
///
/// # Arguments
///
/// * `left` - The first string to compare
/// * `right` - The second string to compare
///
/// # Returns
///
/// Returns the ordering relationship between the strings, or a `ProcessingError`
/// if the collator cannot be created.
///
/// # Examples
///
/// ```rust
/// use xmile::equation::utils;
///
/// use std::cmp::Ordering;
///
/// assert_eq!(utils::uca_compare("hello", "HELLO").unwrap(), Ordering::Equal);
/// assert_eq!(utils::uca_compare("apple", "banana").unwrap(), Ordering::Less);
/// ```
///
/// # Collation Settings
///
/// The collator is configured with:
/// - Primary strength (case-insensitive)
/// - Case level off (ignore case completely)
pub fn uca_compare(left: &str, right: &str) -> Result<std::cmp::Ordering, ProcessingError> {
    let mut options = CollatorOptions::new();
    options.strength = Some(Strength::Primary); // Case-insensitive
    options.case_level = Some(CaseLevel::Off); // Ignore case completely

    let collator = Collator::try_new(&Default::default(), options)
        .map_err(|e| ProcessingError::IcuError(format!("Failed to create Collator: {:?}", e)))?;
    Ok(collator.compare(left, right))
}

/// Checks if two strings are equal according to UCA rules.
///
/// This is a convenience function that uses `uca_compare()` and checks
/// for equality, providing a more readable API for equality testing.
///
/// # Arguments
///
/// * `left` - The first string to compare
/// * `right` - The second string to compare
///
/// # Returns
///
/// Returns `true` if the strings are equal under UCA rules, `false` otherwise,
/// or a `ProcessingError` if comparison fails.
///
/// # Examples
///
/// ```rust
/// use xmile::equation::utils;
///
/// assert!(utils::uca_equal("Hello", "HELLO").unwrap());
/// assert!(utils::uca_equal("STRAßE", "STRASSE").unwrap());
/// assert!(!utils::uca_equal("Hello", "World").unwrap());
/// ```
pub fn uca_equal(left: &str, right: &str) -> Result<bool, ProcessingError> {
    Ok(uca_compare(left, right).unwrap() == std::cmp::Ordering::Equal)
}

/// Normalizes XMILE identifiers according to whitespace and control character rules.
///
/// This function implements the XMILE whitespace equivalence rules from
/// specification section 3.2.2.2:
///
/// - Space (U+0020), underscore (_), newline (\n), and non-breaking space (U+00A0)
///   are all treated as equivalent whitespace
/// - Groups of consecutive whitespace characters are collapsed to a single space
/// - Control characters (below U+0020) are treated as whitespace with warnings
/// - Leading and trailing whitespace is removed
///
/// # Arguments
///
/// * `input` - The string to normalize
///
/// # Returns
///
/// Returns a `WithWarnings<String, String>` containing the normalized string
/// and any warnings about problematic characters encountered.
///
/// # Examples
///
/// ```rust
/// use xmile::equation::utils;
///
/// // Basic whitespace normalization
/// let (result, warnings) = utils::xmile_normalize("test___variable").into();
/// assert_eq!(result, "test variable");
/// assert!(warnings.is_empty());
///
/// // Control character handling with warning
/// let (result, warnings) = utils::xmile_normalize("test\x01variable").into();
/// assert_eq!(result, "test variable");
/// assert_eq!(warnings.len(), 1);
/// ```
///
/// # XMILE Whitespace Rules
///
/// According to the specification:
/// - `wom_multiplier` ≡ `"wom multiplier"` ≡ `"wom\nmultiplier"`
/// - `wom_multiplier` ≡ `wom______multiplier` (multiple whitespace collapsed)
pub fn xmile_normalize(input: &str) -> WithWarnings<String, String> {
    let mut result = String::new();
    let mut warnings = Vec::new();

    // Maintain a flag to track if we are reading whitespace
    let mut reading_whitespace = false;

    // Iterate through characters, applying XMILE whitespace rules
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        match ch {
            // XMILE whitespace equivalences: space, underscore, newline, non-breaking space
            ' ' | '_' | '\n' | '\u{00A0}' => {
                if !reading_whitespace && !result.is_empty() {
                    result.push(' ');
                }
                reading_whitespace = true;
            }
            // Control characters (below U+0020) treated as space
            c if (c as u32) < 0x0020 => {
                warnings.push(format!(
                    "Control character U+{:04X} found in identifier, treating as space",
                    c as u32
                ));

                if !reading_whitespace && !result.is_empty() {
                    result.push(' ');
                }
                reading_whitespace = true;
            }
            // Keep other characters as-is (case folding is done separately)
            c => {
                result.push(c);
                reading_whitespace = false;
            }
        }
    }

    // Remove trailing whitespace
    result = result.trim_end().to_string();

    // If we have warnings, return them
    if warnings.is_empty() {
        WithWarnings::Ok(result)
    } else {
        WithWarnings::Warning(result, warnings)
    }
}

/// Parses escape sequences in quoted identifiers.
///
/// XMILE quoted identifiers support a limited set of escape sequences:
/// - `\"` - Quotation mark
/// - `\n` - Newline
/// - `\\` - Backslash
///
/// Any other character following a backslash is considered an invalid
/// escape sequence.
///
/// # Arguments
///
/// * `input` - The string content inside quotes (without the surrounding quotes)
///
/// # Returns
///
/// Returns the unescaped string, or a `ProcessingError` for invalid escape sequences.
///
/// # Examples
///
/// ```rust
/// use xmile::equation::utils;
///
/// assert_eq!(utils::parse_xmile_escape("hello world").unwrap(), "hello world");
/// assert_eq!(utils::parse_xmile_escape("revenue\\ngap").unwrap(), "revenue\ngap");
/// assert_eq!(utils::parse_xmile_escape("quote: \\\"text\\\"").unwrap(), "quote: \"text\"");
/// assert_eq!(utils::parse_xmile_escape("path\\\\to\\\\file").unwrap(), "path\\to\\file");
///
/// // Invalid escape sequence
/// assert!(utils::parse_xmile_escape("invalid\\xsequence").is_err());
/// ```
///
/// # Error Cases
///
/// - `InvalidEscapeSequence(char)` - A backslash followed by an unsupported character
/// - `UnterminatedEscapeSequence` - A backslash at the end of the string
pub fn parse_xmile_escape(input: &str) -> Result<String, ProcessingError> {
    let mut result = String::new();
    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            // Handle escape sequences
            if let Some(next_ch) = chars.next() {
                match next_ch {
                    'n' => result.push('\n'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    _ => return Err(ProcessingError::InvalidEscapeSequence(next_ch)),
                }
            } else {
                return Err(ProcessingError::UnterminatedEscapeSequence);
            }
        } else {
            // Regular character, just add it
            result.push(ch);
        }
    }

    Ok(result)
}

/// Validates and warns about problematic Unicode characters.
///
/// This function checks individual characters for common Unicode issues
/// that may cause problems in XMILE identifiers:
///
/// - Full-width characters that should be converted to half-width
/// - Unusual Unicode space characters that should be mapped to normal spaces
///
/// # Arguments
///
/// * `ch` - The character to validate
///
/// # Returns
///
/// Returns `WithWarnings<(), String>` with warnings for any problematic
/// characters found. The result value is always `()` since this is
/// purely a validation function.
///
/// # Examples
///
/// ```rust
/// use xmile::equation::utils;
///
/// // Full-width character warning
/// let (_, warnings) = utils::unicode_char_warnings('Ａ').into();
/// assert_eq!(warnings.len(), 1);
/// assert!(warnings[0].contains("Full-width"));
///
/// // Unusual space character warning
/// let (_, warnings) = utils::unicode_char_warnings('\u{2002}').into(); // En-space
/// assert_eq!(warnings.len(), 1);
/// assert!(warnings[0].contains("En-space"));
///
/// // Normal character - no warnings
/// let (_, warnings) = utils::unicode_char_warnings('A').into();
/// assert!(warnings.is_empty());
/// ```
///
/// # Checked Characters
///
/// - **Full-width characters** (U+FF01 to U+FF5E): Recommends half-width equivalents
/// - **En-space** (U+2002): Recommends standard space
/// - **Em-space** (U+2003): Recommends standard space  
/// - **Ideographic space** (U+3000): Recommends standard space
pub fn unicode_char_warnings(ch: char) -> WithWarnings<(), String> {
    let codepoint = ch as u32;
    let mut warnings = Vec::new();

    // Check for full-width ASCII characters
    if codepoint >= 0xFF01 && codepoint <= 0xFF5E {
        warnings.push(format!(
            "Full-width character '{}' (U+{:04X}) found. Consider using half-width equivalent.",
            ch, codepoint
        ));
    }

    // Check for other problematic Unicode spaces mentioned in spec
    match codepoint {
        0x2002 => warnings.push("En-space (U+2002) found. Use standard space instead.".to_string()),
        0x2003 => warnings.push("Em-space (U+2003) found. Use standard space instead.".to_string()),
        0x3000 => warnings
            .push("Ideographic space (U+3000) found. Consider using standard space.".to_string()),
        _ => {}
    }

    if warnings.is_empty() {
        WithWarnings::Ok(())
    } else {
        WithWarnings::Warning((), warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uca_case_folding() {
        // Basic ASCII case folding
        assert_eq!(uca_case_fold("HELLO").unwrap(), "hello");

        // Unicode case folding - German ß
        assert_eq!(uca_case_fold("STRAßE").unwrap(), "strasse");

        // Turkish dotted/dotless i
        assert_eq!(uca_case_fold("İstanbul").unwrap(), "i̇stanbul");
    }

    #[test]
    fn test_uca_comparison() {
        // Case insensitive comparison
        assert!(uca_equal("Hello", "HELLO").unwrap());
        assert!(uca_equal("STRAßE", "STRASSE").unwrap());

        // Different strings should not be equal
        assert!(!uca_equal("Hello", "World").unwrap());
    }

    #[test]
    fn test_nfkc_normalization() {
        // Compatibility characters
        let input = "ﬁle"; // U+FB01 (fi ligature)
        let normalized = nfkc_normalize(input).unwrap();
        assert_eq!(normalized, "file");
    }

    #[test]
    fn test_uca_compare_key() {
        let key1 = uca_compare_key("Cash_Balance").unwrap();
        let key2 = uca_compare_key("CASH BALANCE").unwrap();
        let key3 = uca_compare_key("cash_balance").unwrap();

        assert_eq!(key1, key2);
        assert_eq!(key2, key3);
    }

    #[test]
    fn test_full_width_conversion() {
        assert_eq!(full_to_half_width('Ａ'), 'A');
        assert_eq!(full_to_half_width('１'), '1');
        assert_eq!(full_to_half_width('！'), '!');
        assert_eq!(full_to_half_width('A'), 'A'); // No change for normal chars
    }

    #[test]
    fn test_xmile_normalization() {
        // Basic underscore to space conversion
        let (result, warnings) = xmile_normalize("test_variable").into();
        assert_eq!(result, "test variable");
        assert!(warnings.is_empty());

        // Multiple underscores collapsed
        let (result, warnings) = xmile_normalize("test___variable").into();
        assert_eq!(result, "test variable");
        assert!(warnings.is_empty());

        // Control character with warning
        let (result, warnings) = xmile_normalize("test\x01variable").into();
        assert_eq!(result, "test variable");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Control character"));

        // Leading/trailing whitespace removal
        let (result, _) = xmile_normalize("  test_variable  ").into();
        assert_eq!(result, "test variable");
    }

    #[test]
    fn test_escape_sequence_parsing() {
        // Basic string without escapes
        assert_eq!(parse_xmile_escape("hello world").unwrap(), "hello world");

        // Newline escape
        assert_eq!(parse_xmile_escape("revenue\\ngap").unwrap(), "revenue\ngap");

        // Quote escape
        assert_eq!(
            parse_xmile_escape("quote: \\\"text\\\"").unwrap(),
            "quote: \"text\""
        );

        // Backslash escape
        assert_eq!(
            parse_xmile_escape("path\\\\to\\\\file").unwrap(),
            "path\\to\\file"
        );

        // Invalid escape sequence
        assert!(parse_xmile_escape("invalid\\xsequence").is_err());

        // Unterminated escape sequence
        assert!(parse_xmile_escape("unterminated\\").is_err());
    }

    #[test]
    fn test_unicode_char_warnings() {
        // Full-width character
        let (_, warnings) = unicode_char_warnings('Ａ').into();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Full-width"));

        // En-space
        let (_, warnings) = unicode_char_warnings('\u{2002}').into();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("En-space"));

        // Em-space
        let (_, warnings) = unicode_char_warnings('\u{2003}').into();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Em-space"));

        // Ideographic space
        let (_, warnings) = unicode_char_warnings('\u{3000}').into();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Ideographic space"));

        // Normal character - no warnings
        let (_, warnings) = unicode_char_warnings('A').into();
        assert!(warnings.is_empty());
    }
}
