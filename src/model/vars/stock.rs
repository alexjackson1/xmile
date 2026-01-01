use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    Expression, Identifier, Measure, UnitEquation,
    model::{
        events::EventPoster,
        object::{DeviceRange, DeviceScale, Document, Documentation, FormatOptions, Object},
        vars::{AccessType, NonNegativeContent},
    },
    types::{Validate, ValidationResult},
};

#[cfg(feature = "arrays")]
use crate::model::vars::array::{ArrayElement, VariableDimensions};

use super::Var;

#[derive(Debug, Error)]
pub enum StockConversionError {
    #[error("Missing conveyor length in stock definition")]
    MissingConveyorLength,
}

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RawStock {
    #[serde(rename = "@name")]
    name: Identifier,

    #[serde(rename = "@access")]
    access: Option<AccessType>,

    #[serde(rename = "@autoexport")]
    autoexport: Option<bool>,

    #[serde(rename = "inflow")]
    #[serde(default)]
    inflows: Vec<Identifier>,
    #[serde(rename = "outflow")]
    #[serde(default)]
    outflows: Vec<Identifier>,

    #[serde(rename = "eqn")]
    initial_equation: Expression,

    #[cfg(feature = "mathml")]
    #[serde(rename = "mathml")]
    mathml_equation: Option<String>,

    #[serde(rename = "non_negative")]
    non_negative: Option<NonNegativeContent>,

    #[serde(rename = "conveyor")]
    conveyor: Option<RawConveyor>,
    #[serde(rename = "queue")]
    queue: Option<RawQueue>,

    #[serde(rename = "units")]
    units: Option<UnitEquation>,

    #[serde(rename = "doc")]
    documentation: Option<Documentation>,

    #[serde(rename = "range")]
    range: Option<DeviceRange>,
    #[serde(rename = "scale")]
    scale: Option<DeviceScale>,
    #[serde(rename = "format")]
    format: Option<FormatOptions>,

    #[cfg(feature = "arrays")]
    #[serde(rename = "dimensions")]
    dimensions: Option<VariableDimensions>,
    
    #[cfg(feature = "arrays")]
    #[serde(rename = "element", default)]
    elements: Vec<ArrayElement>,
    
    #[serde(rename = "event_poster")]
    event_poster: Option<EventPoster>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RawConveyor {
    // Conveyor-specific fields
    #[serde(rename = "len")]
    length: Option<Expression>, // required for conveyor stocks
    #[serde(rename = "capacity")]
    capacity: Option<Expression>,
    #[serde(rename = "in_limit")]
    inflow_limit: Option<Expression>,
    #[serde(rename = "sample")]
    sample: Option<Expression>,
    #[serde(rename = "arrest")]
    arrest_value: Option<Expression>,
    #[serde(rename = "@discrete")]
    discrete: Option<bool>,
    #[serde(rename = "@batch_integrity")]
    batch_integrity: Option<bool>,
    #[serde(rename = "@one_at_a_time")]
    one_at_a_time: Option<bool>,
    #[serde(rename = "@exponential_leak")]
    exponential_leakage: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RawQueue;

enum StockKind {
    Normal,
    Conveyor,
    Queue,
    NonNegative,
}

impl RawStock {
    fn stock_kind(&self) -> Result<StockKind, ()> {
        if self.is_conveyor() {
            Ok(StockKind::Conveyor)
        } else if self.is_queue() {
            Ok(StockKind::Queue)
        } else if self.is_non_negative() {
            Ok(StockKind::NonNegative)
        } else if self.is_normal() {
            Ok(StockKind::Normal)
        } else {
            Err(())
        }
    }

    fn is_conveyor(&self) -> bool {
        self.conveyor.is_some()
    }

    fn is_queue(&self) -> bool {
        self.queue.is_some()
    }

    fn is_non_negative(&self) -> bool {
        self.non_negative.map(Into::into).unwrap_or(false)
    }

    fn is_normal(&self) -> bool {
        !self.is_conveyor() && !self.is_queue() && !self.is_non_negative()
    }
}

impl Validate for RawStock {
    fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();
        let warnings = Vec::new();

        if self.is_conveyor() && (self.is_non_negative() || self.is_queue()) {
            errors
                .push("A stock cannot be both a conveyor and non-negative or a queue.".to_string());
        }

        if self.is_queue() && (self.is_non_negative() || self.is_conveyor()) {
            errors
                .push("A stock cannot be both a queue and non-negative or a conveyor.".to_string());
        }

        if self.is_non_negative() && (self.is_conveyor() || self.is_queue()) {
            errors
                .push("A stock cannot be both non-negative and a conveyor or a queue.".to_string());
        }

        if errors.is_empty() {
            ValidationResult::Valid(())
        } else {
            ValidationResult::Invalid(errors, warnings)
        }
    }
}

impl From<BasicStock> for RawStock {
    fn from(stock: BasicStock) -> Self {
        RawStock {
            name: stock.name,
            access: stock.access,
            autoexport: stock.autoexport,
            inflows: stock.inflows,
            outflows: stock.outflows,
            initial_equation: stock.initial_equation,
            #[cfg(feature = "mathml")]
            mathml_equation: stock.mathml_equation,
            non_negative: stock.non_negative.map(Into::into),
            units: stock.units,
            documentation: stock.documentation,
            range: stock.range,
            scale: stock.scale,
            format: stock.format,
            conveyor: None,
            queue: None,
            #[cfg(feature = "arrays")]
            dimensions: stock.dimensions.map(|dims| {
                use crate::model::vars::array::{Dimension, VariableDimensions};
                VariableDimensions {
                    dims: dims.into_iter().map(|name| Dimension { name }).collect(),
                }
            }),
            #[cfg(feature = "arrays")]
            elements: stock.elements,
            event_poster: stock.event_poster,
        }
    }
}

impl From<ConveyorStock> for RawStock {
    fn from(stock: ConveyorStock) -> Self {
        RawStock {
            name: stock.name,
            access: stock.access,
            autoexport: stock.autoexport,
            inflows: stock.inflows,
            outflows: stock.outflows,
            initial_equation: stock.initial_equation,
            #[cfg(feature = "mathml")]
            mathml_equation: stock.mathml_equation,
            non_negative: None, // Conveyors are not marked as non-negative
            units: stock.units,
            documentation: stock.documentation,
            range: stock.range,
            scale: stock.scale,
            format: stock.format,
            conveyor: Some(RawConveyor {
                length: Some(stock.length),
                capacity: stock.capacity,
                inflow_limit: stock.inflow_limit,
                sample: stock.sample,
                arrest_value: stock.arrest_value,
                discrete: stock.discrete,
                batch_integrity: stock.batch_integrity,
                one_at_a_time: stock.one_at_a_time,
                exponential_leakage: stock.exponential_leakage,
            }),
            queue: None, // Conveyors are not queues
            #[cfg(feature = "arrays")]
            dimensions: stock.dimensions.map(|dims| {
                use crate::model::vars::array::{Dimension, VariableDimensions};
                VariableDimensions {
                    dims: dims.into_iter().map(|name| Dimension { name }).collect(),
                }
            }),
            #[cfg(feature = "arrays")]
            elements: stock.elements,
            event_poster: stock.event_poster,
        }
    }
}

impl From<QueueStock> for RawStock {
    fn from(stock: QueueStock) -> Self {
        RawStock {
            name: stock.name,
            access: stock.access,
            autoexport: stock.autoexport,
            inflows: stock.inflows,
            outflows: stock.outflows,
            initial_equation: stock.initial_equation,
            #[cfg(feature = "mathml")]
            mathml_equation: stock.mathml_equation,
            non_negative: None, // Queues are not marked as non-negative
            units: stock.units,
            documentation: stock.documentation,
            range: stock.range,
            scale: stock.scale,
            format: stock.format,
            conveyor: None, // Queues are not conveyors
            queue: Some(RawQueue),
            #[cfg(feature = "arrays")]
            dimensions: stock.dimensions.map(|dims| {
                use crate::model::vars::array::{Dimension, VariableDimensions};
                VariableDimensions {
                    dims: dims.into_iter().map(|name| Dimension { name }).collect(),
                }
            }),
            #[cfg(feature = "arrays")]
            elements: stock.elements,
            event_poster: stock.event_poster,
        }
    }
}

impl From<Stock> for RawStock {
    fn from(stock: Stock) -> Self {
        match stock {
            Stock::Basic(basic) => RawStock::from(basic),
            Stock::Conveyor(conveyor) => RawStock::from(conveyor),
            Stock::Queue(queue) => RawStock::from(queue),
        }
    }
}

impl<'de> Deserialize<'de> for Stock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw_stock = RawStock::deserialize(deserializer)?;
        raw_stock.validate().ok().map_err(|err| {
            let bullets = err
                .iter()
                .map(|e| format!("- {}", e))
                .collect::<Vec<_>>()
                .join("\n");

            serde::de::Error::custom(format!("Invalid stock definition:\n{}", bullets))
        })?;

        let stock_kind = raw_stock.stock_kind().map_err(|_| {
            serde::de::Error::custom(
                "Stock must be either a conveyor, queue, non-negative, or normal stock",
            )
        })?;

        match stock_kind {
            StockKind::Normal => Ok(Stock::Basic(BasicStock::from(raw_stock))),
            StockKind::NonNegative => Ok(Stock::Basic(BasicStock::from(raw_stock))),
            StockKind::Conveyor => Ok(Stock::Conveyor(
                ConveyorStock::try_from(raw_stock).map_err(serde::de::Error::custom)?,
            )),
            StockKind::Queue => Ok(Stock::Queue(QueueStock::from(raw_stock))),
        }
    }
}

impl Serialize for Stock {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let raw_stock: RawStock = self.clone().into();
        raw_stock.serialize(serializer)
    }
}

/// A basic stock variable with inflows, outflows, and an initial value equation.
#[derive(Debug, Clone, PartialEq)]
pub struct BasicStock {
    /// The name of the stock variable.
    pub name: Identifier,

    /// The access type for submodel I/O (input or output).
    pub access: Option<AccessType>,

    /// Whether access is automatically set to output.
    pub autoexport: Option<bool>,

    /// The inflows to the stock variable.
    pub inflows: Vec<Identifier>,

    /// The outflows from the stock variable.
    pub outflows: Vec<Identifier>,

    /// The equation defining the stock's initial value.
    pub initial_equation: Expression,

    /// Whether the stock is non-negative.
    pub non_negative: Option<Option<bool>>,

    /// The units of measure for the stock variable.
    pub units: Option<UnitEquation>,

    /// The documentation for the stock variable.
    pub documentation: Option<Documentation>,

    /// The range of values for the stock variable.
    pub range: Option<DeviceRange>,

    /// The scale of the stock variable.
    pub scale: Option<DeviceScale>,

    /// The format options for the stock variable.
    pub format: Option<FormatOptions>,

    /// The dimensions for this stock variable (if it's an array).
    #[cfg(feature = "arrays")]
    pub dimensions: Option<Vec<String>>,

    /// Array elements for non-apply-to-all arrays.
    #[cfg(feature = "arrays")]
    pub elements: Vec<ArrayElement>,

    /// Optional event poster for triggering events based on stock values.
    pub event_poster: Option<EventPoster>,

    /// Optional MathML representation of the initial equation.
    #[cfg(feature = "mathml")]
    pub mathml_equation: Option<String>,
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
        Some(&self.initial_equation)
    }

    #[cfg(feature = "mathml")]
    fn mathml_equation(&self) -> Option<&String> {
        self.mathml_equation.as_ref()
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
    fn units(&self) -> Option<&UnitEquation> {
        self.units.as_ref()
    }
}

impl From<RawStock> for BasicStock {
    fn from(raw: RawStock) -> Self {
        BasicStock {
            name: raw.name,
            access: raw.access,
            autoexport: raw.autoexport,
            inflows: raw.inflows,
            outflows: raw.outflows,
            initial_equation: raw.initial_equation,
            non_negative: raw.non_negative.map(|nn| nn.value.map(Into::into)),
            units: raw.units,
            documentation: raw.documentation,
            range: raw.range,
            scale: raw.scale,
            format: raw.format,
            #[cfg(feature = "arrays")]
            dimensions: raw.dimensions.map(|dims| dims.dims.into_iter().map(|d| d.name).collect()),
            #[cfg(feature = "arrays")]
            elements: raw.elements,
            event_poster: raw.event_poster,
            #[cfg(feature = "mathml")]
            mathml_equation: raw.mathml_equation,
        }
    }
}

impl BasicStock {}

/// A conveyor stock with inflows, outflows, and an initial value equation.
#[derive(Debug, Clone, PartialEq)]
pub struct ConveyorStock {
    /// The name of the conveyor variable.
    pub name: Identifier,

    /// The access type for submodel I/O (input or output).
    pub access: Option<AccessType>,

    /// Whether access is automatically set to output.
    pub autoexport: Option<bool>,

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
    pub one_at_a_time: Option<bool>,

    /// True if exponential leakage should exponentially decay across the conveyor.
    pub exponential_leakage: Option<bool>,

    /// The units of measure for the conveyor.
    pub units: Option<UnitEquation>,

    /// The documentation for the conveyor.
    pub documentation: Option<Documentation>,

    /// The range of values for the conveyor.
    pub range: Option<DeviceRange>,

    /// The scale of the conveyor.
    pub scale: Option<DeviceScale>,

    /// The format options for the conveyor.
    pub format: Option<FormatOptions>,

    /// The dimensions for this conveyor stock (if it's an array).
    #[cfg(feature = "arrays")]
    pub dimensions: Option<Vec<String>>,

    /// Array elements for non-apply-to-all arrays.
    #[cfg(feature = "arrays")]
    pub elements: Vec<ArrayElement>,

    /// Optional event poster for triggering events based on stock values.
    pub event_poster: Option<EventPoster>,

    /// Optional MathML representation of the initial equation.
    #[cfg(feature = "mathml")]
    pub mathml_equation: Option<String>,
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
        Some(&self.initial_equation)
    }

    #[cfg(feature = "mathml")]
    fn mathml_equation(&self) -> Option<&String> {
        self.mathml_equation.as_ref()
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
    fn units(&self) -> Option<&UnitEquation> {
        self.units.as_ref()
    }
}

impl TryFrom<RawStock> for ConveyorStock {
    type Error = StockConversionError;

    fn try_from(raw: RawStock) -> Result<Self, Self::Error> {
        Ok(ConveyorStock {
            name: raw.name,
            access: raw.access,
            autoexport: raw.autoexport,
            inflows: raw.inflows,
            outflows: raw.outflows,
            initial_equation: raw.initial_equation,
            length: raw
                .conveyor
                .as_ref()
                .and_then(|c| c.length.clone())
                .ok_or(StockConversionError::MissingConveyorLength)?,
            capacity: raw.conveyor.as_ref().and_then(|c| c.capacity.clone()),
            inflow_limit: raw.conveyor.as_ref().and_then(|c| c.inflow_limit.clone()),
            sample: raw.conveyor.as_ref().and_then(|c| c.sample.clone()),
            arrest_value: raw.conveyor.as_ref().and_then(|c| c.arrest_value.clone()),
            discrete: raw.conveyor.as_ref().and_then(|c| c.discrete),
            batch_integrity: raw.conveyor.as_ref().and_then(|c| c.batch_integrity),
            one_at_a_time: raw.conveyor.as_ref().and_then(|c| c.one_at_a_time),
            exponential_leakage: raw.conveyor.as_ref().and_then(|c| c.exponential_leakage),
            units: raw.units,
            documentation: raw.documentation,
            range: raw.range,
            scale: raw.scale,
            format: raw.format,
            #[cfg(feature = "arrays")]
            dimensions: raw.dimensions.map(|dims| dims.dims.into_iter().map(|d| d.name).collect()),
            #[cfg(feature = "arrays")]
            elements: raw.elements,
            event_poster: raw.event_poster,
            #[cfg(feature = "mathml")]
            mathml_equation: raw.mathml_equation,
        })
    }
}

impl ConveyorStock {}

/// A queue stock with inflows, outflows, and an initial value equation.
#[derive(Debug, Clone, PartialEq)]
pub struct QueueStock {
    /// The name of the queue variable.
    pub name: Identifier,

    /// The access type for submodel I/O (input or output).
    pub access: Option<AccessType>,

    /// Whether access is automatically set to output.
    pub autoexport: Option<bool>,

    /// The inflows to the queue variable.
    pub inflows: Vec<Identifier>,

    /// The outflows from the queue variable.
    pub outflows: Vec<Identifier>,

    /// The equation defining the queue's initial value.
    pub initial_equation: Expression,

    /// The units of measure for the queue variable.
    pub units: Option<UnitEquation>,

    /// The documentation for the queue variable.
    pub documentation: Option<Documentation>,

    /// The range of values for the queue variable.
    pub range: Option<DeviceRange>,

    /// The scale of the queue variable.
    pub scale: Option<DeviceScale>,

    /// The format options for the queue variable.
    pub format: Option<FormatOptions>,

    /// The dimensions for this queue stock (if it's an array).
    #[cfg(feature = "arrays")]
    pub dimensions: Option<Vec<String>>,

    /// Array elements for non-apply-to-all arrays.
    #[cfg(feature = "arrays")]
    pub elements: Vec<ArrayElement>,

    /// Optional event poster for triggering events based on stock values.
    pub event_poster: Option<EventPoster>,

    /// Optional MathML representation of the initial equation.
    #[cfg(feature = "mathml")]
    pub mathml_equation: Option<String>,
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
        Some(&self.initial_equation)
    }

    #[cfg(feature = "mathml")]
    fn mathml_equation(&self) -> Option<&String> {
        self.mathml_equation.as_ref()
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
    fn units(&self) -> Option<&UnitEquation> {
        self.units.as_ref()
    }
}

impl From<RawStock> for QueueStock {
    fn from(raw: RawStock) -> Self {
        QueueStock {
            name: raw.name,
            access: raw.access,
            autoexport: raw.autoexport,
            inflows: raw.inflows,
            outflows: raw.outflows,
            initial_equation: raw.initial_equation,
            units: raw.units,
            documentation: raw.documentation,
            range: raw.range,
            scale: raw.scale,
            format: raw.format,
            #[cfg(feature = "arrays")]
            dimensions: raw.dimensions.map(|dims| dims.dims.into_iter().map(|d| d.name).collect()),
            #[cfg(feature = "arrays")]
            elements: raw.elements,
            event_poster: raw.event_poster,
            #[cfg(feature = "mathml")]
            mathml_equation: raw.mathml_equation,
        }
    }
}

impl QueueStock {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_xml_rs::from_str;

    #[test]
    fn test_basic_stock() {
        let xml = r#"
        <stock name="Motivation">
            <eqn>100</eqn>
            <inflow>increasing</inflow>
            <outflow>decreasing</outflow>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse basic stock");

        match stock {
            Stock::Basic(basic_stock) => {
                assert_eq!(basic_stock.name, "Motivation");
                assert_eq!(
                    basic_stock.inflows,
                    vec![Identifier::parse_default("increasing").expect("valid identifier")]
                );
                assert_eq!(
                    basic_stock.outflows,
                    vec![Identifier::parse_default("decreasing").expect("valid identifier")]
                );
                // Note: We'd need to check the expression parsing here
                assert!(basic_stock.non_negative.is_none());
            }
            _ => panic!("Expected BasicStock, got {:?}", stock),
        }
    }

    #[test]
    fn test_stock_with_multiple_inflows_outflows() {
        let xml = r#"
        <stock name="Population">
            <eqn>1000</eqn>
            <inflow>births</inflow>
            <inflow>immigration</inflow>
            <outflow>deaths</outflow>
            <outflow>emigration</outflow>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse stock with multiple flows");

        match stock {
            Stock::Basic(basic_stock) => {
                assert_eq!(basic_stock.name, "Population");
                assert_eq!(basic_stock.inflows.len(), 2);
                assert_eq!(
                    basic_stock.inflows[0],
                    Identifier::parse_default("births").expect("valid identifier")
                );
                assert_eq!(
                    basic_stock.inflows[1],
                    Identifier::parse_default("immigration").expect("valid identifier")
                );
                assert_eq!(basic_stock.outflows.len(), 2);
                assert_eq!(
                    basic_stock.outflows[0],
                    Identifier::parse_default("deaths").expect("valid identifier")
                );
                assert_eq!(
                    basic_stock.outflows[1],
                    Identifier::parse_default("emigration").expect("valid identifier")
                );
            }
            _ => panic!("Expected BasicStock"),
        }
    }

    #[test]
    fn test_stock_no_flows() {
        let xml = r#"
        <stock name="Constants">
            <eqn>42</eqn>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse stock without flows");

        match stock {
            Stock::Basic(basic_stock) => {
                assert_eq!(basic_stock.name, "Constants");
                assert!(basic_stock.inflows.is_empty());
                assert!(basic_stock.outflows.is_empty());
            }
            _ => panic!("Expected BasicStock"),
        }
    }

    #[test]
    fn test_stock_with_non_negative() {
        let xml = r#"
        <stock name="Inventory">
            <eqn>50</eqn>
            <inflow>production</inflow>
            <outflow>sales</outflow>
            <non_negative />
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse non-negative stock");

        match stock {
            Stock::Basic(basic_stock) => {
                assert_eq!(basic_stock.name, "Inventory");
                assert_eq!(basic_stock.non_negative, Some(None));
            }
            _ => panic!("Expected BasicStock"),
        }
    }

    #[test]
    fn test_stock_with_non_negative_true() {
        let xml = r#"
        <stock name="Inventory">
            <eqn>50</eqn>
            <inflow>production</inflow>
            <outflow>sales</outflow>
            <non_negative>true</non_negative>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse non-negative stock");

        match stock {
            Stock::Basic(basic_stock) => {
                assert_eq!(basic_stock.name, "Inventory");
                assert_eq!(basic_stock.non_negative, Some(Some(true)));
            }
            _ => panic!("Expected BasicStock"),
        }
    }

    #[test]
    fn test_stock_with_non_negative_false() {
        let xml = r#"
        <stock name="Balance">
            <eqn>0</eqn>
            <inflow>deposits</inflow>
            <outflow>withdrawals</outflow>
            <non_negative>false</non_negative>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse stock with non_negative=false");

        match stock {
            Stock::Basic(basic_stock) => {
                assert_eq!(basic_stock.name, "Balance");
                assert_eq!(basic_stock.non_negative, Some(Some(false)));
            }
            _ => panic!("Expected BasicStock"),
        }
    }

    #[test]
    fn test_conveyor_stock_basic() {
        let xml = r#"
        <stock name="Students">
            <eqn>1000</eqn>
            <inflow>matriculating</inflow>
            <outflow>graduating</outflow>
            <conveyor>
                <len>4</len>
                <capacity>1200</capacity>
            </conveyor>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse conveyor stock");

        match stock {
            Stock::Conveyor(conveyor_stock) => {
                assert_eq!(conveyor_stock.name, "Students");
                assert_eq!(
                    conveyor_stock.inflows,
                    vec![Identifier::parse_default("matriculating").expect("valid identifier")]
                );
                assert_eq!(
                    conveyor_stock.outflows,
                    vec![Identifier::parse_default("graduating").expect("valid identifier")]
                );
                // The length and capacity would be Expression types that we'd need to verify
                assert!(conveyor_stock.capacity.is_some());
                assert!(conveyor_stock.inflow_limit.is_none()); // Should default to None
                assert_eq!(conveyor_stock.discrete, None); // Should default to None (false)
            }
            _ => panic!("Expected ConveyorStock, got {:?}", stock),
        }
    }

    #[test]
    fn test_conveyor_stock_with_all_options() {
        let xml = r#"
        <stock name="ProductionLine">
            <eqn>500</eqn>
            <inflow>input_flow</inflow>
            <outflow>output_flow</outflow>
            <outflow>leakage_flow</outflow>
            <conveyor discrete="true" batch_integrity="true" one_at_a_time="false" exponential_leak="true">
                <len>8</len>
                <capacity>2000</capacity>
                <in_limit>100</in_limit>
                <sample>TIME > 5</sample>
                <arrest>emergency_stop</arrest>
            </conveyor>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse conveyor with all options");

        match stock {
            Stock::Conveyor(conveyor_stock) => {
                assert_eq!(conveyor_stock.name, "ProductionLine");
                assert_eq!(conveyor_stock.outflows.len(), 2);
                assert_eq!(conveyor_stock.discrete, Some(true));
                assert_eq!(conveyor_stock.batch_integrity, Some(true));
                assert_eq!(conveyor_stock.one_at_a_time, Some(false));
                assert_eq!(conveyor_stock.exponential_leakage, Some(true));
                assert!(conveyor_stock.capacity.is_some());
                assert!(conveyor_stock.inflow_limit.is_some());
                assert!(conveyor_stock.sample.is_some());
                assert!(conveyor_stock.arrest_value.is_some());
            }
            _ => panic!("Expected ConveyorStock"),
        }
    }

    #[test]
    fn test_queue_stock() {
        let xml = r#"
        <stock name="WaitingLine">
            <eqn>0</eqn>
            <inflow>arrivals</inflow>
            <outflow>service</outflow>
            <queue/>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse queue stock");

        match stock {
            Stock::Queue(queue_stock) => {
                assert_eq!(queue_stock.name, "WaitingLine");
                assert_eq!(
                    queue_stock.inflows,
                    vec![Identifier::parse_default("arrivals").expect("valid identifier")]
                );
                assert_eq!(
                    queue_stock.outflows,
                    vec![Identifier::parse_default("service").expect("valid identifier")]
                );
            }
            _ => panic!("Expected QueueStock, got {:?}", stock),
        }
    }

    #[test]
    fn test_stock_with_units_and_documentation() {
        let xml = r#"
        <stock name="Money">
            <eqn>1000</eqn>
            <inflow>income</inflow>
            <outflow>expenses</outflow>
            <units>dollars</units>
            <doc>This represents the amount of money in the account</doc>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse stock with units and docs");

        match stock {
            Stock::Basic(basic_stock) => {
                assert_eq!(basic_stock.name, "Money");
                assert!(basic_stock.units.is_some());
                assert!(basic_stock.documentation.is_some());
            }
            _ => panic!("Expected BasicStock"),
        }
    }

    #[test]
    fn test_stock_with_display_properties() {
        let xml = r#"
        <stock name="Temperature">
            <eqn>20</eqn>
            <inflow>heating</inflow>
            <outflow>cooling</outflow>
            <range min="0" max="100"/>
            <scale min="0" max="100"/>
            <format precision="0.1" scale_by="1" display_as="number" delimit_000s="false"/>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse stock with display properties");

        match stock {
            Stock::Basic(basic_stock) => {
                assert_eq!(basic_stock.name, "Temperature");
                assert!(basic_stock.range.is_some());
                assert!(basic_stock.scale.is_some());
                assert!(basic_stock.format.is_some());
            }
            _ => panic!("Expected BasicStock"),
        }
    }

    #[test]
    fn test_invalid_stock_both_conveyor_and_queue() {
        let xml = r#"
        <stock name="Invalid">
            <eqn>100</eqn>
            <conveyor>
                <len>4</len>
            </conveyor>
            <queue/>
        </stock>
        "#;

        // This should fail validation according to the spec
        let result = from_str::<Stock>(xml);
        // The exact behavior depends on implementation - it might parse but fail validation
        // or fail to parse entirely. We should test both scenarios.

        if let Ok(stock) = result {
            // If it parses, it should fail validation
            let raw_stock: RawStock = stock.into();
            let validation_result = raw_stock.validate();
            assert!(
                validation_result.is_invalid(),
                "Stock with both conveyor and queue should fail validation"
            );
        }
        // If it fails to parse, that's also acceptable behavior
    }

    #[test]
    fn test_conveyor_missing_required_length() {
        let xml = r#"
        <stock name="BrokenConveyor">
            <eqn>100</eqn>
            <inflow>input</inflow>
            <outflow>output</outflow>
            <conveyor>
                <capacity>1000</capacity>
            </conveyor>
        </stock>
        "#;

        // This should fail because length is required for conveyors
        let result = std::panic::catch_unwind(|| {
            let stock: Stock = from_str(xml).expect("Should fail to parse conveyor without length");

            // If parsing succeeds, conversion to ConveyorStock should fail
            if let Stock::Conveyor(_) = stock {
                panic!("ConveyorStock creation should fail without length");
            }
        });

        assert!(result.is_err(), "Conveyor without length should fail");
    }

    #[test]
    fn test_stock_name_with_quotes() {
        let xml = r#"
        <stock name="Complex Name With Spaces">
            <eqn>100</eqn>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse stock with complex name");

        match stock {
            Stock::Basic(basic_stock) => {
                assert_eq!(basic_stock.name, "Complex Name With Spaces");
            }
            _ => panic!("Expected BasicStock"),
        }
    }

    #[test]
    fn test_conveyor_with_minimal_config() {
        let xml = r#"
        <stock name="SimpleConveyor">
            <eqn>0</eqn>
            <inflow>input</inflow>
            <outflow>output</outflow>
            <conveyor>
                <len>2.5</len>
            </conveyor>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse minimal conveyor");

        match stock {
            Stock::Conveyor(conveyor_stock) => {
                assert_eq!(conveyor_stock.name, "SimpleConveyor");
                // All optional fields should be None
                assert!(conveyor_stock.capacity.is_none());
                assert!(conveyor_stock.inflow_limit.is_none());
                assert!(conveyor_stock.sample.is_none());
                assert!(conveyor_stock.arrest_value.is_none());
                assert_eq!(conveyor_stock.discrete, None);
                assert_eq!(conveyor_stock.batch_integrity, None);
                assert_eq!(conveyor_stock.one_at_a_time, None);
                assert_eq!(conveyor_stock.exponential_leakage, None);
            }
            _ => panic!("Expected ConveyorStock"),
        }
    }

    #[test]
    fn test_stock_equation_method() {
        let xml = r#"
        <stock name="TestStock">
            <eqn>100 + 50</eqn>
            <inflow>input</inflow>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse stock");

        match stock {
            Stock::Basic(basic_stock) => {
                // Verify equation() method returns the initial equation
                let equation = basic_stock.equation();
                assert!(equation.is_some(), "equation() should return Some for stocks");
                // The equation should match the initial_equation
                assert_eq!(equation.unwrap(), &basic_stock.initial_equation);
            }
            _ => panic!("Expected BasicStock"),
        }
    }

    #[test]
    fn test_conveyor_stock_equation_method() {
        let xml = r#"
        <stock name="TestConveyor">
            <eqn>200</eqn>
            <inflow>input</inflow>
            <outflow>output</outflow>
            <conveyor>
                <len>5</len>
            </conveyor>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse conveyor stock");

        match stock {
            Stock::Conveyor(conveyor_stock) => {
                // Verify equation() method returns the initial equation
                let equation = conveyor_stock.equation();
                assert!(equation.is_some(), "equation() should return Some for conveyor stocks");
                assert_eq!(equation.unwrap(), &conveyor_stock.initial_equation);
            }
            _ => panic!("Expected ConveyorStock"),
        }
    }

    #[test]
    fn test_queue_stock_equation_method() {
        let xml = r#"
        <stock name="TestQueue">
            <eqn>0</eqn>
            <inflow>arrivals</inflow>
            <outflow>service</outflow>
            <queue/>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse queue stock");

        match stock {
            Stock::Queue(queue_stock) => {
                // Verify equation() method returns the initial equation
                let equation = queue_stock.equation();
                assert!(equation.is_some(), "equation() should return Some for queue stocks");
                assert_eq!(equation.unwrap(), &queue_stock.initial_equation);
            }
            _ => panic!("Expected QueueStock"),
        }
    }

    #[cfg(feature = "mathml")]
    #[test]
    fn test_stock_with_mathml() {
        let xml = r#"
        <stock name="MathMLStock">
            <eqn>100</eqn>
            <mathml>some_mathml_content</mathml>
            <inflow>input</inflow>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse stock with MathML");

        match stock {
            Stock::Basic(basic_stock) => {
                assert_eq!(basic_stock.name, "MathMLStock");
                // Verify MathML is parsed
                assert!(basic_stock.mathml_equation.is_some());
                assert_eq!(
                    basic_stock.mathml_equation.as_ref().unwrap(),
                    "some_mathml_content"
                );
                // Verify mathml_equation() method works
                let mathml = basic_stock.mathml_equation();
                assert!(mathml.is_some());
                assert_eq!(mathml.unwrap(), basic_stock.mathml_equation.as_ref().unwrap());
            }
            _ => panic!("Expected BasicStock"),
        }
    }

    #[cfg(feature = "mathml")]
    #[test]
    fn test_stock_mathml_optional() {
        let xml = r#"
        <stock name="NoMathMLStock">
            <eqn>100</eqn>
            <inflow>input</inflow>
        </stock>
        "#;

        let stock: Stock = from_str(xml).expect("Failed to parse stock without MathML");

        match stock {
            Stock::Basic(basic_stock) => {
                // MathML should be None when not provided
                assert!(basic_stock.mathml_equation.is_none());
                let mathml = basic_stock.mathml_equation();
                assert!(mathml.is_none());
            }
            _ => panic!("Expected BasicStock"),
        }
    }
}
