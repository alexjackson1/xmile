use std::ops::{Index, IndexMut};

use super::{GraphicalFunctionType, Points, GraphicalFunctionScale, Validate, ValidationResult, validation};

/// X-y relationship data for graphical functions.
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
        y_values: Points,
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
        x_values: Points,
        /// Corresponding y-values
        y_values: Points,
    },
}

// GraphicalFunctionData Construction

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
    pub fn xy_pairs(x_values: Vec<f64>, y_values: Vec<f64>, y_scale: Option<(f64, f64)>) -> Self {
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
}

impl GraphicalFunctionData {
    pub fn len(&self) -> usize {
        match self {
            GraphicalFunctionData::UniformScale { y_values, .. } => y_values.len(),
            GraphicalFunctionData::XYPairs { y_values, .. } => y_values.len(),
        }
    }

    pub fn evaluate(&self, function_type: GraphicalFunctionType, x: f64) -> f64 {
        match function_type {
            GraphicalFunctionType::Discrete => self.evaluate_discrete(x),
            GraphicalFunctionType::Continuous => self.evaluate_continuous(x),
            GraphicalFunctionType::Extrapolate => self.evaluate_extrapolate(x),
        }
    }

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
}

// Validation

impl Validate for GraphicalFunctionData {
    /// Validates the graphical function data.
    ///
    /// # Returns
    /// - `Valid(())` if the data is valid.
    /// - `Invalid(warnings, errors)` if there are validation issues.
    fn validate(&self) -> ValidationResult {
        validation::validate_data(self)
    }
}

// Container

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_scale_data_creation() {
        let data =
            GraphicalFunctionData::uniform_scale((0.0, 1.0), vec![0.0, 0.5, 1.0], Some((0.0, 1.0)));

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
        let data = GraphicalFunctionData::xy_pairs(vec![0.0, 0.5, 1.0], vec![0.0, 0.3, 1.0], None);

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
        let data =
            GraphicalFunctionData::uniform_scale((0.0, 1.0), vec![0.2, 0.8, 0.5], Some((0.0, 1.0)));

        let scale = data.y_scale().unwrap();
        assert_eq!(scale.min, 0.0);
        assert_eq!(scale.max, 1.0);
    }
}
