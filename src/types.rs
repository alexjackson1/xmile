use std::fmt::Debug;

/// A result type that can contain warnings alongside the successful result.
///
/// This enum allows functions to return successful results while still
/// providing diagnostic information about potential issues encountered
/// during processing (e.g., problematic Unicode characters, control
/// characters treated as spaces).
///
/// # Type Parameters
///
/// * `T` - The success result type
/// * `W` - The warning type (typically `String` for warning messages)
///
/// # Examples
///
/// ```rust
/// use xmile::types::WithWarnings;
///
/// let result = WithWarnings::Warning("processed".to_string(), vec!["Found control character".to_string()]);
/// assert!(result.is_warning());
/// assert_eq!(result.clone().unwrap(), "processed");
///
/// let warnings = result.warnings();
/// assert_eq!(warnings.len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum WithWarnings<T, W> {
    /// Successful result without warnings
    Ok(T),
    /// Successful result with warnings
    Warning(T, Vec<W>),
}

impl<T, W> WithWarnings<T, W> {
    /// Checks if the result is successful without warnings.
    pub fn is_ok(&self) -> bool {
        matches!(self, WithWarnings::Ok(_))
    }

    /// Checks if the result has warnings.
    pub fn is_warning(&self) -> bool {
        matches!(self, WithWarnings::Warning(_, _))
    }

    /// Extracts the result value, discarding any warnings.
    ///
    /// This consumes the `WithWarnings` and returns the contained value,
    /// regardless of whether there were warnings.
    pub fn unwrap(self) -> T {
        match self {
            WithWarnings::Ok(data) => data,
            WithWarnings::Warning(data, _) => data,
        }
    }

    /// Extracts the warnings, discarding the result value.
    ///
    /// Returns an empty vector if there were no warnings.
    pub fn warnings(self) -> Vec<W> {
        match self {
            WithWarnings::Ok(_) => Vec::new(),
            WithWarnings::Warning(_, warnings) => warnings,
        }
    }
}

impl<T, W> From<WithWarnings<T, W>> for (T, Vec<W>) {
    /// Converts `WithWarnings` into a tuple of (result, warnings).
    ///
    /// This provides a convenient way to destructure the result and
    /// warnings simultaneously.
    fn from(value: WithWarnings<T, W>) -> Self {
        match value {
            WithWarnings::Ok(data) => (data, Vec::new()),
            WithWarnings::Warning(data, warnings) => (data, warnings),
        }
    }
}

/// A validation result type that can represent valid, warning, or invalid states.
///
/// This enum is used to encapsulate the outcome of a validation process,
/// allowing for successful validations with or without warnings, as well as
/// failed validations with error messages.
///
/// # Type Parameters
/// * `T` - The type of the valid result
/// * `W` - The type of warnings (default is `String`)
/// * `E` - The type of error messages (default is `String`)
pub enum ValidationResult<T = (), W = String, E: Debug = String> {
    /// Represents a successful validation with no warnings.
    Valid(T),
    /// Represents a successful validation with warnings.
    Warnings(T, Vec<W>),
    /// Represents a failed validation with an error message.
    Invalid(Vec<W>, Vec<E>),
}

impl<T, W, E: Debug> ValidationResult<T, W, E> {
    /// Checks if the validation result is valid (no warnings or errors).
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid(_))
    }

    /// Checks if the validation result has warnings.
    pub fn has_warnings(&self) -> bool {
        matches!(self, ValidationResult::Warnings(_, _))
    }

    /// Checks if the validation result is invalid (contains errors).
    pub fn is_invalid(&self) -> bool {
        matches!(self, ValidationResult::Invalid(_, _))
    }

    /// Extracts the valid data, discarding any warnings or errors.
    ///
    /// This consumes the `ValidationResult` and returns the contained value,
    /// regardless of whether there were warnings or errors.
    ///
    /// # Arguments
    /// * `msg` - A message to include in the panic if the result is invalid
    ///
    /// # Panics
    /// If the result is `Invalid`, this method will panic with the error messages.
    pub fn expect_valid<S>(self, msg: S) -> T
    where
        S: Into<String>,
    {
        match self {
            ValidationResult::Valid(data) => data,
            ValidationResult::Warnings(data, _) => data,
            ValidationResult::Invalid(_, errors) => {
                panic!("{}: {:?}", msg.into(), errors)
            }
        }
    }

    pub fn ok(self) -> Result<T, Vec<E>> {
        match self {
            ValidationResult::Valid(data) => Ok(data),
            ValidationResult::Warnings(data, _) => Ok(data),
            ValidationResult::Invalid(_, errors) => Err(errors),
        }
    }
}

pub trait Validate<T = (), W = String, E: Debug = String> {
    fn validate(&self) -> ValidationResult<T, W, E>;
}
