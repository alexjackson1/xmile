use crate::{
    Expression, Identifier, Measure, UnitOfMeasure,
    model::object::{Document, Documentation, FormatOptions, Object, Range, Scale},
};

use super::Var;

#[derive(Debug, Clone, PartialEq)]
pub struct Flow {
    pub name: Identifier,
    pub equation: Expression,
    pub mathml_equation: Option<String>,
    pub units: Option<UnitOfMeasure>,
    pub documentation: Option<Documentation>,
    pub range: Option<Range>,
    pub scale: Option<Scale>,
    pub format: Option<FormatOptions>,
}

impl Var<'_> for Flow {
    fn name(&self) -> &Identifier {
        &self.name
    }

    fn equation(&self) -> Option<&Expression> {
        Some(&self.equation)
    }
}

impl Object for Flow {
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

impl Measure for Flow {
    fn units(&self) -> Option<&UnitOfMeasure> {
        self.units.as_ref()
    }
}

impl Document for Flow {
    fn documentation(&self) -> Option<&Documentation> {
        self.documentation.as_ref()
    }
}
