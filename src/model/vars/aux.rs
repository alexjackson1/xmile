use crate::{
    Expression, Identifier, Measure, UnitEquation,
    model::object::{DeviceRange, DeviceScale, Document, Documentation, FormatOptions, Object},
};

use super::Var;

#[derive(Debug, Clone, PartialEq)]
pub struct Auxiliary {
    pub name: Identifier,
    pub documentation: Option<Documentation>,
    pub equation: Expression,
    #[cfg(feature = "mathml")]
    pub mathml_equation: Option<String>,
    pub units: Option<UnitEquation>,
    pub range: Option<DeviceRange>,
    pub scale: Option<DeviceScale>,
    pub format: Option<FormatOptions>,
}

impl Var<'_> for Auxiliary {
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

impl Object for Auxiliary {
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

impl Measure for Auxiliary {
    fn units(&self) -> Option<&UnitEquation> {
        self.units.as_ref()
    }
}

impl Document for Auxiliary {
    fn documentation(&self) -> Option<&Documentation> {
        self.documentation.as_ref()
    }
}
