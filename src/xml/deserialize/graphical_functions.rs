//! Graphical function deserialization module.
//!
//! This module handles deserialization of graphical function definitions,
//! including scales, points, and function data.

use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::{
    Expression,
    equation::{Identifier, parse::unit_equation, units::UnitEquation},
    model::{
        object::{DeviceRange, DeviceScale, DisplayAs, Documentation, FormatOptions},
        vars::gf::{
            GraphicalFunction, GraphicalFunctionData, GraphicalFunctionPoints,
            GraphicalFunctionScale, GraphicalFunctionType,
        },
    },
    xml::{
        deserialize::{DeserializeError, helpers::read_text_content},
        quick::de::{Attrs, skip_element},
    },
};

#[cfg(feature = "arrays")]
use crate::model::vars::array::{ArrayElement, VariableDimensions};

/// Deserialize a GraphicalFunction from XML.
pub fn deserialize_graphical_function<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<GraphicalFunction, DeserializeError> {
    // Expect <gf> start tag
    let (name, r#type) = match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"gf" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let name = attrs
                .get_opt("name")
                .map(|s| {
                    Identifier::parse_from_attribute(s)
                        .map_err(|e| DeserializeError::Custom(format!("Invalid identifier: {}", e)))
                })
                .transpose()?;
            let r#type = attrs
                .get_opt("type")
                .map(|s| {
                    GraphicalFunctionType::from_str(s).map_err(|e| {
                        DeserializeError::Custom(format!("Invalid function type: {}", e))
                    })
                })
                .transpose()?;
            (name, r#type)
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
    };
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
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min_val = attrs.get_opt_f64("min")?;
                        let max_val = attrs.get_opt_f64("max")?;
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
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min_val = attrs.get_opt_f64("min")?;
                        let max_val = attrs.get_opt_f64("max")?;
                        let auto = attrs.get_opt_bool("auto")?;
                        let group = attrs.get_opt_u32("group")?;
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
                        let attrs = Attrs::from_start(&e, reader)?;
                        let precision = attrs.get_opt_f64("precision")?;
                        let scale_by = attrs.get_opt_f64("scale_by")?;
                        let display_as = attrs
                            .get_opt("display_as")
                            .map(|s| match s {
                                "number" => Ok(DisplayAs::Number),
                                "currency" => Ok(DisplayAs::Currency),
                                "percent" => Ok(DisplayAs::Percent),
                                _ => Err(DeserializeError::Custom(format!(
                                    "Invalid display_as: {}",
                                    s
                                ))),
                            })
                            .transpose()?;
                        let delimit_000s = attrs.get_opt_bool("delimit_000s")?;
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
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
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
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
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
                        let attrs = Attrs::from_start(&e, reader)?;
                        let separator = attrs.get_opt_string("sep");
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
                        let attrs = Attrs::from_start(&e, reader)?;
                        let separator = attrs.get_opt_string("sep");
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
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_req_f64("min")?;
                        let max = attrs.get_req_f64("max")?;
                        x_scale = Some(GraphicalFunctionScale { min, max });
                    }
                    b"yscale" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_req_f64("min")?;
                        let max = attrs.get_req_f64("max")?;
                        y_scale = Some(GraphicalFunctionScale { min, max });
                    }
                    b"range" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min_val = attrs.get_opt_f64("min")?;
                        let max_val = attrs.get_opt_f64("max")?;
                        if let (Some(min), Some(max)) = (min_val, max_val) {
                            range = Some(DeviceRange { min, max });
                        }
                    }
                    b"scale" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min_val = attrs.get_opt_f64("min")?;
                        let max_val = attrs.get_opt_f64("max")?;
                        let auto = attrs.get_opt_bool("auto")?;
                        let group = attrs.get_opt_u32("group")?;
                        if let Some(auto_val) = auto {
                            scale = Some(DeviceScale::Auto(auto_val));
                        } else if let Some(group_val) = group {
                            scale = Some(DeviceScale::Group(group_val));
                        } else if let (Some(min), Some(max)) = (min_val, max_val) {
                            scale = Some(DeviceScale::MinMax { min, max });
                        }
                    }
                    b"format" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let precision = attrs.get_opt_f64("precision")?;
                        let scale_by = attrs.get_opt_f64("scale_by")?;
                        let display_as = attrs
                            .get_opt("display_as")
                            .map(|s| match s {
                                "number" => Ok(DisplayAs::Number),
                                "currency" => Ok(DisplayAs::Currency),
                                "percent" => Ok(DisplayAs::Percent),
                                _ => Err(DeserializeError::Custom(format!(
                                    "Invalid display_as: {}",
                                    s
                                ))),
                            })
                            .transpose()?;
                        let delimit_000s = attrs.get_opt_bool("delimit_000s")?;
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
