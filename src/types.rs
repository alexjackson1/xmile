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
