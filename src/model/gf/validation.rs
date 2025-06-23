use super::{
    GraphicalFunction, GraphicalFunctionData, GraphicalFunctionScale, GraphicalFunctionType,
    Points, Validate, ValidationResult,
};

fn _chain<T>(result: ValidationResult<T>, warnings: &mut Vec<String>, errors: &mut Vec<String>) {
    match result {
        ValidationResult::Valid(_) => {}
        ValidationResult::Warnings(_, warns) => {
            warnings.extend(warns);
        }
        ValidationResult::Invalid(warns, errs) => {
            warnings.extend(warns);
            errors.extend(errs);
        }
    }
}

fn _return(warnings: Vec<String>, errors: Vec<String>) -> ValidationResult {
    if errors.is_empty() {
        ValidationResult::Valid(())
    } else {
        ValidationResult::Invalid(warnings, errors)
    }
}

fn _float_equals(a: f64, b: f64) -> bool {
    (a - b).abs() < f64::EPSILON
}

pub fn validate(gf: &GraphicalFunction) -> ValidationResult {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    _chain(gf.data.validate(), &mut warnings, &mut errors);

    if matches!(gf.function_type(), GraphicalFunctionType::Discrete) {
        _chain(validate_discrete(&gf.data), &mut warnings, &mut errors);
    }

    _return(warnings, errors)
}

pub fn validate_data(data: &GraphicalFunctionData) -> ValidationResult {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let w = &mut warnings;
    let e = &mut errors;

    match data {
        GraphicalFunctionData::UniformScale {
            x_scale,
            y_values,
            y_scale,
        } => {
            _chain(validate_y_values(y_values), w, e);
            _chain(validate_x_scale(&Some(*x_scale)), w, e);
            _chain(validate_y_scale(y_scale), w, e);
        }
        GraphicalFunctionData::XYPairs {
            x_values,
            y_values,
            y_scale,
        } => {
            _chain(validate_x_values(x_values, y_values.len()), w, e);
            _chain(validate_y_values(y_values), w, e);
            _chain(validate_y_scale(y_scale), w, e);
        }
    }

    _return(warnings, errors)
}

pub fn validate_scale(scale: &GraphicalFunctionScale) -> ValidationResult {
    let warnings = Vec::new();
    let mut errors = Vec::new();

    if scale.min > scale.max {
        errors.push("Scale minimum cannot be greater than maximum.".to_string());
    }

    if scale.min.is_nan() || scale.max.is_nan() {
        errors.push("Scale values cannot be NaN.".to_string());
    }

    if scale.min.is_infinite() || scale.max.is_infinite() {
        errors.push("Scale values cannot be infinite.".to_string());
    }

    _return(warnings, errors)
}

pub fn validate_points(points: &Points) -> ValidationResult {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    _chain(validate_finite(points), &mut warnings, &mut errors);
    _return(warnings, errors)
}

fn validate_x_values(x_values: &Points, y_len: usize) -> ValidationResult {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let w = &mut warnings;
    let e = &mut errors;

    _chain(x_values.validate(), w, e);
    _chain(validate_length(x_values, y_len), w, e);
    _chain(validate_order(x_values), w, e);
    _return(warnings, errors)
}

fn validate_y_values(y_values: &Points) -> ValidationResult {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let w = &mut warnings;
    let e = &mut errors;

    _chain(y_values.validate(), w, e);
    _chain(validate_non_empty(y_values), w, e);
    _return(warnings, errors)
}

fn validate_x_scale(x_scale: &Option<GraphicalFunctionScale>) -> ValidationResult {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    match x_scale {
        Some(scale) => _chain(scale.validate(), &mut warnings, &mut errors),
        None => {}
    }

    _return(warnings, errors)
}

fn validate_y_scale(y_scale: &Option<GraphicalFunctionScale>) -> ValidationResult {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    match y_scale {
        Some(scale) => _chain(scale.validate(), &mut warnings, &mut errors),
        None => {}
    }

    _return(warnings, errors)
}

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
            } else if !_float_equals(y_values[y_values.len() - 1], y_values[y_values.len() - 2]) {
                errors.push(
                    "Last two points must have the same value for discrete functions.".into(),
                );
            }
        }
    }

    _return(warnings, errors)
}

fn validate_order(points: &[f64]) -> ValidationResult {
    let warnings = Vec::new();
    let mut errors = Vec::new();

    // Check if points are in ascending order
    for i in 1..points.len() {
        if points[i] < points[i - 1] {
            errors.push(format!(
                "Points are not in ascending order: {} > {} at index {}",
                points[i - 1],
                points[i],
                i
            ));
        }
    }

    _return(warnings, errors)
}

fn validate_non_empty(points: &[f64]) -> ValidationResult {
    let warnings = Vec::new();
    let mut errors = Vec::new();

    if points.is_empty() {
        errors.push("Points cannot be empty.".to_string());
    }

    _return(warnings, errors)
}

fn validate_finite(points: &[f64]) -> ValidationResult {
    let warnings = Vec::new();
    let mut errors = Vec::new();

    // Check if all points are finite
    for (i, &value) in points.iter().enumerate() {
        if value.is_nan() || value.is_infinite() {
            errors.push(format!(
                "Point at index {} is not a valid number: {}",
                i, value
            ));
        }
    }

    _return(warnings, errors)
}

fn validate_length(points: &[f64], expected_len: usize) -> ValidationResult {
    let warnings = Vec::new();
    let mut errors = Vec::new();

    if points.len() != expected_len {
        errors.push(format!(
            "Expected {} points, but found {}",
            expected_len,
            points.len()
        ));
    }

    _return(warnings, errors)
}

#[cfg(test)]

mod tests {
    use crate::Identifier;

    use super::*;

    #[test]
    fn test_valid_uniform_scale_function() {
        let gf = GraphicalFunction {
            name: Some(Identifier::parse_default("valid_function").unwrap()),
            data: GraphicalFunctionData::uniform_scale(
                (0.0, 1.0),
                vec![0.0, 0.5, 1.0],
                Some((0.0, 1.0)),
            ),
            function_type: Some(GraphicalFunctionType::Continuous),
        };

        match gf.validate() {
            ValidationResult::Valid(_) => {} // Expected
            _ => panic!("Expected valid function to pass validation"),
        }
    }

    #[test]
    fn test_valid_xy_pairs_function() {
        let gf = GraphicalFunction {
            name: None,
            data: GraphicalFunctionData::xy_pairs(vec![0.0, 0.5, 1.0], vec![0.0, 0.3, 1.0], None),
            function_type: Some(GraphicalFunctionType::Continuous),
        };

        match gf.validate() {
            ValidationResult::Valid(_) => {} // Expected
            _ => panic!("Expected valid function to pass validation"),
        }
    }

    #[test]
    fn test_invalid_discrete_function() {
        let gf = GraphicalFunction {
            name: None,
            data: GraphicalFunctionData::uniform_scale(
                (0.0, 1.0),
                vec![0.0, 0.5, 1.0], // Last two values different
                None,
            ),
            function_type: Some(GraphicalFunctionType::Discrete),
        };

        match gf.validate() {
            ValidationResult::Invalid(_, errors) => {
                assert!(!errors.is_empty());
                assert!(errors.iter().any(|e| e.contains("same value")));
            }
            _ => panic!("Expected discrete function with different last values to fail validation"),
        }
    }

    #[test]
    fn test_valid_discrete_function() {
        let gf = GraphicalFunction {
            name: None,
            data: GraphicalFunctionData::uniform_scale(
                (0.0, 1.0),
                vec![0.0, 0.5, 0.8, 0.8], // Last two values same
                None,
            ),
            function_type: Some(GraphicalFunctionType::Discrete),
        };

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
        let points = Points::new(vec![0.0, f64::NAN, 1.0], None);

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
        let gf = GraphicalFunction {
            name: None,
            data: GraphicalFunctionData::xy_pairs(
                vec![0.0, 1.0, 0.5], // Not in ascending order
                vec![0.0, 0.3, 1.0],
                None,
            ),
            function_type: Some(GraphicalFunctionType::Continuous),
        };

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
        let gf = GraphicalFunction {
            name: None,
            data: GraphicalFunctionData::uniform_scale(
                (0.0, 1.0),
                vec![0.5], // Only one point
                None,
            ),
            function_type: Some(GraphicalFunctionType::Discrete),
        };

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
