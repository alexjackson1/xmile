pub mod containers;
pub mod core;
pub mod equation;
pub mod model;
pub mod namespace;

pub mod types;

pub use crate::core::Uid;
pub use crate::equation::{Expression, Identifier, MeasureUnit, NumericConstant, Operator};
pub use crate::namespace::Namespace;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
