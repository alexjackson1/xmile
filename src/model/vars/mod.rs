#[cfg(feature = "arrays")]
pub mod array;

pub mod aux;
pub mod flow;
pub mod gf;
pub mod stock;

use crate::{
    Expression, Identifier, Measure,
    model::object::{Document, Object},
};

pub use aux::Auxiliary;
pub use flow::BasicFlow;
pub use gf::GraphicalFunction;
use serde::{Deserialize, Serialize};
pub use stock::Stock;

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Auxiliary(Auxiliary),
    Stock(Stock),
    Flow(BasicFlow),
    GraphicalFunction(GraphicalFunction),
}

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

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
struct NonNegativeContent {
    #[serde(rename = "#text")]
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
