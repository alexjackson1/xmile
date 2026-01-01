use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "macros")]
use std::collections::HashMap;

use crate::{
    equation::{Expression, Identifier},
    model::{
        object::Documentation,
        vars::Variable,
    },
    namespace::Namespace,
    specs::SimulationSpecs,
    view::View,
    types::{Validate, ValidationResult},
};

/// Macros allow new built-in functions to be defined, which can also be used to
/// translate built-in functions across vendor packages. They also provide a way
/// to implement non-standard simulation behavior for stocks, flows, and
/// auxiliaries.
///
/// Macros live outside of all other blocks, at the same level as the <model>
/// tag, and MAY be the only thing in a file other than its header. Macros MUST
/// satisfy all the rules and assertions expressed under this section, unless
/// indicated as optional. Macros are defined with a <macro> tag, which has
/// these REQUIRED properties and attributes:
///
/// - Name:  name="…" with the macro name (a valid XMILE identifier).
/// - Equation:  <eqn> w/valid XMILE expression, in a CDATA section if needed.
///
/// Macros also have the following OPTIONAL properties and attributes:
///
/// - Parameter:  <parm> with the name of the formal parameter within the macro
///   (its local name, which must be a valid XMILE identifier) (default: no
///   parameters). There must be one <parm> property for each macro parameter
///   and they must appear in the expected calling order (i.e., the order of the
///   actual parameters). Parameters can optionally have default values so they
///   do not always have to be specified when the macro is called. These are
///   specified with the default="…" attribute on the <parm> tag, using a valid
///   XMILE expression that can refer to any parameter already defined. Every
///   parameter after the first parameter that has a default value specified
///   (i.e., uses default) must also have a default value specified (i.e., the
///   first use of default makes all succeeding parameters optional as well).
///   Since the macro equation often refers to these parameters, it is strongly
///   recommended that the <parm> tag, when used, appears before the <eqn> tag.
/// - Function format:  <format> with text that indicates the proper use of the
///   function, usually with its name and a description of its parameters in
///   order (default: none).
/// - Function documentation:  <doc> with text that describes the purpose of the
///   function, optionally in HTML format (default: none).
/// - Simulation specifications:  <sim_specs> as defined for the model (see
///   Section 2.3) (default: same DT and integration method as model that
///   invokes the macro). This must only appear in conjunction with a
///   <variables> tag. Only <start>, <stop>, <dt>, and method="…" are allowed
///   and all but method are specified with a valid XMILE expression that can
///   include parameters. When <sim_specs> appears, the default DT is one and
///   the default integration method is euler.
/// - Variables:  <variables> as defined for <model> (see Sections 4.1-4.7)
///   (default: no variables).
/// - View:  Exactly one <view> within a <views> tag (see Chapters 5 and 6)
///   (default: no view). This must only appear in conjunction with a
///   <variables> tag and exists only to facilitate editing macros.
/// - Namespace:  namespace="…" with a XMILE namespace, for example,
///   namespace="isee" (default: single namespace specified in the header's
///   <options> tag, or no namespace if either no namespaces or multiple
///   namespaces are specified in the header).
///
/// Macros MAY include submodels. OPTIONALLY, they can also be recursive, i.e.,
/// they can refer to themselves in their equations. In this case, the
/// recursive_macros option must be set to true in the <uses_macros> tag of the
/// XMILE options (see Section 2.2.1).
#[derive(Debug, Clone, PartialEq)]
pub struct Macro {
    /// The name of the macro (a valid XMILE identifier).
    /// This is a REQUIRED attribute: name="…"
    pub name: Identifier,

    /// The equation for the macro, containing a valid XMILE expression.
    /// This is a REQUIRED property: <eqn>...</eqn>
    pub eqn: Expression,

    /// The parameters of the macro, in calling order.
    /// Each parameter is defined with a <parm> tag.
    /// This is an OPTIONAL property (default: no parameters).
    pub parameters: Vec<MacroParameter>,

    /// Function format text that indicates the proper use of the function.
    /// This is an OPTIONAL property: <format>...</format>
    /// (default: none)
    pub format: Option<String>,

    /// Function documentation describing the purpose of the function.
    /// This can be plain text or HTML format.
    /// This is an OPTIONAL property: <doc>...</doc>
    /// (default: none)
    pub doc: Option<Documentation>,

    /// Simulation specifications for the macro.
    /// This must only appear in conjunction with a <variables> tag.
    /// Only <start>, <stop>, <dt>, and method="…" are allowed.
    /// This is an OPTIONAL property: <sim_specs>...</sim_specs>
    /// (default: same DT and integration method as model that invokes the macro)
    pub sim_specs: Option<SimulationSpecs>,

    /// Variables defined within the macro.
    /// This is an OPTIONAL property: <variables>...</variables>
    /// (default: no variables)
    pub variables: Option<Vec<Variable>>,

    /// Exactly one view within a <views> tag.
    /// This must only appear in conjunction with a <variables> tag.
    /// This is an OPTIONAL property: <views><view>...</view></views>
    /// (default: no view)
    pub views: Option<View>,

    /// The namespace for the macro.
    /// This is an OPTIONAL attribute: namespace="…"
    /// (default: single namespace specified in the header's <options> tag,
    /// or no namespace if either no namespaces or multiple namespaces are specified)
    pub namespace: Option<Vec<Namespace>>,
}

/// Raw macro structure for deserialization from XML.
/// Handles the mixed content within a <macro> tag.
#[derive(Debug, Clone, PartialEq, Deserialize)]
struct RawMacro {
    #[serde(rename = "@name")]
    name: Identifier,
    #[serde(rename = "@namespace")]
    namespace: Option<String>,
    #[serde(rename = "parm", default)]
    parameters: Vec<MacroParameter>,
    #[serde(rename = "eqn")]
    eqn: Expression,
    #[serde(rename = "format")]
    format: Option<String>,
    #[serde(rename = "doc")]
    doc: Option<Documentation>,
    #[serde(rename = "sim_specs")]
    sim_specs: Option<SimulationSpecs>,
    #[serde(rename = "variables")]
    variables: Option<crate::xml::schema::Variables>,
    #[serde(rename = "views")]
    views: Option<RawMacroViews>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RawMacroViews {
    #[serde(rename = "view")]
    view: View,
}

impl<'de> Deserialize<'de> for Macro {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw: RawMacro = Deserialize::deserialize(deserializer)?;
        
        let namespace = raw.namespace.map(|ns| {
            // Parse namespace string into Vec<Namespace>
            Namespace::from_str(&ns)
        });
        
        let variables = raw.variables.map(|vars| vars.variables);
        
        let views = raw.views.map(|v| v.view);
        
        Ok(Macro {
            name: raw.name,
            eqn: raw.eqn,
            parameters: raw.parameters,
            format: raw.format,
            doc: raw.doc,
            sim_specs: raw.sim_specs,
            variables,
            views,
            namespace,
        })
    }
}

impl Serialize for Macro {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let field_count = 1 // name
            + if self.namespace.is_some() { 1 } else { 0 }
            + if !self.parameters.is_empty() { 1 } else { 0 }
            + 1 // eqn
            + if self.format.is_some() { 1 } else { 0 }
            + if self.doc.is_some() { 1 } else { 0 }
            + if self.sim_specs.is_some() { 1 } else { 0 }
            + if self.variables.is_some() { 1 } else { 0 }
            + if self.views.is_some() { 1 } else { 0 };
        
        let mut state = serializer.serialize_struct("macro", field_count)?;
        
        // Serialize name as attribute
        state.serialize_field("@name", &self.name.to_string())?;
        
        if let Some(ref ns_vec) = self.namespace {
            if !ns_vec.is_empty() {
                let ns_str = Namespace::as_prefix(ns_vec);
                state.serialize_field("@namespace", &ns_str)?;
            }
        }
        
        if !self.parameters.is_empty() {
            state.serialize_field("parm", &self.parameters)?;
        }
        
        state.serialize_field("eqn", &self.eqn)?;
        
        if let Some(ref format) = self.format {
            state.serialize_field("format", format)?;
        }
        
        if let Some(ref doc) = self.doc {
            state.serialize_field("doc", doc)?;
        }
        
        if let Some(ref sim_specs) = self.sim_specs {
            state.serialize_field("sim_specs", sim_specs)?;
        }
        
        if let Some(ref vars) = self.variables {
            use crate::xml::schema::Variables;
            state.serialize_field("variables", &Variables { variables: vars.clone() })?;
        }
        
        if let Some(ref view) = self.views {
            state.serialize_field("views", &RawMacroViews { view: view.clone() })?;
        }
        
        state.end()
    }
}

/// A macro parameter defined with a <parm> tag.
///
/// Parameters must appear in the expected calling order (i.e., the order of the
/// actual parameters). Parameters can optionally have default values so they
/// do not always have to be specified when the macro is called.
///
/// Every parameter after the first parameter that has a default value specified
/// must also have a default value specified (i.e., the first use of default
/// makes all succeeding parameters optional as well).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacroParameter {
    /// The name of the formal parameter within the macro.
    /// This must be a valid XMILE identifier.
    /// This is specified as the text content of the <parm> tag.
    #[serde(rename = "#text")]
    pub name: Identifier,

    /// The default value for the parameter, specified as a valid XMILE expression.
    /// This expression can refer to any parameter already defined.
    /// This is an OPTIONAL attribute: default="…"
    /// (default: no default value, parameter is required)
    #[serde(rename = "@default")]
    pub default: Option<Expression>,
}

impl Validate for Macro {
    fn validate(&self) -> ValidationResult {
        let warnings = Vec::new();
        let mut errors = Vec::new();

        // Validate that if sim_specs is present, variables must also be present
        if self.sim_specs.is_some() && self.variables.is_none() {
            errors.push(
                "Macro sim_specs can only appear in conjunction with a variables tag.".to_string(),
            );
        }

        // Validate that if views is present, variables must also be present
        if self.views.is_some() && self.variables.is_none() {
            errors.push(
                "Macro views can only appear in conjunction with a variables tag.".to_string(),
            );
        }

        // Validate parameter default values: once a parameter has a default,
        // all subsequent parameters must also have defaults
        let mut found_default = false;
        for (idx, param) in self.parameters.iter().enumerate() {
            if param.default.is_some() {
                found_default = true;
            } else if found_default {
                errors.push(format!(
                    "Macro parameter '{}' (at index {}) must have a default value \
                     because a previous parameter has a default value.",
                    param.name, idx
                ));
            }
        }

        // Validate sim_specs if present (only start, stop, dt, and method are allowed)
        // Note: The spec says only start, stop, dt, and method are allowed.
        // We can't easily validate this at the struct level since SimulationSpecs
        // contains all fields. This would need to be validated during parsing/deserialization.

        if errors.is_empty() {
            ValidationResult::Valid(())
        } else {
            ValidationResult::Invalid(warnings, errors)
        }
    }
}

impl Validate for MacroParameter {
    fn validate(&self) -> ValidationResult {
        let warnings = Vec::new();
        let errors = Vec::new();

        // Parameter name validation is handled by Identifier type itself
        // Default expression validation would be handled by Expression type

        if errors.is_empty() {
            ValidationResult::Valid(())
        } else {
            ValidationResult::Invalid(warnings, errors)
        }
    }
}

/// Registry for macros that maps macro names to their definitions.
/// 
/// This registry is used to resolve macro calls in expressions and validate
/// that macro calls match their definitions (e.g., parameter counts).
#[cfg(feature = "macros")]
#[derive(Debug, Clone, Default)]
pub struct MacroRegistry {
    /// Map from macro name (normalized) to macro definition
    macros: HashMap<Identifier, Macro>,
}

#[cfg(feature = "macros")]
impl MacroRegistry {
    /// Creates a new empty macro registry.
    pub fn new() -> Self {
        MacroRegistry {
            macros: HashMap::new(),
        }
    }

    /// Builds a macro registry from a list of macros.
    /// 
    /// # Arguments
    /// 
    /// * `macros` - A slice of macros to register
    /// 
    /// # Returns
    /// 
    /// A new `MacroRegistry` containing all the provided macros.
    pub fn from_macros(macros: &[Macro]) -> Self {
        let mut registry = MacroRegistry::new();
        for macro_def in macros {
            registry.register(macro_def.clone());
        }
        registry
    }

    /// Registers a macro in the registry.
    /// 
    /// # Arguments
    /// 
    /// * `macro_def` - The macro definition to register
    pub fn register(&mut self, macro_def: Macro) {
        self.macros.insert(macro_def.name.clone(), macro_def);
    }

    /// Looks up a macro by name.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The identifier of the macro to look up
    /// 
    /// # Returns
    /// 
    /// `Some(&Macro)` if the macro is found, `None` otherwise.
    pub fn get(&self, name: &Identifier) -> Option<&Macro> {
        self.macros.get(name)
    }

    /// Checks if a macro with the given name exists in the registry.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The identifier to check
    /// 
    /// # Returns
    /// 
    /// `true` if the macro exists, `false` otherwise.
    pub fn contains(&self, name: &Identifier) -> bool {
        self.macros.contains_key(name)
    }

    /// Returns the number of parameters expected by a macro.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The identifier of the macro
    /// 
    /// # Returns
    /// 
    /// `Some(usize)` if the macro exists, `None` otherwise.
    /// The count includes all parameters, including those with default values.
    pub fn parameter_count(&self, name: &Identifier) -> Option<usize> {
        self.get(name).map(|m| m.parameters.len())
    }

    /// Returns the number of required parameters (those without default values).
    /// 
    /// # Arguments
    /// 
    /// * `name` - The identifier of the macro
    /// 
    /// # Returns
    /// 
    /// `Some(usize)` if the macro exists, `None` otherwise.
    pub fn required_parameter_count(&self, name: &Identifier) -> Option<usize> {
        self.get(name).map(|m| {
            m.parameters
                .iter()
                .take_while(|p| p.default.is_none())
                .count()
        })
    }
}
