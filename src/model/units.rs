#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unit {
    Atom(String),
    Fraction {
        numerator: Vec<Unit>,
        denominator: Vec<Unit>,
    },
}
