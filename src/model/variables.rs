use crate::{
    Expression, GraphicalFunction, Identifier, Measure, UnitOfMeasure,
    model::object::{Document, Documentation, FormatOptions, Object, Range, Scale},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Auxiliary(Auxiliary),
    Stock(Stock),
    Flow(Flow),
    GraphicalFunction(GraphicalFunction),
}

/// All variables have the following REQUIRED property:
///
///  - Name:  name="…" attribute w/valid XMILE identifier
///
/// All variables that are dimensioned have the following REQUIRED property:
///
///  - Dimensions:  <dimensions> w/<dim name="…"> for each dim in order (see
///    Arrays in Section 4.1.4) (default: none)
///
/// All non-apply-to-all arrayed variables, including non-apply-to-all
/// graphical functions, have the following REQUIRED property:
///
///  - Element:  <element> with a valid subscript attribute (default: none).
///    The subscript="…" attribute lists comma-separated indices in dimension
///    order for the array element. This attribute is only valid on the
///    variable type tag for array elements of non-apply-to-all arrays (see
///    Arrays in Section 4.5). There MUST be one <element> tag for each array
///    entry and each MUST encapsulate either an <eqn> tag (non-graphical
///    functions) or a <gf> tag (graphical functions).
///
/// All variables have the following OPTIONAL properties:
///
///  - Access:  access="…" attribute w/valid XMILE access name – see Submodels
///    in Chapter 3 and in Section 4.7 (default: none)
///  - Access automatically set to output:  autoexport="…" attribute with
///    true/false – see Submodels in Section 4.7 (default: false)
///  - Equation:  <eqn> w/valid XMILE expression, in a CDATA section if needed
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
    fn name(&self) -> &Identifier;

    fn equation(&self) -> Option<&Expression>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stock {
    /// The name of the stock variable.
    pub name: Identifier,

    /// The inflows to the stock variable.
    pub inflows: Vec<Identifier>,

    /// The outflows from the stock variable.
    pub outflows: Vec<Identifier>,

    /// The equation defining the stock's initial value.
    pub initial_equation: Expression,

    /// The units of measure for the stock variable.
    pub units: Option<UnitOfMeasure>,

    /// The documentation for the stock variable.
    pub documentation: Option<Documentation>,

    /// The range of values for the stock variable.
    pub range: Option<Range>,

    /// The scale of the stock variable.
    pub scale: Option<Scale>,

    /// The format options for the stock variable.
    pub format: Option<FormatOptions>,
}

impl Object for Stock {
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

impl Document for Stock {
    fn documentation(&self) -> Option<&Documentation> {
        self.documentation.as_ref()
    }
}

impl Measure for Stock {
    fn units(&self) -> Option<&UnitOfMeasure> {
        self.units.as_ref()
    }
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct Auxiliary {
    pub name: Identifier,
    pub documentation: Option<Documentation>,
    pub equation: Expression,
    pub units: Option<UnitOfMeasure>,
    pub range: Option<Range>,
    pub scale: Option<Scale>,
    pub format: Option<FormatOptions>,
}

impl Var<'_> for Auxiliary {
    fn name(&self) -> &Identifier {
        &self.name
    }

    fn equation(&self) -> Option<&Expression> {
        Some(&self.equation)
    }
}

impl Object for Auxiliary {
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

impl Measure for Auxiliary {
    fn units(&self) -> Option<&UnitOfMeasure> {
        self.units.as_ref()
    }
}

impl Document for Auxiliary {
    fn documentation(&self) -> Option<&Documentation> {
        self.documentation.as_ref()
    }
}
