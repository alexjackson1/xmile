#[cfg(feature = "arrays")]
pub mod array;

#[cfg(feature = "arrays")]
pub use array::ArrayRegistry;

pub mod aux;
pub mod flow;
pub mod gf;
pub mod stock;

#[cfg(feature = "submodels")]
pub mod module;

use crate::{
    Expression, Identifier, Measure,
    model::object::{Document, Object},
};

pub use aux::Auxiliary;
pub use flow::BasicFlow;
pub use gf::GraphicalFunction;
use serde::{Deserialize, Serialize};
pub use stock::Stock;

#[cfg(feature = "submodels")]
pub use module::Module;

/// Access type for variables in submodels.
/// Determines whether a variable is an input, output, or neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccessType {
    /// Variable is a submodel input.
    Input,
    /// Variable is a submodel output.
    Output,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Auxiliary(Auxiliary),
    Stock(Stock),
    Flow(BasicFlow),
    GraphicalFunction(GraphicalFunction),
    #[cfg(feature = "submodels")]
    Module(Module),
    Group(crate::model::groups::Group),
}

// Note: Variable enum doesn't implement Serialize/Deserialize directly
// because in XML, each variant appears as a different tag name.
// The individual types (Auxiliary, Stock, Flow, etc.) handle their own serialization.

/// All variables have the following REQUIRED property:
///
///  - Name:  name="…" attribute w/valid XMILE identifier
///
/// All variables that are dimensioned have the following REQUIRED property:
///
///  - Dimensions:  `<dimensions>` w/`<dim name="…">` for each dim in order (see
///    Arrays in Section 4.1.4) (default: none)
///
/// All non-apply-to-all arrayed variables, including non-apply-to-all
/// graphical functions, have the following REQUIRED property:
///
///  - Element:  `<element>` with a valid subscript attribute (default: none).
///    The subscript="…" attribute lists comma-separated indices in dimension
///    order for the array element. This attribute is only valid on the
///    variable type tag for array elements of non-apply-to-all arrays (see
///    Arrays in Section 4.5). There MUST be one `<element>` tag for each array
///    entry and each MUST encapsulate either an `<eqn>` tag (non-graphical
///    functions) or a `<gf>` tag (graphical functions).
///
/// All variables have the following OPTIONAL properties:
///
///  - Access:  access="…" attribute w/valid XMILE access name – see Submodels
///    in Chapter 3 and in Section 4.7 (default: none)
///  - Access automatically set to output:  autoexport="…" attribute with
///    true/false – see Submodels in Section 4.7 (default: false)
///  - Equation:  `<eqn>` w/valid XMILE expression, in a CDATA section if needed
///
/// Of these, the name is REQUIRED for all variables and must be unique across
/// all variables in the containing model. If the intent is to simulate the
/// model, the equation is also required. For a stock, the equation contains
/// the stock’s initial value, rather than the stock’s integration equation.
///
/// The documentation can be plain text or can be HTML. If in plain text, it
/// must use XMILE identifier escape sequences for non-printable characters
/// (i.e., \n for newline, \t for tab, and, necessarily, \\ for backslash),
/// rather than a hexadecimal code such as &#x0A. If in HTML, it must include
/// the proper HTML header. Note this is true for all documentation and
/// user-specified text fields in a XMILE file (i.e., including those in
/// display objects defined in Chapters 5 and 6).
pub trait Var<'a>: Object + Measure + Document {
    fn name(&self) -> Option<&Identifier>;

    fn equation(&self) -> Option<&Expression>;

    #[cfg(feature = "mathml")]
    fn mathml_equation(&self) -> Option<&String>;
}

/// Wrapper for non_negative element content.
///
/// This type handles the semantics of the non_negative element in XMILE:
/// - None = element not present (use default behavior)
/// - Some(None) = empty tag `<non_negative/>` (means true/enabled)
/// - Some(Some(true)) = `<non_negative>true</non_negative>`
/// - Some(Some(false)) = `<non_negative>false</non_negative>`
///
/// Note: XML parsing is handled by quick-xml in `src/xml/deserialize.rs`.
/// This struct's serde derives are used for internal conversions only.
#[derive(Debug, Copy, Clone, PartialEq, Default, Serialize, Deserialize)]
struct NonNegativeContent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    value: Option<bool>,
}

impl From<NonNegativeContent> for bool {
    fn from(content: NonNegativeContent) -> Self {
        content.value.unwrap_or(true)
    }
}

impl From<NonNegativeContent> for Option<bool> {
    fn from(content: NonNegativeContent) -> Self {
        content.value
    }
}

impl From<NonNegativeContent> for Option<Option<bool>> {
    fn from(content: NonNegativeContent) -> Self {
        Some(content.value)
    }
}

impl From<Option<bool>> for NonNegativeContent {
    fn from(value: Option<bool>) -> Self {
        NonNegativeContent { value }
    }
}
