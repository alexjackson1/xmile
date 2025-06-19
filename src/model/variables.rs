use crate::{Expression, Identifier, Measure, UnitOfMeasure};

pub trait Documentation {
    /// Returns the documentation if available.
    fn documentation(&self) -> Option<&String>;
}

pub trait Variable: Documentation {
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
    pub units: Option<UnitOfMeasure>,
}

impl Documentation for Stock {
    fn documentation(&self) -> Option<&String> {
        self.documentation.as_ref()
    }
}

impl Variable for Stock {
    fn name(&self) -> &Identifier {
        &self.name
    }
}

impl Measure for Stock {
    fn units(&self) -> Option<&UnitOfMeasure> {
        self.units.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Flow {
    pub name: Identifier,
    pub documentation: Option<String>,
    pub equation: Expression,
    pub units: Option<UnitOfMeasure>,
}

impl Documentation for Flow {
    fn documentation(&self) -> Option<&String> {
        self.documentation.as_ref()
    }
}

impl Measure for Flow {
    fn units(&self) -> Option<&UnitOfMeasure> {
        self.units.as_ref()
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
    pub units: Option<UnitOfMeasure>,
}

impl Documentation for Auxiliary {
    fn documentation(&self) -> Option<&String> {
        self.documentation.as_ref()
    }
}

impl Measure for Auxiliary {
    fn units(&self) -> Option<&UnitOfMeasure> {
        self.units.as_ref()
    }
}

impl Variable for Auxiliary {
    fn name(&self) -> &Identifier {
        &self.name
    }
}
