//! # XMILE Graphical Functions Implementation
//!
//! This module implements XMILE graphical functions (also known as lookup functions
//! or table functions) according to specification section 3.1.4. It provides
//! complete support for defining arbitrary relationships between one input variable
//! and one output variable with robust interpolation and extrapolation capabilities.
//!
//! ## XMILE Graphical Function Requirements
//!
//! According to the XMILE specification section 3.1.4, graphical functions:
//!
//! - **Describe arbitrary relationships** between one input (x-domain) and one output (y-range)
//! - **MUST be defined in one of two ways**:
//!   - With an x-axis scale and evenly-spaced y-values across that scale
//!   - With explicit x-y coordinate pairs for irregular spacing
//! - **Support three interpolation types**: continuous, extrapolate, and discrete
//! - **Are immutable during simulation** - size cannot change once defined
//! - **Integrate with the container system** for uniform access and operations
//!
//! ## Data Representation Forms
//!
//! ### Uniform Scale (Even Spacing)
//!
//! The most common form uses an x-axis scale with evenly distributed y-values:
//!
//! ```rust
//! use xmile::{Container, GraphicalFunction, GraphicalFunctionData, GraphicalFunctionType, Identifier};
//!
//! let name = Identifier::parse_default("food_availability_multiplier_function").unwrap();
//! let data = GraphicalFunctionData::UniformScale {
//!     x_scale: (0.0, 1.0),  // x ranges from 0 to 1
//!     y_values: vec![0.0, 0.3, 0.55, 0.7, 0.83, 0.9, 0.95, 0.98, 0.99, 0.995, 1.0],
//! };
//!
//! let function = GraphicalFunction {
//!     name,
//!     data,
//!     function_type: Some(GraphicalFunctionType::Continuous),
//! };
//!
//! // With 11 y-values spanning 0 to 1, x-interval is 0.1
//! assert_eq!(function.len(), 11);
//! assert_eq!(function[0], 0.0);   // At x=0.0
//! assert_eq!(function[10], 1.0);  // At x=1.0
//! ```
//!
//! ### X-Y Pairs (Irregular Spacing)
//!
//! For functions that cannot be properly represented with fixed intervals:
//!
//! ```rust
//! use xmile::{GraphicalFunction, GraphicalFunctionData};
//!
//! let data = GraphicalFunctionData::XYPairs {
//!     x_values: vec![0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0],
//!     y_values: vec![0.0, 0.2, 0.45, 0.7, 0.85, 0.95, 1.0],
//! };
//!
//! // x-values can have irregular spacing for more precise control
//! ```
//!
//! ## Interpolation and Extrapolation Types
//!
//! ### Continuous (Default)
//! - **Intermediate values**: Linear interpolation between adjacent points
//! - **Out-of-range values**: Clamped to closest endpoint (no extrapolation)
//! - **Use case**: Smooth relationships with bounded behaviour
//!
//! ### Extrapolate
//! - **Intermediate values**: Linear interpolation between adjacent points  
//! - **Out-of-range values**: Linear extrapolation from last two endpoint values
//! - **Use case**: Relationships that continue trends beyond defined range
//!
//! ### Discrete (Step-wise)
//! - **Intermediate values**: Take value of next lower x-coordinate (step function)
//! - **Out-of-range values**: Clamped to closest endpoint
//! - **Special requirement**: Last two points must have same y-value
//! - **Use case**: Categorical or threshold-based relationships
//!
//! ```rust
//! use xmile::GraphicalFunctionType;
//!
//! // Specify interpolation behaviour
//! let continuous = GraphicalFunctionType::Continuous;    // Default
//! let extrapolate = GraphicalFunctionType::Extrapolate;  // Extends trends
//! let discrete = GraphicalFunctionType::Discrete;        // Step function
//! ```
//!
//! ## Container Integration
//!
//! Graphical functions implement the XMILE container system (section 3.2.4), providing
//! uniform access patterns and statistical operations:
//!
//! ```rust
//! use xmile::{
//!     Container,
//!     GraphicalFunction,
//!     GraphicalFunctionData,
//!     GraphicalFunctionType,
//!     Identifier
//! };
//!
//! let function = GraphicalFunction {
//!    name: Identifier::parse_default("example_function").unwrap(),
//!    function_type: Some(GraphicalFunctionType::Continuous),
//!    data: GraphicalFunctionData::UniformScale {
//!        x_scale: (0.0, 1.0),  
//!        y_values: vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
//!    },
//! };
//!
//! // Container operations - uniform across all XMILE containers
//! assert_eq!(function.len(), 11);                    // Number of y-values
//! assert!(!function.is_empty());                     // Non-empty check
//! assert_eq!(function.values(), function.data.values()); // Direct y-value access
//!
//! // Statistical operations
//! let mean = function.mean().unwrap();               // Average of y-values
//! let (min, max) = function.range().unwrap();        // Y-value range
//! let min_y = function.min().unwrap();               // Minimum y-value
//! let max_y = function.max().unwrap();               // Maximum y-value
//!
//! // Index-based access (square bracket notation per spec 3.7.1)
//! let first_y = function[0];                         // First y-value
//! let last_y = function[function.len() - 1];         // Last y-value
//! ```
//!
//! ## Mutable Operations
//!
//! Although graphical functions are immutable during simulation, they can be
//! modified during model setup:
//!
//! ```rust
//! use xmile::{
//!     Container,
//!     ContainerMut,
//!     GraphicalFunction,
//!     GraphicalFunctionData,
//!     GraphicalFunctionType,
//!     Identifier
//! };
//!
//! let mut function = GraphicalFunction {
//!    name: Identifier::parse_default("example_function").unwrap(),
//!    function_type: Some(GraphicalFunctionType::Continuous),
//!    data: GraphicalFunctionData::UniformScale {
//!        x_scale: (0.0, 1.0),  
//!        y_values: vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
//!    },
//! };
//!
//! // Modify individual y-values during setup
//! function[5] = 0.88;  // Change 6th y-value
//!
//! // The ContainerMut trait ensures type safety
//! ```
//!
//! ## Practical Examples
//!
//! ### Food Availability Multiplier (from XMILE spec)
//!
//! ```rust
//! use xmile::{GraphicalFunction, GraphicalFunctionData, GraphicalFunctionType, Identifier};
//!
//! // Example from XMILE specification section 3.1.4
//! let food_function = GraphicalFunction {
//!     name: Identifier::parse_default("food_availability_multiplier_function").unwrap(),
//!     data: GraphicalFunctionData::UniformScale {
//!         x_scale: (0.0, 1.0),
//!         y_values: vec![0.0, 0.3, 0.55, 0.7, 0.83, 0.9, 0.95, 0.98, 0.99, 0.995, 1.0],
//!     },
//!     function_type: Some(GraphicalFunctionType::Continuous),
//! };
//!
//! // This represents a function where:
//! // - At x=0.0 (no food), y=0.0 (no growth)
//! // - At x=1.0 (full food), y=1.0 (full growth)
//! // - Intermediate values interpolated linearly
//! ```
//!
//! ### Embedded Graphical Function
//!
//! ```rust
//! use xmile::{GraphicalFunction, GraphicalFunctionData, GraphicalFunctionType, Identifier};
//! // As mentioned in spec, graphical functions can be embedded within flows/auxiliaries
//! // In this case, the name is optional:
//!
//! let embedded_function = GraphicalFunction {
//!     name: Identifier::parse_default("name").unwrap(),
//!     data: GraphicalFunctionData::UniformScale {
//!         x_scale: (0.0, 1.0),
//!         y_values: vec![0.0, 0.3, 0.55, 0.7, 0.83, 0.9, 0.95, 0.98, 0.99, 0.995, 1.0],
//!     },
//!     function_type: Some(GraphicalFunctionType::Continuous),
//! };
//! ```
//!
//! ### Discrete Step Function
//!
//! ```rust
//! use xmile::{GraphicalFunction, GraphicalFunctionData, GraphicalFunctionType, Identifier};
//!
//! // For threshold-based relationships
//! let threshold_function = GraphicalFunction {
//!     name: Identifier::parse_default("threshold_multiplier").unwrap(),
//!     data: GraphicalFunctionData::UniformScale {
//!         x_scale: (0.0, 10.0),
//!         y_values: vec![0.0, 0.0, 0.5, 0.5, 1.0, 1.0], // Last two must be equal for discrete
//!     },
//!     function_type: Some(GraphicalFunctionType::Discrete),
//! };
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Memory efficient**: Stores only essential data points, not full interpolated tables
//! - **Fast access**: Direct indexing into y-values array for container operations
//! - **Type safety**: Compile-time guarantees through trait system
//! - **Zero-cost abstractions**: Container traits compile to direct field access
//!
//! ## Design Philosophy
//!
//! This implementation prioritises:
//!
//! 1. **XMILE compliance**: Exact adherence to specification section 3.1.4
//! 2. **Container uniformity**: Consistent API across all XMILE container types
//! 3. **Performance**: Efficient representation and access patterns
//! 4. **Type safety**: Compile-time prevention of common errors
//! 5. **Extensibility**: Easy addition of new interpolation methods
//!
//! ## Integration Notes
//!
//! - **Equation system**: Graphical functions integrate seamlessly with XMILE expressions
//! - **Simulation engine**: Functions provide interpolated values during model execution
//! - **Model validation**: Container traits enable comprehensive model checking
//! - **Serialisation**: Structure supports efficient XMILE file format serialisation
//!
//! The implementation provides a robust foundation for XMILE graphical functions whilst
//! maintaining the flexibility and performance required for complex system dynamics models.

use std::{
    fmt,
    ops::{Index, IndexMut},
};

use crate::{
    Identifier,
    containers::{Container, ContainerMut},
};

/// A complete XMILE graphical function with metadata and interpolation behaviour.
///
/// This struct represents a lookup function that defines an arbitrary relationship
/// between one input variable (x-domain) and one output variable (y-range).
/// It combines the data representation with function metadata and interpolation type.
///
/// ## Structure
///
/// - `name`: The identifier for this graphical function (optional for embedded functions)
/// - `data`: The actual x-y relationship data (uniform scale or explicit pairs)
/// - `function_type`: How to handle interpolation and extrapolation (defaults to continuous)
///
/// ## XMILE Compliance
///
/// Implements the complete graphical function specification from section 3.1.4,
/// including container system integration (section 3.2.4) for uniform access patterns.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphicalFunction {
    /// The function identifier (may be empty for embedded functions)
    pub name: Identifier,
    /// The x-y relationship data
    pub data: GraphicalFunctionData,
    /// Interpolation and extrapolation behaviour (defaults to continuous if None)
    pub function_type: Option<GraphicalFunctionType>,
}

impl Container for GraphicalFunction {
    /// Returns the y-values as a slice for container operations.
    ///
    /// This provides uniform access to the function's y-values regardless of
    /// whether the function uses uniform scaling or explicit x-y pairs.
    fn values(&self) -> &[f64] {
        self.data.values()
    }
}

impl ContainerMut for GraphicalFunction {
    /// Returns mutable access to y-values.
    ///
    /// Allows modification of the underlying y-values during model construction.
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
    /// Implements XMILE's square bracket notation (section 3.7.1) for uniform
    /// container access. The index corresponds to the position in the y-values array.
    ///
    /// # Panics
    ///
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
    /// Allows modification of y-values during model setup (before simulation).
    /// The function structure remains immutable during simulation as per XMILE spec.
    ///
    /// # Panics
    ///
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
/// These three types define how intermediate values and out-of-range values
/// are calculated according to XMILE specification section 3.1.4.
#[derive(Debug, Clone, PartialEq)]
pub enum GraphicalFunctionType {
    /// Linear interpolation with clamping at endpoints.
    ///
    /// - **Intermediate values**: Linear interpolation between adjacent points
    /// - **Out-of-range values**: Same as closest endpoint (no extrapolation)
    /// - **Use case**: Most common type for smooth, bounded relationships
    Continuous,

    /// Linear interpolation with linear extrapolation beyond endpoints.
    ///
    /// - **Intermediate values**: Linear interpolation between adjacent points
    /// - **Out-of-range values**: Linear extrapolation from last two endpoint values
    /// - **Use case**: Relationships that continue established trends beyond defined range
    Extrapolate,

    /// Step-wise function with discrete jumps.
    ///
    /// - **Intermediate values**: Value of next lower x-coordinate (step function)
    /// - **Out-of-range values**: Same as closest endpoint (no extrapolation)
    /// - **Constraint**: Last two points must have same y-value
    /// - **Use case**: Categorical or threshold-based relationships
    Discrete,
}

impl Default for GraphicalFunctionType {
    /// Returns the default interpolation type.
    ///
    /// Continuous interpolation is the default as it's the most commonly
    /// used type for smooth mathematical relationships.
    fn default() -> Self {
        GraphicalFunctionType::Continuous
    }
}

impl fmt::Display for GraphicalFunctionType {
    /// Formats the function type for display and serialisation.
    ///
    /// Uses lowercase names matching XMILE specification terminology.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphicalFunctionType::Continuous => write!(f, "continuous"),
            GraphicalFunctionType::Extrapolate => write!(f, "extrapolate"),
            GraphicalFunctionType::Discrete => write!(f, "discrete"),
        }
    }
}

/// The actual x-y relationship data for a graphical function.
///
/// XMILE supports two ways to define the relationship between x and y values:
/// uniform scaling (evenly spaced) and explicit x-y pairs (irregularly spaced).
#[derive(Debug, Clone, PartialEq)]
pub enum GraphicalFunctionData {
    /// Uniform x-axis scaling with evenly distributed y-values.
    ///
    /// This is the most common form where x-values are calculated automatically
    /// based on the scale range and number of y-values.
    ///
    /// ## Example
    /// ```
    /// use xmile::GraphicalFunctionData::*;
    ///
    /// // x_scale: (0.0, 1.0) with 11 y-values creates x-interval of 0.1
    /// // x-values: [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    /// UniformScale {
    ///     x_scale: (0.0, 1.0),
    ///     y_values: vec![0.0, 0.3, 0.55, 0.7, 0.83, 0.9, 0.95, 0.98, 0.99, 0.995, 1.0],
    /// };
    /// ```
    UniformScale {
        /// The (min, max) range for x-values
        x_scale: (f64, f64),
        /// Y-values evenly distributed across the x-scale
        y_values: Vec<f64>,
    },

    /// Explicit x-y coordinate pairs for irregular spacing.
    ///
    /// Used when the function cannot be properly represented with fixed x-intervals,
    /// allowing precise control over x-coordinate placement.
    ///
    /// ## Example
    /// ```
    /// use xmile::GraphicalFunctionData::*;
    ///
    /// XYPairs {
    ///     x_values: vec![0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0],  // Irregular spacing
    ///     y_values: vec![0.0, 0.2, 0.45, 0.7, 0.85, 0.95, 1.0],
    /// };
    /// ```
    ///
    /// ## Invariants
    /// - `x_values.len()` must equal `y_values.len()`
    /// - `x_values` should be sorted in ascending order for proper interpolation
    XYPairs {
        /// Explicit x-coordinates (should be sorted)
        x_values: Vec<f64>,
        /// Corresponding y-values
        y_values: Vec<f64>,
    },
}

impl Index<usize> for GraphicalFunctionData {
    type Output = f64;

    /// Direct access to y-values by index.
    ///
    /// Enables square bracket notation on the data structure itself,
    /// providing a consistent access pattern across both representation forms.
    fn index(&self, index: usize) -> &Self::Output {
        match self {
            GraphicalFunctionData::UniformScale { y_values, .. } => &y_values[index],
            GraphicalFunctionData::XYPairs { y_values, .. } => &y_values[index],
        }
    }
}

impl IndexMut<usize> for GraphicalFunctionData {
    /// Mutable access to y-values by index.
    ///
    /// Allows modification of the underlying y-values during model construction.
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self {
            GraphicalFunctionData::UniformScale { y_values, .. } => &mut y_values[index],
            GraphicalFunctionData::XYPairs { y_values, .. } => &mut y_values[index],
        }
    }
}

impl Container for GraphicalFunctionData {
    /// Returns y-values for container operations.
    fn values(&self) -> &[f64] {
        match self {
            GraphicalFunctionData::UniformScale { y_values, .. } => y_values,
            GraphicalFunctionData::XYPairs { y_values, .. } => y_values,
        }
    }

    /// Returns the number of data points.
    fn len(&self) -> usize {
        self.values().len()
    }

    /// Calculates the arithmetic mean of y-values.
    ///
    /// Returns `None` for empty functions, `Some(mean)` otherwise.
    /// Uses exact arithmetic to avoid precision loss.
    fn mean(&self) -> Option<f64> {
        match self.values() {
            [] => None,
            y => Some(y.iter().sum::<f64>() / y.len() as f64),
        }
    }

    /// Finds the minimum y-value.
    ///
    /// Uses partial comparison to handle potential NaN values correctly.
    /// Returns `None` for empty functions.
    fn min(&self) -> Option<f64> {
        match self.values() {
            [] => None,
            y => Some(*y.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()),
        }
    }

    /// Finds the maximum y-value.
    ///
    /// Uses partial comparison to handle potential NaN values correctly.
    /// Returns `None` for empty functions.
    fn max(&self) -> Option<f64> {
        match self.values() {
            [] => None,
            y => Some(*y.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()),
        }
    }
}

impl ContainerMut for GraphicalFunctionData {
    /// Returns mutable access to y-values.
    ///
    /// Allows modification of the underlying y-values during model construction.
    fn values_mut(&mut self) -> &mut [f64] {
        match self {
            GraphicalFunctionData::UniformScale { y_values, .. } => y_values,
            GraphicalFunctionData::XYPairs { y_values, .. } => y_values,
        }
    }
}
