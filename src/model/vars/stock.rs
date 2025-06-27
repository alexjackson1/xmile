use crate::{
    Expression, Identifier, Measure, UnitOfMeasure,
    model::object::{DeviceRange, DeviceScale, Document, Documentation, FormatOptions, Object},
};

use super::Var;

/// A trait representing a stock variable in a model.
pub trait StockVar<'a>: Var<'a> + Object + Document + Measure {
    /// Returns the inflows to the stock variable.
    fn inflows(&self) -> &[Identifier];

    /// Returns the outflows from the stock variable.
    fn outflows(&self) -> &[Identifier];

    /// Returns the initial equation defining the stock's value.
    fn initial_equation(&self) -> &Expression;
}

/// Represents a stock variable in a model, which can be of different types:
/// - BasicStock: A basic stock variable with inflows, outflows, and an initial value equation.
/// - ConveyorStock: A conveyor stock is a basic stock with additional properties like length, capacity, and inflow limits.
/// - QueueStock: A queue stock is a specialized stock that manages inflows and outflows in a queue-like manner.
#[derive(Debug, Clone, PartialEq)]
pub enum Stock {
    Basic(BasicStock),
    Conveyor(ConveyorStock),
    Queue(QueueStock),
}

/// A basic stock variable with inflows, outflows, and an initial value equation.
#[derive(Debug, Clone, PartialEq)]
pub struct BasicStock {
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
    pub range: Option<DeviceRange>,

    /// The scale of the stock variable.
    pub scale: Option<DeviceScale>,

    /// The format options for the stock variable.
    pub format: Option<FormatOptions>,
}

impl StockVar<'_> for BasicStock {
    fn inflows(&self) -> &[Identifier] {
        &self.inflows
    }

    fn outflows(&self) -> &[Identifier] {
        &self.outflows
    }

    fn initial_equation(&self) -> &Expression {
        &self.initial_equation
    }
}

impl Var<'_> for BasicStock {
    fn name(&self) -> Option<&Identifier> {
        Some(&self.name)
    }

    fn equation(&self) -> Option<&Expression> {
        todo!()
    }

    #[cfg(feature = "mathml")]
    fn mathml_equation(&self) -> Option<&String> {
        todo!()
    }
}

impl Object for BasicStock {
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

impl Document for BasicStock {
    fn documentation(&self) -> Option<&Documentation> {
        self.documentation.as_ref()
    }
}

impl Measure for BasicStock {
    fn units(&self) -> Option<&UnitOfMeasure> {
        self.units.as_ref()
    }
}

impl BasicStock {}

/// A conveyor stock with inflows, outflows, and an initial value equation.
#[derive(Debug, Clone, PartialEq)]
pub struct ConveyorStock {
    /// The name of the conveyor variable.
    pub name: Identifier,

    /// The inflows to the conveyor.
    pub inflows: Vec<Identifier>,

    /// The outflows from the conveyor.
    pub outflows: Vec<Identifier>,

    /// The equation defining the conveyor's initial value.
    pub initial_equation: Expression,

    /// The length of the conveyor in time units.
    pub length: Expression,

    /// The capacity of the conveyor.
    pub capacity: Option<Expression>,

    /// The inflow limit of the conveyor.
    pub inflow_limit: Option<Expression>,

    /// If true, the conveyor will resample transit time.
    pub sample: Option<Expression>,

    /// If true, the conveyor will stop moving.
    pub arrest_value: Option<Expression>,

    /// If true, the conveyor handles discrete items, otherwise continuous.
    pub discrete: Option<bool>,

    /// If true, batches from an upstream queue cannot be split.
    pub batch_integrity: Option<bool>,

    /// Number of batches able to be taken from an upstream queue (if applicable).
    pub number_of_batches: Option<u32>,

    /// True if exponential leakage should exponentially decay across the conveyor.
    pub exponential_leakage: Option<bool>,

    /// The units of measure for the conveyor.
    pub units: Option<UnitOfMeasure>,

    /// The documentation for the conveyor.
    pub documentation: Option<Documentation>,

    /// The range of values for the conveyor.
    pub range: Option<DeviceRange>,

    /// The scale of the conveyor.
    pub scale: Option<DeviceScale>,

    /// The format options for the conveyor.
    pub format: Option<FormatOptions>,
}

impl StockVar<'_> for ConveyorStock {
    fn inflows(&self) -> &[Identifier] {
        &self.inflows
    }

    fn outflows(&self) -> &[Identifier] {
        &self.outflows
    }

    fn initial_equation(&self) -> &Expression {
        &self.initial_equation
    }
}

impl Var<'_> for ConveyorStock {
    fn name(&self) -> Option<&Identifier> {
        Some(&self.name)
    }

    fn equation(&self) -> Option<&Expression> {
        todo!()
    }

    #[cfg(feature = "mathml")]
    fn mathml_equation(&self) -> Option<&String> {
        todo!()
    }
}

impl Object for ConveyorStock {
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

impl Document for ConveyorStock {
    fn documentation(&self) -> Option<&Documentation> {
        self.documentation.as_ref()
    }
}

impl Measure for ConveyorStock {
    fn units(&self) -> Option<&UnitOfMeasure> {
        self.units.as_ref()
    }
}

impl ConveyorStock {}

/// A queue stock with inflows, outflows, and an initial value equation.
#[derive(Debug, Clone, PartialEq)]
pub struct QueueStock {
    /// The name of the queue variable.
    pub name: Identifier,

    /// The inflows to the queue variable.
    pub inflows: Vec<Identifier>,

    /// The outflows from the queue variable.
    pub outflows: Vec<Identifier>,

    /// The equation defining the queue's initial value.
    pub initial_equation: Expression,

    /// The units of measure for the queue variable.
    pub units: Option<UnitOfMeasure>,

    /// The documentation for the queue variable.
    pub documentation: Option<Documentation>,

    /// The range of values for the queue variable.
    pub range: Option<DeviceRange>,

    /// The scale of the queue variable.
    pub scale: Option<DeviceScale>,

    /// The format options for the queue variable.
    pub format: Option<FormatOptions>,
}

impl StockVar<'_> for QueueStock {
    fn inflows(&self) -> &[Identifier] {
        &self.inflows
    }

    fn outflows(&self) -> &[Identifier] {
        &self.outflows
    }

    fn initial_equation(&self) -> &Expression {
        &self.initial_equation
    }
}

impl Var<'_> for QueueStock {
    fn name(&self) -> Option<&Identifier> {
        Some(&self.name)
    }

    fn equation(&self) -> Option<&Expression> {
        todo!()
    }

    #[cfg(feature = "mathml")]
    fn mathml_equation(&self) -> Option<&String> {
        todo!()
    }
}

impl Object for QueueStock {
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

impl Document for QueueStock {
    fn documentation(&self) -> Option<&Documentation> {
        self.documentation.as_ref()
    }
}

impl Measure for QueueStock {
    fn units(&self) -> Option<&UnitOfMeasure> {
        self.units.as_ref()
    }
}

impl QueueStock {}
