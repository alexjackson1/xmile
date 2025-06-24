pub mod containers;
pub mod core;
pub mod equation;
pub mod model;
pub mod namespace;
pub mod validation_utils;

pub mod types;
// pub mod xml;

#[cfg(test)]
mod test_utils;

pub use crate::containers::{Container, ContainerMut};
pub use crate::core::Uid;
pub use crate::equation::{
    Expression, Identifier, Measure, NumericConstant, Operator, UnitEquation, UnitOfMeasure,
};
pub use crate::model::vars::gf::{GraphicalFunction, GraphicalFunctionData, GraphicalFunctionType};

pub use crate::namespace::Namespace;

pub trait Interpolatable {
    fn interpolate_between(lower: f64, upper: f64, t: f64) -> f64 {
        lower + t * (upper - lower)
    }
}

impl Interpolatable for f64 {}

#[cfg(test)]
mod tests {
    use super::Interpolatable;

    #[test]
    fn test_interpolate_between() {
        let cases = vec![
            (0.0, 10.0, 0.0, 0.0),
            (0.0, 10.0, 1.0, 10.0),
            (0.0, 10.0, 0.5, 5.0),
            (1.0, 3.0, 0.25, 1.5),
            (1.0, 3.0, 0.75, 2.5),
            (-5.0, 5.0, 0.0, -5.0),
            (-5.0, 5.0, 1.0, 5.0),
            (-5.0, 5.0, 0.5, 0.0),
            (100.0, 200.0, 0.1, 110.0),
            (100.0, 200.0, 0.9, 190.0),
            (0.0, 1.0, 0.333, 0.333),
            (0.0, 1.0, 0.666, 0.666),
            (10.0, 20.0, 0.2, 12.0),
            (10.0, 20.0, 0.8, 18.0),
        ];
        for (lower, upper, t, expected) in cases {
            let result = f64::interpolate_between(lower, upper, t);
            assert!(
                (result - expected).abs() < f64::EPSILON,
                "Failed for case: ({}, {}, {})",
                lower,
                upper,
                t
            );
        }
    }
}
