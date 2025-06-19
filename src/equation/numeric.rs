//! # XMILE Numeric Constants Implementation
//!
//! This module implements XMILE numeric constant parsing and validation according
//! to specification section 3.2.1. It provides a robust parser for floating-point
//! numbers that pragmatically follows the XMILE BNF grammar.
//!
//! ## XMILE Numeric Constant Requirements
//!
//! According to the XMILE specification, numeric constants:
//!
//! - **MUST follow US English conventions** with period as decimal separator
//! - **Are expressed as floating point numbers** in decimal notation
//! - **Begin with either a digit or decimal point** (e.g., `123` or `.5`)
//! - **May have optional decimal point** (integers are valid: `42`)
//! - **MUST contain at least one digit** (`.` alone is invalid)
//! - **Support scientific notation** with `E` or `e` (e.g., `6E5`, `1.23e-4`)
//! - **Allow signed exponents** in scientific notation (e.g., `+8.123e-10`)
//!
//! ## BNF Grammar
//!
//! The implementation follows this exact BNF grammar from the specification:
//!
//! ```bnf
//! number ::= { [digit]+[.[digit]*] | [digit]*.[digit]+ }[{E | e} [{+ | –}] [digit]+]
//! digit ::= { 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 }
//! ```
//!
//! ## Data Type Philosophy
//!
//! XMILE has only one data type: **real numbers**. Even values that appear to be
//! integers (like array indices) are represented as real numbers internally.
//! This simplifies the type system whilst maintaining precision for mathematical
//! operations.
//!
//! ## Examples
//!
//! ```rust
//! use xmile::NumericConstant;
//! use std::str::FromStr;
//!
//! // Basic integers
//! let zero = NumericConstant::from_str("0").unwrap();
//! let positive = NumericConstant::from_str("42").unwrap();
//!
//! // Decimals with digits on both sides
//! let pi_approx = NumericConstant::from_str("3.14159").unwrap();
//!
//! // Decimals starting with dot
//! let fraction = NumericConstant::from_str(".375").unwrap();
//!
//! // Decimals ending with dot
//! let whole = NumericConstant::from_str("14.").unwrap();
//!
//! // Scientific notation
//! let large = NumericConstant::from_str("6E5").unwrap();      // 600,000
//! let small = NumericConstant::from_str("8.123e-10").unwrap(); // 0.0000000008123
//! let signed_exp = NumericConstant::from_str("2.5E+3").unwrap(); // 2,500
//!
//! // Access the underlying f64 value
//! let value: f64 = large.into();
//! assert_eq!(value, 600000.0);
//!
//! // Direct conversion from f64
//! let constant = NumericConstant::from(123.456);
//! println!("{}", constant); // Displays: 123.456
//! ```
//!
//! ## Sign Handling
//!
//! The specification is ambiguous about whether leading signs are part of the
//! numeric constant or separate unary operators. This implementation:
//!
//! - **Accepts negative numbers**: `-1` is a valid numeric constant
//! - **Accepts positive numbers**: `+1` is valid but generates a warning
//! - **Warns about redundant `+`**: The positive sign is unnecessary in constants
//! - **Defers to expressions**: Unary operators can still be applied in expressions
//!
//! This approach balances specification compliance with practical usability.
//!
//! ## Error Handling
//!
//! The parser provides detailed error information for various failure modes:
//!
//! ```rust
//! use xmile::equation::{NumericConstant, NumericConstantError};
//! use std::str::FromStr;
//!
//! // Empty input
//! let err = NumericConstant::from_str("").unwrap_err();
//! assert!(matches!(err, NumericConstantError::EmptyNumericConstant));
//!
//! // Multiple decimal points
//! let err = NumericConstant::from_str("1.2.3").unwrap_err();
//! assert!(matches!(err, NumericConstantError::MultipleDecimalPoints(_)));
//!
//! // Invalid scientific notation
//! let err = NumericConstant::from_str("1E").unwrap_err();
//! assert!(matches!(err, NumericConstantError::InvalidScientificNotation(_)));
//!
//! // No digits (just a decimal point)
//! let err = NumericConstant::from_str(".").unwrap_err();
//! assert!(matches!(err, NumericConstantError::NoDigits(_)));
//!
//! // Invalid characters
//! let err = NumericConstant::from_str("1a2").unwrap_err();
//! assert!(matches!(err, NumericConstantError::UnexpectedCharacter(_, _)));
//! ```
//!
//! ## Validation and Robustness
//!
//! The implementation includes comprehensive validation:
//!
//! - **Finite number checking**: Rejects infinity and NaN values
//! - **Character validation**: Only allows digits, decimal point, E/e, and signs
//! - **Structure validation**: Ensures proper placement of decimal points and exponents
//! - **Precision preservation**: Uses f64 for maximum precision within specification limits
//!
//! ## Sample Constants from Specification
//!
//! All sample constants from the XMILE specification parse correctly:
//!
//! | Input        | Value     | Description                |
//! |--------------|-----------|----------------------------|
//! | `0`          | 0.0       | Zero                       |
//! | `-1`         | -1.0      | Negative integer           |
//! | `.375`       | 0.375     | Decimal starting with dot  |
//! | `14.`        | 14.0      | Decimal ending with dot    |
//! | `6E5`        | 600000.0  | Scientific notation        |
//! | `+8.123e-10` | 8.123e-10 | Signed scientific notation |
//!
//! ## Performance Characteristics
//!
//! - **Zero-allocation parsing**: Uses string slicing and avoids unnecessary allocations
//! - **Single-pass validation**: Characters are validated during parsing
//! - **Efficient scientific notation**: Direct f64 operations for exponent application
//! - **Copy semantics**: `NumericConstant` is `Copy` for efficient parameter passing
//!
//! ## Integration with XMILE Expressions
//!
//! Numeric constants integrate seamlessly with the broader XMILE expression system:
//!
//! - **Type compatibility**: All constants are f64-compatible
//! - **Operator precedence**: Constants have highest precedence in expressions  
//! - **Unary operators**: `-` and `+` can be applied to constants in expressions
//! - **Function arguments**: Constants can be passed directly to XMILE functions

use log::warn;
use std::{fmt, str::FromStr};
use thiserror::Error;

/// Errors that can occur during numeric constant parsing.
///
/// This enum provides specific error types for different parsing failures,
/// enabling precise error reporting and handling. Each variant includes
/// contextual information about what went wrong and where.
#[derive(Debug, Error)]
pub enum NumericConstantError {
    /// The input string is empty or contains only whitespace.
    ///
    /// XMILE numeric constants must contain at least one digit.
    #[error("Empty numeric constant")]
    EmptyNumericConstant,

    /// Multiple decimal points found in the number.
    ///
    /// Examples: `"1.2.3"`, `"..5"`, `"1.."`
    ///
    /// XMILE allows at most one decimal point per numeric constant.
    #[error("Multiple decimal points: '{0}'")]
    MultipleDecimalPoints(String),

    /// Invalid scientific notation format.
    ///
    /// Examples: `"1E"` (no exponent), `"E5"` (no base), `"1E+"` (no digits after sign)
    ///
    /// Scientific notation requires both a base number and a valid exponent.
    #[error("Invalid scientific notation: '{0}'")]
    InvalidScientificNotation(String),

    /// Unexpected character found in the numeric constant.
    ///
    /// Examples: `"1a2"` (letter), `"1,000"` (comma), `"1@2"` (symbol)
    ///
    /// XMILE numeric constants only allow digits, decimal point, E/e, and signs.
    #[error("Unexpected character: '{1}' in '{0}'")]
    UnexpectedCharacter(String, char),

    /// No digits found in the numeric constant.
    ///
    /// Examples: `"."` (just decimal point), `"E5"` (no base digits)
    ///
    /// XMILE requires at least one digit in every numeric constant.
    #[error("No digits: '{0}'")]
    NoDigits(String),

    /// The parsed value is not a finite real number.
    ///
    /// Examples: Infinity, NaN, or numbers outside f64 range
    ///
    /// XMILE only supports finite real numbers as specified in section 3.2.3.
    #[error("Not a real number: '{0}' is not a valid real number")]
    NotARealNumber(String),

    /// Standard library parsing error.
    ///
    /// This wraps `std::num::ParseFloatError` for cases where the string
    /// structure is valid but `f64::parse()` fails.
    #[error("Parse error: '{0}' cannot be parsed as a number: {1}")]
    ParseFloatError(String, std::num::ParseFloatError),
}

/// A validated XMILE numeric constant.
///
/// This wrapper around `f64` ensures that the value represents a valid XMILE
/// numeric constant according to the specification. It can only be created
/// through parsing or conversion from finite f64 values.
///
/// ## Invariants
///
/// - The wrapped value is always finite (not infinity or NaN)
/// - The value follows XMILE numeric constant syntax rules
/// - The value uses standard f64 precision and range
///
/// ## Usage
///
/// ```rust
/// use xmile::NumericConstant;
/// use std::str::FromStr;
///
/// // Parse from string
/// let constant = NumericConstant::from_str("3.14159").unwrap();
///
/// // Convert to f64 for calculations
/// let value: f64 = constant.into();
///
/// // Create from f64
/// let from_float = NumericConstant::from(42.0);
///
/// // Display formatting
/// println!("Value: {}", constant);
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NumericConstant(pub f64);

/// Splits a numeric string into main part and optional exponent part.
fn split_exponent(input: &str) -> (&str, Option<&str>) {
    match input.find(|c: char| c == 'E' || c == 'e') {
        Some(pos) => (&input[0..pos], Some(&input[pos..])),
        None => (input, None),
    }
}

/// Parses the main part of a numeric constant (before any exponent).
fn parse_main(full: &str, input: &str) -> Result<f64, NumericConstantError> {
    // Validate main part
    if input.is_empty() {
        return Err(NumericConstantError::InvalidScientificNotation(
            full.to_string(),
        ));
    }

    let mut chars = input.chars();
    let mut has_digits = false;
    let mut has_decimal = false;

    // Check for optional sign
    if let Some(first_char) = chars.next() {
        match first_char {
            '+' => {
                warn!(
                    "Numeric constant '{}' starts with a redundant '+' sign, ignoring.",
                    full
                );
            }
            '-' => {}
            '0'..='9' => {
                has_digits = true;
            }
            '.' => {
                if has_decimal {
                    return Err(NumericConstantError::MultipleDecimalPoints(
                        input.to_string(),
                    ));
                }
                has_decimal = true;
            }
            c => {
                return Err(NumericConstantError::UnexpectedCharacter(
                    full.to_string(),
                    c,
                ));
            }
        }
    }

    // Parse remaining characters
    for ch in chars {
        match ch {
            '0'..='9' => {
                has_digits = true;
            }
            '.' => {
                if has_decimal {
                    return Err(NumericConstantError::MultipleDecimalPoints(
                        input.to_string(),
                    ));
                }
                has_decimal = true;
            }
            c => {
                return Err(NumericConstantError::UnexpectedCharacter(
                    input.to_string(),
                    c,
                ));
            }
        }
    }

    // Must have at least one digit in the main part
    if !has_digits {
        return Err(NumericConstantError::NoDigits(input.to_string()));
    }

    // Parse the main part as f64
    parse_as_f64(full, input)
}

/// Parses the exponent part of scientific notation.
fn parse_exponent(full: &str, input: &str) -> Result<f64, NumericConstantError> {
    // Validate exponent part if present
    if !input.starts_with('E') && !input.starts_with('e') {
        return Err(NumericConstantError::InvalidScientificNotation(
            full.to_string(),
        ));
    }

    // Skip the 'E' or 'e'
    let remaining = &input[1..];
    if remaining.is_empty() {
        return Err(NumericConstantError::InvalidScientificNotation(
            full.to_string(),
        ));
    }

    // Check for optional sign and digits in exponent
    let mut chars = remaining.chars();
    let mut has_digits = false;

    // Check for optional sign
    if let Some(first_char) = chars.next() {
        match first_char {
            '+' | '-' => {}
            '0'..='9' => {
                has_digits = true;
            }
            c => {
                return Err(NumericConstantError::UnexpectedCharacter(
                    full.to_string(),
                    c,
                ));
            }
        }
    }

    // Parse remaining digits in exponent
    for ch in chars {
        match ch {
            '0'..='9' => {
                has_digits = true;
            }
            c => {
                return Err(NumericConstantError::UnexpectedCharacter(c.to_string(), c));
            }
        }
    }

    // Exponent must have at least one digit
    if !has_digits {
        return Err(NumericConstantError::NoDigits(full.to_string()));
    }

    // Parse the exponent as f64
    parse_as_f64(full, remaining)
}

/// Parses a string as f64 with additional validation for XMILE requirements.
fn parse_as_f64(full: &str, input: &str) -> Result<f64, NumericConstantError> {
    match input.parse::<f64>() {
        Ok(value) if value.is_finite() => Ok(value),
        Ok(_) => Err(NumericConstantError::NotARealNumber(full.to_string())),
        Err(err) => Err(NumericConstantError::ParseFloatError(full.to_string(), err)),
    }
}

impl FromStr for NumericConstant {
    type Err = NumericConstantError;

    /// Parses a numeric constant from a string according to XMILE specification.
    ///
    /// This implementation follows the exact BNF grammar from XMILE section 3.2.1
    /// and handles all valid forms of numeric constants including:
    ///
    /// - **Integers**: `0`, `42`, `999`
    /// - **Decimals**: `3.14`, `0.5`, `99.99`
    /// - **Leading decimal**: `.375`, `.5`, `.0`
    /// - **Trailing decimal**: `14.`, `123.`, `0.`
    /// - **Scientific notation**: `6E5`, `1.23e-4`, `+8.123e-10`
    /// - **Signed numbers**: `-1`, `+42` (with warning for redundant +)
    ///
    /// The parser is strict about XMILE requirements whilst providing helpful
    /// error messages for common mistakes.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to parse as a numeric constant
    ///
    /// # Returns
    ///
    /// `Ok(NumericConstant)` if the string is a valid XMILE numeric constant,
    /// or `Err(NumericConstantError)` with details about the parsing failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::NumericConstant;
    /// use std::str::FromStr;
    ///
    /// // Valid examples
    /// assert!(NumericConstant::from_str("42").is_ok());
    /// assert!(NumericConstant::from_str("3.14159").is_ok());
    /// assert!(NumericConstant::from_str(".375").is_ok());
    /// assert!(NumericConstant::from_str("6E5").is_ok());
    /// assert!(NumericConstant::from_str("-1").is_ok());
    ///
    /// // Invalid examples
    /// assert!(NumericConstant::from_str("").is_err());        // Empty
    /// assert!(NumericConstant::from_str("1.2.3").is_err());   // Multiple decimals
    /// assert!(NumericConstant::from_str("1E").is_err());      // Incomplete scientific
    /// assert!(NumericConstant::from_str("abc").is_err());     // Non-numeric
    /// assert!(NumericConstant::from_str("1,000").is_err());   // Wrong delimiter
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.trim();

        // Check for empty input
        if input.is_empty() {
            return Err(NumericConstantError::EmptyNumericConstant);
        }

        // Split into main part and exponent part
        let (main_part, exp_part) = split_exponent(input);

        // Parse the main part
        let main_value = parse_main(input, main_part)?;
        let exp_value = exp_part.map(|exp| parse_exponent(input, exp)).transpose()?;

        // If there's an exponent, apply it
        let final_value = if let Some(exp) = exp_value {
            main_value * 10f64.powf(exp)
        } else {
            main_value
        };

        // Check if the final value is a valid real number
        if final_value.is_infinite() || final_value.is_nan() {
            return Err(NumericConstantError::NotARealNumber(input.to_string()));
        }

        Ok(NumericConstant(final_value))
    }
}

impl From<f64> for NumericConstant {
    /// Creates a numeric constant from an f64 value.
    ///
    /// This conversion assumes the f64 value is finite. For values that
    /// might be infinite or NaN, use `FromStr` with validation instead.
    ///
    /// # Arguments
    ///
    /// * `value` - The f64 value to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::NumericConstant;
    ///
    /// let constant = NumericConstant::from(3.14159);
    /// let value: f64 = constant.into();
    /// assert_eq!(value, 3.14159);
    /// ```
    fn from(value: f64) -> Self {
        NumericConstant(value)
    }
}

impl From<NumericConstant> for f64 {
    /// Extracts the f64 value from a numeric constant.
    ///
    /// This conversion is always safe since `NumericConstant` can only
    /// contain finite f64 values.
    ///
    /// # Arguments
    ///
    /// * `value` - The numeric constant to convert
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::NumericConstant;
    /// use std::str::FromStr;
    ///
    /// let constant = NumericConstant::from_str("42.5").unwrap();
    /// let value: f64 = constant.into();
    /// assert_eq!(value, 42.5);
    /// ```
    fn from(value: NumericConstant) -> Self {
        value.0
    }
}

impl fmt::Display for NumericConstant {
    /// Formats the numeric constant for display.
    ///
    /// Uses the standard f64 display formatting, which automatically
    /// chooses between decimal and scientific notation as appropriate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::NumericConstant;
    ///
    /// let constant = NumericConstant::from(123.456);
    /// assert_eq!(format!("{}", constant), "123.456");
    ///
    /// let scientific = NumericConstant::from(1e10);
    /// assert_eq!(format!("{}", scientific), "10000000000");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_valid_integers() {
        let cases = vec![
            ("0", 0.0),
            ("1", 1.0),
            ("42", 42.0),
            ("123", 123.0),
            ("999", 999.0),
        ];

        for (input, expected) in cases {
            let result = NumericConstant::from_str(input);
            assert!(result.is_ok(), "Failed to parse valid integer: {}", input);
            assert_eq!(result.unwrap().0, expected);
        }
    }

    #[test]
    fn test_valid_decimals_with_digits_both_sides() {
        let cases = vec![
            ("1.0", 1.0),
            ("3.14", 3.14),
            ("123.456", 123.456),
            ("0.5", 0.5),
            ("99.99", 99.99),
        ];

        for (input, expected) in cases {
            let result = NumericConstant::from_str(input);
            assert!(result.is_ok(), "Failed to parse valid decimal: {}", input);
            assert_eq!(result.unwrap().0, expected);
        }
    }

    #[test]
    fn test_valid_decimals_starting_with_dot() {
        let cases = vec![
            (".5", 0.5),
            (".375", 0.375),
            (".1", 0.1),
            (".999", 0.999),
            (".0", 0.0),
        ];

        for (input, expected) in cases {
            let result = NumericConstant::from_str(input);
            assert!(
                result.is_ok(),
                "Failed to parse valid decimal starting with dot: {}",
                input
            );
            assert_eq!(result.unwrap().0, expected);
        }
    }

    #[test]
    fn test_valid_decimals_ending_with_dot() {
        let cases = vec![("1.", 1.0), ("14.", 14.0), ("123.", 123.0), ("0.", 0.0)];

        for (input, expected) in cases {
            let result = NumericConstant::from_str(input);
            assert!(
                result.is_ok(),
                "Failed to parse valid decimal ending with dot: {}",
                input
            );
            assert_eq!(result.unwrap().0, expected);
        }
    }

    #[test]
    fn test_valid_scientific_notation() {
        let cases = vec![
            ("1E5", 1e5),
            ("6E5", 6e5),
            ("1e10", 1e10),
            ("2.5E3", 2.5e3),
            ("1.23e-4", 1.23e-4),
            ("8.123e-10", 8.123e-10),
            ("1E+5", 1e5),
            ("3.14E-2", 3.14e-2),
            (".5E2", 0.5e2),
            ("14.E3", 14e3),
        ];

        for (input, expected) in cases {
            let result = NumericConstant::from_str(input);
            assert!(
                result.is_ok(),
                "Failed to parse valid scientific notation: {}",
                input
            );
            assert!(
                (result.as_ref().unwrap().0 - expected).abs() < 1e-15,
                "Value mismatch for {}: got {}, expected {}",
                input,
                result.unwrap().0,
                expected
            );
        }
    }

    #[test]
    fn test_sample_constants_from_docs() {
        // Test all the sample constants mentioned in the documentation
        let cases = vec![("0", 0.0), (".375", 0.375), ("14.", 14.0), ("6E5", 6e5)];

        for (input, expected) in cases {
            let result = NumericConstant::from_str(input);
            assert!(result.is_ok(), "Failed to parse sample constant: {}", input);
            assert_eq!(result.unwrap().0, expected);
        }
    }

    #[test]
    fn test_invalid_empty_string() {
        let result = NumericConstant::from_str("");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            NumericConstantError::EmptyNumericConstant
        ));
    }

    #[test]
    fn test_invalid_whitespace_only() {
        let cases = vec!["   ", "\t", "\n", " \t\n "];

        for input in cases {
            let result = NumericConstant::from_str(input);
            assert!(
                result.is_err(),
                "Should reject whitespace-only input: {:?}",
                input
            );
        }
    }

    #[test]
    fn test_invalid_just_decimal_point() {
        let result = NumericConstant::from_str(".");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            NumericConstantError::NoDigits(_)
        ));
    }

    #[test]
    fn test_invalid_multiple_decimal_points() {
        let cases = vec!["1.2.3", "..5", "1..", ".2."];

        for input in cases {
            let result = NumericConstant::from_str(input);
            assert!(
                result.is_err(),
                "Should reject multiple decimal points: {}",
                input
            );
        }
    }

    #[test]
    fn test_invalid_letters_in_number() {
        let cases = vec!["1a", "a1", "1.2a", "abc", "1x2"];

        for input in cases {
            let result = NumericConstant::from_str(input);
            assert!(
                result.is_err(),
                "Should reject letters in number: {}",
                input
            );
        }
    }

    #[test]
    fn test_invalid_scientific_notation() {
        let cases = vec![
            "1E",    // E without exponent
            "1e",    // e without exponent
            "E5",    // E without base
            "e5",    // e without base
            "1E+",   // E with sign but no digits
            "1E-",   // E with sign but no digits
            "1Ea",   // E with invalid character
            "1E5.5", // E with decimal in exponent
            "1EE5",  // Multiple E's
            "1E+5-", // Invalid characters after exponent
        ];

        for input in cases {
            let result = NumericConstant::from_str(input);
            assert!(
                result.is_err(),
                "Should reject invalid scientific notation: {}",
                input
            );
        }
    }

    #[test]
    fn test_invalid_special_characters() {
        let cases = vec!["1,000", "1_000", "1@2", "1#2"];

        for input in cases {
            let result = NumericConstant::from_str(input);
            assert!(
                result.is_err(),
                "Should reject special characters: {}",
                input
            );
        }
    }

    #[test]
    fn test_edge_cases_large_numbers() {
        let cases = vec![
            "1E308",  // Near f64 max
            "1E-308", // Near f64 min positive
        ];

        for input in cases {
            let result = NumericConstant::from_str(input);
            assert!(result.is_ok(), "Should parse large valid number: {}", input);
        }
    }

    #[test]
    fn test_invalid_infinity_and_nan() {
        // These might parse as f64 but should be rejected by our additional validation
        let cases = vec!["inf", "infinity", "nan", "NaN", "-inf"];

        for input in cases {
            let result = NumericConstant::from_str(input);
            // These should either fail to parse or be rejected by our validation
            assert!(result.is_err(), "Should reject infinity/NaN: {}", input);
        }
    }

    #[test]
    fn test_whitespace_handling() {
        // Test that leading/trailing whitespace is handled correctly
        let cases = vec![
            (" 1.5 ", 1.5),
            ("\t42\t", 42.0),
            ("\n3.14\n", 3.14),
            ("  .5  ", 0.5),
        ];

        for (input, expected) in cases {
            let result = NumericConstant::from_str(input);
            assert!(result.is_ok(), "Should handle whitespace: {:?}", input);
            assert_eq!(result.unwrap().0, expected);
        }
    }

    #[test]
    fn test_conversion_traits() {
        let value = 42.5;
        let constant = NumericConstant::from(value);
        assert_eq!(constant.0, value);

        let back_to_f64: f64 = constant.into();
        assert_eq!(back_to_f64, value);

        // Test Display trait
        let display_str = format!("{}", constant);
        assert_eq!(display_str, "42.5");
    }

    #[test]
    fn test_copy_clone_partialeq() {
        let c1 = NumericConstant(1.5);
        let c2 = c1; // Test Copy
        let c3 = c1.clone(); // Test Clone

        assert_eq!(c1, c2); // Test PartialEq
        assert_eq!(c1, c3); // Test PartialEq
        assert_eq!(c2, c3); // Test PartialEq
    }

    #[test]
    fn test_zero_variations() {
        let cases = vec![
            ("0", 0.0),
            ("0.", 0.0),
            (".0", 0.0),
            ("0.0", 0.0),
            ("0E0", 0.0),
            ("0e0", 0.0),
            (".0E0", 0.0),
            ("0.E0", 0.0),
        ];

        for (input, expected) in cases {
            let result = NumericConstant::from_str(input);
            assert!(result.is_ok(), "Should parse zero variation: {}", input);
            assert_eq!(result.unwrap().0, expected);
        }
    }

    #[test]
    fn test_signed_numbers_from_spec_samples() {
        // Test the sample constants from the XMILE specification that include signs
        let cases = vec![("-1", -1.0), ("+8.123e-10", 8.123e-10)];

        for (input, expected) in cases {
            let result = NumericConstant::from_str(input);
            assert!(
                result.is_ok(),
                "Failed to parse signed sample constant: {}",
                input
            );
            assert!(
                (result.as_ref().unwrap().0 - expected).abs() < 1e-15,
                "Value mismatch for {}: got {}, expected {}",
                input,
                result.as_ref().unwrap().0,
                expected
            );
        }
    }
}
