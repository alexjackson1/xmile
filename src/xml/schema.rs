use serde::{Deserialize, Deserializer, Serialize};

fn default_xmlns() -> String {
    "http://docs.oasis-open.org/xmile/ns/XMILE/v1.0".to_string()
}

use crate::{
    behavior::Behavior, data::Data, dimensions::Dimensions, header::Header, 
    model::vars::Variable,
    model::vars::flow::Flow,
    specs::SimulationSpecs, units::ModelUnits, view::{Style, View},
    types::{Validate, ValidationResult},
    xml::validation::*,
};

#[cfg(feature = "macros")]
use crate::r#macro::Macro;

/// A XMILE file contains information about a whole-model, with a
/// well-specified structure. The file MUST be encoded in UTF-8. The entire
/// XMILE file is enclosed within a <xmile> tag as follows:
///
/// ```xml
/// <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
///    ...
/// </xmile>
/// ```
///
/// The version number MUST refer to the version of XMILE used (presently 1.0).
/// The XML namespace refers to tags and attributes used in this specification.
/// Both of these attributes are REQUIRED. Inside of the <xmile> tag are a
/// number of top-level tags, listed below. These tags are marked req (a single
/// instance is REQUIRED), opt (a single instance is OPTIONAL), * (zero or more
/// tags MAY occur) and + (one or more tags MAY occur). Top level tags MAY
/// occur in any order, but are RECOMMENDED to occur in the following order:
///
/// - `<header>` (req) - information about the origin of the file and required
///   capabilities.
/// - `<sim_specs>` (opt) - default simulation specifications for this file.
/// - `<model_units>` (opt) - definitions of model units used in this file.
/// - `<dimensions>` (opt) - definitions of array dimensions specific to this
///   file.
/// - `<behavior>` (opt) - simulation style definitions that are
///   inherited/cascaded through all models defined in this XMILE file.
/// - `<style>` (opt) - display style definitions that are inherited/cascaded
///   through all views defined in this XMILE file.
/// - `<data>` (opt) - definitions of persistent data import/export
///   connections.
/// - `<model>+` - definition of model equations and (optionally) diagrams.
/// - `<macro>*` - definition of macros that can be used in model equations.
///
/// These tags are specified in the subsequent sections of this chapter, after
/// XMILE namespaces are discussed.
///
/// When an XMILE file includes references to models contained in separate files
/// or at a specific URL, each such file may contain overlapping information,
/// most commonly in sim_specs, model_units and dimensions. When such overlap
/// is consistent, combining parts is done by taking the union of the different
/// component files. When an inconsistency is found, (for example, a dimension
/// with two distinct definitions) software reading the files MUST resolve the
/// inconsistency and SHOULD provide user feedback in doing so. Some
/// inconsistencies, such as conflicting Macro or Model names MUST be resolved
/// as detailed in section 2.11.3.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "xmile")]
pub struct XmileFile {
    /// The version of the XMILE specification used in this file.
    #[serde(rename = "@version")]
    pub version: String,
    /// The XML namespace for XMILE.
    #[serde(rename = "@xmlns", default = "default_xmlns")]
    pub xmlns: String,
    /// The header information for the XMILE file.
    pub header: Header,
    /// Optional simulation specifications for the XMILE file.
    pub sim_specs: Option<SimulationSpecs>,
    /// Optional model units defined in the XMILE file.
    pub model_units: Option<ModelUnits>,
    /// Optional dimensions defined in the XMILE file.
    pub dimensions: Option<Dimensions>,
    /// Optional behavior specifications for the XMILE file.
    pub behavior: Option<Behavior>,
    /// Optional style definitions for the XMILE file.
    pub style: Option<Style>,
    /// Optional data definitions for the XMILE file.
    pub data: Option<Data>,
    /// A list of models defined in the XMILE file.
    #[serde(rename = "model")]
    pub models: Vec<Model>,
    /// A list of macros defined in the XMILE file.
    #[cfg(feature = "macros")]
    #[serde(rename = "macro", default)]
    pub macros: Vec<Macro>,
}

/// The overall structure of a <model> tag appears below (sub-tags MUST appear in this order):
///
/// ```xml
/// <model name="..." resource="...">
///    <sim_specs>    <!-- OPTIONAL – see Chapter 2 -->
///      ...
///    </sim_specs>
///    <behavior>     <!-- OPTIONAL – see Chapter 2 -->
///      ...
///    </behavior>
///    <variables>    <!-- REQUIRED -->
///      ...
///    </variables>
///    <views>        <!-- OPTIONAL – see Chapters 5 & 6 -->
///      ...
///    </views>
/// </model>
/// ```
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Optional name attribute for the model (required if model is a submodel).
    #[serde(rename = "@name")]
    pub name: Option<String>,
    /// Optional resource attribute referencing an external file containing the model.
    #[serde(rename = "@resource")]
    pub resource: Option<String>,
    /// Optional simulation specifications for this model.
    pub sim_specs: Option<SimulationSpecs>,
    /// Optional behavior specifications for this model.
    pub behavior: Option<Behavior>,
    /// The variables defined in this model (REQUIRED).
    pub variables: Variables,
    /// Optional views for this model.
    pub views: Option<Views>,
}

impl Validate for Model {
    fn validate(&self) -> ValidationResult {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        
        // Validate variable name uniqueness
        match validate_variable_name_uniqueness(&self.variables.variables) {
            ValidationResult::Valid(_) => {}
            ValidationResult::Warnings(_, warns) => warnings.extend(warns),
            ValidationResult::Invalid(warns, errs) => {
                warnings.extend(warns);
                errors.extend(errs);
            }
        }
        
        // Validate dimension references
        // TODO: Get dimensions from file-level or model-level dimensions
        // For now, we'll skip this validation as we need access to the full file context
        // This would require passing dimensions from XmileFile to Model validation
        
        // Validate view object references
        if let Some(ref views) = self.views {
            for view in &views.views {
                match validate_view_object_references(view, &self.variables.variables) {
                    ValidationResult::Valid(_) => {}
                    ValidationResult::Warnings(_, warns) => warnings.extend(warns),
                    ValidationResult::Invalid(warns, errs) => {
                        warnings.extend(warns);
                        errors.extend(errs);
                    }
                }
                
                // Validate UID uniqueness within each view
                match validate_view_uids_unique(view) {
                    ValidationResult::Valid(_) => {}
                    ValidationResult::Warnings(_, warns) => warnings.extend(warns),
                    ValidationResult::Invalid(warns, errs) => {
                        warnings.extend(warns);
                        errors.extend(errs);
                    }
                }
            }
        }
        
        // Validate group entity references
        let groups: Vec<_> = self.variables.variables
            .iter()
            .filter_map(|v| {
                if let Variable::Group(g) = v {
                    Some(g.clone())
                } else {
                    None
                }
            })
            .collect();
        
        if !groups.is_empty() {
            match validate_group_entity_references(&groups, &self.variables.variables) {
                ValidationResult::Valid(_) => {}
                ValidationResult::Warnings(_, warns) => warnings.extend(warns),
                ValidationResult::Invalid(warns, errs) => {
                    warnings.extend(warns);
                    errors.extend(errs);
                }
            }
        }
        
        if errors.is_empty() {
            if warnings.is_empty() {
                ValidationResult::Valid(())
            } else {
                ValidationResult::Warnings((), warnings)
            }
        } else {
            ValidationResult::Invalid(warnings, errors)
        }
    }
}

/// A wrapper for deserializing variables from XML.
/// The <variables> tag contains a mix of <stock>, <flow>, <aux>, <gf>, and <module> tags.
#[derive(Debug, PartialEq, Clone)]
pub struct Variables {
    pub variables: Vec<Variable>,
}

impl Variables {
    pub fn new(variables: Vec<Variable>) -> Self {
        Variables { variables }
    }
}

// Custom deserialization for Variables to handle mixed tag names
// In XML, <variables> contains a mix of <stock>, <flow>, <aux>, <gf>, and <module> tags
impl<'de> Deserialize<'de> for Variables {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct VariablesVisitor;

        impl<'de> Visitor<'de> for VariablesVisitor {
            type Value = Variables;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a variables element")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut variables = Vec::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "stock" => {
                            let stock: crate::model::vars::Stock = map.next_value()?;
                            variables.push(Variable::Stock(stock));
                        }
                        "flow" => {
                            let flow: Flow = map.next_value()?;
                            match flow {
                                Flow::Basic(basic) => {
                                    variables.push(Variable::Flow(basic));
                                }
                                _ => {
                                    return Err(de::Error::custom(
                                        "Only basic flows are supported in variables section"
                                    ));
                                }
                            }
                        }
                        "aux" => {
                            let aux: crate::model::vars::Auxiliary = map.next_value()?;
                            variables.push(Variable::Auxiliary(aux));
                        }
                        "gf" => {
                            let gf: crate::model::vars::GraphicalFunction = map.next_value()?;
                            variables.push(Variable::GraphicalFunction(gf));
                        }
                        #[cfg(feature = "submodels")]
                        "module" => {
                            let module: crate::model::vars::Module = map.next_value()?;
                            variables.push(Variable::Module(module));
                        }
                        "group" => {
                            let group: crate::model::groups::Group = map.next_value()?;
                            variables.push(Variable::Group(group));
                        }
                        _ => {
                            // Skip unknown tags
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                Ok(Variables { variables })
            }
        }

        deserializer.deserialize_map(VariablesVisitor)
    }
}

impl Serialize for Variables {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.variables.len()))?;
        
        for var in &self.variables {
            match var {
                Variable::Stock(stock) => {
                    map.serialize_entry("stock", stock)?;
                }
                Variable::Flow(flow) => {
                    map.serialize_entry("flow", flow)?;
                }
                Variable::Auxiliary(aux) => {
                    map.serialize_entry("aux", aux)?;
                }
                Variable::GraphicalFunction(gf) => {
                    map.serialize_entry("gf", gf)?;
                }
                #[cfg(feature = "submodels")]
                Variable::Module(module) => {
                    map.serialize_entry("module", module)?;
                }
                Variable::Group(group) => {
                    map.serialize_entry("group", group)?;
                }
            }
        }
        map.end()
    }
}

/// The <views> tag contains a list of one or many <view> tags which describes
/// the layout, content and appearance of the user interface and stock and flow diagram.
/// The <views> tag can also contain an OPTIONAL visible_view attribute specifying
/// the index of the view which the user desires to be active upon loading of the file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Views {
    /// The index of the view which should be active upon loading.
    /// The index refers to the full list of views regardless of the view's type.
    #[serde(rename = "@visible_view")]
    pub visible_view: Option<u32>,
    /// A list of views defined in this model.
    #[serde(rename = "view")]
    pub views: Vec<View>,
    /// Optional style definitions that apply to all views within this <views> tag.
    pub style: Option<Style>,
}
