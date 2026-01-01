use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    Expression, Identifier, Measure, UnitEquation,
    model::{
        events::EventPoster,
        object::{DeviceRange, DeviceScale, Document, Documentation, FormatOptions, Object},
        vars::{AccessType, NonNegativeContent},
    },
};

#[cfg(feature = "arrays")]
use crate::model::vars::array::{ArrayElement, VariableDimensions};

use super::Var;

#[derive(Debug, Error)]
pub enum FlowConversionError {
    #[error("Missing leakage for conveyor flow")]
    MissingLeakage,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Flow {
    Basic(BasicFlow),
    QueueOverflow(QueueOverflow),
    ConveyorLeakage(ConveyorLeakage),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RawFlow {
    // Flow fields
    #[serde(rename = "@name")]
    name: Identifier,
    #[serde(rename = "@access")]
    access: Option<AccessType>,
    #[serde(rename = "@autoexport")]
    autoexport: Option<bool>,
    #[serde(rename = "@leak_start")]
    leak_start: Option<f64>,
    #[serde(rename = "@leak_end")]
    leak_end: Option<f64>,
    #[serde(rename = "eqn")]
    equation: Option<Expression>,
    #[serde(rename = "mathml")]
    mathml_equation: Option<String>,
    #[serde(rename = "multiplier")]
    multiplier: Option<f64>,
    // Non-negative content
    #[serde(rename = "non_negative")]
    non_negative: Option<NonNegativeContent>,
    // QueueOverflow specific fields
    #[serde(rename = "queue_overflow")]
    queue_overflow: Option<OverflowFlag>,
    // ConveyorLeakage specific fields
    #[serde(rename = "leak")]
    leak: Option<LeakContent>,
    #[serde(rename = "leak_integers")]
    leak_integers: Option<LeakIntegersFlag>,
    // Common fields
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

enum FlowKind {
    Normal,
    QueueOverflow,
    ConveyorLeakage,
    NonNegative,
}

impl RawFlow {
    fn flow_kind(&self) -> Result<FlowKind, ()> {
        if self.is_leakage() {
            Ok(FlowKind::ConveyorLeakage)
        } else if self.is_overflow() {
            Ok(FlowKind::QueueOverflow)
        } else if self.is_non_negative() {
            Ok(FlowKind::NonNegative)
        } else if self.is_normal() {
            Ok(FlowKind::Normal)
        } else {
            Err(())
        }
    }

    fn is_leakage(&self) -> bool {
        self.leak.is_some()
    }

    fn is_overflow(&self) -> bool {
        self.queue_overflow.is_some()
    }

    fn is_non_negative(&self) -> bool {
        self.non_negative.map(Into::into).unwrap_or(false)
    }

    fn is_normal(&self) -> bool {
        !self.is_leakage() && !self.is_overflow() && !self.is_non_negative()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
struct LeakIntegersFlag;

impl From<LeakIntegersFlag> for Option<bool> {
    fn from(_: LeakIntegersFlag) -> Self {
        Some(true)
    }
}

impl From<Option<bool>> for LeakIntegersFlag {
    fn from(_: Option<bool>) -> Self {
        LeakIntegersFlag
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
struct OverflowFlag;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
struct LeakContent {
    #[serde(rename = "#text")]
    fraction: Option<f64>,
}

impl From<Option<f64>> for LeakContent {
    fn from(fraction: Option<f64>) -> Self {
        LeakContent { fraction }
    }
}

impl From<&Flow> for RawFlow {
    fn from(flow: &Flow) -> Self {
        match flow {
            Flow::Basic(basic_flow) => RawFlow::from(basic_flow),
            Flow::QueueOverflow(queue_overflow) => RawFlow::from(queue_overflow),
            Flow::ConveyorLeakage(conveyor_leakage) => RawFlow::from(conveyor_leakage),
        }
    }
}

impl From<&BasicFlow> for RawFlow {
    fn from(flow: &BasicFlow) -> Self {
        RawFlow {
            name: flow.name.clone(),
            access: flow.access,
            autoexport: flow.autoexport,
            equation: flow.equation.clone(),
            mathml_equation: flow.mathml_equation.clone(),
            multiplier: flow.multiplier,
            non_negative: flow.non_negative.map(Into::into),
            queue_overflow: None,
            leak: None,
            leak_integers: None,
            leak_start: None,
            leak_end: None,
            units: flow.units.clone(),
            documentation: flow.documentation.clone(),
            range: flow.range,
            scale: flow.scale,
            format: flow.format,
            #[cfg(feature = "arrays")]
            dimensions: flow.dimensions.as_ref().map(|dims| {
                use crate::model::vars::array::{Dimension, VariableDimensions};
                VariableDimensions {
                    dims: dims.iter().map(|name| Dimension { name: name.clone() }).collect(),
                }
            }),
            #[cfg(feature = "arrays")]
            elements: flow.elements.clone(),
            event_poster: flow.event_poster.clone(),
        }
    }
}

impl From<&QueueOverflow> for RawFlow {
    fn from(flow: &QueueOverflow) -> Self {
        RawFlow {
            name: flow.name.clone(),
            access: flow.access,
            autoexport: flow.autoexport,
            equation: flow.equation.clone(),
            mathml_equation: flow.mathml_equation.clone(),
            multiplier: flow.multiplier,
            non_negative: None,
            queue_overflow: Some(OverflowFlag),
            leak: None,
            leak_integers: None,
            leak_start: None,
            leak_end: None,
            units: flow.units.clone(),
            documentation: flow.documentation.clone(),
            range: flow.range,
            scale: flow.scale,
            format: flow.format,
            #[cfg(feature = "arrays")]
            dimensions: flow.dimensions.as_ref().map(|dims| {
                use crate::model::vars::array::{Dimension, VariableDimensions};
                VariableDimensions {
                    dims: dims.iter().map(|name| Dimension { name: name.clone() }).collect(),
                }
            }),
            #[cfg(feature = "arrays")]
            elements: flow.elements.clone(),
            event_poster: flow.event_poster.clone(),
        }
    }
}

impl From<&ConveyorLeakage> for RawFlow {
    fn from(flow: &ConveyorLeakage) -> Self {
        RawFlow {
            name: flow.name.clone(),
            access: flow.access,
            autoexport: flow.autoexport,
            equation: flow.equation.clone(),
            mathml_equation: flow.mathml_equation.clone(),
            multiplier: flow.multiplier,
            non_negative: None,
            queue_overflow: None,
            leak: Some(flow.leak.into()),
            leak_integers: flow.leak_integers.map(Into::into),
            leak_start: flow.leak_start,
            leak_end: flow.leak_end,
            units: flow.units.clone(),
            documentation: flow.documentation.clone(),
            range: flow.range,
            scale: flow.scale,
            format: flow.format,
            #[cfg(feature = "arrays")]
            dimensions: flow.dimensions.as_ref().map(|dims| {
                use crate::model::vars::array::{Dimension, VariableDimensions};
                VariableDimensions {
                    dims: dims.iter().map(|name| Dimension { name: name.clone() }).collect(),
                }
            }),
            #[cfg(feature = "arrays")]
            elements: flow.elements.clone(),
            event_poster: flow.event_poster.clone(),
        }
    }
}

impl<'de> Deserialize<'de> for Flow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw_flow = RawFlow::deserialize(deserializer)?;

        let flow_kind = raw_flow.flow_kind().map_err(|_| {
            serde::de::Error::custom(
                "Stock must be either a conveyor, queue, non-negative, or normal stock",
            )
        })?;

        match flow_kind {
            FlowKind::Normal | FlowKind::NonNegative => Ok(Flow::Basic(BasicFlow::from(raw_flow))),
            FlowKind::QueueOverflow => Ok(Flow::QueueOverflow(QueueOverflow::from(raw_flow))),
            FlowKind::ConveyorLeakage => Ok(Flow::ConveyorLeakage(
                ConveyorLeakage::try_from(raw_flow).map_err(serde::de::Error::custom)?,
            )),
        }
    }
}

impl Serialize for Flow {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let raw_flow = RawFlow::from(self);
        raw_flow.serialize(serializer)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicFlow {
    pub name: Identifier,

    pub access: Option<AccessType>,
    pub autoexport: Option<bool>,

    pub equation: Option<Expression>,
    pub mathml_equation: Option<String>,

    pub multiplier: Option<f64>,

    pub non_negative: Option<Option<bool>>,

    pub units: Option<UnitEquation>,

    pub documentation: Option<Documentation>,

    pub range: Option<DeviceRange>,
    pub scale: Option<DeviceScale>,
    pub format: Option<FormatOptions>,

    /// The dimensions for this flow (if it's an array).
    #[cfg(feature = "arrays")]
    pub dimensions: Option<Vec<String>>,

    /// Array elements for non-apply-to-all arrays.
    #[cfg(feature = "arrays")]
    pub elements: Vec<ArrayElement>,

    /// Optional event poster for triggering events based on flow values.
    pub event_poster: Option<EventPoster>,
}

// BasicFlow serializes/deserializes via RawFlow
impl<'de> Deserialize<'de> for BasicFlow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw: RawFlow = Deserialize::deserialize(deserializer)?;
        Ok(BasicFlow::from(raw))
    }
}

impl Serialize for BasicFlow {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let raw = RawFlow::from(self);
        raw.serialize(serializer)
    }
}

impl Var<'_> for BasicFlow {
    fn name(&self) -> Option<&Identifier> {
        Some(&self.name)
    }

    fn equation(&self) -> Option<&Expression> {
        self.equation.as_ref()
    }

    #[cfg(feature = "mathml")]
    fn mathml_equation(&self) -> Option<&String> {
        self.mathml_equation.as_ref()
    }
}

impl Object for BasicFlow {
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

impl Measure for BasicFlow {
    fn units(&self) -> Option<&UnitEquation> {
        self.units.as_ref()
    }
}

impl Document for BasicFlow {
    fn documentation(&self) -> Option<&Documentation> {
        self.documentation.as_ref()
    }
}

impl From<RawFlow> for BasicFlow {
    fn from(raw: RawFlow) -> Self {
        BasicFlow {
            name: raw.name,
            access: raw.access,
            autoexport: raw.autoexport,
            equation: raw.equation,
            mathml_equation: raw.mathml_equation,
            multiplier: raw.multiplier,
            non_negative: raw.non_negative.map(Into::into),
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
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueueOverflow {
    pub name: Identifier,

    pub access: Option<AccessType>,
    pub autoexport: Option<bool>,

    pub equation: Option<Expression>,
    pub mathml_equation: Option<String>,

    pub multiplier: Option<f64>,

    pub units: Option<UnitEquation>,

    pub documentation: Option<Documentation>,

    pub range: Option<DeviceRange>,
    pub scale: Option<DeviceScale>,
    pub format: Option<FormatOptions>,

    /// The dimensions for this queue overflow flow (if it's an array).
    #[cfg(feature = "arrays")]
    pub dimensions: Option<Vec<String>>,

    /// Array elements for non-apply-to-all arrays.
    #[cfg(feature = "arrays")]
    pub elements: Vec<ArrayElement>,

    /// Optional event poster for triggering events based on flow values.
    pub event_poster: Option<EventPoster>,
}

impl Var<'_> for QueueOverflow {
    fn name(&self) -> Option<&Identifier> {
        Some(&self.name)
    }

    fn equation(&self) -> Option<&Expression> {
        self.equation.as_ref()
    }

    #[cfg(feature = "mathml")]
    fn mathml_equation(&self) -> Option<&String> {
        self.mathml_equation.as_ref()
    }
}

impl Object for QueueOverflow {
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

impl Measure for QueueOverflow {
    fn units(&self) -> Option<&UnitEquation> {
        self.units.as_ref()
    }
}

impl Document for QueueOverflow {
    fn documentation(&self) -> Option<&Documentation> {
        self.documentation.as_ref()
    }
}

impl From<RawFlow> for QueueOverflow {
    fn from(raw: RawFlow) -> Self {
        QueueOverflow {
            name: raw.name,
            access: raw.access,
            autoexport: raw.autoexport,
            equation: raw.equation,
            mathml_equation: raw.mathml_equation,
            multiplier: raw.multiplier,
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
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConveyorLeakage {
    pub name: Identifier,

    pub access: Option<AccessType>,
    pub autoexport: Option<bool>,

    pub equation: Option<Expression>,
    pub mathml_equation: Option<String>,

    pub multiplier: Option<f64>,

    pub leak: Option<f64>,
    pub leak_integers: Option<Option<bool>>,
    pub leak_start: Option<f64>,
    pub leak_end: Option<f64>,

    pub units: Option<UnitEquation>,

    pub documentation: Option<Documentation>,

    pub range: Option<DeviceRange>,
    pub scale: Option<DeviceScale>,
    pub format: Option<FormatOptions>,

    /// The dimensions for this conveyor leakage flow (if it's an array).
    #[cfg(feature = "arrays")]
    pub dimensions: Option<Vec<String>>,

    /// Array elements for non-apply-to-all arrays.
    #[cfg(feature = "arrays")]
    pub elements: Vec<ArrayElement>,

    /// Optional event poster for triggering events based on flow values.
    pub event_poster: Option<EventPoster>,
}

impl Var<'_> for ConveyorLeakage {
    fn name(&self) -> Option<&Identifier> {
        Some(&self.name)
    }

    fn equation(&self) -> Option<&Expression> {
        self.equation.as_ref()
    }

    #[cfg(feature = "mathml")]
    fn mathml_equation(&self) -> Option<&String> {
        self.mathml_equation.as_ref()
    }
}

impl Object for ConveyorLeakage {
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

impl Measure for ConveyorLeakage {
    fn units(&self) -> Option<&UnitEquation> {
        self.units.as_ref()
    }
}

impl Document for ConveyorLeakage {
    fn documentation(&self) -> Option<&Documentation> {
        self.documentation.as_ref()
    }
}

impl TryFrom<RawFlow> for ConveyorLeakage {
    type Error = FlowConversionError;

    fn try_from(raw: RawFlow) -> Result<Self, Self::Error> {
        Ok(ConveyorLeakage {
            name: raw.name,
            access: raw.access,
            autoexport: raw.autoexport,
            equation: raw.equation,
            mathml_equation: raw.mathml_equation,
            multiplier: raw.multiplier,
            leak: raw
                .leak
                .ok_or(FlowConversionError::MissingLeakage)?
                .fraction,
            leak_integers: raw.leak_integers.map(Into::into),
            leak_start: raw.leak_start,
            leak_end: raw.leak_end,
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_xml_rs::from_str;

    #[test]
    fn test_basic_flow() {
        let xml = r#"<flow name="increasing">
   <eqn>rewards*reward_multiplier</eqn>
</flow>"#;

        let flow: Flow = from_str(xml).expect("Failed to parse basic flow");

        match flow {
            Flow::Basic(basic_flow) => {
                assert_eq!(basic_flow.name, "increasing");
                assert!(basic_flow.equation.is_some());
                // assert_eq!(basic_flow.equation.unwrap(), "rewards*reward_multiplier");
                assert!(basic_flow.multiplier.is_none());
                assert!(basic_flow.non_negative.is_none());
            }
            _ => panic!("Expected Basic flow"),
        }
    }

    #[test]
    fn test_flow_with_multiplier() {
        let xml = r#"<flow name="unit_converter">
   <eqn>base_flow</eqn>
   <multiplier>3</multiplier>
</flow>"#;

        let flow: Flow = from_str(xml).expect("Failed to parse flow with multiplier");

        match flow {
            Flow::Basic(basic_flow) => {
                // Note: raw() preserves quotes as they appear in XML
                assert_eq!(basic_flow.name.raw(), "\"unit_converter\"");
                assert!(basic_flow.equation.is_some());
                // assert_eq!(basic_flow.equation.unwrap(), "base_flow");
                assert!(basic_flow.multiplier.is_some());
                assert_eq!(basic_flow.multiplier.unwrap(), 3.0);
            }
            _ => panic!("Expected Basic flow"),
        }
    }

    #[test]
    fn test_non_negative_flow() {
        let xml = r#"<flow name="increasing">
   <eqn>rewards*reward_multiplier</eqn>
   <non_negative/>
</flow>"#;

        let flow: Flow = from_str(xml).expect("Failed to parse non-negative flow");

        match flow {
            Flow::Basic(basic_flow) => {
                assert_eq!(basic_flow.name, "increasing");
                assert!(basic_flow.equation.is_some());
                // assert_eq!(basic_flow.equation.unwrap(), "rewards*reward_multiplier");
                assert!(basic_flow.non_negative.is_some());
                assert_eq!(basic_flow.non_negative.unwrap(), None);
            }
            _ => panic!("Expected Basic flow"),
        }
    }

    #[test]
    fn test_non_negative_false_flow() {
        let xml = r#"<flow name="bidirectional">
   <eqn>some_expression</eqn>
   <non_negative>false</non_negative>
</flow>"#;

        let flow: Flow = from_str(xml).expect("Failed to parse non-negative false flow");

        match flow {
            Flow::Basic(basic_flow) => {
                assert_eq!(basic_flow.name, "bidirectional");
                assert!(basic_flow.non_negative.is_some());
                assert_eq!(basic_flow.non_negative.unwrap(), Some(false));
            }
            _ => panic!("Expected Basic flow"),
        }
    }

    #[test]
    fn test_queue_overflow_flow() {
        let xml = r#"<flow name="overflow_flow">
   <queue_overflow/>
</flow>"#;

        let flow: Flow = from_str(xml).expect("Failed to parse queue overflow flow");

        match flow {
            Flow::QueueOverflow(queue_overflow) => {
                assert_eq!(queue_overflow.name.raw(), "\"overflow_flow\""); // TODO
                assert!(queue_overflow.equation.is_none()); // Queue outflows don't have equations
            }
            _ => panic!("Expected QueueOverflow flow"),
        }
    }

    #[test]
    fn test_conveyor_leakage_simple() {
        let xml = r#"<flow name="shrinking">
   <leak/>
</flow>"#;

        let flow: Flow = from_str(xml).expect("Failed to parse simple conveyor leakage");

        match flow {
            Flow::ConveyorLeakage(conveyor_leakage) => {
                // Note: raw() preserves quotes as they appear in XML
                assert_eq!(conveyor_leakage.name.raw(), "\"shrinking\"");
                assert!(conveyor_leakage.equation.is_none());
                assert!(conveyor_leakage.leak.is_none()); // No fraction specified
                assert!(conveyor_leakage.leak_integers.is_none());
                assert!(conveyor_leakage.leak_start.is_none());
                assert!(conveyor_leakage.leak_end.is_none());
            }
            _ => panic!("Expected ConveyorLeakage flow"),
        }
    }

    #[test]
    fn test_conveyor_leakage_with_fraction() {
        let xml = r#"<flow name="attriting" leak_end="0.25">
   <leak>0.1</leak>
</flow>"#;

        let flow: Flow = from_str(xml).expect("Failed to parse conveyor leakage with fraction");

        match flow {
            Flow::ConveyorLeakage(conveyor_leakage) => {
                assert_eq!(conveyor_leakage.name.raw(), "\"attriting\""); // TODO
                assert!(conveyor_leakage.equation.is_none());
                assert!(conveyor_leakage.leak.is_some());
                assert_eq!(conveyor_leakage.leak.unwrap(), 0.1);
                assert!(conveyor_leakage.leak_end.is_some());
                assert_eq!(conveyor_leakage.leak_end.unwrap(), 0.25);
            }
            _ => panic!("Expected ConveyorLeakage flow"),
        }
    }

    #[test]
    fn test_conveyor_leakage_full_options() {
        let xml = r#"<flow name="complex_leak" leak_start="0.2" leak_end="0.8">
   <eqn>some_calculation</eqn>
   <leak>0.05</leak>
   <leak_integers/>
   <units>items/day</units>
   <doc>A complex leakage flow</doc>
</flow>"#;

        let flow: Flow = from_str(xml).expect("Failed to parse complex conveyor leakage");

        match flow {
            Flow::ConveyorLeakage(conveyor_leakage) => {
                // Note: raw() preserves quotes as they appear in XML
                assert_eq!(conveyor_leakage.name.raw(), "\"complex_leak\"");
                assert!(conveyor_leakage.equation.is_some());
                // assert_eq!(conveyor_leakage.equation.unwrap(), "some_calculation");
                assert!(conveyor_leakage.leak.is_some());
                assert_eq!(conveyor_leakage.leak.unwrap(), 0.05);
                assert!(conveyor_leakage.leak_integers.is_some());
                assert_eq!(conveyor_leakage.leak_integers.unwrap(), Some(true));
                assert!(conveyor_leakage.leak_start.is_some());
                assert_eq!(conveyor_leakage.leak_start.unwrap(), 0.2);
                assert!(conveyor_leakage.leak_end.is_some());
                assert_eq!(conveyor_leakage.leak_end.unwrap(), 0.8);
                assert!(conveyor_leakage.units.is_some());
                assert!(conveyor_leakage.documentation.is_some());
            }
            _ => panic!("Expected ConveyorLeakage flow"),
        }
    }

    #[test]
    fn test_flow_with_common_properties() {
        let xml = r#"<flow name="documented_flow">
   <eqn>complex_expression</eqn>
   <mathml>some_mathml</mathml>
   <multiplier>2.5</multiplier>
   <units>kg/s</units>
   <doc>This is a documented flow</doc>
   <range min="0" max="100"/>
   <scale min="0" max="1000"/>
   <format precision="0.01" delimit_000s="true"/>
</flow>"#;

        let flow: Flow = from_str(xml).expect("Failed to parse flow with common properties");

        match flow {
            Flow::Basic(basic_flow) => {
                assert_eq!(basic_flow.name.raw(), "\"documented_flow\""); // TODO
                assert!(basic_flow.equation.is_some());
                // assert_eq!(basic_flow.equation.unwrap(), "complex_expression");
                assert!(basic_flow.mathml_equation.is_some());
                assert_eq!(basic_flow.mathml_equation.unwrap(), "some_mathml");
                assert!(basic_flow.multiplier.is_some());
                assert_eq!(basic_flow.multiplier.unwrap(), 2.5);
                assert!(basic_flow.units.is_some());
                assert!(basic_flow.documentation.is_some());
                assert!(basic_flow.range.is_some());
                assert!(basic_flow.scale.is_some());
                assert!(basic_flow.format.is_some());
            }
            _ => panic!("Expected Basic flow"),
        }
    }

    #[test]
    fn test_flow_name_only() {
        let xml = r#"<flow name="minimal_flow">
</flow>"#;

        let flow: Flow = from_str(xml).expect("Failed to parse minimal flow");

        match flow {
            Flow::Basic(basic_flow) => {
                // Note: raw() preserves quotes as they appear in XML
                assert_eq!(basic_flow.name.raw(), "\"minimal_flow\"");
                assert!(basic_flow.equation.is_none());
                assert!(basic_flow.multiplier.is_none());
                assert!(basic_flow.non_negative.is_none());
            }
            _ => panic!("Expected Basic flow"),
        }
    }

    #[test]
    fn test_flow_type_detection() {
        // Test that flows are correctly classified based on their content

        // Normal flow
        let normal_xml = r#"<flow name="normal"><eqn>x</eqn></flow>"#;
        let normal_flow: Flow = from_str(normal_xml).unwrap();
        assert!(matches!(normal_flow, Flow::Basic(_)));

        // Non-negative flow
        let non_neg_xml = r#"<flow name="non_neg"><eqn>x</eqn><non_negative/></flow>"#;
        let non_neg_flow: Flow = from_str(non_neg_xml).unwrap();
        assert!(matches!(non_neg_flow, Flow::Basic(_)));

        // Queue overflow flow
        let queue_xml = r#"<flow name="queue"><queue_overflow/></flow>"#;
        let queue_flow: Flow = from_str(queue_xml).unwrap();
        assert!(matches!(queue_flow, Flow::QueueOverflow(_)));

        // Conveyor leakage flow
        let leak_xml = r#"<flow name="leak"><leak/></flow>"#;
        let leak_flow: Flow = from_str(leak_xml).unwrap();
        assert!(matches!(leak_flow, Flow::ConveyorLeakage(_)));
    }

    #[test]
    fn test_quoted_flow_names() {
        let xml = r#"<flow name="flow with spaces">
   <eqn>some_value</eqn>
</flow>"#;

        let flow: Flow = from_str(xml).expect("Failed to parse flow with quoted name");

        match flow {
            Flow::Basic(basic_flow) => {
                // Note: raw() preserves quotes as they appear in XML
                assert_eq!(basic_flow.name.raw(), "\"flow with spaces\"");
            }
            _ => panic!("Expected Basic flow"),
        }
    }

    #[test]
    fn test_flow_serialization_roundtrip() {
        use serde_xml_rs::to_string;

        let original_xml = r#"<flow name="test_flow">
   <eqn>x + y</eqn>
   <multiplier>1.5</multiplier>
   <non_negative/>
</flow>"#;

        let flow: Flow = from_str(original_xml).expect("Failed to parse flow");
        let serialized = to_string(&flow).expect("Failed to serialize flow");
        let reparsed: Flow = from_str(&serialized).expect("Failed to reparse flow");

        // Verify the roundtrip preserves the data
        match (&flow, &reparsed) {
            (Flow::Basic(orig), Flow::Basic(reparsed)) => {
                assert_eq!(orig.name, reparsed.name);
                assert_eq!(orig.equation, reparsed.equation);
                assert_eq!(orig.multiplier, reparsed.multiplier);
                assert_eq!(orig.non_negative, reparsed.non_negative);
            }
            _ => panic!("Flow types don't match after roundtrip"),
        }
    }
}
