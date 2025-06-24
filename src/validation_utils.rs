use std::fmt;

use crate::types::ValidationResult;

pub fn _chain<T>(
    result: ValidationResult<T>,
    warnings: &mut Vec<String>,
    errors: &mut Vec<String>,
) {
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

pub fn _return(warnings: Vec<String>, errors: Vec<String>) -> ValidationResult {
    if errors.is_empty() {
        ValidationResult::Valid(())
    } else {
        ValidationResult::Invalid(warnings, errors)
    }
}

pub fn _float_equals(a: f64, b: f64) -> bool {
    (a - b).abs() < f64::EPSILON
}

pub fn validate_ascending<V: PartialOrd + fmt::Display>(points: &[V]) -> ValidationResult {
    let warnings = Vec::new();
    let mut errors = Vec::new();

    // Check if values are in ascending order
    for i in 1..points.len() {
        if points[i] < points[i - 1] {
            errors.push(format!(
                "values are not in ascending order: {} > {} at index {}",
                points[i - 1],
                points[i],
                i
            ));
        }
    }

    _return(warnings, errors)
}

pub fn validate_non_empty(points: &[f64]) -> ValidationResult {
    let warnings = Vec::new();
    let mut errors = Vec::new();

    if points.is_empty() {
        errors.push("values cannot be empty.".to_string());
    }

    _return(warnings, errors)
}

pub fn validate_finite(points: &[f64]) -> ValidationResult {
    let warnings = Vec::new();
    let mut errors = Vec::new();

    // Check if all points are finite
    for (i, &value) in points.iter().enumerate() {
        if value.is_nan() || value.is_infinite() {
            errors.push(format!(
                "value at index {} is not a valid number: {}",
                i, value
            ));
        }
    }

    _return(warnings, errors)
}

pub fn validate_length<V>(points: &[V], expected_len: usize) -> ValidationResult {
    let warnings = Vec::new();
    let mut errors = Vec::new();

    if points.len() != expected_len {
        errors.push(format!(
            "expected length {}, but received {}",
            expected_len,
            points.len()
        ));
    }

    _return(warnings, errors)
}
