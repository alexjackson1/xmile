//! Comprehensive error types for XMILE parsing and validation.

use std::fmt;
use std::path::PathBuf;

use thiserror::Error;

/// A comprehensive error type for XMILE file parsing and validation.
///
/// This enum provides detailed error information including context like
/// file locations, line numbers, and specific failure reasons.
#[derive(Debug, Error)]
pub enum XmileError {
    /// IO error occurred while reading the file.
    #[error("IO error reading file: {0}")]
    Io(#[from] std::io::Error),

    /// XML parsing error (malformed XML structure).
    #[error("XML parsing error{context}: {message}")]
    Xml {
        message: String,
        context: ErrorContext,
    },

    /// Deserialization error (XML structure doesn't match expected format).
    #[error("Deserialization error{context}: {message}")]
    Deserialize {
        message: String,
        context: ErrorContext,
    },

    /// Validation error (file structure is valid but violates XMILE rules).
    #[error("Validation error{context}: {message}")]
    Validation {
        message: String,
        context: ErrorContext,
        warnings: Vec<String>,
        errors: Vec<String>,
    },

    /// Multiple errors occurred during parsing or validation.
    #[error("Multiple errors occurred:\n{}", format_errors(.0))]
    Multiple(Vec<XmileError>),
}

fn format_errors(errors: &[XmileError]) -> String {
    errors
        .iter()
        .enumerate()
        .map(|(idx, error)| format!("  {}. {}", idx + 1, error))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Context information for error reporting.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorContext {
    /// The file path where the error occurred (if available).
    pub file_path: Option<PathBuf>,
    /// The line number where the error occurred (if available).
    pub line: Option<usize>,
    /// The column number where the error occurred (if available).
    pub column: Option<usize>,
    /// Additional context about what was being parsed.
    pub parsing: Option<String>,
}

impl ErrorContext {
    /// Create a new empty error context.
    pub fn new() -> Self {
        Self {
            file_path: None,
            line: None,
            column: None,
            parsing: None,
        }
    }

    /// Create an error context with file path.
    pub fn with_file_path<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            file_path: Some(path.into()),
            line: None,
            column: None,
            parsing: None,
        }
    }

    /// Create an error context with line number.
    pub fn with_line(line: usize) -> Self {
        Self {
            file_path: None,
            line: Some(line),
            column: None,
            parsing: None,
        }
    }

    /// Create an error context with file path and line number.
    pub fn with_file_and_line<P: Into<PathBuf>>(path: P, line: usize) -> Self {
        Self {
            file_path: Some(path.into()),
            line: Some(line),
            column: None,
            parsing: None,
        }
    }

    /// Add parsing context information.
    pub fn with_parsing<S: Into<String>>(mut self, parsing: S) -> Self {
        self.parsing = Some(parsing.into());
        self
    }

    /// Add column information.
    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        if let Some(ref path) = self.file_path {
            parts.push(format!(" in file '{}'", path.display()));
        }

        if let Some(line) = self.line {
            if let Some(column) = self.column {
                parts.push(format!(" at line {}, column {}", line, column));
            } else {
                parts.push(format!(" at line {}", line));
            }
        }

        if let Some(ref parsing) = self.parsing {
            parts.push(format!(" while parsing {}", parsing));
        }

        if parts.is_empty() {
            return Ok(());
        }

        write!(f, "{}", parts.join(","))
    }
}

/// A collection of errors that can be aggregated.
#[derive(Debug)]
pub struct ErrorCollection {
    errors: Vec<XmileError>,
}

impl ErrorCollection {
    /// Create a new empty error collection.
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    /// Add an error to the collection.
    pub fn push(&mut self, error: XmileError) {
        self.errors.push(error);
    }

    /// Check if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the number of errors in the collection.
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Convert the collection into a single `XmileError::Multiple` error.
    pub fn into_error(self) -> Option<XmileError> {
        if self.errors.is_empty() {
            None
        } else if self.errors.len() == 1 {
            Some(self.errors.into_iter().next().unwrap())
        } else {
            Some(            XmileError::Multiple(self.errors))
        }
    }

    /// Get all errors as a slice.
    pub fn errors(&self) -> &[XmileError] {
        &self.errors
    }
}

impl Default for ErrorCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<XmileError>> for ErrorCollection {
    fn from(errors: Vec<XmileError>) -> Self {
        Self { errors }
    }
}

impl From<ErrorCollection> for XmileError {
    fn from(collection: ErrorCollection) -> Self {
        collection.into_error().unwrap_or_else(|| {
            XmileError::Validation {
                message: "Unknown error".to_string(),
                context: ErrorContext::new(),
                warnings: Vec::new(),
                errors: Vec::new(),
            }
        })
    }
}

/// Helper trait for converting validation results to XmileError.
pub trait ToXmileError {
    fn to_xmile_error(self, context: ErrorContext) -> XmileError;
}

impl ToXmileError for crate::types::ValidationResult {
    fn to_xmile_error(self, context: ErrorContext) -> XmileError {
        match self {
            crate::types::ValidationResult::Valid(_) => {
                XmileError::Validation {
                    message: "Validation passed".to_string(),
                    context,
                    warnings: Vec::new(),
                    errors: Vec::new(),
                }
            }
            crate::types::ValidationResult::Warnings(_, warnings) => {
                XmileError::Validation {
                    message: format!("Validation passed with {} warning(s)", warnings.len()),
                    context,
                    warnings,
                    errors: Vec::new(),
                }
            }
            crate::types::ValidationResult::Invalid(warnings, errors) => {
                let error_count = errors.len();
                let message = if error_count == 1 {
                    errors[0].clone()
                } else {
                    format!("{} validation errors", error_count)
                };
                XmileError::Validation {
                    message,
                    context,
                    warnings,
                    errors,
                }
            }
        }
    }
}
