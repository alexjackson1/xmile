//! # XMILE Graphical Functions
//!
//! Implementation of XMILE graphical functions (lookup tables) from specification
//! section 3.1.4. Provides arbitrary relationships between input and output variables
//! with interpolation support.
//!
//! ## Quick Start
//!
//! ```rust
//! use xmile::{GraphicalFunction, GraphicalFunctionData, GraphicalFunctionType};
//!
//! // Uniform scale (most common)
//! let data = GraphicalFunctionData::uniform_scale(
//!     (0.0, 1.0),
//!     vec![0.0, 0.5, 0.8, 0.95, 1.0],
//!     Some((0.0, 1.0)), // Optional y-scale
//! );
//!
//! let function = GraphicalFunction {
//!     name: None, // Anonymous/embedded
//!     data,
//!     function_type: Some(GraphicalFunctionType::Continuous),
//! };
//! ```
//!
//! ## Data Representation
//!
//! - **Uniform Scale**: Evenly spaced x-values with explicit y-values
//! - **X-Y Pairs**: Explicit coordinate pairs for irregular spacing
//!
//! ## Interpolation Types
//!
//! - **Continuous**: Linear interpolation, clamped at endpoints
//! - **Extrapolate**: Linear interpolation with extrapolation beyond range  
//! - **Discrete**: Step function with discrete jumps

pub mod data;
pub mod interpolation;
pub mod validation;
pub mod xml;

use std::{
    fmt,
    ops::{Deref, DerefMut, Index, IndexMut},
    str::FromStr,
};

use crate::{
    Identifier,
    containers::{Container, ContainerMut},
    types::{Validate, ValidationResult},
};

pub use data::GraphicalFunctionData;
use thiserror::Error;

/// XMILE graphical function with metadata and interpolation behaviour.
///
/// Represents a lookup function defining relationships between input (x) and
/// output (y) variables. Can be standalone (named) or embedded in other variables.
///
/// # Examples
///
/// ```rust
/// use xmile::{GraphicalFunction, GraphicalFunctionData, GraphicalFunctionType, Identifier};
///
/// // Standalone named function
/// let named_function = GraphicalFunction {
///     name: Some(Identifier::parse_default("growth_rate").unwrap()),
///     data: GraphicalFunctionData::uniform_scale((0.0, 1.0), vec![0.0, 0.8, 1.0], None),
///     function_type: Some(GraphicalFunctionType::Continuous),
/// };
///
/// // Anonymous embedded function  
/// let embedded_function = GraphicalFunction {
///     name: None,
///     data: GraphicalFunctionData::uniform_scale((0.0, 1.0), vec![0.0, 0.8, 1.0], None),
///     function_type: None, // Defaults to Continuous
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GraphicalFunction {
    /// The function identifier (may be empty for embedded functions)
    pub name: Option<Identifier>,

    /// Interpolation and extrapolation behaviour (defaults to continuous if None)
    pub function_type: Option<GraphicalFunctionType>,

    /// The x-y relationship data
    pub data: GraphicalFunctionData,
}

impl GraphicalFunction {
    /// Returns true if this function has no name (embedded function).
    ///
    /// # Returns
    /// True if the function is anonymous (no name).
    pub fn is_anonymous(&self) -> bool {
        self.name.is_none()
    }

    /// Returns the function type, defaulting to Continuous if not specified.
    ///
    /// # Returns
    /// The function type as `GraphicalFunctionType`.
    pub fn function_type(&self) -> GraphicalFunctionType {
        self.function_type.clone().unwrap_or_default()
    }

    /// Evaluates the function at a given x-value.
    ///
    /// # Arguments
    /// - `x`: The input value to evaluate the function at.
    ///
    /// # Returns
    /// The evaluated y-value based on the function's data and type.
    ///
    /// # Note
    /// This method handles different function types:
    /// - Continuous: Linear interpolation with clamping at endpoints.
    /// - Extrapolate: Linear interpolation with extrapolation beyond endpoints.
    /// - Discrete: Step-wise function with discrete jumps.
    pub fn evaluate(&self, x: f64) -> f64 {
        match self.function_type() {
            GraphicalFunctionType::Continuous => self.data.evaluate_continuous(x),
            GraphicalFunctionType::Extrapolate => self.data.evaluate_extrapolate(x),
            GraphicalFunctionType::Discrete => self.data.evaluate_discrete(x),
        }
    }
}

impl Validate for GraphicalFunction {
    /// Validates the graphical function.
    ///
    /// # Returns
    /// - `Valid(())` if the function is valid.
    /// - `Invalid(warnings, errors)` if there are validation issues.
    fn validate(&self) -> ValidationResult {
        validation::validate(self)
    }
}

impl Container for GraphicalFunction {
    /// Returns the y-values as a slice for container operations.
    fn values(&self) -> &[f64] {
        match &self.data {
            GraphicalFunctionData::UniformScale { y_values, .. } => y_values,
            GraphicalFunctionData::XYPairs { y_values, .. } => y_values,
        }
    }
}

impl ContainerMut for GraphicalFunction {
    /// Returns mutable access to y-values.
    fn values_mut(&mut self) -> &mut [f64] {
        match &mut self.data {
            GraphicalFunctionData::UniformScale { y_values, .. } => y_values,
            GraphicalFunctionData::XYPairs { y_values, .. } => y_values,
        }
    }
}

impl Index<usize> for GraphicalFunction {
    type Output = f64;

    /// Provides direct access to y-values by index.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    fn index(&self, index: usize) -> &Self::Output {
        match &self.data {
            GraphicalFunctionData::UniformScale { y_values, .. } => &y_values[index],
            GraphicalFunctionData::XYPairs { y_values, .. } => &y_values[index],
        }
    }
}

impl IndexMut<usize> for GraphicalFunction {
    /// Provides mutable access to y-values by index.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match &mut self.data {
            GraphicalFunctionData::UniformScale { y_values, .. } => &mut y_values[index],
            GraphicalFunctionData::XYPairs { y_values, .. } => &mut y_values[index],
        }
    }
}

/// Interpolation and extrapolation behaviour for graphical functions.
///
/// Defines how intermediate values and out-of-range values are calculated
/// according to XMILE specification section 3.1.4.
#[derive(Debug, Clone, PartialEq)]
pub enum GraphicalFunctionType {
    /// Linear interpolation with clamping at endpoints.
    Continuous,
    /// Linear interpolation with linear extrapolation beyond endpoints.
    Extrapolate,
    /// Step-wise function with discrete jumps.
    ///
    /// # Note
    /// Last two points must have same y-value.
    Discrete,
}

impl Default for GraphicalFunctionType {
    /// Returns the default interpolation type (Continuous).
    fn default() -> Self {
        GraphicalFunctionType::Continuous
    }
}

impl fmt::Display for GraphicalFunctionType {
    /// Formats the function type for display and serialisation.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphicalFunctionType::Continuous => write!(f, "continuous"),
            GraphicalFunctionType::Extrapolate => write!(f, "extrapolate"),
            GraphicalFunctionType::Discrete => write!(f, "discrete"),
        }
    }
}

#[derive(Debug, Error)]
pub enum GraphicalFunctionTypeParseError {
    /// Error when parsing an invalid function type string.
    #[error("Invalid GraphicalFunctionType: {0}")]
    InvalidValue(String),
}

impl FromStr for GraphicalFunctionType {
    type Err = GraphicalFunctionTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "continuous" => Ok(GraphicalFunctionType::Continuous),
            "extrapolate" => Ok(GraphicalFunctionType::Extrapolate),
            "discrete" => Ok(GraphicalFunctionType::Discrete),
            _ => Err(GraphicalFunctionTypeParseError::InvalidValue(s.to_string())),
        }
    }
}

/// Range for scaling graphical functions.
///
/// Defines the minimum and maximum values for scaling y-values in graphical functions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphicalFunctionScale {
    /// The minimum value for scaling.
    pub min: f64,
    /// The maximum value for scaling.
    pub max: f64,
}

impl GraphicalFunctionScale {
    /// Creates a new Scale instance with the given min and max values.
    ///
    /// # Arguments
    /// - `min`: The minimum value for the scale.
    /// - `max`: The maximum value for the scale.
    pub fn new(min: f64, max: f64) -> Self {
        GraphicalFunctionScale { min, max }
    }

    /// Returns the difference between max and min values.
    ///
    /// # Returns
    /// The delta (max - min) of the scale.
    pub fn delta(&self) -> f64 {
        self.max - self.min
    }
}

impl From<(f64, f64)> for GraphicalFunctionScale {
    /// Converts a tuple (min, max) into a Scale instance.
    fn from(range: (f64, f64)) -> Self {
        GraphicalFunctionScale {
            min: range.0,
            max: range.1,
        }
    }
}

impl Validate for GraphicalFunctionScale {
    /// Validates the scale range.
    ///
    /// # Returns
    /// - `Valid(())` if the range is valid.
    /// - `Invalid(warnings, errors)` if there are validation issues.
    fn validate(&self) -> ValidationResult {
        validation::validate_scale(self)
    }
}

/// X-axis or y-axis points for graphical functions.
///
/// Represents points used in `<xpts>` and `<ypts>` XML tags with optional
/// separator specification.
#[derive(Debug, Clone, PartialEq)]
pub struct Points {
    pub values: Vec<f64>,
    pub separator: Option<String>,
}

impl Points {
    /// Creates a new Points instance with the given values and optional separator.
    ///
    /// # Arguments
    /// - `values`: Vector of f64 values representing points.
    /// - `separator`: Optional string separator for parsing (default is None).
    pub fn new(values: Vec<f64>, separator: Option<String>) -> Self {
        Points { values, separator }
    }

    /// Returns the separator used for parsing points, if any.
    ///
    /// # Returns
    /// An `Option<&str>` containing the separator string, or None if not specified.
    pub fn separator(&self) -> Option<&str> {
        self.separator.as_deref()
    }
}

impl Deref for Points {
    type Target = [f64];

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl DerefMut for Points {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

impl Index<usize> for Points {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl IndexMut<usize> for Points {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.values[index]
    }
}

impl From<Vec<f64>> for Points {
    /// Converts a vector of f64 into Points with default separator.
    fn from(values: Vec<f64>) -> Self {
        Points {
            values,
            separator: None,
        }
    }
}

impl Validate for Points {
    fn validate(&self) -> ValidationResult {
        validation::validate_points(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ValidationResult;

    mod graphical_function_tests {
        use super::*;

        #[test]
        fn test_uniform_scale_creation() {
            let gf = GraphicalFunction {
                name: Some(Identifier::parse_default("test_function").unwrap()),
                data: GraphicalFunctionData::uniform_scale(
                    (0.0, 1.0),
                    vec![0.0, 0.3, 0.55, 0.7, 0.83, 0.9, 0.95, 0.98, 0.99, 0.995, 1.0],
                    None,
                ),
                function_type: Some(GraphicalFunctionType::Continuous),
            };

            assert!(!gf.is_anonymous());
            assert_eq!(gf.function_type(), GraphicalFunctionType::Continuous);
        }

        #[test]
        fn test_anonymous_function() {
            let gf = GraphicalFunction {
                name: None,
                data: GraphicalFunctionData::uniform_scale((0.0, 1.0), vec![0.0, 0.5, 1.0], None),
                function_type: None,
            };

            assert!(gf.is_anonymous());
            assert_eq!(gf.function_type(), GraphicalFunctionType::Continuous); // Default
        }

        #[test]
        fn test_xy_pairs_creation() {
            let gf = GraphicalFunction {
                name: Some(Identifier::parse_default("xy_function").unwrap()),
                data: GraphicalFunctionData::xy_pairs(
                    vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5],
                    vec![0.05, 0.1, 0.2, 0.25, 0.3, 0.33],
                    Some((0.0, 1.0)),
                ),
                function_type: Some(GraphicalFunctionType::Extrapolate),
            };

            assert_eq!(gf.function_type(), GraphicalFunctionType::Extrapolate);
        }

        #[test]
        fn test_indexing() {
            let gf = GraphicalFunction {
                name: None,
                data: GraphicalFunctionData::uniform_scale((0.0, 1.0), vec![0.0, 0.5, 1.0], None),
                function_type: None,
            };

            assert_eq!(gf[0], 0.0);
            assert_eq!(gf[1], 0.5);
            assert_eq!(gf[2], 1.0);
        }

        #[test]
        fn test_mutable_indexing() {
            let mut gf = GraphicalFunction {
                name: None,
                data: GraphicalFunctionData::uniform_scale((0.0, 1.0), vec![0.0, 0.5, 1.0], None),
                function_type: None,
            };

            gf[1] = 0.7;
            assert_eq!(gf[1], 0.7);
        }
    }

    mod graphical_function_type_tests {
        use super::*;

        #[test]
        fn test_default_function_type() {
            assert_eq!(
                GraphicalFunctionType::default(),
                GraphicalFunctionType::Continuous
            );
        }

        #[test]
        fn test_function_type_display() {
            assert_eq!(
                format!("{}", GraphicalFunctionType::Continuous),
                "continuous"
            );
            assert_eq!(
                format!("{}", GraphicalFunctionType::Extrapolate),
                "extrapolate"
            );
            assert_eq!(format!("{}", GraphicalFunctionType::Discrete), "discrete");
        }
    }

    mod scale_tests {
        use super::*;

        #[test]
        fn test_scale_from_tuple() {
            let scale: GraphicalFunctionScale = (0.0, 1.0).into();
            assert_eq!(scale.min, 0.0);
            assert_eq!(scale.max, 1.0);
        }

        #[test]
        fn test_scale_validation() {
            let valid_scale = GraphicalFunctionScale { min: 0.0, max: 1.0 };
            assert!(matches!(valid_scale.validate(), ValidationResult::Valid(_)));

            let invalid_scale = GraphicalFunctionScale { min: 1.0, max: 0.0 };
            assert!(matches!(
                invalid_scale.validate(),
                ValidationResult::Invalid(_, _)
            ));

            let nan_scale = GraphicalFunctionScale {
                min: f64::NAN,
                max: 1.0,
            };
            assert!(matches!(
                nan_scale.validate(),
                ValidationResult::Invalid(_, _)
            ));

            let infinite_scale = GraphicalFunctionScale {
                min: 0.0,
                max: f64::INFINITY,
            };
            assert!(matches!(
                infinite_scale.validate(),
                ValidationResult::Invalid(_, _)
            ));
        }
    }

    mod points_tests {
        use super::*;

        #[test]
        fn test_points_creation() {
            let points = Points::new(vec![0.0, 0.5, 1.0], Some(";".to_string()));
            assert_eq!(points.len(), 3);
            assert_eq!(points.separator(), Some(";"));
        }

        #[test]
        fn test_points_from_vec() {
            let points: Points = vec![0.0, 0.5, 1.0].into();
            assert_eq!(points.len(), 3);
            assert_eq!(points.separator(), None);
        }

        #[test]
        fn test_points_deref() {
            let points = Points::new(vec![0.0, 0.5, 1.0], None);
            assert_eq!(points[0], 0.0);
            assert_eq!(points[1], 0.5);
            assert_eq!(points[2], 1.0);
        }

        #[test]
        fn test_points_deref_mut() {
            let mut points = Points::new(vec![0.0, 0.5, 1.0], None);
            points[1] = 0.7;
            assert_eq!(points[1], 0.7);
        }
    }

    mod edge_case_tests {
        use crate::test_utils::assert_float_eq;

        use super::*;

        #[test]
        fn test_zero_range_scale() {
            let gf = GraphicalFunction {
                name: None,
                data: GraphicalFunctionData::uniform_scale(
                    (5.0, 5.0), // Zero range
                    vec![0.5],
                    None,
                ),
                function_type: Some(GraphicalFunctionType::Continuous),
            };

            // All evaluations should return the single y-value
            assert_float_eq(gf.evaluate(4.0), 0.5, 1e-10);
            assert_float_eq(gf.evaluate(5.0), 0.5, 1e-10);
            assert_float_eq(gf.evaluate(6.0), 0.5, 1e-10);
        }

        #[test]
        fn test_extrapolation_edge_cases() {
            let gf = GraphicalFunction {
                name: None,
                data: GraphicalFunctionData::xy_pairs(vec![0.0, 1.0], vec![0.0, 1.0], None),
                function_type: Some(GraphicalFunctionType::Extrapolate),
            };

            // Test extrapolation with linear function (should maintain linearity)
            assert_float_eq(gf.evaluate(-1.0), -1.0, 1e-10);
            assert_float_eq(gf.evaluate(2.0), 2.0, 1e-10);
        }

        #[test]
        fn test_negative_values() {
            let gf = GraphicalFunction {
                name: None,
                data: GraphicalFunctionData::uniform_scale((-1.0, 1.0), vec![-0.5, 0.0, 0.5], None),
                function_type: Some(GraphicalFunctionType::Continuous),
            };

            assert_float_eq(gf.evaluate(-1.0), -0.5, 1e-10);
            assert_float_eq(gf.evaluate(0.0), 0.0, 1e-10);
            assert_float_eq(gf.evaluate(1.0), 0.5, 1e-10);
        }

        #[test]
        fn test_large_scale_values() {
            let gf = GraphicalFunction {
                name: None,
                data: GraphicalFunctionData::uniform_scale(
                    (0.0, 1000000.0),
                    vec![0.0, 500000.0, 1000000.0],
                    None,
                ),
                function_type: Some(GraphicalFunctionType::Continuous),
            };

            assert_float_eq(gf.evaluate(250000.0), 250000.0, 1.0); // Allow larger tolerance for large numbers
        }
    }
}
