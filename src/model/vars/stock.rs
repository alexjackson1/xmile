use crate::{
    Expression, Identifier, Measure, UnitOfMeasure,
    model::object::{Document, Documentation, FormatOptions, Object, Range, Scale},
};

use super::Var;

#[derive(Debug, Clone, PartialEq)]
pub struct Stock {
    /// The name of the stock variable.
    pub name: Identifier,

    /// The inflows to the stock variable.
    pub inflows: Vec<Identifier>,

    /// The outflows from the stock variable.
    pub outflows: Vec<Identifier>,

    /// The equation defining the stock's initial value.
    pub initial_equation: Expression,

    /// The units of measure for the stock variable.
    pub units: Option<UnitOfMeasure>,

    /// The documentation for the stock variable.
    pub documentation: Option<Documentation>,

    /// The range of values for the stock variable.
    pub range: Option<Range>,

    /// The scale of the stock variable.
    pub scale: Option<Scale>,

    /// The format options for the stock variable.
    pub format: Option<FormatOptions>,
}

impl Object for Stock {
    fn range(&self) -> Option<&Range> {
        self.range.as_ref()
    }

    fn scale(&self) -> Option<&Scale> {
        self.scale.as_ref()
    }

    fn format(&self) -> Option<&FormatOptions> {
        self.format.as_ref()
    }
}

impl Document for Stock {
    fn documentation(&self) -> Option<&Documentation> {
        self.documentation.as_ref()
    }
}

impl Measure for Stock {
    fn units(&self) -> Option<&UnitOfMeasure> {
        self.units.as_ref()
    }
}
