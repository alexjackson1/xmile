// Helper function to assert floating point equality with tolerance
pub fn assert_float_eq(a: f64, b: f64, tolerance: f64) {
    assert!(
        (a - b).abs() < tolerance,
        "Expected {} to be approximately equal to {} (tolerance: {})",
        a,
        b,
        tolerance
    );
}
