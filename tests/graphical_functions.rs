#![cfg(test)]

mod test_utils;

use test_utils::assert_float_eq;

use xmile::{
    GraphicalFunction, GraphicalFunctionData, GraphicalFunctionType, Identifier,
    types::{Validate, ValidationResult},
};
#[test]
fn test_xmile_spec_example_complete() {
    // Complete example from XMILE specification
    let food_availability = GraphicalFunction {
        name: Some(Identifier::parse_default("food_availability_multiplier_function").unwrap()),
        data: GraphicalFunctionData::uniform_scale(
            (0.0, 1.0),
            vec![0.0, 0.3, 0.55, 0.7, 0.83, 0.9, 0.95, 0.98, 0.99, 0.995, 1.0],
            None,
        ),
        function_type: Some(GraphicalFunctionType::Continuous),
    };

    // Validate the function
    assert!(matches!(
        food_availability.validate(),
        ValidationResult::Valid(_)
    ));

    // Test some known interpolations
    assert_float_eq(food_availability.evaluate(0.0), 0.0, 1e-10);
    assert_float_eq(food_availability.evaluate(1.0), 1.0, 1e-10);

    // Test interpolation at quarter points
    let quarter_value = food_availability.evaluate(0.25);
    assert!(quarter_value > 0.55 && quarter_value < 0.7);
}

#[test]
fn test_equivalent_representations() {
    #[cfg(test)]
    let uniform = GraphicalFunction {
        name: None,
        data: GraphicalFunctionData::uniform_scale((0.0, 1.0), vec![0.0, 0.5, 1.0], None),
        function_type: Some(GraphicalFunctionType::Continuous),
    };

    let xy_pairs = GraphicalFunction {
        name: None,
        data: GraphicalFunctionData::xy_pairs(vec![0.0, 0.5, 1.0], vec![0.0, 0.5, 1.0], None),
        function_type: Some(GraphicalFunctionType::Continuous),
    };

    // Test multiple evaluation points
    let test_points = vec![-0.5, 0.0, 0.25, 0.5, 0.75, 1.0, 1.5];
    for x in test_points {
        println!("Testing x = {}", x);
        assert_float_eq(uniform.evaluate(x), xy_pairs.evaluate(x), 1e-10);
    }
}
