use serde::{Deserialize, Serialize};

use crate::{
    Identifier,
    model::object::{Document, Documentation, Object},
};

use super::Var;

/// A module is a placeholder in the variables section for a submodel.
/// Modules connect submodel inputs and outputs to variables in the parent model.
///
/// According to XMILE spec section 4.7.1, a module has:
/// - Name: name="…" attribute (REQUIRED)
/// - Resource: resource="…" attribute (OPTIONAL) - reference to submodel file
/// - Connections: <connect to="…" from="…"/> tags (OPTIONAL)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    /// The name of the module (must match submodel name if resource not specified).
    #[serde(rename = "@name")]
    pub name: Identifier,

    /// Optional resource reference to the submodel's file (URL, relative, or absolute path).
    #[serde(rename = "@resource")]
    pub resource: Option<String>,

    /// Connections between this module and the parent model.
    /// Each connection maps a submodel input (to) to a submodel output (from).
    #[serde(rename = "connect", default)]
    pub connections: Vec<ModuleConnection>,

    /// Optional documentation for the module.
    #[serde(rename = "doc")]
    pub documentation: Option<Documentation>,
}

/// A connection between a module and the parent model.
/// Maps a submodel input to a submodel output.
///
/// According to XMILE spec section 4.7.1:
/// - `to`: The name of the submodel input within this submodel (or qualified with submodel name)
/// - `from`: The qualified name of the submodel output being assigned to that input
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleConnection {
    /// The name of the submodel input that is being assigned.
    #[serde(rename = "@to")]
    pub to: String,

    /// The qualified name of the submodel output that is being assigned to the input.
    #[serde(rename = "@from")]
    pub from: String,
}

impl Var<'_> for Module {
    fn name(&self) -> Option<&Identifier> {
        Some(&self.name)
    }

    fn equation(&self) -> Option<&crate::Expression> {
        None // Modules don't have equations
    }

    #[cfg(feature = "mathml")]
    fn mathml_equation(&self) -> Option<&String> {
        None
    }
}

impl Object for Module {
    fn range(&self) -> Option<&crate::model::object::DeviceRange> {
        None // Modules don't have ranges
    }

    fn scale(&self) -> Option<&crate::model::object::DeviceScale> {
        None // Modules don't have scales
    }

    fn format(&self) -> Option<&crate::model::object::FormatOptions> {
        None // Modules don't have format options
    }
}

impl crate::Measure for Module {
    fn units(&self) -> Option<&crate::UnitEquation> {
        None // Modules don't have units
    }
}

impl Document for Module {
    fn documentation(&self) -> Option<&Documentation> {
        self.documentation.as_ref()
    }
}
