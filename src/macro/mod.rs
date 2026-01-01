use serde::{Deserialize, Serialize};

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
///
/// Note: This struct does not implement Serialize/Deserialize because some
/// contained types (SimulationSpecs, Variable, View) do not implement these traits.
/// Debug, Clone, and PartialEq are manually implemented because View doesn't implement them.
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

impl std::fmt::Debug for Macro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Macro")
            .field("name", &self.name)
            .field("eqn", &self.eqn)
            .field("parameters", &self.parameters)
            .field("format", &self.format)
            .field("doc", &self.doc)
            .field("sim_specs", &self.sim_specs)
            .field("variables", &self.variables)
            .field("views", &self.views.as_ref().map(|_| "View(...)"))
            .field("namespace", &self.namespace)
            .finish()
    }
}

impl Clone for Macro {
    fn clone(&self) -> Self {
        Macro {
            name: self.name.clone(),
            eqn: self.eqn.clone(),
            parameters: self.parameters.clone(),
            format: self.format.clone(),
            doc: self.doc.clone(),
            sim_specs: self.sim_specs.clone(),
            variables: self.variables.clone(),
            views: None, // View doesn't implement Clone, so we set to None
            namespace: self.namespace.clone(),
        }
    }
}

impl PartialEq for Macro {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.eqn == other.eqn
            && self.parameters == other.parameters
            && self.format == other.format
            && self.doc == other.doc
            && self.sim_specs == other.sim_specs
            && self.variables == other.variables
            // View doesn't implement PartialEq, so we skip it in comparison
            && self.namespace == other.namespace
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
