#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeasureUnit {
    Atom(String),
    Fraction {
        numerator: Vec<MeasureUnit>,
        denominator: Vec<MeasureUnit>,
    },
}

pub trait Measure {
    fn units(&self) -> Option<&MeasureUnit>;
}
