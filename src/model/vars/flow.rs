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
                    dims: dims
                        .iter()
                        .map(|name| Dimension { name: name.clone() })
                        .collect(),
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
                    dims: dims
                        .iter()
                        .map(|name| Dimension { name: name.clone() })
                        .collect(),
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
                    dims: dims
                        .iter()
                        .map(|name| Dimension { name: name.clone() })
                        .collect(),
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
            dimensions: raw
                .dimensions
                .map(|dims| dims.dims.into_iter().map(|d| d.name).collect()),
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
            dimensions: raw
                .dimensions
                .map(|dims| dims.dims.into_iter().map(|d| d.name).collect()),
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
            dimensions: raw
                .dimensions
                .map(|dims| dims.dims.into_iter().map(|d| d.name).collect()),
            #[cfg(feature = "arrays")]
            elements: raw.elements,
            event_poster: raw.event_poster,
        })
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::test_utils::parse_flow;

    #[test]
    fn test_basic_flow() {
        let xml = r#"<flow name="increasing">
   <eqn>rewards*reward_multiplier</eqn>
</flow>"#;

        let basic_flow = parse_flow(xml);
        assert_eq!(basic_flow.name, "increasing");
        assert!(basic_flow.equation.is_some());
        assert!(basic_flow.multiplier.is_none());
        assert!(basic_flow.non_negative.is_none());
    }

    #[test]
    fn test_flow_with_multiplier() {
        let xml = r#"<flow name="unit_converter">
   <eqn>base_flow</eqn>
   <multiplier>3</multiplier>
</flow>"#;

        let basic_flow = parse_flow(xml);
        assert_eq!(basic_flow.name, "unit_converter");
        assert!(basic_flow.equation.is_some());
        assert!(basic_flow.multiplier.is_some());
        assert_eq!(basic_flow.multiplier.unwrap(), 3.0);
    }

    #[test]
    fn test_non_negative_flow() {
        let xml = r#"<flow name="increasing">
   <eqn>rewards*reward_multiplier</eqn>
   <non_negative/>
</flow>"#;

        let basic_flow = parse_flow(xml);
        assert_eq!(basic_flow.name, "increasing");
        assert!(basic_flow.equation.is_some());
        assert!(basic_flow.non_negative.is_some());
        assert_eq!(basic_flow.non_negative.unwrap(), None);
    }

    #[test]
    fn test_non_negative_false_flow() {
        let xml = r#"<flow name="bidirectional">
   <eqn>some_expression</eqn>
   <non_negative>false</non_negative>
</flow>"#;

        let basic_flow = parse_flow(xml);
        assert_eq!(basic_flow.name, "bidirectional");
        assert!(basic_flow.non_negative.is_some());
        assert_eq!(basic_flow.non_negative.unwrap(), Some(false));
    }

    #[test]
    fn test_flow_with_common_properties() {
        let xml = r#"<flow name="documented_flow">
   <eqn>complex_expression</eqn>
   <multiplier>2.5</multiplier>
   <units>kg/s</units>
   <doc>This is a documented flow</doc>
   <range min="0" max="100"/>
   <scale min="0" max="1000"/>
   <format precision="0.01" delimit_000s="true"/>
</flow>"#;

        let basic_flow = parse_flow(xml);
        assert_eq!(basic_flow.name, "documented_flow");
        assert!(basic_flow.equation.is_some());
        assert!(basic_flow.multiplier.is_some());
        assert_eq!(basic_flow.multiplier.unwrap(), 2.5);
        assert!(basic_flow.units.is_some());
        assert!(basic_flow.documentation.is_some());
        assert!(basic_flow.range.is_some());
        assert!(basic_flow.scale.is_some());
        assert!(basic_flow.format.is_some());
    }

    #[test]
    fn test_flow_name_only() {
        let xml = r#"<flow name="minimal_flow"></flow>"#;

        let basic_flow = parse_flow(xml);
        assert_eq!(basic_flow.name, "minimal_flow");
        assert!(basic_flow.equation.is_none());
        assert!(basic_flow.multiplier.is_none());
        assert!(basic_flow.non_negative.is_none());
    }

    #[test]
    fn test_quoted_flow_names() {
        let xml = r#"<flow name="flow with spaces">
   <eqn>some_value</eqn>
</flow>"#;

        let basic_flow = parse_flow(xml);
        // Identifier normalizes names with spaces
        assert_eq!(basic_flow.name, "flow with spaces");
    }

    #[test]
    fn test_flow_serialization_roundtrip() {
        use crate::test_utils::wrap_variable_xml;
        use crate::xml::XmileFile;

        let flow_xml = r#"<flow name="test_flow">
   <eqn>x + y</eqn>
   <multiplier>1.5</multiplier>
   <non_negative/>
</flow>"#;

        let full_xml = wrap_variable_xml(flow_xml);
        let file = XmileFile::from_str(&full_xml).expect("Failed to parse XMILE file");

        // Serialize to XML
        let serialized = file.to_xml().expect("Failed to serialize");

        // Re-parse the serialized XML
        let file2 = XmileFile::from_str(&serialized).expect("Failed to re-parse XMILE file");

        // Verify the roundtrip preserves the data
        let orig = &file.models[0].variables.variables[0];
        let reparsed = &file2.models[0].variables.variables[0];

        match (orig, reparsed) {
            (
                crate::model::vars::Variable::Flow(orig),
                crate::model::vars::Variable::Flow(reparsed),
            ) => {
                assert_eq!(orig.name, reparsed.name);
                assert_eq!(orig.equation, reparsed.equation);
                assert_eq!(orig.multiplier, reparsed.multiplier);
                assert_eq!(orig.non_negative, reparsed.non_negative);
            }
            _ => panic!("Flow types don't match after roundtrip"),
        }
    }
}
