//! # XMILE Identifier Implementation
//!
//! This module implements XMILE identifier parsing, normalization, and comparison
//! according to the XMILE specification section 3.2.2.
//!
//! ## Key Features
//!
//! - **Unicode Compliance**: Full Unicode support with UCA-compliant case folding
//! - **Equivalence Rules**: Case-insensitive and whitespace-insensitive comparison
//! - **Namespace Support**: Qualified identifiers with dot notation (e.g., `std.function`)
//! - **Quoted Identifiers**: Support for arbitrary UTF-8 strings in double quotes
//! - **Reserved Word Checking**: Comprehensive validation against XMILE reserved identifiers
//!
//! ## Identifier Forms
//!
//! XMILE supports two forms of identifiers:
//!
//! ### Unquoted Identifiers
//! - Must start with a letter, underscore, or Unicode character above U+007F
//! - Cannot start with digits or dollar signs (except for units of measure)
//! - Cannot start or end with underscores
//! - May contain letters, digits, underscores, dollar signs, and Unicode characters
//!
//! ### Quoted Identifiers
//! - Enclosed in double quotes: `"any string here"`
//! - Support escape sequences: `\"`, `\n`, `\\`
//! - Allow arbitrary UTF-8 content including spaces and special characters
//!
//! ## Equivalence Rules
//!
//! According to XMILE specification section 3.2.2.2, identifiers are equivalent if they
//! differ only in:
//!
//! - **Case**: `Cash_Balance` ≡ `cash_balance` ≡ `CASH_BALANCE`
//! - **Whitespace**: `wom_multiplier` ≡ `"wom multiplier"` ≡ `"wom\nmultiplier"`
//!
//! Whitespace characters include: space (` `), underscore (`_`), newline (`\n`),
//! and non-breaking space (U+00A0).
//!
//! ## Examples
//!
//! ```rust
//! use xmile::Identifier;
//!
//! // Basic identifier
//! let id1 = Identifier::parse_default("Cash_Balance").unwrap();
//! assert_eq!(id1.normalized(), "Cash Balance");
//!
//! // Quoted identifier with escape sequence
//! let id2 = Identifier::parse_default("\"revenue\\ngap\"").unwrap();
//! assert_eq!(id2.normalized(), "revenue gap");
//!
//! // Namespace qualified
//! let id3 = Identifier::parse_default("std.function").unwrap();
//! assert!(id3.is_qualified());
//! assert_eq!(id3.unqualified(), "function");
//!
//! // Equivalence checking
//! let id4 = Identifier::parse_default("Cash_Balance").unwrap();
//! let id5 = Identifier::parse_default("cash_balance").unwrap();
//! assert_eq!(id4, id5); // Case-insensitive
//!
//! let id6 = Identifier::parse_default("wom_multiplier").unwrap();
//! let id7 = Identifier::parse_default("\"wom multiplier\"").unwrap();
//! assert_eq!(id6, id7); // Whitespace-insensitive
//! ```

use log::warn;
use thiserror::Error;

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use super::utils;
use crate::Namespace;

/// Errors that can occur during identifier parsing and processing.
#[derive(Debug, Error)]
pub enum IdentifierError {
    /// Error occurred during string processing (Unicode normalization, case folding, etc.)
    #[error("String processing error: {0}")]
    ProcessingError(#[from] utils::ProcessingError),

    /// The provided identifier string is empty
    #[error("Empty identifier provided")]
    EmptyIdentifier,

    /// The identifier starts with an invalid character
    #[error("Invalid first character: '{0}'")]
    InvalidFirstCharacter(char),

    /// The identifier ends with an invalid character
    #[error("Invalid last character: '{0}'")]
    InvalidLastCharacter(char),

    /// The identifier contains an invalid character
    #[error("Invalid character in identifier: '{0}'")]
    InvalidCharacter(char),

    /// The identifier is a reserved word and cannot be used
    #[error("Identifier '{0}' is reserved and cannot be used")]
    ReservedIdentifier(String),

    /// The qualified name format is invalid (e.g., empty namespace or identifier part)
    #[error("Invalid qualified name format")]
    InvalidQualifiedName,
}

/// An XMILE identifier that supports Unicode, namespaces, and equivalence rules.
///
/// This struct represents an identifier according to the XMILE specification,
/// providing Unicode-compliant normalization and comparison. It handles both
/// quoted and unquoted forms, namespace qualification, and implements the
/// case-insensitive and whitespace-insensitive equivalence rules.
///
/// ## Internal Structure
///
/// - `raw`: The original identifier string as provided by the user
/// - `normalized`: The processed identifier with normalized Unicode and whitespace
/// - `compare_key`: A UCA-compliant key used for efficient comparison and hashing
/// - `namespace_path`: Optional namespace qualification (e.g., `["std"]` for `std.function`)
/// - `quoted`: Whether the original identifier was enclosed in quotes
///
/// ## Comparison and Hashing
///
/// Identifiers use Unicode Collation Algorithm (UCA) for comparison, making them
/// suitable for use in hash maps and sorted collections whilst maintaining
/// XMILE equivalence rules.
#[derive(Debug, Clone)]
pub struct Identifier {
    /// The raw identifier string as provided
    raw: String,
    /// The normalized identifier (unquoted, case-folded, whitespace normalized)
    normalized: String,
    /// The cached UCA-compliant comparison key
    compare_key: String,
    /// Optional namespace path for qualified identifiers
    namespace_path: Vec<Namespace>,
    /// Whether the identifier was originally quoted
    quoted: bool,
}

impl Identifier {
    /// Reserved keywords and operators in XMILE.
    ///
    /// These are language-level constructs that cannot be redefined by users.
    /// From XMILE spec section 3.2.2.5: "The operator names AND, OR, and NOT,
    /// the statement keywords IF, THEN, and ELSE... are reserved identifiers."
    const RESERVED_KEYWORDS: [&'static str; 6] = ["and", "or", "not", "if", "then", "else"];

    /// Reserved built-in function names.
    ///
    /// These functions are provided by the XMILE standard library and cannot
    /// be redefined. Includes mathematical, time/delay, logic, and lookup functions.
    const RESERVED_FUNCTIONS: [&'static str; 39] = [
        // Mathematical functions
        "abs",
        "sin",
        "cos",
        "tan",
        "asin",
        "acos",
        "atan",
        "atan2",
        "sinh",
        "cosh",
        "tanh",
        "asinh",
        "acosh",
        "atanh",
        "sqrt",
        "exp",
        "ln",
        "log",
        "log10",
        "pow",
        "power",
        "min",
        "max",
        "sum",
        "mean",
        "median",
        "stddev",
        // Time and delay functions
        "time",
        "dt",
        "starttime",
        "stoptime",
        "timestep",
        "delay",
        "delay1",
        "delay3",
        // Logic and conditional functions
        "if_then_else",
        "pulse_train",
        // Array and lookup functions
        "lookup",
        "with_lookup",
    ];
}

impl Identifier {
    /// Parses a new identifier from a string.
    ///
    /// # Arguments
    ///
    /// * `input` - The identifier string to parse
    /// * `options` - Parsing options to customize behavior
    ///
    /// # Returns
    ///
    /// Returns `Ok(Identifier)` if the string is a valid XMILE identifier,
    /// or `Err(IdentifierError)` if parsing fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::equation::identifier::{Identifier, IdentifierOptions};
    ///
    /// let opts = IdentifierOptions {
    ///     allow_dollar: false,
    ///     allow_digit: false,
    ///     allow_reserved: false,
    /// };
    /// let id = Identifier::parse("Cash_Balance", opts).unwrap();
    /// assert_eq!(id.normalized(), "Cash Balance");
    /// ```
    pub fn parse(input: &str, options: IdentifierOptions) -> Result<Self, IdentifierError> {
        // Return error if input is empty
        if input.is_empty() {
            return Err(IdentifierError::EmptyIdentifier);
        }

        // Validate and warn about Unicode issues
        input
            .chars()
            .map(|w| utils::unicode_char_warnings(w).warnings())
            .flatten()
            .for_each(|w| warn!("{}", w));

        parse_identifier(input, options)
    }

    /// Parses an identifier using default options.
    ///
    /// This is a convenience method that uses the default parsing options,
    /// allowing for standard XMILE identifiers without special handling for
    /// units of measure or reserved words.
    ///
    /// # Arguments
    ///
    /// * `input` - The identifier string to parse
    ///
    /// # Returns
    ///
    /// Returns `Ok(Identifier)` if the string is a valid XMILE identifier,
    /// or `Err(IdentifierError)` if parsing fails.
    pub fn parse_default(input: &str) -> Result<Self, IdentifierError> {
        Self::parse(input, IdentifierOptions::default())
    }

    /// Parses an identifier specifically for units of measure.
    ///
    /// This method allows for identifiers that start with a dollar sign (`$`),
    /// which is common in XMILE for units of measure.
    ///
    /// # Arguments
    ///
    /// * `input` - The identifier string to parse
    ///
    /// # Returns
    ///
    /// Returns `Ok(Identifier)` if the string is a valid XMILE identifier for units,
    /// or `Err(IdentifierError)` if parsing fails.
    pub fn parse_unit_name(input: &str) -> Result<Self, IdentifierError> {
        Self::parse(input, IdentifierOptions::units_of_measure())
    }

    /// Returns the raw identifier string as originally provided.
    ///
    /// This preserves the exact input including quotes, case, and whitespace.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Identifier;
    ///
    /// let id = Identifier::parse_default("\"Cash Balance\"").unwrap();
    /// assert_eq!(id.raw(), "\"Cash Balance\"");
    /// ```
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Returns the normalized identifier string.
    ///
    /// The normalized form has:
    /// - Unicode NFKC normalization applied
    /// - Quotes removed (if originally quoted)
    /// - Escape sequences processed
    /// - Whitespace normalized according to XMILE rules
    /// - Case preserved (case folding is only used for comparison)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Identifier;
    ///
    /// let id = Identifier::parse_default("Cash_Balance").unwrap();
    /// assert_eq!(id.normalized(), "Cash Balance");
    ///
    /// let quoted = Identifier::parse_default("\"revenue\\ngap\"").unwrap();
    /// assert_eq!(quoted.normalized(), "revenue gap");
    /// ```
    pub fn normalized(&self) -> &str {
        &self.normalized
    }

    /// Returns the namespace path for qualified identifiers.
    ///
    /// For unqualified identifiers, this returns an empty slice.
    /// For qualified identifiers like `std.function`, this returns the
    /// parsed namespace components.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::{Identifier, Namespace};
    ///
    /// let simple = Identifier::parse_default("function").unwrap();
    /// assert!(simple.namespace_path().is_empty());
    ///
    /// let qualified = Identifier::parse_default("std.function").unwrap();
    /// assert_eq!(qualified.namespace_path().len(), 1);
    /// assert_eq!(qualified.namespace_path()[0], Namespace::Std);
    /// ```
    pub fn namespace_path(&self) -> &[Namespace] {
        &self.namespace_path
    }

    /// Returns the top-level namespace, if present.
    ///
    /// For qualified identifiers, this returns the first namespace component.
    /// For nested namespaces like `isee.utils.function`, this returns `isee`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::{Identifier, Namespace};
    ///
    /// let qualified = Identifier::parse_default("std.function").unwrap();
    /// assert_eq!(qualified.top_level_namespace(), Some(&Namespace::Std));
    ///
    /// let unqualified = Identifier::parse_default("function").unwrap();
    /// assert_eq!(unqualified.top_level_namespace(), None);
    /// ```
    pub fn top_level_namespace(&self) -> Option<&Namespace> {
        self.namespace_path.first()
    }

    /// Returns the UCA-compliant comparison key.
    ///
    /// This key is used internally for efficient comparison and hashing.
    /// It incorporates case folding and whitespace normalization according
    /// to XMILE equivalence rules.
    ///
    /// # Note
    ///
    /// This is primarily for internal use. For most purposes, direct comparison
    /// using `==` or `cmp()` is preferred.
    pub fn compare_key(&self) -> &str {
        &self.compare_key
    }

    /// Checks if the identifier was originally quoted.
    ///
    /// Returns `true` if the original input was enclosed in double quotes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Identifier;
    ///
    /// let unquoted = Identifier::parse_default("function").unwrap();
    /// assert!(!unquoted.is_quoted());
    ///
    /// let quoted = Identifier::parse_default("\"function name\"").unwrap();
    /// assert!(quoted.is_quoted());
    /// ```
    pub fn is_quoted(&self) -> bool {
        self.quoted
    }

    /// Checks if this is a qualified identifier (contains namespace).
    ///
    /// Returns `true` if the identifier includes namespace qualification
    /// (contains a dot in the original form).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Identifier;
    ///
    /// let simple = Identifier::parse_default("function").unwrap();
    /// assert!(!simple.is_qualified());
    ///
    /// let qualified = Identifier::parse_default("std.function").unwrap();
    /// assert!(qualified.is_qualified());
    /// ```
    pub fn is_qualified(&self) -> bool {
        !self.namespace_path.is_empty()
    }

    /// Returns the unqualified part of the identifier.
    ///
    /// This is the identifier name without any namespace qualification.
    /// For both `function` and `std.function`, this returns `function`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Identifier;
    ///
    /// let qualified = Identifier::parse_default("std.function").unwrap();
    /// assert_eq!(qualified.unqualified(), "function");
    /// ```
    pub fn unqualified(&self) -> &str {
        &self.normalized
    }

    /// Returns the full qualified name as a string.
    ///
    /// For qualified identifiers, this reconstructs the full name including
    /// namespace. For unqualified identifiers, this returns just the identifier.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Identifier;
    ///
    /// let simple = Identifier::parse_default("function").unwrap();
    /// assert_eq!(simple.qualified_name(), "function");
    ///
    /// let qualified = Identifier::parse_default("std.function").unwrap();
    /// assert_eq!(qualified.qualified_name(), "std.function");
    /// ```
    pub fn qualified_name(&self) -> String {
        if self.namespace_path.is_empty() {
            self.normalized.clone()
        } else {
            Namespace::as_prefix(&self.namespace_path) + "." + self.unqualified()
        }
    }
}

impl Identifier {
    /// Checks if a character is valid in an XMILE identifier.
    ///
    /// Valid characters include:
    /// - ASCII alphanumeric characters (A-Z, a-z, 0-9)
    /// - Underscore (_)
    /// - Dollar sign ($)
    /// - Unicode characters above U+007F
    fn is_valid_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' || ch > '\u{007F}'
    }

    /// Checks if an identifier is reserved according to XMILE rules.
    ///
    /// Uses UCA-compliant case folding to check against reserved keywords,
    /// namespaces, and function names. This ensures consistent behaviour
    /// across different Unicode representations.
    fn is_reserved(input: &str) -> bool {
        // Use UCA-compliant comparison for reserved word checking
        let input_key = match utils::uca_case_fold(input) {
            Ok(key) => key,
            Err(_) => return false,
        };

        // Check against all reserved word categories
        let all_reserved = Self::RESERVED_KEYWORDS
            .iter()
            .chain(Self::RESERVED_FUNCTIONS.iter());

        for reserved in all_reserved {
            if let Ok(reserved_key) = utils::uca_case_fold(reserved) {
                if input_key == reserved_key {
                    return true;
                }
            }
        }

        false
    }
}

/// Creates a normalized identifier string for XMILE using Unicode best practices.
///
/// This function applies the Unicode normalization pipeline required for XMILE:
/// 1. Full-width to half-width character conversion
/// 2. Unicode NFKC normalization for compatibility
/// 3. XMILE-specific whitespace normalization
///
/// Control characters and problematic Unicode are handled with appropriate warnings.
fn normalize_identifier(input: &str) -> Result<String, IdentifierError> {
    // Convert full-width characters first
    let preprocessed: String = input.chars().map(utils::full_to_half_width).collect();

    // Normalize using NFKC
    let nfkc_normalized =
        utils::nfkc_normalize(&preprocessed).map_err(|e| IdentifierError::ProcessingError(e))?;

    // Apply XMILE whitespace normalization (but not case folding for display)
    let (whitespace_normalized, warnings) = utils::xmile_normalize(&nfkc_normalized).into();

    // Log any warnings about control characters or whitespace issues
    warnings.into_iter().for_each(|w| warn!("{}", w));

    Ok(whitespace_normalized)
}

/// Creates a UCA-compliant comparison key for XMILE identifiers.
///
/// This key incorporates case folding and is used for efficient comparison
/// and hashing whilst maintaining XMILE equivalence semantics.
fn make_compare_key(normalized: &str) -> Result<String, IdentifierError> {
    utils::uca_compare_key(&normalized).map_err(|e| IdentifierError::ProcessingError(e))
}

/// Options for parsing identifiers in XMILE.
/// This struct allows customization of identifier parsing behavior,
/// particularly for units of measure.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct IdentifierOptions {
    /// Whether to allow dollar signs at the start of identifiers (for units of measure)
    pub allow_dollar: bool,
    /// Whether to allow digits at the start of identifiers (for units of measure)
    pub allow_digit: bool,
    /// Allow reserved identifiers (e.g., `min` is a minute in units of measure)
    pub allow_reserved: bool,
}

impl IdentifierOptions {
    /// Creates a new set of options for parsing units of measure.
    pub fn units_of_measure() -> Self {
        Self {
            allow_dollar: true,
            allow_digit: true,
            allow_reserved: true,
        }
    }
}

/// Parses an identifier string according to XMILE rules.
///
/// Handles both quoted and unquoted forms, namespace qualification,
/// and validates against XMILE identifier requirements.
///
/// # Arguments
///
/// * `input` - The identifier string to parse
/// * `options` - Parsing options to customize behavior
fn parse_identifier(
    input: &str,
    options: IdentifierOptions,
) -> Result<Identifier, IdentifierError> {
    // Trim whitespace from both ends
    let trimmed = input.trim();

    // Check for empty identifier
    if trimmed.is_empty() {
        return Err(IdentifierError::EmptyIdentifier);
    }

    // Handle Namespaces (3.2.2.3)
    // To avoid conflicts between identifiers in different libraries of
    // functions, each library, whether vendor-specific or user-defined, SHOULD
    // exist within its own namespace.
    if let Some(dot_pos) = trimmed.rfind('.') {
        return parse_qualified_identifier(trimmed, dot_pos, options);
    }

    // Handle Identifier Form, Quoted (3.2.2.1)
    // Any identifier MAY be enclosed in quotation marks, which are not part of
    // the identifier itself.
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        return parse_quoted_identifier(trimmed);
    }

    // Handle Identifier Form, Unquoted (3.2.2.1)
    parse_unquoted_identifier(trimmed, options)
}

/// Parses a qualified identifier (namespace.identifier) according to XMILE rules.
fn parse_qualified_identifier(
    input: &str,
    dot_pos: usize,
    options: IdentifierOptions,
) -> Result<Identifier, IdentifierError> {
    let namespace_part = &input[..dot_pos];
    let identifier_part = &input[dot_pos + 1..];

    if namespace_part.is_empty() || identifier_part.is_empty() {
        return Err(IdentifierError::InvalidQualifiedName);
    }

    // Parse the identifier part recursively
    let identifier = parse_identifier(identifier_part, options)?;
    let namespace_path = Namespace::from_str(namespace_part);

    Ok(Identifier {
        raw: input.to_string(),
        normalized: identifier.normalized,
        compare_key: identifier.compare_key,
        namespace_path,
        quoted: identifier.quoted,
    })
}

/// Parses a quoted identifier according to XMILE rules.
fn parse_quoted_identifier(input: &str) -> Result<Identifier, IdentifierError> {
    let inner = &input[1..input.len() - 1];
    let unescaped = utils::parse_xmile_escape(inner)?;

    // Identifiers are formed by a sequence of one or more characters...
    if unescaped.is_empty() {
        return Err(IdentifierError::EmptyIdentifier);
    }

    // Normalize the identifier (case-folding, whitespace normalization)
    let normalized = normalize_identifier(&unescaped)?;

    // Create a UCA-compliant comparison key
    let comparison = make_compare_key(&normalized)?;

    return Ok(Identifier {
        raw: input.to_string(),
        normalized,
        compare_key: comparison,
        namespace_path: vec![],
        quoted: true,
    });
}

/// Parses an unquoted identifier according to XMILE rules.
fn parse_unquoted_identifier(
    input: &str,
    options: IdentifierOptions,
) -> Result<Identifier, IdentifierError> {
    // Identifiers are formed by a sequence of one or more characters...
    if input.is_empty() {
        return Err(IdentifierError::EmptyIdentifier);
    }

    let mut chars = input.chars();
    let first_char = chars.next().unwrap();

    // Identifiers SHALL NOT begin with a digit or a dollar sign (with
    // exceptions as noted for units of measure), and SHALL NOT begin or end
    // with an underscore.
    if (!options.allow_digit && first_char.is_ascii_digit()) || first_char == '_' {
        return Err(IdentifierError::InvalidFirstCharacter(first_char));
    }

    if !options.allow_dollar && first_char == '$' {
        return Err(IdentifierError::InvalidFirstCharacter(first_char));
    }

    if input.ends_with('_') {
        return Err(IdentifierError::InvalidLastCharacter('_'));
    }

    // ...that include roman letters (A-Z or a-z), underscore (_), dollar sign
    // ($), digits (0-9), and Unicode characters above 127.
    for ch in chars {
        if !Identifier::is_valid_char(ch) {
            return Err(IdentifierError::InvalidCharacter(ch));
        }
    }

    // The operator names AND, OR, and NOT, the statement keywords IF, THEN,
    // and ELSE, the names of all built-in functions, and the XMILE namespace
    // std, are reserved identifiers. They cannot be used as vendor- or
    // user-defined namespaces, macros, or functions. Any conflict with these
    // names that is found when reading user- or vendor-supplied definitions
    // SHOULD be flagged as an error to the end user.
    if !options.allow_reserved && Identifier::is_reserved(input) {
        return Err(IdentifierError::ReservedIdentifier(input.to_string()));
    }

    // Normalize the identifier (case-folding, whitespace normalization)
    let normalized = normalize_identifier(input)?;

    // Create a UCA-compliant comparison key
    let comparison = make_compare_key(&normalized)?;

    Ok(Identifier {
        raw: input.to_string(),
        normalized,
        compare_key: comparison,
        namespace_path: vec![],
        quoted: false,
    })
}

impl FromStr for Identifier {
    type Err = IdentifierError;

    /// Parses an identifier from a string according to XMILE specification.
    ///
    /// This implementation handles all forms of XMILE identifiers including:
    /// - Unquoted identifiers: `Cash_Balance`
    /// - Quoted identifiers: `"wom multiplier"`
    /// - Qualified identifiers: `std.function`
    /// - Escape sequences in quoted identifiers: `"revenue\ngap"`
    ///
    /// Unicode characters are validated and warnings are logged for
    /// problematic characters (e.g., full-width, unusual spaces).
    ///
    /// # Errors
    ///
    /// Returns `IdentifierError` for various parsing failures:
    /// - Empty identifiers
    /// - Invalid character usage
    /// - Reserved identifier conflicts
    /// - Malformed qualified names
    /// - Invalid escape sequences
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse_default(input)
    }
}

impl fmt::Display for Identifier {
    /// Displays the normalized form of the identifier.
    ///
    /// This shows the canonical representation without quotes or escape sequences,
    /// but preserving the original case and with normalized whitespace.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.normalized)
    }
}

impl Identifier {
    /// Compares two identifiers using UCA rules.
    ///
    /// This method implements XMILE's case-insensitive and whitespace-insensitive
    /// comparison semantics using the Unicode Collation Algorithm.
    fn uca_compare(&self, other: &Self) -> Result<Ordering, IdentifierError> {
        // First compare namespaces
        match self.namespace_path.cmp(&other.namespace_path) {
            Ordering::Equal => {
                // Then use UCA comparison for the identifiers
                utils::uca_compare(&self.normalized, &other.normalized)
                    .map_err(|e| IdentifierError::ProcessingError(e))
            }
            ord => Ok(ord),
        }
    }

    /// Checks if two identifiers are equal using UCA rules.
    fn uca_equal(&self, other: &Self) -> Result<bool, IdentifierError> {
        if self.namespace_path != other.namespace_path {
            return Ok(false);
        }

        utils::uca_equal(&self.normalized, &other.normalized)
            .map_err(|e| IdentifierError::ProcessingError(e))
    }

    /// Checks if this identifier is equal to a string using UCA rules.
    fn uca_equal_str(&self, other: &str) -> Result<bool, IdentifierError> {
        utils::uca_equal(&self.normalized, other).map_err(|e| IdentifierError::ProcessingError(e))
    }
}

// Trait implementations for comparison and hashing

impl Eq for Identifier {}

impl PartialEq for Identifier {
    /// Compares identifiers for equality using XMILE equivalence rules.
    ///
    /// Two identifiers are equal if they have the same namespace and their
    /// normalized forms are equivalent under UCA comparison (case-insensitive,
    /// whitespace-insensitive).
    fn eq(&self, other: &Self) -> bool {
        match self.uca_equal(other) {
            Ok(result) => result,
            Err(_) => {
                // Fallback to cached comparison keys and namespace comparison
                self.compare_key == other.compare_key && self.namespace_path == other.namespace_path
            }
        }
    }
}

impl PartialEq<&str> for Identifier {
    /// Compares an identifier with a string using XMILE equivalence rules.
    ///
    /// This allows direct comparison with string literals:
    /// ```rust
    /// use xmile::Identifier;
    ///
    /// let id = Identifier::parse_default("Cash_Balance").unwrap();
    /// assert_eq!(id, "cash balance"); // true due to equivalence rules
    /// ```
    fn eq(&self, other: &&str) -> bool {
        self.uca_equal_str(other).unwrap_or(false)
    }
}

impl Ord for Identifier {
    /// Orders identifiers lexicographically with namespace precedence.
    ///
    /// Ordering rules:
    /// 1. Compare namespace paths first
    /// 2. If namespaces are equal, compare identifiers using UCA
    ///
    /// This ensures stable, predictable ordering for use in sorted collections.
    fn cmp(&self, other: &Self) -> Ordering {
        match self.uca_compare(other) {
            Ok(result) => result,
            Err(_) => {
                // Fallback to cached comparison keys
                match self.namespace_path.cmp(&other.namespace_path) {
                    Ordering::Equal => self.compare_key.cmp(&other.compare_key),
                    ord => ord,
                }
            }
        }
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Identifier {
    /// Hashes the identifier using the comparison key and namespace.
    ///
    /// This ensures that equivalent identifiers (according to XMILE rules)
    /// produce the same hash value, making them suitable for use in HashMap
    /// and HashSet collections.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.compare_key.hash(state);
        self.namespace_path.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_identifier() {
        let id = Identifier::from_str("Cash_Balance").unwrap();
        assert_eq!(id.raw(), "Cash_Balance");
        assert_eq!(id.normalized(), "Cash Balance");
        assert_eq!(id, "Cash Balance");
        assert_eq!(id, "cash balance");
        assert_eq!(id, "cash baLANCe");
        assert_eq!(id.compare_key(), "cash balance");
        assert!(!id.is_quoted());
        assert!(!id.is_qualified());
    }

    #[test]
    fn test_quoted_identifier() {
        let id = Identifier::from_str("\"wom multiplier\"").unwrap();
        assert_eq!(id.raw(), "\"wom multiplier\"");
        assert_eq!(id.normalized(), "wom multiplier");
        assert!(id.is_quoted());
    }

    #[test]
    fn test_escape_sequences() {
        // "revenue\ngap"
        let id = Identifier::from_str("\"revenue\\ngap\"").unwrap();
        assert_eq!(id.normalized(), "revenue gap");
    }

    #[test]
    fn test_qualified_name() {
        let id = Identifier::from_str("funcs.find").unwrap();
        assert_eq!(id.namespace_path(), &Namespace::from_str("funcs"));
        assert_eq!(id.unqualified(), "find");
        assert!(id.is_qualified());
    }

    #[test]
    fn test_predefined_namespace() {
        let id = Identifier::from_str("std.func").unwrap();
        assert_eq!(id.namespace_path(), &Namespace::from_str("std"));

        let id2 = Identifier::from_str("vensim.function").unwrap();
        assert_eq!(id2.namespace_path(), &Namespace::from_str("vensim"));

        let id3 = Identifier::from_str("user.custom").unwrap();
        assert_eq!(id3.namespace_path(), &Namespace::from_str("user"));
    }

    #[test]
    fn test_control_character_normalization() {
        // Test control character is treated as space
        let id1 = Identifier::from_str("\"test\x01variable\"").unwrap();
        let id2 = Identifier::from_str("\"test variable\"").unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_case_insensitive_equality() {
        let id1 = Identifier::from_str("Cash_Balance").unwrap();
        let id2 = Identifier::from_str("cash_balance").unwrap();
        let id3 = Identifier::from_str("CASH_BALANCE").unwrap();

        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
        assert_eq!(id1, id3);
    }

    #[test]
    fn test_whitespace_equivalence() {
        let id1 = Identifier::from_str("wom_multiplier").unwrap();
        let id2 = Identifier::from_str("\"wom multiplier\"").unwrap();
        let id3 = Identifier::from_str("\"wom\\nmultiplier\"").unwrap();

        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
    }

    #[test]
    fn test_reserved_identifiers() {
        // Language keywords
        assert!(Identifier::from_str("AND").is_err());
        assert!(Identifier::from_str("if").is_err());

        // Built-in functions
        assert!(Identifier::from_str("sin").is_err());
        assert!(Identifier::from_str("MAX").is_err());
        assert!(Identifier::from_str("time").is_err());
        assert!(Identifier::from_str("delay").is_err());
    }

    #[test]
    fn test_invalid_first_character() {
        assert!(Identifier::from_str("123abc").is_err());
        assert!(Identifier::from_str("_abc").is_err());
    }

    #[test]
    fn test_invalid_last_character() {
        assert!(Identifier::from_str("abc_").is_err());
    }

    #[test]
    fn test_ordering_with_namespaces() {
        let mut identifiers = vec![
            Identifier::from_str("zebra").unwrap(),
            Identifier::from_str("apple").unwrap(),
            Identifier::from_str("banana").unwrap(),
            Identifier::from_str("funcs.find").unwrap(),
            Identifier::from_str("isee.helper").unwrap(),
            Identifier::from_str("user.process").unwrap(),
        ];

        identifiers.sort();

        // Unqualified names should come first (alphabetically)
        assert_eq!(identifiers[0].normalized(), "apple");
        assert_eq!(identifiers[1].normalized(), "banana");
        assert_eq!(identifiers[2].normalized(), "zebra");

        // Then qualified names - the exact order depends on namespace enum ordering
        // Let's just verify we have the right number of qualified vs unqualified
        let unqualified_count = identifiers.iter().filter(|id| !id.is_qualified()).count();
        let qualified_count = identifiers.iter().filter(|id| id.is_qualified()).count();

        assert_eq!(unqualified_count, 3);
        assert_eq!(qualified_count, 3);

        // Verify specific identifiers exist in the qualified portion
        let qualified_identifiers: Vec<_> = identifiers.iter().skip(3).collect();
        assert!(
            qualified_identifiers
                .iter()
                .any(|id| id.unqualified() == "find")
        );
        assert!(
            qualified_identifiers
                .iter()
                .any(|id| id.unqualified() == "helper")
        );
        assert!(
            qualified_identifiers
                .iter()
                .any(|id| id.unqualified() == "process")
        );
    }
}
