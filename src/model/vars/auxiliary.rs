use serde::{Deserialize, Serialize};

use crate::{
    Expression, Identifier, Measure, UnitEquation,
    model::{
        events::EventPoster,
        object::{DeviceRange, DeviceScale, Document, Documentation, FormatOptions, Object},
        vars::AccessType,
    },
};

#[cfg(feature = "arrays")]
use crate::model::vars::array::{ArrayElement, VariableDimensions};

use super::Var;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "aux")]
pub struct Auxiliary {
    #[serde(rename = "@name")]
    pub name: Identifier,
    #[serde(rename = "@access")]
    pub access: Option<AccessType>,
    #[serde(rename = "@autoexport")]
    pub autoexport: Option<bool>,
    pub documentation: Option<Documentation>,
    #[serde(rename = "eqn")]
    pub equation: Expression,
    #[cfg(feature = "mathml")]
    pub mathml_equation: Option<String>,
    pub units: Option<UnitEquation>,
    pub range: Option<DeviceRange>,
    pub scale: Option<DeviceScale>,
    pub format: Option<FormatOptions>,

    /// The dimensions for this auxiliary variable (if it's an array).
    #[cfg(feature = "arrays")]
    #[serde(rename = "dimensions")]
    pub dimensions: Option<VariableDimensions>,

    /// Array elements for non-apply-to-all arrays.
    #[cfg(feature = "arrays")]
    #[serde(rename = "element", default)]
    pub elements: Vec<ArrayElement>,

    /// Optional event poster for triggering events based on auxiliary values.
    #[serde(rename = "event_poster")]
    pub event_poster: Option<EventPoster>,
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
