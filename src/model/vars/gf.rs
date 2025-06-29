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
//! let function: GraphicalFunction = data.into();  // default values
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
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use std::{
    ops::{Index, IndexMut},
    str::FromStr,
};

use crate::{
    Expression, Identifier, Measure, UnitEquation,
    containers::{Container, ContainerMut},
    equation::IdentifierError,
    model::{
        object::{DeviceRange, DeviceScale, Document, Documentation, FormatOptions, Object},
        vars::{
            Var,
            gf::data::{GraphicalFunctionDataParseError, RawGraphicalFunctionData},
        },
    },
    types::{Validate, ValidationResult},
    validation_utils,
};

pub use data::GraphicalFunctionData;
pub use function_type::GraphicalFunctionType;
pub use points::GraphicalFunctionPoints;
pub use scale::GraphicalFunctionScale;

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
/// let named_function = GraphicalFunction::continuous(
///     Some(Identifier::parse_default("growth_rate").unwrap()),
///     GraphicalFunctionData::uniform_scale((0.0, 1.0), vec![0.0, 0.8, 1.0], None),
/// );
///
/// // Anonymous embedded function  
/// let embedded_function: GraphicalFunction = GraphicalFunctionData::uniform_scale(
///     (0.0, 1.0),
///     vec![0.0, 0.8, 1.0],
///     None
/// ).into();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GraphicalFunction {
    /// The function identifier (may be empty for embedded functions)
    pub name: Option<Identifier>,

    /// Interpolation and extrapolation behaviour (defaults to continuous if None)
    pub r#type: Option<GraphicalFunctionType>,

    /// The x-y relationship data
    pub data: GraphicalFunctionData,

    /// Optional equation describing the function
    pub equation: Option<Expression>,

    /// Optional MathML representation of the equation
    pub mathml_equation: Option<String>,

    /// Optional units of measure for this graphical function
    pub units: Option<UnitEquation>,

    /// Optional documentation for this graphical function
    pub documentation: Option<Documentation>,

    /// Optional display range for this graphical function
    pub range: Option<DeviceRange>,

    /// Optional display scale of this graphical function
    pub scale: Option<DeviceScale>,

    /// Format options for this graphical function
    pub format: Option<FormatOptions>,
}

impl GraphicalFunction {
    /// Creates a new graphical function with the specified parameters.
    ///
    /// # Arguments
    /// - `name`: Optional identifier for the function (None for anonymous functions).
    /// - `r#type`: Optional function type (defaults to Continuous if None).
    /// - `data`: The x-y relationship data for the function.
    ///
    /// # Returns
    /// A new `GraphicalFunction` instance with the provided parameters.
    pub fn new(
        name: Option<Identifier>,
        r#type: Option<GraphicalFunctionType>,
        data: GraphicalFunctionData,
    ) -> Self {
        GraphicalFunction {
            name,
            r#type,
            data,
            equation: None,
            mathml_equation: None,
            units: None,
            documentation: None,
            range: None,
            scale: None,
            format: None,
        }
    }

    /// Creates a continuous graphical function with the specified data.
    ///
    /// # Arguments
    /// - `name`: Optional identifier for the function (None for anonymous functions).
    /// - `data`: The x-y relationship data for the function.
    ///
    /// # Returns
    /// A new `GraphicalFunction` instance with type set to Continuous.
    pub fn continuous(name: Option<Identifier>, data: GraphicalFunctionData) -> Self {
        GraphicalFunction {
            name,
            r#type: Some(GraphicalFunctionType::Continuous),
            data,
            equation: None,
            mathml_equation: None,
            units: None,
            documentation: None,
            range: None,
            scale: None,
            format: None,
        }
    }

    /// Creates a discrete graphical function with the specified data.
    ///
    /// # Arguments
    /// - `name`: Optional identifier for the function (None for anonymous functions).
    /// - `data`: The x-y relationship data for the function.
    ///
    /// # Returns
    /// A new `GraphicalFunction` instance with type set to Discrete.
    pub fn discrete(name: Option<Identifier>, data: GraphicalFunctionData) -> Self {
        GraphicalFunction {
            name,
            r#type: Some(GraphicalFunctionType::Discrete),
            data,
            equation: None,
            mathml_equation: None,
            units: None,
            documentation: None,
            range: None,
            scale: None,
            format: None,
        }
    }

    /// Creates an extrapolating graphical function with the specified data.
    ///
    /// # Arguments
    /// - `name`: Optional identifier for the function (None for anonymous functions).
    /// - `data`: The x-y relationship data for the function.
    ///
    /// # Returns
    /// A new `GraphicalFunction` instance with type set to Extrapolate.
    pub fn extrapolate(name: Option<Identifier>, data: GraphicalFunctionData) -> Self {
        GraphicalFunction {
            name,
            r#type: Some(GraphicalFunctionType::Extrapolate),
            data,
            equation: None,
            mathml_equation: None,
            units: None,
            documentation: None,
            range: None,
            scale: None,
            format: None,
        }
    }

    /// Sets the equation of the graphical function and returns it.
    pub fn with_equation(mut self, equation: Expression) -> Self {
        self.equation = Some(equation);
        self
    }

    /// Sets the units of measure for this graphical function and returns it.
    pub fn with_units(mut self, units: UnitEquation) -> Self {
        self.units = Some(units);
        self
    }

    /// Sets the documentation for this graphical function and returns it.
    pub fn with_documentation(mut self, documentation: Documentation) -> Self {
        self.documentation = Some(documentation);
        self
    }

    /// Sets the range of values for this graphical function and returns it.
    pub fn with_range(mut self, range: DeviceRange) -> Self {
        self.range = Some(range);
        self
    }

    /// Sets the scale of this graphical function and returns it.
    pub fn with_scale(mut self, scale: DeviceScale) -> Self {
        self.scale = Some(scale);
        self
    }

    /// Sets the format options for this graphical function and returns it.
    pub fn with_format(mut self, format: FormatOptions) -> Self {
        self.format = Some(format);
        self
    }

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
        self.r#type.clone().unwrap_or_default()
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

// VARIABLE IMPLEMENTATIONS

impl Var<'_> for GraphicalFunction {
    /// Returns the name of this graphical function.
    ///
    /// # Returns
    /// An optional reference to the function name.
    fn name(&self) -> Option<&Identifier> {
        self.name.as_ref()
    }

    /// Returns the equation for this graphical function.
    ///
    /// # Returns
    /// An optional reference to the equation expression.
    fn equation(&self) -> Option<&Expression> {
        self.equation.as_ref()
    }

    /// Returns the MathML representation of the equation, if available.
    ///
    /// # Returns
    /// An optional reference to the MathML equation string.
    #[cfg(feature = "mathml")]
    fn mathml_equation(&self) -> Option<&String> {
        self.mathml_equation.as_ref()
    }
}

impl Document for GraphicalFunction {
    /// Returns the documentation for this graphical function.
    ///
    /// # Returns
    /// An optional reference to the documentation.
    fn documentation(&self) -> Option<&crate::model::object::Documentation> {
        None // No documentation field in GraphicalFunction
    }
}

impl Measure for GraphicalFunction {
    /// Returns the units of measure for this graphical function.
    ///
    /// # Returns
    /// An optional reference to the units of measure.
    fn units(&self) -> Option<&crate::UnitEquation> {
        None // No units field in GraphicalFunction
    }
}

impl Object for GraphicalFunction {
    /// Returns the range of values for this graphical function.
    ///
    /// # Returns
    /// An optional reference to the range.
    fn range(&self) -> Option<&DeviceRange> {
        None // No range field in GraphicalFunction
    }

    /// Returns the scale of this graphical function.
    ///
    /// # Returns
    /// An optional reference to the scale.
    fn scale(&self) -> Option<&DeviceScale> {
        None // No scale field in GraphicalFunction
    }

    /// Returns the format options for this graphical function.
    ///
    /// # Returns
    /// An optional reference to the format options.
    fn format(&self) -> Option<&FormatOptions> {
        None // No format field in GraphicalFunction
    }
}

// VALIDATION LOGIC

impl Validate for GraphicalFunction {
    /// Validates the graphical function.
    ///
    /// # Returns
    /// - `Valid(())` if the function is valid.
    /// - `Invalid(warnings, errors)` if there are validation issues.
    fn validate(&self) -> ValidationResult {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Validate the internal data structure
        validation_utils::_chain(self.data.validate(), &mut warnings, &mut errors);

        // Validate discrete functions specifically
        if matches!(self.function_type(), GraphicalFunctionType::Discrete) {
            validation_utils::_chain(
                Self::validate_discrete(&self.data),
                &mut warnings,
                &mut errors,
            );
        }

        validation_utils::_return(warnings, errors)
    }
}

impl GraphicalFunction {
    /// Validates the graphical function data for discrete functions.
    fn validate_discrete(data: &GraphicalFunctionData) -> ValidationResult {
        let mut errors = Vec::new();
        let warnings = Vec::new();

        // Ensure y-values are valid
        match data {
            GraphicalFunctionData::UniformScale { y_values, .. }
            | GraphicalFunctionData::XYPairs { y_values, .. } => {
                // Validate at least two y-values for discrete functions
                if y_values.len() < 2 {
                    errors.push("Discrete functions require at least two y-values.".into());
                } else if !validation_utils::_float_equals(
                    y_values[y_values.len() - 1],
                    y_values[y_values.len() - 2],
                ) {
                    errors.push(
                        "Last two points must have the same value for discrete functions.".into(),
                    );
                }
            }
        }

        validation_utils::_return(warnings, errors)
    }
}

// CONTAINER IMPLEMENTATIONS

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

// XML SERIALIZATION AND DESERIALIZATION

/// Helper struct for deserializing the raw XML structure
#[derive(Debug, Serialize, Deserialize)]
struct RawGraphicalFunction {
    #[serde(rename = "@name")]
    name: Option<String>,
    #[serde(rename = "@type")]
    r#type: Option<String>,
    #[serde(rename = "eqn")]
    equation: Option<Expression>,
    #[serde(rename = "mathml")]
    mathml_equation: Option<String>,
    #[serde(rename = "units")]
    units: Option<UnitEquation>,
    #[serde(rename = "doc")]
    documentation: Option<Documentation>,
    #[serde(rename = "range")]
    range: Option<DeviceRange>,
    #[serde(rename = "scale")]
    scale: Option<DeviceScale>,
    #[serde(rename = "format")]
    format: Option<FormatOptions>,
    // serde-xml-rs doesn't support flattening the below enum directly,
    // so we deserialize it as additional fields
    #[serde(rename = "xscale")]
    x_scale: Option<GraphicalFunctionScale>,
    #[serde(rename = "yscale")]
    y_scale: Option<GraphicalFunctionScale>,
    #[serde(rename = "ypts")]
    y_pts: Option<GraphicalFunctionPoints>,
    #[serde(rename = "xpts")]
    x_pts: Option<GraphicalFunctionPoints>,
}

impl From<RawGraphicalFunction> for RawGraphicalFunctionData {
    /// Converts the raw XML representation into structured data.
    fn from(raw: RawGraphicalFunction) -> Self {
        RawGraphicalFunctionData {
            x_scale: raw.x_scale,
            y_scale: raw.y_scale,
            y_pts: raw.y_pts,
            x_pts: raw.x_pts,
        }
    }
}

impl From<GraphicalFunctionData> for GraphicalFunction {
    /// Converts structured GraphicalFunctionData into a raw XML representation.
    fn from(data: GraphicalFunctionData) -> GraphicalFunction {
        GraphicalFunction::new(None, None, data)
    }
}

/// Error types for parsing graphical functions from XML.
#[derive(Debug, Error)]
pub enum GraphicalFunctionParseError {
    /// Error parsing the function name as an Identifier.
    #[error("Invalid name: {0}")]
    InvalidName(#[from] IdentifierError),

    /// Error parsing the function type from a string.
    #[error("Invalid function type: {0}")]
    InvalidFunctionType(String),

    /// Error converting raw data into structured GraphicalFunctionData.
    #[error("Data conversion error: {0}")]
    DataError(#[from] GraphicalFunctionDataParseError),
}

impl TryFrom<RawGraphicalFunction> for GraphicalFunction {
    type Error = GraphicalFunctionParseError;

    /// Converts raw XML data into a structured GraphicalFunction.
    fn try_from(raw: RawGraphicalFunction) -> Result<Self, Self::Error> {
        // Optionally parse name if present using Identifier::from_str
        let name = raw
            .name
            .as_ref()
            .map(|name_str| Identifier::parse_default(name_str))
            .transpose()?;

        // Optionally parse type if present using GraphicalFunctionType::from_str
        let r#type = raw
            .r#type
            .as_ref()
            .map(|type_str| {
                GraphicalFunctionType::from_str(type_str)
                    .map_err(GraphicalFunctionParseError::InvalidFunctionType)
            })
            .transpose()?;

        // Get the equation if present
        // todo cloning
        let equation = raw.equation.clone();
        let mathml_equation = raw.mathml_equation.clone();
        let units = raw.units.clone();
        let documentation = raw.documentation.clone();
        let range = raw.range.clone();
        let scale = raw.scale.clone();
        let format = raw.format.clone();

        // Convert raw data into GraphicalFunctionData
        let data = Into::<RawGraphicalFunctionData>::into(raw).try_into()?;

        let mut gf = GraphicalFunction::new(name, r#type, data);
        gf.equation = equation;
        gf.mathml_equation = mathml_equation;
        gf.units = units;
        gf.documentation = documentation;
        gf.range = range;
        gf.scale = scale;
        gf.format = format;

        Ok(gf)
    }
}

impl From<GraphicalFunction> for RawGraphicalFunction {
    /// Converts a structured GraphicalFunction into raw XML representation.
    fn from(gf: GraphicalFunction) -> Self {
        let x_scale = match gf.data {
            GraphicalFunctionData::UniformScale { x_scale, .. } => Some(x_scale),
            GraphicalFunctionData::XYPairs { .. } => None,
        };
        let y_scale = match gf.data {
            GraphicalFunctionData::UniformScale { y_scale, .. } => y_scale,
            GraphicalFunctionData::XYPairs { y_scale, .. } => y_scale,
        };
        // TODO: Should be able to eliminate cloning here
        let y_pts = match gf.data.clone() {
            GraphicalFunctionData::UniformScale { y_values, .. } => Some(y_values),
            GraphicalFunctionData::XYPairs { y_values, .. } => Some(y_values),
        };
        let x_pts = match gf.data {
            GraphicalFunctionData::UniformScale { .. } => None,
            GraphicalFunctionData::XYPairs { x_values, .. } => Some(x_values),
        };
        RawGraphicalFunction {
            name: gf.name.as_ref().map(|n| n.to_string()),
            r#type: gf.r#type.as_ref().map(|t| t.to_string()),
            equation: gf.equation,
            mathml_equation: gf.mathml_equation,
            units: gf.units,
            documentation: gf.documentation,
            range: gf.range,
            scale: gf.scale,
            format: gf.format,
            x_scale,
            y_scale,
            y_pts,
            x_pts,
        }
    }
}

impl<'de> Deserialize<'de> for GraphicalFunction {
    /// Deserialises the graphical function from XML.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RawGraphicalFunction::deserialize(deserializer)?
            .try_into()
            .map_err(|e| match e {
                GraphicalFunctionParseError::InvalidName(err) => serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(err.to_string().as_str()),
                    &"a valid Identifier for the function name",
                ),
                GraphicalFunctionParseError::InvalidFunctionType(invalid) => {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(invalid.as_str()),
                        &"a valid GraphicalFunctionType (continuous, extrapolate, discrete)",
                    )
                }
                GraphicalFunctionParseError::DataError(data_error) => serde::de::Error::custom(
                    format!("Failed to parse GraphicalFunction data: {}", data_error),
                ),
            })
    }
}

impl Serialize for GraphicalFunction {
    /// Serialises the graphical function to XML.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO: Should be able to eliminate cloning here
        RawGraphicalFunction::from(self.clone()).serialize(serializer)
    }
}

/// Data representation for graphical function relationships.
///
/// This module contains the core data structures for representing the x-y relationships
/// in graphical functions. XMILE supports two main data formats:
///
/// - **Uniform Scale**: Evenly spaced x-values with explicit y-values, defined by an x-scale range
/// - **XY Pairs**: Explicit coordinate pairs allowing irregular x-spacing
///
/// The module handles interpolation, extrapolation, and discrete evaluation of these
/// relationships, with proper validation of data consistency and ordering requirements.
///
/// # Examples
///
/// Creating uniform scale data:
/// ```rust
/// use xmile::model::vars::gf::GraphicalFunctionData;
/// let data = GraphicalFunctionData::uniform_scale(
///     (0.0, 1.0),
///     vec![0.0, 0.5, 1.0],
///     None
/// );
/// ```
///
/// Creating XY pairs data:
/// ```rust
/// use xmile::model::vars::gf::GraphicalFunctionData;
/// let data = GraphicalFunctionData::xy_pairs(
///     vec![0.0, 0.3, 1.0],  // Irregular spacing
///     vec![0.0, 0.8, 1.0],
///     None
/// );
/// ```
pub mod data {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use thiserror::Error;

    use std::ops::{Index, IndexMut};

    use crate::{Interpolatable, validation_utils};

    use super::{
        GraphicalFunctionPoints, GraphicalFunctionScale, GraphicalFunctionType, Validate,
        ValidationResult,
    };

    /// X-Y relationship data for graphical functions.
    ///
    /// XMILE supports two representations: uniform scaling (evenly spaced x-values)
    /// and explicit x-y pairs (irregular spacing).
    #[derive(Debug, Clone, PartialEq)]
    pub enum GraphicalFunctionData {
        /// Uniform x-axis scaling with evenly distributed y-values.
        ///
        /// # Example
        /// ```rust
        /// use xmile::GraphicalFunctionData;
        ///
        /// // x_scale: (0.0, 1.0) with 5 y-values creates x-interval of 0.25
        /// let data = GraphicalFunctionData::uniform_scale(
        ///     (0.0, 1.0),
        ///     vec![0.0, 0.3, 0.7, 0.9, 1.0],
        ///     None,
        /// );
        /// ```
        UniformScale {
            /// The (min, max) range for x-values
            x_scale: GraphicalFunctionScale,
            /// Option scale for y-values
            y_scale: Option<GraphicalFunctionScale>,
            /// Y-values evenly distributed across the x-scale
            y_values: GraphicalFunctionPoints,
        },

        /// Explicit x-y coordinate pairs for irregular spacing.
        ///
        /// # Example
        /// ```rust
        /// use xmile::GraphicalFunctionData;
        ///
        /// let data = GraphicalFunctionData::xy_pairs(
        ///     vec![0.0, 0.1, 0.5, 0.9, 1.0],  // Irregular x-spacing
        ///     vec![0.0, 0.2, 0.7, 0.95, 1.0],
        ///     None,
        /// );
        /// ```
        ///
        /// # Requirements
        /// - x_values.len() must equal y_values.len()
        /// - x_values should be sorted in ascending order
        XYPairs {
            /// Option scale for y-values
            y_scale: Option<GraphicalFunctionScale>,
            /// Explicit x-coordinates (should be sorted)
            x_values: GraphicalFunctionPoints,
            /// Corresponding y-values
            y_values: GraphicalFunctionPoints,
        },
    }

    impl GraphicalFunctionData {
        /// Creates uniform scale data with evenly spaced x-values.
        ///
        /// # Panics
        /// Panics if y_values is empty.
        pub fn uniform_scale(
            x_scale: (f64, f64),
            y_values: Vec<f64>,
            y_scale: Option<(f64, f64)>,
        ) -> Self {
            assert!(
                !y_values.is_empty(),
                "y-values cannot be empty for uniform scale"
            );
            GraphicalFunctionData::UniformScale {
                x_scale: x_scale.into(),
                y_values: y_values.into(),
                y_scale: y_scale.map(GraphicalFunctionScale::from),
            }
        }

        /// Creates explicit x-y pairs data with irregular spacing.
        ///
        /// # Panics
        /// Panics if x_values and y_values have different lengths.
        pub fn xy_pairs(
            x_values: Vec<f64>,
            y_values: Vec<f64>,
            y_scale: Option<(f64, f64)>,
        ) -> Self {
            assert_eq!(
                x_values.len(),
                y_values.len(),
                "x-values and y-values must have the same length"
            );
            GraphicalFunctionData::XYPairs {
                x_values: x_values.into(),
                y_values: y_values.into(),
                y_scale: y_scale.map(Into::into),
            }
        }

        /// Returns the y-scale for this graphical function data.
        ///
        /// If no explicit y-scale is provided, it infers the scale from the y-values.
        ///
        /// # Returns
        /// - `Some(Scale)` if a y-scale is defined or can be inferred.
        /// - `None` if y-values are empty or cannot be compared.
        pub fn y_scale(&self) -> Option<GraphicalFunctionScale> {
            match self {
                GraphicalFunctionData::UniformScale { y_scale, .. }
                | GraphicalFunctionData::XYPairs { y_scale, .. } => {
                    y_scale.or_else(|| self.infer_y_scale())
                }
            }
        }

        /// Infers the y-scale from the y-values if no explicit scale is provided.
        ///
        /// # Panics
        /// Panics if y-values are empty or cannot be compared.
        fn infer_y_scale(&self) -> Option<GraphicalFunctionScale> {
            let y_values = match self {
                GraphicalFunctionData::UniformScale { y_values, .. }
                | GraphicalFunctionData::XYPairs { y_values, .. } => y_values,
            };

            if y_values.is_empty() {
                return None;
            }

            let mut min = f64::INFINITY;
            let mut max = f64::NEG_INFINITY;

            for &value in y_values.iter() {
                if value.is_finite() {
                    min = min.min(value);
                    max = max.max(value);
                }
            }

            if min.is_infinite() || max.is_infinite() {
                None
            } else {
                Some(GraphicalFunctionScale { min, max })
            }
        }

        /// Returns the number of y-values in this graphical function data.
        ///
        /// # Returns
        /// The length of the y-values slice.
        pub fn len(&self) -> usize {
            match self {
                GraphicalFunctionData::UniformScale { y_values, .. } => y_values.len(),
                GraphicalFunctionData::XYPairs { y_values, .. } => y_values.len(),
            }
        }

        /// Evaluates the function at a given x-value based on the specified function type.
        ///
        /// # Arguments
        /// - `function_type`: The type of function to use for evaluation (Discrete, Continuous, Extrapolate).
        /// - `x`: The input value to evaluate the function at.
        ///
        /// # Returns
        /// The evaluated y-value based on the function's data and type.
        pub fn evaluate(&self, function_type: GraphicalFunctionType, x: f64) -> f64 {
            match function_type {
                GraphicalFunctionType::Discrete => self.evaluate_discrete(x),
                GraphicalFunctionType::Continuous => self.evaluate_continuous(x),
                GraphicalFunctionType::Extrapolate => self.evaluate_extrapolate(x),
            }
        }

        /// Evaluates the function at a given x-value using discrete steps.
        pub fn evaluate_discrete(&self, x: f64) -> f64 {
            match self {
                GraphicalFunctionData::UniformScale {
                    y_values, x_scale, ..
                } => self.step_uniform(x, x_scale, y_values),
                GraphicalFunctionData::XYPairs {
                    x_values, y_values, ..
                } => self.step_xy(x, x_values, y_values),
            }
        }

        /// Evaluates the function at a given x-value using linear interpolation
        /// without extrapolation.
        pub fn evaluate_continuous(&self, x: f64) -> f64 {
            match self {
                GraphicalFunctionData::UniformScale {
                    x_scale, y_values, ..
                } => self.interpolate_uniform(x, x_scale, y_values, false),
                GraphicalFunctionData::XYPairs {
                    x_values, y_values, ..
                } => self.interpolate_xy(x, x_values, y_values, false),
            }
        }

        /// Evaluates the function at a given x-value using linear interpolation
        /// with extrapolation.
        pub fn evaluate_extrapolate(&self, x: f64) -> f64 {
            match self {
                GraphicalFunctionData::UniformScale {
                    x_scale, y_values, ..
                } => self.interpolate_uniform(x, x_scale, y_values, true),
                GraphicalFunctionData::XYPairs {
                    x_values, y_values, ..
                } => self.interpolate_xy(x, x_values, y_values, true),
            }
        }
    }

    // INTERPOLATION AND GRADIENT CALCULATION

    /// Represents the position of a value in a uniform scale.
    #[derive(Debug)]
    enum UniformPosition {
        BeforeRange(f64),
        AfterRange(f64),
        Single(f64),
        Between { lower_y: f64, upper_y: f64, t: f64 },
    }

    /// Represents the position of a value in x-y pairs.
    #[derive(Debug)]
    enum XYPosition {
        BeforeRange(f64),
        AfterRange(f64),
        Single(f64),
        Between { lower_y: f64, upper_y: f64, t: f64 },
    }

    impl GraphicalFunctionData {
        /// Get the range of valid indices for gradient calculation
        fn gradient_range(&self) -> Option<std::ops::Range<usize>> {
            let len = self.len();
            if len >= 2 { Some(0..len) } else { None }
        }

        /// Calculate gradient between two points safely
        fn gradient(&self, i_1: usize, i_2: usize) -> Option<f64> {
            let range = self.gradient_range()?;
            if !range.contains(&i_1) || !range.contains(&i_2) || i_1 == i_2 {
                return None;
            }

            match self {
                GraphicalFunctionData::UniformScale {
                    y_values, x_scale, ..
                } => {
                    let step_size = x_scale.delta() / (y_values.len() - 1) as f64;
                    if step_size.abs() < f64::EPSILON {
                        None
                    } else {
                        Some((y_values[i_2] - y_values[i_1]) / (step_size * (i_2 - i_1) as f64))
                    }
                }
                GraphicalFunctionData::XYPairs {
                    x_values, y_values, ..
                } => {
                    let dx = x_values[i_2] - x_values[i_1];
                    if dx.abs() < f64::EPSILON {
                        None
                    } else {
                        Some((y_values[i_2] - y_values[i_1]) / dx)
                    }
                }
            }
        }

        /// Get the left (starting) gradient for extrapolation
        fn left_gradient(&self) -> Option<f64> {
            self.gradient(0, 1)
        }

        /// Get the right (ending) gradient for extrapolation
        fn right_gradient(&self) -> Option<f64> {
            let len = self.len();
            if len >= 2 {
                self.gradient(len - 2, len - 1)
            } else {
                None
            }
        }

        /// Find the position in a uniform scale
        fn find_uniform_position(
            &self,
            x: f64,
            x_scale: &GraphicalFunctionScale,
            y_values: &[f64],
        ) -> UniformPosition {
            if y_values.is_empty() {
                return UniformPosition::Single(0.0);
            }

            if x <= x_scale.min {
                return UniformPosition::BeforeRange(y_values[0]);
            }

            if x >= x_scale.max {
                return UniformPosition::AfterRange(*y_values.last().unwrap());
            }

            let delta = x_scale.delta();
            if delta.abs() < f64::EPSILON {
                return UniformPosition::Single(y_values[0]);
            }

            let step = delta / (y_values.len() - 1) as f64;
            if step.abs() < f64::EPSILON {
                return UniformPosition::Single(y_values[0]);
            }

            let exact_index = (x - x_scale.min) / step;
            let lower_index = exact_index.floor() as usize;
            let upper_index = (lower_index + 1).min(y_values.len() - 1);

            if lower_index >= y_values.len() - 1 {
                return UniformPosition::Single(y_values[lower_index.min(y_values.len() - 1)]);
            }

            let t = exact_index - lower_index as f64;

            UniformPosition::Between {
                lower_y: y_values[lower_index],
                upper_y: y_values[upper_index],
                t,
            }
        }

        /// Find the position in XY pairs
        fn find_xy_position(&self, x: f64, x_values: &[f64], y_values: &[f64]) -> XYPosition {
            if x_values.is_empty() || y_values.is_empty() {
                return XYPosition::Single(0.0);
            }

            if x <= x_values[0] {
                return XYPosition::BeforeRange(y_values[0]);
            }

            if x >= *x_values.last().unwrap() {
                return XYPosition::AfterRange(*y_values.last().unwrap());
            }

            // Binary search would be more efficient for large datasets
            let upper_index = x_values
                .iter()
                .position(|&x_val| x_val > x)
                .unwrap_or(x_values.len());

            if upper_index == 0 {
                return XYPosition::Single(y_values[0]);
            }

            let lower_index = upper_index - 1;
            if lower_index >= y_values.len() - 1 {
                return XYPosition::Single(*y_values.last().unwrap());
            }

            let lower_x = x_values[lower_index];
            let upper_x = x_values[upper_index];
            let dx = upper_x - lower_x;

            if dx.abs() < f64::EPSILON {
                return XYPosition::Single(y_values[lower_index]);
            }

            let t = (x - lower_x) / dx;

            XYPosition::Between {
                lower_y: y_values[lower_index],
                upper_y: y_values[upper_index],
                t,
            }
        }

        /// Evaluate using step interpolation for uniform scale
        fn step_uniform(&self, x: f64, x_scale: &GraphicalFunctionScale, y_values: &[f64]) -> f64 {
            match self.find_uniform_position(x, x_scale, y_values) {
                UniformPosition::BeforeRange(y)
                | UniformPosition::AfterRange(y)
                | UniformPosition::Single(y) => y,
                UniformPosition::Between { lower_y, .. } => lower_y,
            }
        }

        /// Evaluate using step interpolation for XY pairs
        fn step_xy(&self, x: f64, x_values: &[f64], y_values: &[f64]) -> f64 {
            match self.find_xy_position(x, x_values, y_values) {
                XYPosition::BeforeRange(y) | XYPosition::AfterRange(y) | XYPosition::Single(y) => y,
                XYPosition::Between { lower_y, .. } => lower_y,
            }
        }

        /// Evaluate using linear interpolation for uniform scale
        fn interpolate_uniform(
            &self,
            x: f64,
            x_scale: &GraphicalFunctionScale,
            y_values: &[f64],
            extrapolate: bool,
        ) -> f64 {
            match self.find_uniform_position(x, x_scale, y_values) {
                UniformPosition::BeforeRange(y) => {
                    if extrapolate {
                        let gradient = self.left_gradient().unwrap_or(0.0);
                        y + gradient * (x - x_scale.min)
                    } else {
                        y
                    }
                }
                UniformPosition::AfterRange(y) => {
                    if extrapolate {
                        let gradient = self.right_gradient().unwrap_or(0.0);
                        y + gradient * (x - x_scale.max)
                    } else {
                        y
                    }
                }
                UniformPosition::Single(y) => y,
                UniformPosition::Between {
                    lower_y,
                    upper_y,
                    t,
                    ..
                } => f64::interpolate_between(lower_y, upper_y, t),
            }
        }

        /// Evaluate using linear interpolation for XY pairs
        fn interpolate_xy(
            &self,
            x: f64,
            x_values: &[f64],
            y_values: &[f64],
            extrapolate: bool,
        ) -> f64 {
            match self.find_xy_position(x, x_values, y_values) {
                XYPosition::BeforeRange(y) => {
                    if extrapolate && x_values.len() >= 2 {
                        let gradient = (y_values[1] - y_values[0]) / (x_values[1] - x_values[0]);
                        y + gradient * (x - x_values[0])
                    } else {
                        y
                    }
                }
                XYPosition::AfterRange(y) => {
                    if extrapolate && x_values.len() >= 2 {
                        let len = x_values.len();
                        let gradient = (y_values[len - 1] - y_values[len - 2])
                            / (x_values[len - 1] - x_values[len - 2]);
                        y + gradient * (x - x_values[len - 1])
                    } else {
                        y
                    }
                }
                XYPosition::Single(y) => y,
                XYPosition::Between {
                    lower_y,
                    upper_y,
                    t,
                    ..
                } => f64::interpolate_between(lower_y, upper_y, t),
            }
        }
    }

    // VALIDATION LOGIC

    impl Validate for GraphicalFunctionData {
        /// Validates the graphical function data.
        fn validate(&self) -> ValidationResult {
            let mut warnings = Vec::new();
            let mut errors = Vec::new();

            let w = &mut warnings;
            let e = &mut errors;

            match &self {
                GraphicalFunctionData::UniformScale {
                    x_scale,
                    y_values,
                    y_scale,
                } => {
                    validation_utils::_chain(Self::validate_y_values(y_values), w, e);
                    validation_utils::_chain(Self::validate_x_scale(&Some(*x_scale)), w, e);
                    validation_utils::_chain(Self::validate_y_scale(y_scale), w, e);
                }
                GraphicalFunctionData::XYPairs {
                    x_values,
                    y_values,
                    y_scale,
                } => {
                    validation_utils::_chain(
                        Self::validate_x_values(x_values, y_values.len()),
                        w,
                        e,
                    );
                    validation_utils::_chain(Self::validate_y_values(y_values), w, e);
                    validation_utils::_chain(Self::validate_y_scale(y_scale), w, e);
                }
            }

            validation_utils::_return(warnings, errors)
        }
    }

    impl GraphicalFunctionData {
        fn validate_x_values(x_values: &GraphicalFunctionPoints, y_len: usize) -> ValidationResult {
            let mut warnings = Vec::new();
            let mut errors = Vec::new();

            let w = &mut warnings;
            let e = &mut errors;

            validation_utils::_chain(x_values.validate(), w, e);
            validation_utils::_chain(validation_utils::validate_length(x_values, y_len), w, e);
            validation_utils::_chain(validation_utils::validate_ascending(x_values), w, e);

            validation_utils::_return(warnings, errors)
        }

        fn validate_y_values(y_values: &GraphicalFunctionPoints) -> ValidationResult {
            let mut warnings = Vec::new();
            let mut errors = Vec::new();

            let w = &mut warnings;
            let e = &mut errors;

            validation_utils::_chain(y_values.validate(), w, e);
            validation_utils::_chain(validation_utils::validate_non_empty(y_values), w, e);

            validation_utils::_return(warnings, errors)
        }

        fn validate_x_scale(x_scale: &Option<GraphicalFunctionScale>) -> ValidationResult {
            let mut warnings = Vec::new();
            let mut errors = Vec::new();

            match x_scale {
                Some(scale) => {
                    validation_utils::_chain(scale.validate(), &mut warnings, &mut errors)
                }
                None => {}
            }

            validation_utils::_return(warnings, errors)
        }

        fn validate_y_scale(y_scale: &Option<GraphicalFunctionScale>) -> ValidationResult {
            let mut warnings = Vec::new();
            let mut errors = Vec::new();

            match y_scale {
                Some(scale) => {
                    validation_utils::_chain(scale.validate(), &mut warnings, &mut errors)
                }
                None => {}
            }

            validation_utils::_return(warnings, errors)
        }
    }

    // XML SERIALIZATION AND DESERIALIZATION

    /// Graphical Function Data XML representation.
    #[derive(Debug, Serialize, Deserialize)]
    pub(super) struct RawGraphicalFunctionData {
        #[serde(rename = "xscale")]
        pub(super) x_scale: Option<GraphicalFunctionScale>,
        #[serde(rename = "yscale")]
        pub(super) y_scale: Option<GraphicalFunctionScale>,
        #[serde(rename = "ypts")]
        pub(super) y_pts: Option<GraphicalFunctionPoints>,
        #[serde(rename = "xpts")]
        pub(super) x_pts: Option<GraphicalFunctionPoints>,
    }

    /// Error types for parsing graphical function data from XML.
    #[derive(Debug, Error)]
    pub enum GraphicalFunctionDataParseError {
        #[error("ypts is required for graphical function data")]
        MissingYPoints,
        #[error("y-values cannot be empty")]
        EmptyYValues,
        #[error("x-values and y-values must have the same length")]
        MismatchedLengths(GraphicalFunctionPoints, GraphicalFunctionPoints),
        #[error("Cannot have both xscale and xpts")]
        Overspecified,
        #[error("Either xscale or xpts must be provided")]
        Underspecified,
    }

    impl TryFrom<RawGraphicalFunctionData> for GraphicalFunctionData {
        type Error = GraphicalFunctionDataParseError;

        /// Converts raw XML data into a structured GraphicalFunctionData.
        fn try_from(raw: RawGraphicalFunctionData) -> Result<Self, Self::Error> {
            // Parse y-values (required for both variants)
            let y_values = raw
                .y_pts
                .ok_or(GraphicalFunctionDataParseError::MissingYPoints)?;

            // Validate y-values
            if y_values.is_empty() {
                return Err(GraphicalFunctionDataParseError::EmptyYValues);
            }

            // Helper functions to create the variants
            fn from_scale(
                x_scale: GraphicalFunctionScale,
                y_values: GraphicalFunctionPoints,
                y_scale: Option<GraphicalFunctionScale>,
            ) -> Result<GraphicalFunctionData, GraphicalFunctionDataParseError> {
                Ok(GraphicalFunctionData::UniformScale {
                    x_scale,
                    y_scale,
                    y_values,
                })
            }

            fn from_pairs(
                x_values: GraphicalFunctionPoints,
                y_values: GraphicalFunctionPoints,
                y_scale: Option<GraphicalFunctionScale>,
            ) -> Result<GraphicalFunctionData, GraphicalFunctionDataParseError> {
                if x_values.len() != y_values.len() {
                    return Err(GraphicalFunctionDataParseError::MismatchedLengths(
                        x_values, y_values,
                    ));
                }

                Ok(GraphicalFunctionData::XYPairs {
                    y_scale,
                    x_values,
                    y_values,
                })
            }

            // Determine which variant to create based on presence of x-scale vs x-pts
            match (raw.x_scale, raw.x_pts) {
                (Some(x_scale), None) => from_scale(x_scale, y_values, raw.y_scale),
                (None, Some(raw_x)) => from_pairs(raw_x, y_values, raw.y_scale),
                (Some(_), Some(_)) => Err(GraphicalFunctionDataParseError::Overspecified),
                (None, None) => Err(GraphicalFunctionDataParseError::Underspecified),
            }
        }
    }

    impl<'de> Deserialize<'de> for GraphicalFunctionData {
        /// Deserialises the graphical function data from XML.
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            RawGraphicalFunctionData::deserialize(deserializer)?
                .try_into()
                .map_err(|parse_err| match parse_err {
                    GraphicalFunctionDataParseError::MissingYPoints => {
                        serde::de::Error::missing_field("ypts")
                    }
                    GraphicalFunctionDataParseError::EmptyYValues => {
                        serde::de::Error::invalid_value(
                            serde::de::Unexpected::Unit,
                            &"y-values cannot be empty",
                        )
                    }
                    GraphicalFunctionDataParseError::MismatchedLengths(_, _) => {
                        serde::de::Error::invalid_value(
                            serde::de::Unexpected::Seq,
                            &"x-values and y-values must have the same length",
                        )
                    }
                    GraphicalFunctionDataParseError::Overspecified => {
                        serde::de::Error::custom("Cannot have both xscale and xpts")
                    }
                    GraphicalFunctionDataParseError::Underspecified => {
                        serde::de::Error::custom("Either xscale or xpts must be provided")
                    }
                })
        }
    }

    impl Serialize for GraphicalFunctionData {
        /// Serialises the graphical function data to XML.
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let raw = match self {
                GraphicalFunctionData::UniformScale {
                    x_scale,
                    y_scale,
                    y_values,
                } => RawGraphicalFunctionData {
                    x_scale: Some(*x_scale),
                    y_scale: *y_scale,
                    y_pts: Some(y_values.clone()),
                    x_pts: None,
                },
                GraphicalFunctionData::XYPairs {
                    x_values,
                    y_values,
                    y_scale,
                } => RawGraphicalFunctionData {
                    x_scale: None,
                    y_scale: *y_scale,
                    y_pts: Some(y_values.clone()),
                    x_pts: Some(x_values.clone()),
                },
            };
            raw.serialize(serializer)
        }
    }

    // OTHER TRAITS

    impl Index<usize> for GraphicalFunctionData {
        type Output = f64;

        /// Direct access to y-values by index.
        fn index(&self, index: usize) -> &Self::Output {
            match self {
                GraphicalFunctionData::UniformScale { y_values, .. } => &y_values[index],
                GraphicalFunctionData::XYPairs { y_values, .. } => &y_values[index],
            }
        }
    }

    impl IndexMut<usize> for GraphicalFunctionData {
        /// Mutable access to y-values by index.
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            match self {
                GraphicalFunctionData::UniformScale { y_values, .. } => &mut y_values[index],
                GraphicalFunctionData::XYPairs { y_values, .. } => &mut y_values[index],
            }
        }
    }
}

/// Interpolation and extrapolation behavior definitions for graphical functions.
///
/// This module defines the three interpolation types supported by XMILE graphical functions:
///
/// - **Continuous**: Linear interpolation with clamping at endpoints (no extrapolation)
/// - **Extrapolate**: Linear interpolation with linear extrapolation beyond range
/// - **Discrete**: Step-wise function with discrete jumps between values
///
/// The function type determines how intermediate values between data points are calculated
/// and how out-of-range values are handled during evaluation.
///
/// # XMILE Specification
///
/// These types correspond directly to the XMILE specification section 3.1.4 requirements
/// for graphical function behavior.
pub mod function_type {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::{fmt, str::FromStr};

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

    impl FromStr for GraphicalFunctionType {
        type Err = String;

        /// Parses a string into a GraphicalFunctionType.
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_lowercase().as_str() {
                "continuous" => Ok(GraphicalFunctionType::Continuous),
                "extrapolate" => Ok(GraphicalFunctionType::Extrapolate),
                "discrete" => Ok(GraphicalFunctionType::Discrete),
                _ => Err(s.to_string()),
            }
        }
    }

    impl<'de> Deserialize<'de> for GraphicalFunctionType {
        /// Deserialises a string into a GraphicalFunctionType.
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            GraphicalFunctionType::from_str(&s).map_err(|invalid| {
                serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(invalid.as_str()),
                    &"a valid GraphicalFunctionType (continuous, extrapolate, discrete)",
                )
            })
        }
    }

    impl Serialize for GraphicalFunctionType {
        /// Serialises the GraphicalFunctionType as a string.
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.to_string())
        }
    }
}

/// Scale definitions for graphical function axes.
///
/// This module provides the `GraphicalFunctionScale` struct for defining minimum and maximum
/// ranges for both x-axis and y-axis scaling in graphical functions. Scales are used to:
///
/// - Define the range of x-values for uniform scale functions
/// - Optionally specify explicit y-value ranges for display and validation
/// - Support proper XML serialization with min/max attributes
///
/// Scales include validation to ensure minimum values don't exceed maximum values
/// and that neither contains invalid floating-point values (NaN, infinity).
///
/// # Usage
///
/// Scales can be created from tuples for convenience:
/// ```rust
/// use xmile::model::vars::gf::GraphicalFunctionScale;
/// let scale: GraphicalFunctionScale = (0.0, 1.0).into();
/// ```
pub mod scale {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::{
        types::{Validate, ValidationResult},
        validation_utils,
    };

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

    // VALIDATION LOGIC

    impl Validate for GraphicalFunctionScale {
        /// Validates the scale range.
        ///
        /// # Returns
        /// - `Valid(())` if the range is valid.
        /// - `Invalid(warnings, errors)` if there are validation issues.
        fn validate(&self) -> ValidationResult {
            let warnings = Vec::new();
            let mut errors = Vec::new();

            if self.min > self.max {
                errors.push("Scale minimum cannot be greater than maximum.".to_string());
            }

            if self.min.is_nan() || self.max.is_nan() {
                errors.push("Scale values cannot be NaN.".to_string());
            }

            if self.min.is_infinite() || self.max.is_infinite() {
                errors.push("Scale values cannot be infinite.".to_string());
            }

            validation_utils::_return(warnings, errors)
        }
    }

    // XML SERIALIZATION AND DESERIALIZATION

    /// Graphical Function Scale XML representation.
    #[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
    struct RawGraphicalFunctionScale {
        #[serde(rename = "@min")]
        pub min: f64,
        #[serde(rename = "@max")]
        pub max: f64,
    }

    impl From<RawGraphicalFunctionScale> for GraphicalFunctionScale {
        /// Converts a RawGraphicalFunctionScale into a GraphicalFunctionScale.
        fn from(raw: RawGraphicalFunctionScale) -> Self {
            GraphicalFunctionScale {
                min: raw.min,
                max: raw.max,
            }
        }
    }

    impl<'de> Deserialize<'de> for GraphicalFunctionScale {
        /// Deserialises a GraphicalFunctionScale from XML.
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            RawGraphicalFunctionScale::deserialize(deserializer).map(Into::into)
        }
    }

    impl Serialize for GraphicalFunctionScale {
        /// Serialises the GraphicalFunctionScale to XML.
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            RawGraphicalFunctionScale {
                min: self.min,
                max: self.max,
            }
            .serialize(serializer)
        }
    }

    // OTHER TRAITS

    impl From<(f64, f64)> for GraphicalFunctionScale {
        /// Converts a tuple (min, max) into a Scale instance.
        fn from(range: (f64, f64)) -> Self {
            GraphicalFunctionScale {
                min: range.0,
                max: range.1,
            }
        }
    }
}

/// Point collections for graphical function coordinates.
///
/// This module handles collections of x-axis or y-axis points used in graphical functions,
/// with support for flexible parsing from XML with customizable separators.
///
/// The `GraphicalFunctionPoints` wrapper around `Vec<f64>` provides:
///
/// - Custom separator support for XML parsing (comma, semicolon, pipe, space, tab, etc.)
/// - Validation of numeric values (finite, non-NaN)
/// - Convenient container operations and indexing
/// - Proper XML serialization with separator preservation
///
/// # Separator Support
///
/// Points can be parsed from XML with various separators:
/// - `"0,0.5,1"` (comma - default)
/// - `"0;0.5;1"` (semicolon)
/// - `"0|0.5|1"` (pipe)
/// - `"0 0.5 1"` (space)
/// - Tab-separated values
///
/// The original separator is preserved for round-trip XML serialization.
pub mod points {
    use std::ops::{Deref, DerefMut, Index, IndexMut};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::{
        types::{Validate, ValidationResult},
        validation_utils,
    };

    /// X-axis or y-axis points for graphical functions.
    ///
    /// Represents points used in `<xpts>` and `<ypts>` XML tags with optional
    /// separator specification.
    #[derive(Debug, Clone, PartialEq)]
    pub struct GraphicalFunctionPoints {
        pub values: Vec<f64>,
        pub separator: Option<String>,
    }

    impl GraphicalFunctionPoints {
        /// Creates a new Points instance with the given values and optional separator.
        ///
        /// # Arguments
        /// - `values`: Vector of f64 values representing points.
        /// - `separator`: Optional string separator for parsing (default is None).
        pub fn new(values: Vec<f64>, separator: Option<String>) -> Self {
            GraphicalFunctionPoints { values, separator }
        }

        /// Returns the separator used for parsing points, if any.
        ///
        /// # Returns
        /// An `Option<&str>` containing the separator string, or None if not specified.
        pub fn separator(&self) -> Option<&str> {
            self.separator.as_deref()
        }
    }

    // VALIDATION LOGIC

    impl Validate for GraphicalFunctionPoints {
        /// Validates the points data.
        fn validate(&self) -> ValidationResult {
            let mut warnings = Vec::new();
            let mut errors = Vec::new();

            validation_utils::_chain(
                validation_utils::validate_finite(self),
                &mut warnings,
                &mut errors,
            );
            validation_utils::_return(warnings, errors)
        }
    }

    // XML SERIALIZATION AND DESERIALIZATION

    /// Points XML representation.
    #[derive(Debug, Serialize, Deserialize)]
    struct RawGraphicalFunctionPoints {
        #[serde(rename = "@sep")]
        separator: Option<String>,
        #[serde(rename = "#text")]
        data: String,
    }

    impl TryFrom<RawGraphicalFunctionPoints> for GraphicalFunctionPoints {
        type Error = String;

        /// Converts a RawGraphicalFunctionPoints into GraphicalFunctionPoints.
        fn try_from(raw: RawGraphicalFunctionPoints) -> Result<Self, Self::Error> {
            let sep = raw.separator.as_deref().unwrap_or(",");
            raw.data
                .split(sep)
                .map(|val_str| {
                    val_str
                        .trim()
                        .parse::<f64>()
                        .map_err(|_| val_str.to_string())
                })
                .collect::<Result<Vec<f64>, _>>()
                .map(|vals| GraphicalFunctionPoints::new(vals, raw.separator))
        }
    }

    impl<'de> Deserialize<'de> for GraphicalFunctionPoints {
        /// Deserialises GraphicalFunctionPoints from XML.
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let raw: RawGraphicalFunctionPoints =
                RawGraphicalFunctionPoints::deserialize(deserializer)?;
            GraphicalFunctionPoints::try_from(raw).map_err(|invalid_float| {
                serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(invalid_float.as_str()),
                    &"a valid f64 value (e.g. '1.0', '2.5', '-3.14')",
                )
            })
        }
    }

    impl Serialize for GraphicalFunctionPoints {
        /// Serialises GraphicalFunctionPoints to XML.
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let sep = self.separator.as_deref().unwrap_or(",");
            let raw = RawGraphicalFunctionPoints {
                separator: self.separator.clone(),
                data: self
                    .values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(sep),
            };
            raw.serialize(serializer)
        }
    }

    // OTHER TRAITS

    impl Deref for GraphicalFunctionPoints {
        type Target = [f64];

        /// Provides read-only access to the underlying values.
        fn deref(&self) -> &Self::Target {
            &self.values
        }
    }

    impl DerefMut for GraphicalFunctionPoints {
        /// Provides mutable access to the underlying values.
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.values
        }
    }

    impl Index<usize> for GraphicalFunctionPoints {
        type Output = f64;

        /// Provides direct access to points by index.
        fn index(&self, index: usize) -> &Self::Output {
            &self.values[index]
        }
    }

    impl IndexMut<usize> for GraphicalFunctionPoints {
        /// Provides mutable access to points by index.
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            &mut self.values[index]
        }
    }

    impl From<Vec<f64>> for GraphicalFunctionPoints {
        /// Converts a vector of f64 into Points with default separator.
        fn from(values: Vec<f64>) -> Self {
            GraphicalFunctionPoints {
                values,
                separator: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_scale_creation() {
        let gf = GraphicalFunction::new(
            Some(Identifier::parse_default("test_function").unwrap()),
            Some(GraphicalFunctionType::Continuous),
            GraphicalFunctionData::uniform_scale(
                (0.0, 1.0),
                vec![0.0, 0.3, 0.55, 0.7, 0.83, 0.9, 0.95, 0.98, 0.99, 0.995, 1.0],
                None,
            ),
        );

        assert!(!gf.is_anonymous());
        assert_eq!(gf.function_type(), GraphicalFunctionType::Continuous);
    }

    #[test]
    fn test_anonymous_function() {
        let gf: GraphicalFunction =
            GraphicalFunctionData::uniform_scale((0.0, 1.0), vec![0.0, 0.5, 1.0], None).into();

        assert!(gf.is_anonymous());
        assert_eq!(gf.function_type(), GraphicalFunctionType::Continuous); // Default
    }

    #[test]
    fn test_xy_pairs_creation() {
        let gf: GraphicalFunction = GraphicalFunction::new(
            Some(Identifier::parse_default("xy_function").unwrap()),
            Some(GraphicalFunctionType::Extrapolate),
            GraphicalFunctionData::xy_pairs(
                vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5],
                vec![0.05, 0.1, 0.2, 0.25, 0.3, 0.33],
                Some((0.0, 1.0)),
            ),
        );

        assert_eq!(gf.function_type(), GraphicalFunctionType::Extrapolate);
    }

    #[test]
    fn test_indexing() {
        let gf: GraphicalFunction =
            GraphicalFunctionData::uniform_scale((0.0, 1.0), vec![0.0, 0.5, 1.0], None).into();

        assert_eq!(gf[0], 0.0);
        assert_eq!(gf[1], 0.5);
        assert_eq!(gf[2], 1.0);
    }

    #[test]
    fn test_mutable_indexing() {
        let mut gf: GraphicalFunction =
            GraphicalFunctionData::uniform_scale((0.0, 1.0), vec![0.0, 0.5, 1.0], None).into();

        gf[1] = 0.7;
        assert_eq!(gf[1], 0.7);
    }

    mod data {
        #[cfg(test)]
        use super::*;

        #[test]
        fn test_uniform_scale_data_creation() {
            let data = GraphicalFunctionData::uniform_scale(
                (0.0, 1.0),
                vec![0.0, 0.5, 1.0],
                Some((0.0, 1.0)),
            );

            match data {
                GraphicalFunctionData::UniformScale {
                    x_scale,
                    y_values,
                    y_scale,
                } => {
                    assert_eq!(x_scale.min, 0.0);
                    assert_eq!(x_scale.max, 1.0);
                    assert_eq!(y_values.len(), 3);
                    assert!(y_scale.is_some());
                }
                _ => panic!("Expected UniformScale variant"),
            }
        }

        #[test]
        fn test_xy_pairs_data_creation() {
            let data =
                GraphicalFunctionData::xy_pairs(vec![0.0, 0.5, 1.0], vec![0.0, 0.3, 1.0], None);

            match data {
                GraphicalFunctionData::XYPairs {
                    x_values,
                    y_values,
                    y_scale,
                } => {
                    assert_eq!(x_values.len(), 3);
                    assert_eq!(y_values.len(), 3);
                    assert!(y_scale.is_none());
                }
                _ => panic!("Expected XYPairs variant"),
            }
        }

        #[test]
        #[should_panic(expected = "y-values cannot be empty for uniform scale")]
        fn test_uniform_scale_empty_y_values() {
            GraphicalFunctionData::uniform_scale((0.0, 1.0), vec![], None);
        }

        #[test]
        #[should_panic(expected = "x-values and y-values must have the same length")]
        fn test_xy_pairs_mismatched_lengths() {
            GraphicalFunctionData::xy_pairs(vec![0.0, 0.5], vec![0.0, 0.3, 1.0], None);
        }

        #[test]
        fn test_y_scale_inference() {
            let data = GraphicalFunctionData::uniform_scale((0.0, 1.0), vec![0.2, 0.8, 0.5], None);

            let inferred_scale = data.y_scale().unwrap();
            assert_eq!(inferred_scale.min, 0.2);
            assert_eq!(inferred_scale.max, 0.8);
        }

        #[test]
        fn test_y_scale_explicit() {
            let data = GraphicalFunctionData::uniform_scale(
                (0.0, 1.0),
                vec![0.2, 0.8, 0.5],
                Some((0.0, 1.0)),
            );

            let scale = data.y_scale().unwrap();
            assert_eq!(scale.min, 0.0);
            assert_eq!(scale.max, 1.0);
        }
    }

    mod function_type {
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

    mod scale {
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

    mod points {
        use super::*;

        #[test]
        fn test_points_creation() {
            let points = GraphicalFunctionPoints::new(vec![0.0, 0.5, 1.0], Some(";".to_string()));
            assert_eq!(points.len(), 3);
            assert_eq!(points.separator(), Some(";"));
        }

        #[test]
        fn test_points_from_vec() {
            let points: GraphicalFunctionPoints = vec![0.0, 0.5, 1.0].into();
            assert_eq!(points.len(), 3);
            assert_eq!(points.separator(), None);
        }

        #[test]
        fn test_points_deref() {
            let points = GraphicalFunctionPoints::new(vec![0.0, 0.5, 1.0], None);
            assert_eq!(points[0], 0.0);
            assert_eq!(points[1], 0.5);
            assert_eq!(points[2], 1.0);
        }

        #[test]
        fn test_points_deref_mut() {
            let mut points = GraphicalFunctionPoints::new(vec![0.0, 0.5, 1.0], None);
            points[1] = 0.7;
            assert_eq!(points[1], 0.7);
        }
    }

    mod edge_case_tests {
        use crate::test_utils::assert_float_eq;

        use super::*;

        #[test]
        fn test_zero_range_scale() {
            let gf: GraphicalFunction = GraphicalFunctionData::uniform_scale(
                (5.0, 5.0), // Zero range
                vec![0.5],
                None,
            )
            .into();

            // All evaluations should return the single y-value
            assert_float_eq(gf.evaluate(4.0), 0.5, 1e-10);
            assert_float_eq(gf.evaluate(5.0), 0.5, 1e-10);
            assert_float_eq(gf.evaluate(6.0), 0.5, 1e-10);
        }

        #[test]
        fn test_extrapolation_edge_cases() {
            let gf = GraphicalFunction::new(
                None,
                Some(GraphicalFunctionType::Extrapolate),
                GraphicalFunctionData::xy_pairs(vec![0.0, 1.0], vec![0.0, 1.0], None).into(),
            );

            // Test extrapolation with linear function (should maintain linearity)
            assert_float_eq(gf.evaluate(-1.0), -1.0, 1e-10);
            assert_float_eq(gf.evaluate(2.0), 2.0, 1e-10);
        }

        #[test]
        fn test_negative_values() {
            let gf: GraphicalFunction =
                GraphicalFunctionData::uniform_scale((-1.0, 1.0), vec![-0.5, 0.0, 0.5], None)
                    .into();

            assert_float_eq(gf.evaluate(-1.0), -0.5, 1e-10);
            assert_float_eq(gf.evaluate(0.0), 0.0, 1e-10);
            assert_float_eq(gf.evaluate(1.0), 0.5, 1e-10);
        }

        #[test]
        fn test_large_scale_values() {
            let gf: GraphicalFunction = GraphicalFunctionData::uniform_scale(
                (0.0, 1000000.0),
                vec![0.0, 500000.0, 1000000.0],
                None,
            )
            .into();

            assert_float_eq(gf.evaluate(250000.0), 250000.0, 1.0); // Allow larger tolerance for large numbers
        }
    }

    #[cfg(test)]
    mod xml_tests {
        use crate::Identifier;
        use crate::model::vars::gf::{
            GraphicalFunction, GraphicalFunctionData, GraphicalFunctionScale, GraphicalFunctionType,
        };

        // Tests for examples directly from the XMILE specification
        mod specification_examples {
            use crate::types::Validate;

            use super::*;

            #[test]
            fn test_spec_minimal_named_function() {
                // From specification: smallest possible named graphical function
                let xml = r#"<gf name="rising">
                <xscale min="0" max="1"/>
                <ypts>0,0.1,0.5,0.9,1</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                assert_eq!(
                    function.name,
                    Some(Identifier::parse_default("rising").unwrap())
                );
                assert!(function.r#type.is_none()); // Should default to continuous

                match function.data {
                    GraphicalFunctionData::UniformScale {
                        x_scale,
                        y_values,
                        y_scale,
                    } => {
                        assert_eq!(x_scale.min, 0.0);
                        assert_eq!(x_scale.max, 1.0);
                        assert_eq!(y_values.values, vec![0.0, 0.1, 0.5, 0.9, 1.0]);
                        assert!(y_scale.is_none());
                    }
                    _ => panic!("Expected UniformScale variant"),
                }
            }

            #[test]
            fn test_spec_food_availability_uniform_scale() {
                // From specification: food_availability_multiplier_function with uniform scale
                let xml = r#"<gf name="food_availability_multiplier_function" type="continuous">
                <xscale min="0" max="1"/>
                <ypts>0,0.3,0.55,0.7,0.83,0.9,0.95,0.98,0.99,0.995,1</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                assert_eq!(
                    function.name,
                    Some(
                        Identifier::parse_default("food_availability_multiplier_function").unwrap()
                    )
                );
                assert_eq!(function.r#type, Some(GraphicalFunctionType::Continuous));

                match function.data {
                    GraphicalFunctionData::UniformScale {
                        x_scale,
                        y_values,
                        y_scale,
                    } => {
                        assert_eq!(x_scale.min, 0.0);
                        assert_eq!(x_scale.max, 1.0);
                        assert_eq!(
                            y_values.values,
                            vec![0.0, 0.3, 0.55, 0.7, 0.83, 0.9, 0.95, 0.98, 0.99, 0.995, 1.0]
                        );
                        assert!(y_scale.is_none());
                    }
                    _ => panic!("Expected UniformScale variant"),
                }
            }

            #[test]
            fn test_spec_food_availability_xy_pairs() {
                // From specification: food_availability_multiplier_function with x-y pairs
                let xml = r#"<gf name="food_availability_multiplier_function" type="continuous">
                <xpts>0,0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8,0.9,1</xpts>
                <ypts>0,0.3,0.55,0.7,0.83,0.9,0.95,0.98,0.99,0.995,1</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                assert_eq!(
                    function.name,
                    Some(
                        Identifier::parse_default("food_availability_multiplier_function").unwrap()
                    )
                );
                assert_eq!(function.r#type, Some(GraphicalFunctionType::Continuous));

                match function.data {
                    GraphicalFunctionData::XYPairs {
                        x_values,
                        y_values,
                        y_scale,
                    } => {
                        assert_eq!(
                            x_values.values,
                            vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
                        );
                        assert_eq!(
                            y_values.values,
                            vec![0.0, 0.3, 0.55, 0.7, 0.83, 0.9, 0.95, 0.98, 0.99, 0.995, 1.0]
                        );
                        assert!(y_scale.is_none());
                    }
                    _ => panic!("Expected XYPairs variant"),
                }
            }

            #[test]
            fn test_spec_embedded_function() {
                // From specification: embedded function (anonymous)
                let xml = r#"<gf>
                <xscale min="0.0" max="1.0"/>
                <ypts>0,0.3,0.55,0.7,0.83,0.9,0.95,0.98,0.99,0.995,1</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                assert!(function.name.is_none()); // Anonymous/embedded
                assert!(function.r#type.is_none()); // Should default

                match function.data {
                    GraphicalFunctionData::UniformScale {
                        x_scale,
                        y_values,
                        y_scale,
                    } => {
                        assert_eq!(x_scale.min, 0.0);
                        assert_eq!(x_scale.max, 1.0);
                        assert_eq!(
                            y_values.values,
                            vec![0.0, 0.3, 0.55, 0.7, 0.83, 0.9, 0.95, 0.98, 0.99, 0.995, 1.0]
                        );
                        assert!(y_scale.is_none());
                    }
                    _ => panic!("Expected UniformScale variant"),
                }
            }

            #[test]
            fn test_spec_overspecified_function_should_fail() {
                // From specification: overspecified function (has both xscale and xpts)
                let xml = r#"<gf name="overspecified">
                <xscale min="0" max="0.5"/>
                <yscale min="0" max="1"/>
                <xpts>0,0.1,0.2,0.3,0.4,0.5</xpts>
                <ypts>0.05,0.1,0.2,0.25,0.3,0.33</ypts>
            </gf>"#;

                let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
                assert!(result.is_err());
            }

            #[test]
            fn test_spec_inconsistent_function_should_fail() {
                // From specification: inconsistent function (xpts don't match xscale)
                let xml = r#"<gf name="inconsistent">
                <xscale min="0" max="0.5"/>
                <yscale min="0" max="1"/>
                <xpts>0,1,2,3,4,5</xpts>
                <ypts>0.05,0.1,0.2,0.25,0.3,0.33</ypts>
            </gf>"#;

                let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
                assert!(result.is_err());
            }

            #[test]
            fn test_spec_invalid_xpts_order_should_fail() {
                // From specification: invalid function with unordered x-values
                let xml = r#"<gf name="invalid">
                <yscale min="0" max="1"/>
                <xpts>2,1,3,0</xpts>
                <ypts>0.05,0.1,0.2,0.25</ypts>
            </gf>"#;

                let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
                // This should fail during validation, but may parse successfully initially
                // The validation would catch the unordered x-values
                if let Ok(function) = result {
                    // If it parses, validation should catch the error
                    assert!(function.validate().is_invalid());
                }
            }

            #[test]
            fn test_spec_mismatched_xy_length_should_fail() {
                // From specification: mismatched x and y value counts
                let xml = r#"<gf name="invalid">
                <yscale min="0" max="1"/>
                <xpts>2,1,3,0</xpts>
                <ypts>0.05,0.1,0.2,0.25,0.3,0.33</ypts>
            </gf>"#;

                let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
                assert!(result.is_err());
            }
        }

        mod additional_properties {
            use super::*;

            #[cfg(feature = "full")]
            #[test]
            fn all_additional() {
                use crate::{
                    Expression, NumericConstant, UnitEquation,
                    model::object::{
                        DeviceRange, DeviceScale, DisplayAs, Documentation, FormatOptions,
                    },
                };

                let xml = r#"<gf name="additional_properties" type="continuous">
                <!-- Data -->
                <xscale min="0" max="1"/>
                <yscale min="-1" max="2"/>
                <ypts>0,0.5,1</ypts>
                <!-- Additional properties -->
                <eqn>x^2 + 2 * x + 4</eqn>
                <mathml>x^2 + 2x + 4</mathml>
                <units>(people * research_output / time) / energy</units>
                <doc>This is a test function with additional properties.</doc>
                <range min="1" max="2" />
                <scale min="1.0" max="2.0"></scale>
                <format precision="0.01" scale_by="1000" display_as="percent" delimit_000s="true" />
            </gf>"#;
                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();
                println!("{:#?}", function);

                assert_eq!(
                    function.name,
                    Some(Identifier::parse_default("additional_properties").unwrap())
                );
                assert_eq!(function.r#type, Some(GraphicalFunctionType::Continuous));

                match function.data {
                    GraphicalFunctionData::UniformScale {
                        x_scale,
                        y_values,
                        y_scale,
                    } => {
                        assert_eq!(x_scale.min, 0.0);
                        assert_eq!(x_scale.max, 1.0);
                        assert_eq!(y_values.values, vec![0.0, 0.5, 1.0]);
                        assert_eq!(y_scale.unwrap().min, -1.0);
                        assert_eq!(y_scale.unwrap().max, 2.0);
                    }
                    _ => panic!("Expected UniformScale variant"),
                }

                let expected_equation = Expression::add(
                    Expression::add(
                        Expression::exponentiation(
                            Expression::subscript(Identifier::parse_default("x").unwrap(), vec![]),
                            Expression::constant(NumericConstant::from(2.0)),
                        ),
                        Expression::multiply(
                            Expression::constant(NumericConstant::from(2.0)),
                            Expression::subscript(Identifier::parse_default("x").unwrap(), vec![]),
                        ),
                    ),
                    Expression::constant(NumericConstant::from(4.0)),
                );

                let expected_units = UnitEquation::division(
                    UnitEquation::parentheses(UnitEquation::division(
                        UnitEquation::multiplication(
                            UnitEquation::Alias(
                                Identifier::parse_unit_name("people").expect("valid unit"),
                            ),
                            UnitEquation::Alias(
                                Identifier::parse_unit_name("research_output").expect("valid unit"),
                            ),
                        ),
                        UnitEquation::Alias(
                            Identifier::parse_unit_name("time").expect("valid unit"),
                        ),
                    )),
                    UnitEquation::Alias(Identifier::parse_unit_name("energy").expect("valid unit")),
                );

                // Additional properties
                assert_eq!(function.equation, Some(expected_equation));
                assert_eq!(function.mathml_equation, Some("x^2 + 2x + 4".to_string()));
                assert_eq!(function.units, Some(expected_units));
                assert_eq!(
                    function.documentation,
                    Some(Documentation::PlainText(
                        "This is a test function with additional properties.".to_string()
                    ))
                );
                assert_eq!(function.range, Some(DeviceRange::new(1.0, 2.0)));
                assert_eq!(function.scale, Some(DeviceScale::new(1.0, 2.0)));
                assert_eq!(
                    function.format,
                    Some(FormatOptions {
                        precision: Some(0.01),
                        scale_by: Some(1000.0),
                        display_as: Some(DisplayAs::Percent),
                        delimit_000s: Some(true),
                    })
                );
            }
        }

        // Tests for valid configurations not explicitly shown in specification
        mod additional_valid_configurations {
            use super::*;

            #[test]
            fn test_all_function_types() {
                let continuous_xml = r#"<gf name="continuous_func" type="continuous">
                <xscale min="0" max="1"/>
                <ypts>0,0.5,1</ypts>
            </gf>"#;

                let extrapolate_xml = r#"<gf name="extrapolate_func" type="extrapolate">
                <xscale min="0" max="1"/>
                <ypts>0,0.5,1</ypts>
            </gf>"#;

                let discrete_xml = r#"<gf name="discrete_func" type="discrete">
                <xscale min="0" max="1"/>
                <ypts>0,0.5,1</ypts>
            </gf>"#;

                let continuous_func: GraphicalFunction =
                    serde_xml_rs::from_str(continuous_xml).unwrap();
                let extrapolate_func: GraphicalFunction =
                    serde_xml_rs::from_str(extrapolate_xml).unwrap();
                let discrete_func: GraphicalFunction =
                    serde_xml_rs::from_str(discrete_xml).unwrap();

                assert_eq!(
                    continuous_func.r#type,
                    Some(GraphicalFunctionType::Continuous)
                );
                assert_eq!(
                    extrapolate_func.r#type,
                    Some(GraphicalFunctionType::Extrapolate)
                );
                assert_eq!(discrete_func.r#type, Some(GraphicalFunctionType::Discrete));
            }

            #[test]
            fn test_case_insensitive_function_types() {
                let xml = r#"<gf name="test_func" type="CONTINUOUS">
                <xscale min="0" max="1"/>
                <ypts>0,1</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();
                assert_eq!(function.r#type, Some(GraphicalFunctionType::Continuous));
            }

            #[test]
            fn test_with_y_scale() {
                let xml = r#"<gf name="scaled_func">
                <xscale min="0" max="1"/>
                <yscale min="-5" max="10"/>
                <ypts>0,0.5,1</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                match function.data {
                    GraphicalFunctionData::UniformScale { y_scale, .. } => {
                        assert_eq!(y_scale, Some(GraphicalFunctionScale::new(-5.0, 10.0)));
                    }
                    _ => panic!("Expected UniformScale variant"),
                }
            }

            #[test]
            fn test_negative_values() {
                let xml = r#"<gf name="negative_func">
                <xscale min="-10" max="10"/>
                <ypts>-1,-0.5,0,0.5,1</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                match function.data {
                    GraphicalFunctionData::UniformScale {
                        x_scale, y_values, ..
                    } => {
                        assert_eq!(x_scale.min, -10.0);
                        assert_eq!(x_scale.max, 10.0);
                        assert_eq!(y_values.values, vec![-1.0, -0.5, 0.0, 0.5, 1.0]);
                    }
                    _ => panic!("Expected UniformScale variant"),
                }
            }

            #[test]
            fn test_single_point_function() {
                let xml = r#"<gf name="single_point">
                <xscale min="5" max="5"/>
                <ypts>42</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                match function.data {
                    GraphicalFunctionData::UniformScale {
                        x_scale, y_values, ..
                    } => {
                        assert_eq!(x_scale.min, 5.0);
                        assert_eq!(x_scale.max, 5.0);
                        assert_eq!(y_values.values, vec![42.0]);
                    }
                    _ => panic!("Expected UniformScale variant"),
                }
            }

            #[test]
            fn test_fractional_values() {
                let xml = r#"<gf name="fractional">
                <xscale min="0.1" max="0.9"/>
                <ypts>0.01,0.25,0.75,0.99</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                match function.data {
                    GraphicalFunctionData::UniformScale {
                        x_scale, y_values, ..
                    } => {
                        assert_eq!(x_scale.min, 0.1);
                        assert_eq!(x_scale.max, 0.9);
                        assert_eq!(y_values.values, vec![0.01, 0.25, 0.75, 0.99]);
                    }
                    _ => panic!("Expected UniformScale variant"),
                }
            }

            #[test]
            fn test_scientific_notation() {
                let xml = r#"<gf name="scientific">
                <xscale min="1e-3" max="1e3"/>
                <ypts>1e-6,1e-3,1e0,1e3,1e6</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                match function.data {
                    GraphicalFunctionData::UniformScale {
                        x_scale, y_values, ..
                    } => {
                        assert_eq!(x_scale.min, 0.001);
                        assert_eq!(x_scale.max, 1000.0);
                        assert_eq!(
                            y_values.values,
                            vec![0.000001, 0.001, 1.0, 1000.0, 1000000.0]
                        );
                    }
                    _ => panic!("Expected UniformScale variant"),
                }
            }

            #[test]
            fn test_xy_pairs_with_y_scale() {
                let xml = r#"<gf name="xy_with_yscale">
                <yscale min="0" max="100"/>
                <xpts>0,0.5,1</xpts>
                <ypts>10,50,90</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                match function.data {
                    GraphicalFunctionData::XYPairs {
                        x_values,
                        y_values,
                        y_scale,
                    } => {
                        assert_eq!(x_values.values, vec![0.0, 0.5, 1.0]);
                        assert_eq!(y_values.values, vec![10.0, 50.0, 90.0]);
                        assert_eq!(y_scale, Some(GraphicalFunctionScale::new(0.0, 100.0)));
                    }
                    _ => panic!("Expected XYPairs variant"),
                }
            }

            #[test]
            fn test_irregular_x_spacing() {
                let xml = r#"<gf name="irregular">
                <xpts>0,0.1,0.15,0.2,0.8,0.95,1</xpts>
                <ypts>0,0.2,0.3,0.4,0.7,0.9,1</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                match function.data {
                    GraphicalFunctionData::XYPairs {
                        x_values, y_values, ..
                    } => {
                        assert_eq!(x_values.values, vec![0.0, 0.1, 0.15, 0.2, 0.8, 0.95, 1.0]);
                        assert_eq!(y_values.values, vec![0.0, 0.2, 0.3, 0.4, 0.7, 0.9, 1.0]);
                    }
                    _ => panic!("Expected XYPairs variant"),
                }
            }
        }

        // Tests for custom separators
        mod separator_tests {
            use super::*;

            #[test]
            fn test_semicolon_separator() {
                let xml = r#"<gf name="semicolon_sep">
                <xscale min="0" max="1"/>
                <ypts sep=";">0;0.25;0.5;0.75;1</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                match function.data {
                    GraphicalFunctionData::UniformScale { y_values, .. } => {
                        assert_eq!(y_values.values, vec![0.0, 0.25, 0.5, 0.75, 1.0]);
                        assert_eq!(y_values.separator(), Some(";"));
                    }
                    _ => panic!("Expected UniformScale variant"),
                }
            }

            #[test]
            fn test_pipe_separator() {
                let xml = r#"<gf name="pipe_sep">
                <xpts sep="|">0|0.5|1</xpts>
                <ypts sep="|">10|50|90</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                match function.data {
                    GraphicalFunctionData::XYPairs {
                        x_values, y_values, ..
                    } => {
                        assert_eq!(x_values.values, vec![0.0, 0.5, 1.0]);
                        assert_eq!(y_values.values, vec![10.0, 50.0, 90.0]);
                        assert_eq!(x_values.separator(), Some("|"));
                        assert_eq!(y_values.separator(), Some("|"));
                    }
                    _ => panic!("Expected XYPairs variant"),
                }
            }

            #[test]
            fn test_space_separator() {
                let xml = r#"<gf name="space_sep">
                <xscale min="0" max="2"/>
                <ypts sep=" ">0 1 4</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                match function.data {
                    GraphicalFunctionData::UniformScale { y_values, .. } => {
                        assert_eq!(y_values.values, vec![0.0, 1.0, 4.0]);
                        assert_eq!(y_values.separator(), Some(" "));
                    }
                    _ => panic!("Expected UniformScale variant"),
                }
            }

            #[test]
            fn test_tab_separator() {
                let xml = r#"<gf name="tab_sep">
                <xscale min="0" max="1"/>
                <ypts sep="	">0	0.5	1</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                match function.data {
                    GraphicalFunctionData::UniformScale { y_values, .. } => {
                        assert_eq!(y_values.values, vec![0.0, 0.5, 1.0]);
                        assert_eq!(y_values.separator(), Some("\t"));
                    }
                    _ => panic!("Expected UniformScale variant"),
                }
            }

            #[test]
            fn test_mixed_separators_xy_pairs() {
                // Different separators for x and y points
                let xml = r#"<gf name="mixed_sep">
                <xpts sep=";">0;0.5;1</xpts>
                <ypts sep=",">10,50,90</ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                match function.data {
                    GraphicalFunctionData::XYPairs {
                        x_values, y_values, ..
                    } => {
                        assert_eq!(x_values.values, vec![0.0, 0.5, 1.0]);
                        assert_eq!(y_values.values, vec![10.0, 50.0, 90.0]);
                        assert_eq!(x_values.separator(), Some(";"));
                        assert_eq!(y_values.separator(), Some(","));
                    }
                    _ => panic!("Expected XYPairs variant"),
                }
            }
        }

        // Tests for edge cases and error conditions
        mod error_condition_tests {
            use super::*;

            #[test]
            fn test_missing_ypts_should_fail() {
                let xml = r#"<gf name="no_ypts">
                <xscale min="0" max="1"/>
            </gf>"#;

                let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
                assert!(result.is_err());
            }

            #[test]
            fn test_missing_xscale_and_xpts_should_fail() {
                let xml = r#"<gf name="no_x_data">
                <ypts>0,0.5,1</ypts>
            </gf>"#;

                let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
                assert!(result.is_err());
            }

            #[test]
            fn test_empty_ypts_should_fail() {
                let xml = r#"<gf name="empty_ypts">
                <xscale min="0" max="1"/>
                <ypts></ypts>
            </gf>"#;

                let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
                assert!(result.is_err());
            }

            #[test]
            fn test_invalid_number_in_ypts_should_fail() {
                let xml = r#"<gf name="invalid_number">
                <xscale min="0" max="1"/>
                <ypts>0,invalid,1</ypts>
            </gf>"#;

                let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
                assert!(result.is_err());
            }

            #[test]
            fn test_invalid_function_type_should_fail() {
                let xml = r#"<gf name="invalid_type" type="invalid_type">
                <xscale min="0" max="1"/>
                <ypts>0,1</ypts>
            </gf>"#;

                let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
                assert!(result.is_err());
            }

            #[test]
            fn test_both_xscale_and_xpts_should_fail() {
                let xml = r#"<gf name="both_x">
                <xscale min="0" max="1"/>
                <xpts>0,0.5,1</xpts>
                <ypts>0,0.5,1</ypts>
            </gf>"#;

                let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
                assert!(result.is_err());
            }

            #[test]
            fn test_mismatched_xy_lengths_should_fail() {
                let xml = r#"<gf name="mismatched">
                <xpts>0,0.5,1</xpts>
                <ypts>0,0.5</ypts>
            </gf>"#;

                let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
                assert!(result.is_err());
            }

            #[test]
            fn test_invalid_separator_parsing() {
                let xml = r#"<gf name="bad_sep">
                <xscale min="0" max="1"/>
                <ypts sep=",">0;0.5;1</ypts>
            </gf>"#;

                // This should fail because separator is "," but values use ";"
                let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
                assert!(result.is_err());
            }
        }

        // Tests for whitespace handling
        mod whitespace_tests {
            use super::*;

            #[test]
            fn test_whitespace_in_values() {
                let xml = r#"<gf name="whitespace">
                <xscale min="0" max="1"/>
                <ypts> 0 , 0.25 , 0.5 , 0.75 , 1 </ypts>
            </gf>"#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                match function.data {
                    GraphicalFunctionData::UniformScale { y_values, .. } => {
                        assert_eq!(y_values.values, vec![0.0, 0.25, 0.5, 0.75, 1.0]);
                    }
                    _ => panic!("Expected UniformScale variant"),
                }
            }

            #[test]
            fn test_multiline_xml() {
                let xml = r#"
            <gf name="multiline">
                <xscale min="0" max="1"/>
                <yscale min="0" max="100"/>
                <ypts>
                    0,
                    25,
                    50,
                    75,
                    100
                </ypts>
            </gf>
            "#;

                let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

                match function.data {
                    GraphicalFunctionData::UniformScale {
                        y_values, y_scale, ..
                    } => {
                        assert_eq!(y_values.values, vec![0.0, 25.0, 50.0, 75.0, 100.0]);
                        assert_eq!(y_scale, Some(GraphicalFunctionScale::new(0.0, 100.0)));
                    }
                    _ => panic!("Expected UniformScale variant"),
                }
            }
        }

        // Tests for large datasets
        mod large_dataset_tests {
            use super::*;

            #[test]
            fn test_large_uniform_scale_function() {
                let y_values: Vec<String> =
                    (0..1000).map(|i| (i as f64 / 1000.0).to_string()).collect();
                let ypts_str = y_values.join(",");

                let xml = format!(
                    r#"<gf name="large_function">
                <xscale min="0" max="999"/>
                <ypts>{}</ypts>
            </gf>"#,
                    ypts_str
                );

                let function: GraphicalFunction = serde_xml_rs::from_str(&xml).unwrap();

                match function.data {
                    GraphicalFunctionData::UniformScale { y_values, .. } => {
                        assert_eq!(y_values.len(), 1000);
                        assert_eq!(y_values[0], 0.0);
                        assert_eq!(y_values[999], 0.999);
                    }
                    _ => panic!("Expected UniformScale variant"),
                }
            }

            #[test]
            fn test_large_xy_pairs_function() {
                let xy_pairs: Vec<(String, String)> = (0..100)
                    .map(|i| {
                        let x = (i as f64 / 10.0).to_string();
                        let y = (i as f64 * i as f64 / 100.0).to_string();
                        (x, y)
                    })
                    .collect();

                let x_values: Vec<String> = xy_pairs.iter().map(|(x, _)| x.clone()).collect();
                let y_values: Vec<String> = xy_pairs.iter().map(|(_, y)| y.clone()).collect();

                let xml = format!(
                    r#"<gf name="large_xy_function">
                <xpts>{}</xpts>
                <ypts>{}</ypts>
            </gf>"#,
                    x_values.join(","),
                    y_values.join(",")
                );

                let function: GraphicalFunction = serde_xml_rs::from_str(&xml).unwrap();

                match function.data {
                    GraphicalFunctionData::XYPairs {
                        x_values, y_values, ..
                    } => {
                        assert_eq!(x_values.len(), 100);
                        assert_eq!(y_values.len(), 100);
                        assert_eq!(x_values[0], 0.0);
                        assert_eq!(x_values[99], 9.9);
                    }
                    _ => panic!("Expected XYPairs variant"),
                }
            }
        }
    }

    #[cfg(test)]
    mod validation_tests {
        use crate::{
            GraphicalFunction, GraphicalFunctionData, Identifier,
            model::vars::gf::{GraphicalFunctionPoints, GraphicalFunctionScale},
            types::Validate,
        };

        use super::*;

        #[test]
        fn test_valid_uniform_scale_function() {
            let gf = GraphicalFunction::continuous(
                Some(Identifier::parse_default("valid_function").unwrap()),
                GraphicalFunctionData::uniform_scale(
                    (0.0, 1.0),
                    vec![0.0, 0.5, 1.0],
                    Some((0.0, 1.0)),
                ),
            );

            match gf.validate() {
                ValidationResult::Valid(_) => {} // Expected
                _ => panic!("Expected valid function to pass validation"),
            }
        }

        #[test]
        fn test_valid_xy_pairs_function() {
            let gf: GraphicalFunction =
                GraphicalFunctionData::xy_pairs(vec![0.0, 0.5, 1.0], vec![0.0, 0.3, 1.0], None)
                    .into();

            match gf.validate() {
                ValidationResult::Valid(_) => {} // Expected
                _ => panic!("Expected valid function to pass validation"),
            }
        }

        #[test]
        fn test_invalid_discrete_function() {
            let gf = GraphicalFunction::discrete(
                None,
                GraphicalFunctionData::uniform_scale(
                    (0.0, 1.0),
                    vec![0.0, 0.5, 1.0], // Last two values different
                    None,
                ),
            );

            match gf.validate() {
                ValidationResult::Invalid(_, errors) => {
                    assert!(!errors.is_empty());
                    assert!(errors.iter().any(|e| e.contains("same value")));
                }
                _ => panic!(
                    "Expected discrete function with different last values to fail validation"
                ),
            }
        }

        #[test]
        fn test_valid_discrete_function() {
            let gf = GraphicalFunction::discrete(
                None,
                GraphicalFunctionData::uniform_scale(
                    (0.0, 1.0),
                    vec![0.0, 0.5, 0.8, 0.8], // Last two values same
                    None,
                ),
            );

            match gf.validate() {
                ValidationResult::Valid(_) => {} // Expected
                _ => panic!("Expected valid discrete function to pass validation"),
            }
        }

        #[test]
        fn test_invalid_scale() {
            let scale = GraphicalFunctionScale { min: 1.0, max: 0.0 }; // Invalid: min > max

            match scale.validate() {
                ValidationResult::Invalid(_, errors) => {
                    assert!(!errors.is_empty());
                    assert!(
                        errors
                            .iter()
                            .any(|e| e.contains("minimum cannot be greater than maximum"))
                    );
                }
                _ => panic!("Expected invalid scale to fail validation"),
            }
        }

        #[test]
        fn test_nan_values_validation() {
            let points = GraphicalFunctionPoints::new(vec![0.0, f64::NAN, 1.0], None);

            match points.validate() {
                ValidationResult::Invalid(_, errors) => {
                    assert!(!errors.is_empty());
                    assert!(errors.iter().any(|e| e.contains("not a valid number")));
                }
                _ => panic!("Expected NaN values to fail validation"),
            }
        }

        #[test]
        fn test_unordered_x_values() {
            let gf: GraphicalFunction = GraphicalFunctionData::xy_pairs(
                vec![0.0, 1.0, 0.5], // Not in ascending order
                vec![0.0, 0.3, 1.0],
                None,
            )
            .into();

            match gf.validate() {
                ValidationResult::Invalid(_, errors) => {
                    assert!(!errors.is_empty());
                    assert!(errors.iter().any(|e| e.contains("not in ascending order")));
                }
                _ => panic!("Expected unordered x-values to fail validation"),
            }
        }

        #[test]
        fn test_insufficient_discrete_points() {
            let gf = GraphicalFunction::new(
                None,
                Some(GraphicalFunctionType::Discrete),
                GraphicalFunctionData::uniform_scale(
                    (0.0, 1.0),
                    vec![0.5], // Only one point
                    None,
                ),
            );

            match gf.validate() {
                ValidationResult::Invalid(_, errors) => {
                    assert!(!errors.is_empty());
                    assert!(errors.iter().any(|e| e.contains("at least two y-values")));
                }
                _ => {
                    panic!("Expected discrete function with insufficient points to fail validation")
                }
            }
        }
    }
}
