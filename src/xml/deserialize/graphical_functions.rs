//! Graphical function deserialization module.
//!
//! This module handles deserialization of graphical function definitions,
//! including scales, points, and function data.

use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::BufRead;
use std::str::FromStr;

use crate::Expression;
use crate::equation::Identifier;
use crate::equation::units::UnitEquation;
use crate::model::object::{DeviceRange, DeviceScale, DisplayAs, Documentation, FormatOptions};
#[cfg(feature = "arrays")]
use crate::model::vars::array::{ArrayElement, VariableDimensions};
use crate::model::vars::gf::{
    GraphicalFunction, GraphicalFunctionData, GraphicalFunctionPoints, GraphicalFunctionScale,
    GraphicalFunctionType,
};
use crate::xml::deserialize::DeserializeError;
use crate::xml::deserialize::helpers::read_text_content;
use crate::xml::quick::de::skip_element;

/// Deserialize a GraphicalFunction from XML.
pub fn deserialize_graphical_function<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<GraphicalFunction, DeserializeError> {
    // Expect <gf> start tag
    let mut name: Option<Identifier> = None;
    let mut r#type: Option<GraphicalFunctionType> = None;

    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"gf" => {
            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"name" => {
                        let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                        name = Some(Identifier::parse_from_attribute(&name_str).map_err(|e| {
                            DeserializeError::Custom(format!("Invalid identifier: {}", e))
                        })?);
                    }
                    b"type" => {
                        let type_str = attr.decode_and_unescape_value(reader)?.to_string();
                        r#type = Some(GraphicalFunctionType::from_str(&type_str).map_err(|e| {
                            DeserializeError::Custom(format!("Invalid function type: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }
        }
        Event::Start(e) => {
            return Err(DeserializeError::UnexpectedElement {
                expected: "gf".to_string(),
                found: String::from_utf8_lossy(e.name().as_ref()).to_string(),
            });
        }
        _ => {
            return Err(DeserializeError::Custom(
                "Expected gf start tag".to_string(),
            ));
        }
    }
    buf.clear();
    deserialize_graphical_function_impl(reader, buf, name, r#type)
}

/// Internal implementation of GraphicalFunction deserialization.
/// Used when the start tag has already been consumed.
pub(crate) fn deserialize_graphical_function_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<Identifier>,
    r#type: Option<GraphicalFunctionType>,
) -> Result<GraphicalFunction, DeserializeError> {
    // Import functions from deserialize module that we still need
    // These functions are still in deserialize.rs (not yet extracted to separate modules)
    // Note: We need to use the full path since these are in a sibling module
    // For now, we'll call them directly using the full path

    let mut equation: Option<Expression> = None;
    let mut mathml_equation: Option<String> = None;
    let mut documentation: Option<Documentation> = None;
    let mut units: Option<UnitEquation> = None;
    let mut range: Option<DeviceRange> = None;
    let mut scale: Option<DeviceScale> = None;
    let mut format: Option<FormatOptions> = None;
    let mut x_scale: Option<GraphicalFunctionScale> = None;
    let mut y_scale: Option<GraphicalFunctionScale> = None;
    let mut y_pts: Option<GraphicalFunctionPoints> = None;
    let mut x_pts: Option<GraphicalFunctionPoints> = None;
    #[cfg(feature = "arrays")]
    let mut dimensions: Option<VariableDimensions> = None;
    #[cfg(feature = "arrays")]
    let mut elements: Vec<ArrayElement> = Vec::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"eqn" => {
                        equation = Some(crate::xml::deserialize::read_expression(reader, buf)?);
                    }
                    b"mathml" => {
                        mathml_equation = Some(read_text_content(reader, buf)?);
                    }
                    b"doc" => {
                        let doc_text = read_text_content(reader, buf)?;
                        documentation = Some(
                            if doc_text.trim().contains('<') && doc_text.trim().contains('>') {
                                Documentation::Html(doc_text)
                            } else {
                                Documentation::PlainText(doc_text)
                            },
                        );
                    }
                    b"units" => {
                        let units_str = read_text_content(reader, buf)?;
                        use crate::equation::parse::unit_equation;
                        let (remaining, unit_eqn) = unit_equation(&units_str).map_err(|e| {
                            DeserializeError::Custom(format!(
                                "Failed to parse unit equation: {}",
                                e
                            ))
                        })?;
                        if !remaining.is_empty() {
                            return Err(DeserializeError::Custom(format!(
                                "Unexpected trailing characters after unit equation: '{}'",
                                remaining
                            )));
                        }
                        units = Some(unit_eqn);
                    }
                    b"range" => {
                        // Extract attributes before clearing buf
                        let mut min_val: Option<f64> = None;
                        let mut max_val: Option<f64> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min_val = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max_val = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid max: {}", err))
                                    })?);
                                }
                                _ => {}
                            }
                        }
                        buf.clear();
                        // Skip to end of range element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == b"range" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        if let (Some(min), Some(max)) = (min_val, max_val) {
                            range = Some(DeviceRange { min, max });
                        }
                    }
                    b"scale" => {
                        // Extract attributes before clearing buf
                        let mut min_val: Option<f64> = None;
                        let mut max_val: Option<f64> = None;
                        let mut auto: Option<bool> = None;
                        let mut group: Option<u32> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min_val = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max_val = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid max: {}", err))
                                    })?);
                                }
                                b"auto" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    auto = Some(s == "true");
                                }
                                b"group" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    group = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid group: {}", err))
                                    })?);
                                }
                                _ => {}
                            }
                        }
                        buf.clear();
                        // Skip to end of scale element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == b"scale" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        if let Some(auto_val) = auto {
                            scale = Some(DeviceScale::Auto(auto_val));
                        } else if let Some(group_val) = group {
                            scale = Some(DeviceScale::Group(group_val));
                        } else if let (Some(min), Some(max)) = (min_val, max_val) {
                            scale = Some(DeviceScale::MinMax { min, max });
                        }
                    }
                    b"format" => {
                        // Extract attributes before clearing buf
                        let mut precision: Option<f64> = None;
                        let mut scale_by: Option<f64> = None;
                        let mut display_as: Option<DisplayAs> = None;
                        let mut delimit_000s: Option<bool> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"precision" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    precision = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid precision: {}",
                                            err
                                        ))
                                    })?);
                                }
                                b"scale_by" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    scale_by = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid scale_by: {}",
                                            err
                                        ))
                                    })?);
                                }
                                b"display_as" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    display_as = Some(match s.as_str() {
                                        "number" => DisplayAs::Number,
                                        "currency" => DisplayAs::Currency,
                                        "percent" => DisplayAs::Percent,
                                        _ => {
                                            return Err(DeserializeError::Custom(format!(
                                                "Invalid display_as: {}",
                                                s
                                            )));
                                        }
                                    });
                                }
                                b"delimit_000s" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    delimit_000s = Some(s == "true");
                                }
                                _ => {}
                            }
                        }
                        buf.clear();
                        // Skip to end of format element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == b"format" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        format = Some(FormatOptions {
                            precision,
                            scale_by,
                            display_as,
                            delimit_000s,
                        });
                    }
                    b"xscale" => {
                        // Handle xscale with content (if any) - already read start event
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let val = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(val.parse::<f64>().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let val = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(val.parse::<f64>().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid max: {}", err))
                                    })?);
                                }
                                _ => {}
                            }
                        }
                        buf.clear();
                        // Read to end of xscale element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end_e) if end_e.name().as_ref() == b"xscale" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        x_scale = Some(GraphicalFunctionScale {
                            min: min.ok_or_else(|| {
                                DeserializeError::MissingField("xscale.min".to_string())
                            })?,
                            max: max.ok_or_else(|| {
                                DeserializeError::MissingField("xscale.max".to_string())
                            })?,
                        });
                    }
                    b"yscale" => {
                        // Handle yscale with content (if any) - already read start event
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let val = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(val.parse::<f64>().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let val = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(val.parse::<f64>().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid max: {}", err))
                                    })?);
                                }
                                _ => {}
                            }
                        }
                        buf.clear();
                        // Read to end of yscale element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end_e) if end_e.name().as_ref() == b"yscale" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        y_scale = Some(GraphicalFunctionScale {
                            min: min.ok_or_else(|| {
                                DeserializeError::MissingField("yscale.min".to_string())
                            })?,
                            max: max.ok_or_else(|| {
                                DeserializeError::MissingField("yscale.max".to_string())
                            })?,
                        });
                    }
                    b"ypts" => {
                        // Extract separator from attributes
                        let mut separator: Option<String> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            if attr.key.as_ref() == b"sep" {
                                separator =
                                    Some(attr.decode_and_unescape_value(reader)?.to_string());
                            }
                        }
                        // Read text content
                        let data_text = read_text_content(reader, buf)?;
                        let sep = separator.as_deref().unwrap_or(",");
                        let values: Result<Vec<f64>, _> = data_text
                            .split(sep)
                            .map(|s| s.trim().parse::<f64>())
                            .collect();
                        y_pts = Some(GraphicalFunctionPoints {
                            values: values.map_err(|err| {
                                DeserializeError::Custom(format!("Invalid point value: {}", err))
                            })?,
                            separator,
                        });
                    }
                    b"xpts" => {
                        // Extract separator from attributes
                        let mut separator: Option<String> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            if attr.key.as_ref() == b"sep" {
                                separator =
                                    Some(attr.decode_and_unescape_value(reader)?.to_string());
                            }
                        }
                        // Read text content
                        let data_text = read_text_content(reader, buf)?;
                        let sep = separator.as_deref().unwrap_or(",");
                        let values: Result<Vec<f64>, _> = data_text
                            .split(sep)
                            .map(|s| s.trim().parse::<f64>())
                            .collect();
                        x_pts = Some(GraphicalFunctionPoints {
                            values: values.map_err(|err| {
                                DeserializeError::Custom(format!("Invalid point value: {}", err))
                            })?,
                            separator,
                        });
                    }
                    #[cfg(feature = "arrays")]
                    b"dimensions" => {
                        dimensions = Some(crate::xml::deserialize::deserialize_dimensions(
                            reader, buf,
                        )?);
                    }
                    #[cfg(feature = "arrays")]
                    b"element" => {
                        let element =
                            crate::xml::deserialize::deserialize_array_element(reader, buf)?;
                        elements.push(element);
                    }
                    _ => {
                        // Skip unknown elements using the helper
                        let element_name = e.name().as_ref().to_vec();
                        skip_element(reader, buf, &element_name)?;
                    }
                }
            }
            Event::Empty(e) => {
                match e.name().as_ref() {
                    b"xscale" => {
                        // Handle empty xscale with attributes
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let val = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(val.parse::<f64>().map_err(|parse_err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid min: {}",
                                            parse_err
                                        ))
                                    })?);
                                }
                                b"max" => {
                                    let val = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(val.parse::<f64>().map_err(|parse_err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid max: {}",
                                            parse_err
                                        ))
                                    })?);
                                }
                                _ => {}
                            }
                        }
                        x_scale = Some(GraphicalFunctionScale {
                            min: min.ok_or_else(|| {
                                DeserializeError::MissingField("xscale.min".to_string())
                            })?,
                            max: max.ok_or_else(|| {
                                DeserializeError::MissingField("xscale.max".to_string())
                            })?,
                        });
                    }
                    b"yscale" => {
                        // Handle empty yscale with attributes
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let val = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(val.parse::<f64>().map_err(|parse_err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid min: {}",
                                            parse_err
                                        ))
                                    })?);
                                }
                                b"max" => {
                                    let val = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(val.parse::<f64>().map_err(|parse_err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid max: {}",
                                            parse_err
                                        ))
                                    })?);
                                }
                                _ => {}
                            }
                        }
                        y_scale = Some(GraphicalFunctionScale {
                            min: min.ok_or_else(|| {
                                DeserializeError::MissingField("yscale.min".to_string())
                            })?,
                            max: max.ok_or_else(|| {
                                DeserializeError::MissingField("yscale.max".to_string())
                            })?,
                        });
                    }
                    b"range" => {
                        let mut min_val: Option<f64> = None;
                        let mut max_val: Option<f64> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min_val = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max_val = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid max: {}", err))
                                    })?);
                                }
                                _ => {}
                            }
                        }
                        if let (Some(min), Some(max)) = (min_val, max_val) {
                            range = Some(DeviceRange { min, max });
                        }
                    }
                    b"scale" => {
                        let mut min_val: Option<f64> = None;
                        let mut max_val: Option<f64> = None;
                        let mut auto: Option<bool> = None;
                        let mut group: Option<u32> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min_val = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max_val = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid max: {}", err))
                                    })?);
                                }
                                b"auto" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    auto = Some(s == "true");
                                }
                                b"group" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    group = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid group: {}", err))
                                    })?);
                                }
                                _ => {}
                            }
                        }
                        if let Some(auto_val) = auto {
                            scale = Some(DeviceScale::Auto(auto_val));
                        } else if let Some(group_val) = group {
                            scale = Some(DeviceScale::Group(group_val));
                        } else if let (Some(min), Some(max)) = (min_val, max_val) {
                            scale = Some(DeviceScale::MinMax { min, max });
                        }
                    }
                    b"format" => {
                        let mut precision: Option<f64> = None;
                        let mut scale_by: Option<f64> = None;
                        let mut display_as: Option<DisplayAs> = None;
                        let mut delimit_000s: Option<bool> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"precision" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    precision = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid precision: {}",
                                            err
                                        ))
                                    })?);
                                }
                                b"scale_by" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    scale_by = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid scale_by: {}",
                                            err
                                        ))
                                    })?);
                                }
                                b"display_as" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    display_as = Some(match s.as_str() {
                                        "number" => DisplayAs::Number,
                                        "currency" => DisplayAs::Currency,
                                        "percent" => DisplayAs::Percent,
                                        _ => {
                                            return Err(DeserializeError::Custom(format!(
                                                "Invalid display_as: {}",
                                                s
                                            )));
                                        }
                                    });
                                }
                                b"delimit_000s" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    delimit_000s = Some(s == "true");
                                }
                                _ => {}
                            }
                        }
                        format = Some(FormatOptions {
                            precision,
                            scale_by,
                            display_as,
                            delimit_000s,
                        });
                    }
                    _ => {
                        // Ignore other empty elements
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"gf" => {
                break;
            }
            Event::Eof => {
                return Err(DeserializeError::UnexpectedEof);
            }
            _ => {}
        }
        buf.clear();
    }

    // Construct GraphicalFunctionData
    let data = if let Some(x_scale_val) = x_scale {
        // UniformScale variant
        let y_values = y_pts.ok_or_else(|| DeserializeError::MissingField("ypts".to_string()))?;
        GraphicalFunctionData::UniformScale {
            x_scale: x_scale_val,
            y_scale,
            y_values,
        }
    } else if let Some(x_pts_val) = x_pts {
        // XYPairs variant
        let y_values = y_pts.ok_or_else(|| DeserializeError::MissingField("ypts".to_string()))?;
        if x_pts_val.values.len() != y_values.values.len() {
            return Err(DeserializeError::Custom(format!(
                "x-values and y-values must have the same length ({} vs {})",
                x_pts_val.values.len(),
                y_values.values.len()
            )));
        }
        GraphicalFunctionData::XYPairs {
            y_scale,
            x_values: x_pts_val,
            y_values,
        }
    } else {
        return Err(DeserializeError::Custom(
            "Either xscale or xpts must be provided for graphical function data".to_string(),
        ));
    };

    Ok(GraphicalFunction {
        name,
        r#type,
        data,
        equation,
        mathml_equation,
        units,
        documentation,
        range,
        scale,
        format,
        #[cfg(feature = "arrays")]
        dimensions: dimensions.map(|dims| dims.dims.into_iter().map(|d| d.name).collect()),
        #[cfg(feature = "arrays")]
        elements,
    })
}
