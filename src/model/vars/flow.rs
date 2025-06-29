use crate::{
    Expression, Identifier, Measure, UnitEquation,
    model::object::{DeviceRange, DeviceScale, Document, Documentation, FormatOptions, Object},
};

use super::Var;

#[derive(Debug, Clone, PartialEq)]
pub struct Flow {
    pub name: Identifier,
    pub equation: Expression,
    pub mathml_equation: Option<String>,
    pub units: Option<UnitEquation>,
    pub documentation: Option<Documentation>,
    pub range: Option<DeviceRange>,
    pub scale: Option<DeviceScale>,
    pub format: Option<FormatOptions>,
}

impl Var<'_> for Flow {
    fn name(&self) -> Option<&Identifier> {
        Some(&self.name)
    }

    fn equation(&self) -> Option<&Expression> {
        Some(&self.equation)
    }

    #[cfg(feature = "mathml")]
    fn mathml_equation(&self) -> Option<&String> {
        self.mathml_equation.as_ref()
    }
}

impl Object for Flow {
    fn range(&self) -> Option<&DeviceRange> {
        self.range.as_ref()
    }

    fn scale(&self) -> Option<&DeviceScale> {
        self.scale.as_ref()
    }

    fn format(&self) -> Option<&FormatOptions> {
        self.format.as_ref()
    }
}

impl Measure for Flow {
    fn units(&self) -> Option<&UnitEquation> {
        self.units.as_ref()
    }
}

impl Document for Flow {
    fn documentation(&self) -> Option<&Documentation> {
        self.documentation.as_ref()
    }
}
