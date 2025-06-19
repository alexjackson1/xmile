use crate::Identifier;

use super::{Expression, Unit};

pub trait Documentation {
    /// Returns the documentation if available.
    fn documentation(&self) -> Option<&String>;
}

pub trait UnitOfMeasure {
    /// Returns the unit of measure.
    fn unit(&self) -> Option<&Unit>;
}

pub trait Variable: Documentation + UnitOfMeasure {
    /// Returns the name of the variable.
    fn name(&self) -> &Identifier;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stock {
    pub name: Identifier,
    pub documentation: Option<String>,
    pub inflows: Vec<Identifier>,
    pub outflows: Vec<Identifier>,
    pub initial_equation: Expression,
    pub units: Unit,
}

impl Documentation for Stock {
    fn documentation(&self) -> Option<&String> {
        self.documentation.as_ref()
    }
}

impl UnitOfMeasure for Stock {
    fn unit(&self) -> Option<&Unit> {
        Some(&self.units)
    }
}

impl Variable for Stock {
    fn name(&self) -> &Identifier {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Flow {
    pub name: Identifier,
    pub documentation: Option<String>,
    pub equation: Expression,
    pub units: Unit,
}

impl Documentation for Flow {
    fn documentation(&self) -> Option<&String> {
        self.documentation.as_ref()
    }
}

impl UnitOfMeasure for Flow {
    fn unit(&self) -> Option<&Unit> {
        Some(&self.units)
    }
}

impl Variable for Flow {
    fn name(&self) -> &Identifier {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Auxiliary {
    pub name: Identifier,
    pub documentation: Option<String>,
    pub equation: Expression,
    pub units: Unit,
}

impl Documentation for Auxiliary {
    fn documentation(&self) -> Option<&String> {
        self.documentation.as_ref()
    }
}

impl UnitOfMeasure for Auxiliary {
    fn unit(&self) -> Option<&Unit> {
        Some(&self.units)
    }
}

impl Variable for Auxiliary {
    fn name(&self) -> &Identifier {
        &self.name
    }
}
