//! Variables deserialization module.
//!
//! This module handles deserialization of all variable types:
//! stocks, flows, auxiliaries, modules, groups, and graphical functions.

use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::{
    Expression,
    equation::{Identifier, parse::unit_equation, units::UnitEquation},
    model::{
        events::{Event as ModelEvent, EventPoster, Threshold},
        groups::{Group, GroupEntity},
        object::{DeviceRange, DeviceScale, DisplayAs, Documentation, FormatOptions},
        vars::{
            Variable,
            aux::Auxiliary,
            flow::BasicFlow,
            gf::{GraphicalFunctionPoints, GraphicalFunctionScale, GraphicalFunctionType},
            stock::{BasicStock, ConveyorStock, QueueStock, Stock},
        },
    },
    xml::{
        deserialize::{
            DeserializeError,
            graphical_functions::deserialize_graphical_function_impl,
            helpers::{read_number_content, read_text_content},
        },
        quick::de::{Attrs, skip_element},
        schema::Variables,
    },
};

#[cfg(feature = "arrays")]
use crate::{
    model::vars::{
        array::{ArrayElement, VariableDimensions},
        gf::GraphicalFunction,
    },
    xml::deserialize::graphical_functions::deserialize_graphical_function,
};

#[cfg(feature = "submodels")]
use crate::model::vars::module::{Module, ModuleConnection};

/// Helper to parse common variable attributes (name, access, autoexport) from Attrs.
fn parse_var_attrs(
    attrs: &Attrs,
) -> Result<
    (
        Option<Identifier>,
        Option<crate::model::vars::AccessType>,
        Option<bool>,
    ),
    DeserializeError,
> {
    let name = attrs
        .get_opt("name")
        .map(|s| {
            Identifier::parse_from_attribute(s)
                .map_err(|e| DeserializeError::Custom(format!("Invalid identifier: {}", e)))
        })
        .transpose()?;
    let access = attrs
        .get_opt("access")
        .map(|s| match s {
            "input" => Ok(crate::model::vars::AccessType::Input),
            "output" => Ok(crate::model::vars::AccessType::Output),
            _ => Err(DeserializeError::Custom(format!(
                "Invalid access type: {}",
                s
            ))),
        })
        .transpose()?;
    let autoexport = attrs.get_opt_bool("autoexport")?;
    Ok((name, access, autoexport))
}

pub fn deserialize_variables<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Variables, DeserializeError> {
    // Expect <variables> start tag
    let event = reader.read_event_into(buf)?;
    match event {
        Event::Start(e) if e.name().as_ref() == b"variables" => {}
        Event::Start(e) => {
            return Err(DeserializeError::UnexpectedElement {
                expected: "variables".to_string(),
                found: String::from_utf8_lossy(e.name().as_ref()).to_string(),
            });
        }
        _ => {
            return Err(DeserializeError::Custom(
                "Expected variables start tag".to_string(),
            ));
        }
    }
    buf.clear();
    deserialize_variables_impl(reader, buf)
}

/// Internal implementation of variables deserialization.
pub(crate) fn deserialize_variables_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Variables, DeserializeError> {
    let mut variables = Vec::new();

    // Check if variables is empty by reading the first event
    let first_event = reader.read_event_into(buf)?;
    let is_empty = matches!(&first_event, Event::End(e) if e.name().as_ref() == b"variables");

    if is_empty {
        buf.clear();
        return Ok(Variables { variables });
    }

    // Process the first variable element we already read
    // Extract all data into owned types - we need to do this before first_event goes out of scope
    // but we also need to make sure first_event is dropped before we clear buf
    let (element_name, extracted_attrs, is_empty_element) = {
        let is_empty = matches!(&first_event, Event::Empty(_));
        let name = match &first_event {
            Event::Start(e) | Event::Empty(e) => e.name().as_ref().to_vec(),
            _ => Vec::new(),
        };
        let attrs = match &first_event {
            Event::Start(e) | Event::Empty(e) => Attrs::from_start(e, reader)?.to_vec(),
            _ => Vec::new(),
        };
        // first_event is still borrowed here, but we return owned data
        (name, attrs, is_empty)
    };
    // Now first_event goes out of scope, so we can clear buf
    buf.clear();

    match element_name.as_slice() {
        b"stock" => {
            // Extract attributes from the already-read start event
            let mut name: Option<Identifier> = None;
            let mut access: Option<crate::model::vars::AccessType> = None;
            let mut autoexport: Option<bool> = None;
            for (key, value) in &extracted_attrs {
                match key.as_slice() {
                    b"name" => {
                        name = Some(Identifier::parse_from_attribute(value).map_err(|e| {
                            DeserializeError::Custom(format!("Invalid identifier: {}", e))
                        })?);
                    }
                    b"access" => {
                        access = Some(match value.as_str() {
                            "input" => crate::model::vars::AccessType::Input,
                            "output" => crate::model::vars::AccessType::Output,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid access type: {}",
                                    value
                                )));
                            }
                        });
                    }
                    b"autoexport" => {
                        autoexport = Some(value.parse::<bool>().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid autoexport value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }
            let stock =
                deserialize_stock_impl(reader, buf, name, access, autoexport, is_empty_element)?;
            variables.push(crate::model::vars::Variable::Stock(stock));
        }
        b"flow" => {
            // Extract attributes from the already-read start event
            let mut name: Option<Identifier> = None;
            let mut access: Option<crate::model::vars::AccessType> = None;
            let mut autoexport: Option<bool> = None;
            for (key, value) in &extracted_attrs {
                match key.as_slice() {
                    b"name" => {
                        name = Some(Identifier::parse_from_attribute(value).map_err(|e| {
                            DeserializeError::Custom(format!("Invalid identifier: {}", e))
                        })?);
                    }
                    b"access" => {
                        access = Some(match value.as_str() {
                            "input" => crate::model::vars::AccessType::Input,
                            "output" => crate::model::vars::AccessType::Output,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid access type: {}",
                                    value
                                )));
                            }
                        });
                    }
                    b"autoexport" => {
                        autoexport = Some(value.parse::<bool>().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid autoexport value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }
            let flow = deserialize_basic_flow_impl(
                reader,
                buf,
                name,
                access,
                autoexport,
                is_empty_element,
            )?;
            variables.push(crate::model::vars::Variable::Flow(flow));
        }
        b"aux" => {
            // Extract attributes from the already-read start event
            let mut name: Option<Identifier> = None;
            let mut access: Option<crate::model::vars::AccessType> = None;
            let mut autoexport: Option<bool> = None;
            for (key, value) in &extracted_attrs {
                match key.as_slice() {
                    b"name" => {
                        name = Some(Identifier::parse_from_attribute(value).map_err(|e| {
                            DeserializeError::Custom(format!("Invalid identifier: {}", e))
                        })?);
                    }
                    b"access" => {
                        access = Some(match value.as_str() {
                            "input" => crate::model::vars::AccessType::Input,
                            "output" => crate::model::vars::AccessType::Output,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid access type: {}",
                                    value
                                )));
                            }
                        });
                    }
                    b"autoexport" => {
                        autoexport = Some(value.parse::<bool>().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid autoexport value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }
            let aux = deserialize_auxiliary_impl(
                reader,
                buf,
                name,
                access,
                autoexport,
                is_empty_element,
            )?;
            variables.push(crate::model::vars::Variable::Auxiliary(aux));
        }
        b"gf" => {
            // Extract attributes from the already-read start event
            let mut name: Option<Identifier> = None;
            let mut gf_type: Option<GraphicalFunctionType> = None;
            for (key, value) in &extracted_attrs {
                match key.as_slice() {
                    b"name" => {
                        name = Some(Identifier::parse_from_attribute(value).map_err(|e| {
                            DeserializeError::Custom(format!("Invalid identifier: {}", e))
                        })?);
                    }
                    b"type" => {
                        gf_type = Some(GraphicalFunctionType::from_str(value).map_err(|e| {
                            DeserializeError::Custom(format!("Invalid function type: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }
            let gf = deserialize_graphical_function_impl(reader, buf, name, gf_type)?;
            variables.push(Variable::GraphicalFunction(gf));
        }
        #[cfg(feature = "submodels")]
        b"module" => {
            // Extract name and resource attributes from already-read start event
            let mut name: Option<Identifier> = None;
            let mut resource: Option<String> = None;
            for (key, value) in &extracted_attrs {
                match key.as_slice() {
                    b"name" => {
                        name = Some(Identifier::parse_from_attribute(value).map_err(|e| {
                            DeserializeError::Custom(format!("Invalid module name: {}", e))
                        })?);
                    }
                    b"resource" => {
                        resource = Some(value.clone());
                    }
                    _ => {}
                }
            }
            buf.clear();
            let module = deserialize_module_impl(reader, buf, name, resource, is_empty_element)?;
            variables.push(crate::model::vars::Variable::Module(module));
        }
        b"group" => {
            // Extract name attribute from already-read start event
            let mut name: Option<Identifier> = None;
            for (key, value) in &extracted_attrs {
                if key.as_slice() == b"name" {
                    name = Some(Identifier::parse_from_attribute(value).map_err(|e| {
                        DeserializeError::Custom(format!("Invalid group name: {}", e))
                    })?);
                }
            }
            let group = deserialize_group_impl(reader, buf, name, is_empty_element)?;
            variables.push(crate::model::vars::Variable::Group(group));
        }
        _ => {
            // Skip unknown elements using the helper
            if !is_empty_element {
                skip_element(reader, buf, &element_name)?;
            }
        }
    }
    buf.clear();

    // Continue processing remaining events
    loop {
        let event = reader.read_event_into(buf)?;
        let is_empty_element = matches!(event, Event::Empty(_));
        match event {
            Event::Start(e) | Event::Empty(e) => {
                match e.name().as_ref() {
                    b"stock" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let (name, access, autoexport) = parse_var_attrs(&attrs)?;
                        buf.clear();
                        let stock = deserialize_basic_stock_impl(
                            reader,
                            buf,
                            name,
                            access,
                            autoexport,
                            is_empty_element,
                        )?;
                        variables.push(crate::model::vars::Variable::Stock(
                            crate::model::vars::stock::Stock::Basic(stock),
                        ));
                    }
                    b"flow" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let (name, access, autoexport) = parse_var_attrs(&attrs)?;
                        buf.clear();
                        let flow = deserialize_basic_flow_impl(
                            reader,
                            buf,
                            name,
                            access,
                            autoexport,
                            is_empty_element,
                        )?;
                        variables.push(crate::model::vars::Variable::Flow(flow));
                    }
                    b"aux" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let (name, access, autoexport) = parse_var_attrs(&attrs)?;
                        buf.clear();
                        let aux = deserialize_auxiliary_impl(
                            reader,
                            buf,
                            name,
                            access,
                            autoexport,
                            is_empty_element,
                        )?;
                        variables.push(crate::model::vars::Variable::Auxiliary(aux));
                    }
                    b"gf" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let name = attrs
                            .get_opt("name")
                            .map(|s| {
                                Identifier::parse_from_attribute(s).map_err(|e| {
                                    DeserializeError::Custom(format!("Invalid identifier: {}", e))
                                })
                            })
                            .transpose()?;
                        let gf_type = attrs
                            .get_opt("type")
                            .map(|s| {
                                GraphicalFunctionType::from_str(s).map_err(|e| {
                                    DeserializeError::Custom(format!(
                                        "Invalid function type: {}",
                                        e
                                    ))
                                })
                            })
                            .transpose()?;
                        buf.clear();
                        let gf = deserialize_graphical_function_impl(reader, buf, name, gf_type)?;
                        variables.push(Variable::GraphicalFunction(gf));
                    }
                    #[cfg(feature = "submodels")]
                    b"module" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let name = attrs
                            .get_opt("name")
                            .map(|s| {
                                Identifier::parse_from_attribute(s).map_err(|err| {
                                    DeserializeError::Custom(format!(
                                        "Invalid module name: {}",
                                        err
                                    ))
                                })
                            })
                            .transpose()?;
                        let resource = attrs.get_opt_string("resource");
                        buf.clear();
                        let module = deserialize_module_impl(reader, buf, name, resource, false)?;
                        variables.push(crate::model::vars::Variable::Module(module));
                    }
                    b"group" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let name = attrs
                            .get_opt("name")
                            .map(|s| {
                                Identifier::parse_from_attribute(s).map_err(|err| {
                                    DeserializeError::Custom(format!("Invalid group name: {}", err))
                                })
                            })
                            .transpose()?;
                        buf.clear();
                        let group = deserialize_group_impl(reader, buf, name, is_empty_element)?;
                        variables.push(crate::model::vars::Variable::Group(group));
                    }
                    _ => {
                        // Skip unknown elements using the helper
                        let element_name = e.name().as_ref().to_vec();
                        buf.clear();
                        if !is_empty_element {
                            skip_element(reader, buf, &element_name)?;
                        }
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"variables" => {
                break;
            }
            Event::Eof => {
                return Err(DeserializeError::UnexpectedEof);
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(Variables { variables })
}

/// Helper to read an expression from an element.
pub fn read_expression<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Expression, DeserializeError> {
    let text = read_text_content(reader, buf)?;
    // Parse the expression string
    use crate::equation::parse::expression;
    let (remaining, expr) = expression(&text)
        .map_err(|e| DeserializeError::Custom(format!("Failed to parse expression: {}", e)))?;
    if !remaining.is_empty() {
        return Err(DeserializeError::Custom(format!(
            "Unexpected trailing characters after expression: '{}'",
            remaining
        )));
    }
    Ok(expr)
}

/// Helper to read a non_negative element.
///
/// Returns Option<Option<bool>>:
/// - None = element not present
/// - Some(None) = empty tag <non_negative/> (defaults to true)
/// - Some(Some(false)) = <non_negative>false</non_negative>
/// - Some(Some(true)) = <non_negative>true</non_negative>
pub fn read_non_negative<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Option<Option<bool>>, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Empty(_) => {
            // Empty tag means true (default)
            Ok(Some(None))
        }
        Event::Start(e) if e.name().as_ref() == b"non_negative" => {
            let text = read_text_content(reader, buf)?;
            let value = match text.trim() {
                "true" => Some(Some(true)),
                "false" => Some(Some(false)),
                "" => Some(None), // Empty content means true
                _ => {
                    return Err(DeserializeError::Custom(format!(
                        "Invalid non_negative value: {}",
                        text
                    )));
                }
            };
            Ok(value)
        }
        _ => Ok(None),
    }
}

/// Deserialize an ArrayElement from XML.
///
/// This function expects the reader to be positioned at the start of an <element> tag.
#[cfg(feature = "arrays")]
pub fn deserialize_array_element<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<ArrayElement, DeserializeError> {
    // Expect <element> start tag
    let (subscript, is_empty) = match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"element" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let subscript = attrs.get_opt_string("subscript");
            (subscript, false)
        }
        Event::Empty(e) if e.name().as_ref() == b"element" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let subscript = attrs.get_opt_string("subscript");
            (subscript, true)
        }
        Event::Start(e) => {
            return Err(DeserializeError::UnexpectedElement {
                expected: "element".to_string(),
                found: String::from_utf8_lossy(e.name().as_ref()).to_string(),
            });
        }
        _ => {
            return Err(DeserializeError::Custom(
                "Expected element start tag".to_string(),
            ));
        }
    };
    buf.clear();

    if is_empty {
        return Ok(ArrayElement {
            subscript: subscript
                .ok_or_else(|| DeserializeError::MissingField("subscript".to_string()))?,
            eqn: None,
            gf: None,
        });
    }

    let mut eqn: Option<Expression> = None;
    let mut gf: Option<GraphicalFunction> = None;

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"eqn" => {
                        eqn = Some(read_expression(reader, buf)?);
                    }
                    b"gf" => {
                        gf = Some(deserialize_graphical_function(reader, buf)?);
                    }
                    _ => {
                        // Skip unknown elements using the helper
                        let element_name = e.name().as_ref().to_vec();
                        skip_element(reader, buf, &element_name)?;
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"element" => {
                break;
            }
            Event::Eof => {
                return Err(DeserializeError::UnexpectedEof);
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(ArrayElement {
        subscript: subscript
            .ok_or_else(|| DeserializeError::MissingField("subscript".to_string()))?,
        eqn,
        gf,
    })
}

/// Deserialize a DeviceRange from XML.
pub fn deserialize_range<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<DeviceRange, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Empty(e) | Event::Start(e) if e.name().as_ref() == b"range" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let min = attrs.get_req_f64("min")?;
            let max = attrs.get_req_f64("max")?;

            // If it's a start tag, read until end
            if matches!(reader.read_event_into(buf)?, Event::Start(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"range" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(DeviceRange { min, max })
        }
        _ => Err(DeserializeError::Custom(
            "Expected range element".to_string(),
        )),
    }
}

/// Deserialize a DeviceRange from an already-read start tag.
pub fn deserialize_range_from_start<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start_event: &quick_xml::events::BytesStart,
) -> Result<DeviceRange, DeserializeError> {
    let attrs = Attrs::from_start(start_event, reader)?;
    let min = attrs.get_req_f64("min")?;
    let max = attrs.get_req_f64("max")?;

    // Read until end tag
    loop {
        match reader.read_event_into(buf)? {
            Event::End(e) if e.name().as_ref() == b"range" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(DeviceRange { min, max })
}

/// Deserialize a DeviceScale from XML.
pub fn deserialize_scale<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<DeviceScale, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Empty(e) | Event::Start(e) if e.name().as_ref() == b"scale" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let min = attrs.get_opt_f64("min")?;
            let max = attrs.get_opt_f64("max")?;
            let auto = attrs.get_opt_bool("auto")?;
            let group = attrs.get_opt_u32("group")?;

            // If it's a start tag, read until end
            if matches!(reader.read_event_into(buf)?, Event::Start(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"scale" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            // Determine scale type
            if let Some(auto_val) = auto {
                Ok(DeviceScale::Auto(auto_val))
            } else if let Some(group_val) = group {
                Ok(DeviceScale::Group(group_val))
            } else if let (Some(min_val), Some(max_val)) = (min, max) {
                Ok(DeviceScale::MinMax {
                    min: min_val,
                    max: max_val,
                })
            } else {
                Err(DeserializeError::Custom(
                    "DeviceScale must specify min/max, auto, or group".to_string(),
                ))
            }
        }
        _ => Err(DeserializeError::Custom(
            "Expected scale element".to_string(),
        )),
    }
}

/// Deserialize a DeviceScale from an already-read start tag.
pub fn deserialize_scale_from_start<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start_event: &quick_xml::events::BytesStart,
) -> Result<DeviceScale, DeserializeError> {
    let attrs = Attrs::from_start(start_event, reader)?;
    let min = attrs.get_opt_f64("min")?;
    let max = attrs.get_opt_f64("max")?;
    let auto = attrs.get_opt_bool("auto")?;
    let group = attrs.get_opt_u32("group")?;

    // Read until end tag
    loop {
        match reader.read_event_into(buf)? {
            Event::End(e) if e.name().as_ref() == b"scale" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    // Determine scale type
    if let Some(auto_val) = auto {
        Ok(DeviceScale::Auto(auto_val))
    } else if let Some(group_val) = group {
        Ok(DeviceScale::Group(group_val))
    } else if let (Some(min_val), Some(max_val)) = (min, max) {
        Ok(DeviceScale::MinMax {
            min: min_val,
            max: max_val,
        })
    } else {
        Err(DeserializeError::Custom(
            "DeviceScale must specify min/max, auto, or group".to_string(),
        ))
    }
}

/// Deserialize FormatOptions from XML.
pub fn deserialize_format<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<FormatOptions, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Empty(e) | Event::Start(e) if e.name().as_ref() == b"format" => {
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
                        "Invalid display_as value: {}",
                        s
                    ))),
                })
                .transpose()?;
            let delimit_000s = attrs.get_opt_bool("delimit_000s")?;

            // If it's a start tag, read until end
            if matches!(reader.read_event_into(buf)?, Event::Start(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"format" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(FormatOptions {
                precision,
                scale_by,
                display_as,
                delimit_000s,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected format element".to_string(),
        )),
    }
}

/// Deserialize FormatOptions from an already-read start tag.
pub fn deserialize_format_from_start<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start_event: &quick_xml::events::BytesStart,
) -> Result<FormatOptions, DeserializeError> {
    let attrs = Attrs::from_start(start_event, reader)?;
    let precision = attrs.get_opt_f64("precision")?;
    let scale_by = attrs.get_opt_f64("scale_by")?;
    let display_as = attrs
        .get_opt("display_as")
        .map(|s| match s {
            "number" => Ok(DisplayAs::Number),
            "currency" => Ok(DisplayAs::Currency),
            "percent" => Ok(DisplayAs::Percent),
            _ => Err(DeserializeError::Custom(format!(
                "Invalid display_as value: {}",
                s
            ))),
        })
        .transpose()?;
    let delimit_000s = attrs.get_opt_bool("delimit_000s")?;

    // Read until end tag
    loop {
        match reader.read_event_into(buf)? {
            Event::End(e) if e.name().as_ref() == b"format" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(FormatOptions {
        precision,
        scale_by,
        display_as,
        delimit_000s,
    })
}

/// Deserialize VariableDimensions from XML.
#[cfg(feature = "arrays")]
pub fn deserialize_dimensions<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<VariableDimensions, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"dimensions" => {
            let mut dims = Vec::new();

            loop {
                match reader.read_event_into(buf)? {
                    Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"dim" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let name = attrs.get_opt_string("name");

                        if let Some(dim_name) = name {
                            dims.push(crate::model::vars::array::Dimension { name: dim_name });
                        }

                        // If it's a start tag (not empty), read until end
                        if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                            loop {
                                match reader.read_event_into(buf)? {
                                    Event::End(e) if e.name().as_ref() == b"dim" => break,
                                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                    _ => {}
                                }
                                buf.clear();
                            }
                        }
                    }
                    Event::End(e) if e.name().as_ref() == b"dimensions" => break,
                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                    _ => {}
                }
                buf.clear();
            }

            Ok(VariableDimensions { dims })
        }
        _ => Err(DeserializeError::Custom(
            "Expected dimensions element".to_string(),
        )),
    }
}

/// Deserialize EventPoster from XML.
pub fn deserialize_event_poster<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<EventPoster, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"event_poster" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let min = attrs.get_opt_f64("min")?;
            let max = attrs.get_opt_f64("max")?;

            let mut thresholds = Vec::new();

            loop {
                let event = reader.read_event_into(buf)?;
                match &event {
                    Event::Start(e) if e.name().as_ref() == b"threshold" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let value = attrs.get_opt_f64("value")?;
                        let direction = attrs.get_opt_string("direction");
                        let repeat = attrs.get_opt_string("repeat");
                        let interval = attrs.get_opt_f64("interval")?;
                        buf.clear();

                        // Now read events
                        let mut events = Vec::new();
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::Start(e) if e.name().as_ref() == b"event" => {
                                    let attrs = Attrs::from_start(&e, reader)?;
                                    let sim_action = attrs.get_opt_string("sim_action");
                                    let actions_text = read_text_content(reader, buf)?;
                                    let actions = if actions_text.trim().is_empty() {
                                        Vec::new()
                                    } else {
                                        vec![actions_text]
                                    };
                                    events.push(ModelEvent {
                                        sim_action,
                                        actions,
                                    });
                                }
                                Event::End(e) if e.name().as_ref() == b"threshold" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }

                        thresholds.push(Threshold {
                            value: value.ok_or_else(|| {
                                DeserializeError::MissingField("value".to_string())
                            })?,
                            direction,
                            repeat,
                            interval,
                            events,
                        });
                    }
                    Event::End(e) if e.name().as_ref() == b"event_poster" => break,
                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                    _ => {}
                }
                buf.clear();
            }

            Ok(EventPoster {
                min: min.ok_or_else(|| DeserializeError::MissingField("min".to_string()))?,
                max: max.ok_or_else(|| DeserializeError::MissingField("max".to_string()))?,
                thresholds,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected event_poster element".to_string(),
        )),
    }
}

/// Deserialize GraphicalFunctionScale from XML.
#[allow(dead_code)]
fn deserialize_gf_scale<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<GraphicalFunctionScale, DeserializeError> {
    let event = reader.read_event_into(buf)?;
    match &event {
        Event::Empty(e) | Event::Start(e) => {
            let attrs = Attrs::from_start(e, reader)?;
            let min = attrs.get_req_f64("min")?;
            let max = attrs.get_req_f64("max")?;

            // If it's a start tag, read until end
            if matches!(event, Event::Start(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e)
                            if e.name().as_ref() == b"xscale" || e.name().as_ref() == b"yscale" =>
                        {
                            break;
                        }
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(GraphicalFunctionScale { min, max })
        }
        _ => Err(DeserializeError::Custom(
            "Expected scale element".to_string(),
        )),
    }
}

/// Deserialize GraphicalFunctionPoints from XML.
#[allow(dead_code)]
fn deserialize_gf_points<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<GraphicalFunctionPoints, DeserializeError> {
    let event = reader.read_event_into(buf)?;
    match &event {
        Event::Start(e) | Event::Empty(e) => {
            let attrs = Attrs::from_start(e, reader)?;
            let separator = attrs.get_opt_string("sep");

            // Read text content
            let data_text = read_text_content(reader, buf)?;
            let sep = separator.as_deref().unwrap_or(",");
            let values: Result<Vec<f64>, _> = data_text
                .split(sep)
                .map(|s| s.trim().parse::<f64>())
                .collect();

            Ok(GraphicalFunctionPoints {
                values: values
                    .map_err(|e| DeserializeError::Custom(format!("Invalid point value: {}", e)))?,
                separator,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected points element".to_string(),
        )),
    }
}
#[cfg(feature = "submodels")]
pub fn deserialize_module<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Module, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"module" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let name = attrs
                .get_opt("name")
                .map(|s| {
                    Identifier::parse_from_attribute(s).map_err(|err| {
                        DeserializeError::Custom(format!("Invalid module name: {}", err))
                    })
                })
                .transpose()?;
            let resource = attrs.get_opt_string("resource");

            let mut connections = Vec::new();
            let mut documentation: Option<Documentation> = None;

            loop {
                match reader.read_event_into(buf)? {
                    Event::Start(e) if e.name().as_ref() == b"connect" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let to = attrs.get_opt_string("to");
                        let from = attrs.get_opt_string("from");

                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(e) if e.name().as_ref() == b"connect" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        buf.clear();

                        if let (Some(to_val), Some(from_val)) = (to, from) {
                            connections.push(ModuleConnection {
                                to: to_val,
                                from: from_val,
                            });
                        }
                    }
                    Event::Empty(e) if e.name().as_ref() == b"connect" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let to = attrs.get_opt_string("to");
                        let from = attrs.get_opt_string("from");

                        if let (Some(to_val), Some(from_val)) = (to, from) {
                            connections.push(ModuleConnection {
                                to: to_val,
                                from: from_val,
                            });
                        }

                        buf.clear();
                    }
                    Event::Start(e) if e.name().as_ref() == b"doc" => {
                        let doc_text = read_text_content(reader, buf)?;
                        // Determine if it's HTML or plain text
                        documentation = Some(
                            if doc_text.trim().contains('<') && doc_text.trim().contains('>') {
                                Documentation::Html(doc_text)
                            } else {
                                Documentation::PlainText(doc_text)
                            },
                        );
                    }
                    Event::End(e) if e.name().as_ref() == b"module" => break,
                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                    _ => {}
                }
                buf.clear();
            }

            buf.clear();

            Ok(Module {
                name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
                resource,
                connections,
                documentation,
            })
        }
        Event::Empty(e) if e.name().as_ref() == b"module" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let name = attrs
                .get_opt("name")
                .map(|s| {
                    Identifier::parse_from_attribute(s).map_err(|err| {
                        DeserializeError::Custom(format!("Invalid module name: {}", err))
                    })
                })
                .transpose()?;
            let resource = attrs.get_opt_string("resource");

            buf.clear();

            Ok(Module {
                name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
                resource,
                connections: Vec::new(),
                documentation: None,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected module element".to_string(),
        )),
    }
}

/// Deserialize a Module from XML when the start tag has already been consumed.
///
/// This is called when deserialize_variables_impl has already matched the <module> start tag.
#[cfg(feature = "submodels")]
pub(crate) fn deserialize_module_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<Identifier>,
    resource: Option<String>,
    is_empty_tag: bool,
) -> Result<Module, DeserializeError> {
    let mut connections = Vec::new();
    let mut documentation: Option<Documentation> = None;

    if !is_empty_tag {
        loop {
            match reader.read_event_into(buf)? {
                Event::Start(e) if e.name().as_ref() == b"connect" => {
                    let attrs = Attrs::from_start(&e, reader)?;
                    let to = attrs.get_opt_string("to");
                    let from = attrs.get_opt_string("from");

                    loop {
                        match reader.read_event_into(buf)? {
                            Event::End(e) if e.name().as_ref() == b"connect" => break,
                            Event::Eof => return Err(DeserializeError::UnexpectedEof),
                            _ => {}
                        }
                        buf.clear();
                    }
                    buf.clear();

                    if let (Some(to_val), Some(from_val)) = (to, from) {
                        connections.push(ModuleConnection {
                            to: to_val,
                            from: from_val,
                        });
                    }
                }
                Event::Empty(e) if e.name().as_ref() == b"connect" => {
                    let attrs = Attrs::from_start(&e, reader)?;
                    let to = attrs.get_opt_string("to");
                    let from = attrs.get_opt_string("from");

                    if let (Some(to_val), Some(from_val)) = (to, from) {
                        connections.push(ModuleConnection {
                            to: to_val,
                            from: from_val,
                        });
                    }

                    buf.clear();
                }
                Event::Start(e) if e.name().as_ref() == b"doc" => {
                    let doc_text = read_text_content(reader, buf)?;
                    // Determine if it's HTML or plain text
                    documentation = Some(
                        if doc_text.trim().contains('<') && doc_text.trim().contains('>') {
                            Documentation::Html(doc_text)
                        } else {
                            Documentation::PlainText(doc_text)
                        },
                    );
                }
                Event::End(e) if e.name().as_ref() == b"module" => break,
                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                _ => {}
            }
            buf.clear();
        }
    }

    buf.clear();

    Ok(Module {
        name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
        resource,
        connections,
        documentation,
    })
}

/// Deserialize a Group from XML.
pub fn deserialize_group<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Group, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"group" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let name = attrs
                .get_opt("name")
                .map(|s| {
                    Identifier::parse_from_attribute(s)
                        .map_err(|e| DeserializeError::Custom(format!("Invalid group name: {}", e)))
                })
                .transpose()?;

            let mut doc: Option<Documentation> = None;
            let mut entities = Vec::new();

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"doc" => {
                            let doc_text = read_text_content(reader, buf)?;
                            // Determine if it's HTML or plain text
                            doc = Some(
                                if doc_text.trim().contains('<') && doc_text.trim().contains('>') {
                                    Documentation::Html(doc_text)
                                } else {
                                    Documentation::PlainText(doc_text)
                                },
                            );
                        }
                        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"entity" => {
                            let attrs = Attrs::from_start(&e, reader)?;
                            let entity_name = attrs
                                .get_opt("name")
                                .map(|s| {
                                    Identifier::parse_from_attribute(s).map_err(|e| {
                                        DeserializeError::Custom(format!(
                                            "Invalid entity name: {}",
                                            e
                                        ))
                                    })
                                })
                                .transpose()?;
                            let run = attrs.get_opt_bool("run")?;

                            // If it's a start tag, read until end
                            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                                loop {
                                    match reader.read_event_into(buf)? {
                                        Event::End(e) if e.name().as_ref() == b"entity" => break,
                                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                        _ => {}
                                    }
                                    buf.clear();
                                }
                            }
                            buf.clear();

                            if let Some(name) = entity_name {
                                entities.push(GroupEntity {
                                    name,
                                    run: run.unwrap_or(false),
                                });
                            }
                        }
                        Event::End(e) if e.name().as_ref() == b"group" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(Group {
                name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
                doc,
                entities,
                display: Vec::new(), // Display UIDs are handled separately in views
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected group element".to_string(),
        )),
    }
}

/// Internal implementation of Group deserialization when start tag is already read.
/// This function expects the reader to be positioned at the start of an <aux> element.
pub fn deserialize_auxiliary<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Auxiliary, DeserializeError> {
    // Expect <aux> start tag
    let (name, access, autoexport) = match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"aux" => {
            let attrs = Attrs::from_start(&e, reader)?;
            parse_var_attrs(&attrs)?
        }
        Event::Start(e) => {
            return Err(DeserializeError::UnexpectedElement {
                expected: "aux".to_string(),
                found: String::from_utf8_lossy(e.name().as_ref()).to_string(),
            });
        }
        _ => {
            return Err(DeserializeError::Custom(
                "Expected aux start tag".to_string(),
            ));
        }
    };
    buf.clear();

    let mut equation: Option<Expression> = None;
    #[cfg(feature = "mathml")]
    let mut mathml_equation: Option<String> = None;
    let mut documentation: Option<Documentation> = None;
    let mut units: Option<UnitEquation> = None;
    let mut range: Option<DeviceRange> = None;
    let mut scale: Option<DeviceScale> = None;
    let mut format: Option<FormatOptions> = None;
    let mut _event_poster: Option<EventPoster> = None;
    #[cfg(feature = "arrays")]
    let mut _dimensions: Option<VariableDimensions> = None;
    #[cfg(feature = "arrays")]
    let mut elements: Vec<ArrayElement> = Vec::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"eqn" => {
                        equation = Some(read_expression(reader, buf)?);
                    }
                    #[cfg(feature = "mathml")]
                    b"mathml" => {
                        mathml_equation = Some(read_text_content(reader, buf)?);
                    }
                    b"doc" => {
                        let doc_text = read_text_content(reader, buf)?;
                        // Determine if it's HTML or plain text
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
                        range = Some(deserialize_range(reader, buf)?);
                    }
                    b"scale" => {
                        scale = Some(deserialize_scale(reader, buf)?);
                    }
                    b"format" => {
                        format = Some(deserialize_format(reader, buf)?);
                    }
                    b"event_poster" => {
                        _event_poster = Some(deserialize_event_poster(reader, buf)?);
                    }
                    #[cfg(feature = "arrays")]
                    b"dimensions" => {
                        _dimensions = Some(deserialize_dimensions(reader, buf)?);
                    }
                    #[cfg(feature = "arrays")]
                    b"element" => {
                        let element = deserialize_array_element(reader, buf)?;
                        elements.push(element);
                    }
                    _ => {
                        // Skip unknown elements using the helper
                        let element_name = e.name().as_ref().to_vec();
                        skip_element(reader, buf, &element_name)?;
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"aux" => {
                break;
            }
            Event::Eof => {
                return Err(DeserializeError::UnexpectedEof);
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(Auxiliary {
        name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
        access,
        autoexport,
        documentation,
        equation: equation.ok_or_else(|| DeserializeError::MissingField("eqn".to_string()))?,
        #[cfg(feature = "mathml")]
        mathml_equation,
        units,
        range,
        scale,
        format,
        #[cfg(feature = "arrays")]
        dimensions: _dimensions,
        #[cfg(feature = "arrays")]
        elements,
        event_poster: _event_poster,
    })
}

/// Internal implementation of auxiliary deserialization.
pub(crate) fn deserialize_auxiliary_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<Identifier>,
    access: Option<crate::model::vars::AccessType>,
    autoexport: Option<bool>,
    _is_empty_tag: bool,
) -> Result<Auxiliary, DeserializeError> {
    let mut equation: Option<Expression> = None;
    #[cfg(feature = "mathml")]
    let mut mathml_equation: Option<String> = None;
    let mut documentation: Option<Documentation> = None;
    let mut units: Option<UnitEquation> = None;
    let mut range: Option<DeviceRange> = None;
    let mut scale: Option<DeviceScale> = None;
    let mut format: Option<FormatOptions> = None;
    let mut _event_poster: Option<EventPoster> = None;
    #[cfg(feature = "arrays")]
    let mut _dimensions: Option<VariableDimensions> = None;
    #[cfg(feature = "arrays")]
    let mut elements: Vec<ArrayElement> = Vec::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"eqn" => {
                        equation = Some(read_expression(reader, buf)?);
                    }
                    #[cfg(feature = "mathml")]
                    b"mathml" => {
                        mathml_equation = Some(read_text_content(reader, buf)?);
                    }
                    b"doc" => {
                        let doc_text = read_text_content(reader, buf)?;
                        // Determine if it's HTML or plain text
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
                        range = Some(deserialize_range(reader, buf)?);
                    }
                    b"scale" => {
                        scale = Some(deserialize_scale(reader, buf)?);
                    }
                    b"format" => {
                        format = Some(deserialize_format(reader, buf)?);
                    }
                    b"event_poster" => {
                        _event_poster = Some(deserialize_event_poster(reader, buf)?);
                    }
                    #[cfg(feature = "arrays")]
                    b"dimensions" => {
                        _dimensions = Some(deserialize_dimensions(reader, buf)?);
                    }
                    #[cfg(feature = "arrays")]
                    b"element" => {
                        let element = deserialize_array_element(reader, buf)?;
                        elements.push(element);
                    }
                    _ => {
                        // Skip unknown elements using the helper
                        let element_name = e.name().as_ref().to_vec();
                        skip_element(reader, buf, &element_name)?;
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"aux" => {
                break;
            }
            Event::Eof => {
                return Err(DeserializeError::UnexpectedEof);
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(Auxiliary {
        name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
        access,
        autoexport,
        documentation,
        equation: equation.ok_or_else(|| DeserializeError::MissingField("eqn".to_string()))?,
        #[cfg(feature = "mathml")]
        mathml_equation,
        units,
        range,
        scale,
        format,
        #[cfg(feature = "arrays")]
        dimensions: _dimensions,
        #[cfg(feature = "arrays")]
        elements,
        event_poster: _event_poster,
    })
}

/// Internal implementation of basic flow deserialization.
pub(crate) fn deserialize_basic_flow_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<Identifier>,
    access: Option<crate::model::vars::AccessType>,
    autoexport: Option<bool>,
    _is_empty_tag: bool,
) -> Result<BasicFlow, DeserializeError> {
    let mut equation: Option<Expression> = None;
    let mut mathml_equation: Option<String> = None;
    let mut documentation: Option<Documentation> = None;
    let mut units: Option<UnitEquation> = None;
    let mut non_negative: Option<Option<bool>> = None;
    let mut multiplier: Option<f64> = None;
    let mut range: Option<DeviceRange> = None;
    let mut scale: Option<DeviceScale> = None;
    let mut format: Option<FormatOptions> = None;
    let mut _event_poster: Option<EventPoster> = None;
    #[cfg(feature = "arrays")]
    let mut _dimensions: Option<VariableDimensions> = None;
    #[cfg(feature = "arrays")]
    let mut elements: Vec<ArrayElement> = Vec::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"eqn" => {
                        equation = Some(read_expression(reader, buf)?);
                    }
                    b"mathml" => {
                        mathml_equation = Some(read_text_content(reader, buf)?);
                    }
                    b"multiplier" => {
                        multiplier = Some(read_number_content(reader, buf)?);
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
                    b"non_negative" => {
                        // Read the text content of the non_negative element
                        let text = read_text_content(reader, buf)?;
                        non_negative = Some(match text.trim() {
                            "true" => Some(true),
                            "false" => Some(false),
                            "" => None, // Empty content means default (true)
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid non_negative value: {}",
                                    text
                                )));
                            }
                        });
                    }
                    b"range" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
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
                        if let (Some(min_val), Some(max_val)) = (min, max) {
                            range = Some(DeviceRange {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"scale" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
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
                        } else if let (Some(min_val), Some(max_val)) = (min, max) {
                            scale = Some(DeviceScale::MinMax {
                                min: min_val,
                                max: max_val,
                            });
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
                    #[cfg(feature = "arrays")]
                    b"element" => {
                        buf.clear();
                        let element = deserialize_array_element(reader, buf)?;
                        elements.push(element);
                    }
                    _ => {
                        // Skip unknown elements
                        let element_name = e.name().as_ref().to_vec();
                        buf.clear();
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end)
                                    if end.name().as_ref() == element_name.as_slice() =>
                                {
                                    break;
                                }
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                    }
                }
            }
            Event::Empty(e) => {
                // Handle empty tags like <non_negative/>
                match e.name().as_ref() {
                    b"non_negative" => {
                        // Empty tag <non_negative/> means default (true)
                        non_negative = Some(None);
                    }
                    b"range" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
                        if let (Some(min_val), Some(max_val)) = (min, max) {
                            range = Some(DeviceRange {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"scale" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
                        let auto = attrs.get_opt_bool("auto")?;
                        let group = attrs.get_opt_u32("group")?;
                        if let Some(auto_val) = auto {
                            scale = Some(DeviceScale::Auto(auto_val));
                        } else if let Some(group_val) = group {
                            scale = Some(DeviceScale::Group(group_val));
                        } else if let (Some(min_val), Some(max_val)) = (min, max) {
                            scale = Some(DeviceScale::MinMax {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"format" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let precision = attrs.get_opt_f64("precision")?;
                        let scale_by = attrs.get_opt_f64("scale_by")?;
                        let display_as = attrs.get_opt("display_as").and_then(|s| match s {
                            "number" => Some(DisplayAs::Number),
                            "currency" => Some(DisplayAs::Currency),
                            "percent" => Some(DisplayAs::Percent),
                            _ => None,
                        });
                        let delimit_000s = attrs.get_opt_bool("delimit_000s")?;
                        format = Some(FormatOptions {
                            precision,
                            scale_by,
                            display_as,
                            delimit_000s,
                        });
                    }
                    _ => {}
                }
            }
            Event::End(end) if end.name().as_ref() == b"flow" => {
                break;
            }
            Event::Eof => {
                return Err(DeserializeError::UnexpectedEof);
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(BasicFlow {
        name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
        access,
        autoexport,
        equation,
        mathml_equation,
        multiplier,
        non_negative,
        units,
        documentation,
        range,
        scale,
        format,
        #[cfg(feature = "arrays")]
        dimensions: _dimensions.map(|dims| dims.dims.into_iter().map(|d| d.name).collect()),
        #[cfg(feature = "arrays")]
        elements,
        event_poster: _event_poster,
    })
}

/// Deserialize a BasicStock variable from XML.
///
/// This function expects the reader to be positioned at the start of a <stock> element.
pub fn deserialize_basic_stock<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<BasicStock, DeserializeError> {
    // Expect <stock> start tag
    let (name, access, autoexport) = match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"stock" => {
            let attrs = Attrs::from_start(&e, reader)?;
            parse_var_attrs(&attrs)?
        }
        Event::Start(e) => {
            return Err(DeserializeError::UnexpectedElement {
                expected: "stock".to_string(),
                found: String::from_utf8_lossy(e.name().as_ref()).to_string(),
            });
        }
        _ => {
            return Err(DeserializeError::Custom(
                "Expected stock start tag".to_string(),
            ));
        }
    };
    buf.clear();
    deserialize_basic_stock_impl(reader, buf, name, access, autoexport, false)
}

/// Parsed conveyor element data.
#[derive(Debug, Default)]
struct ConveyorData {
    length: Option<Expression>,
    capacity: Option<Expression>,
    inflow_limit: Option<Expression>,
    sample: Option<Expression>,
    arrest_value: Option<Expression>,
    discrete: Option<bool>,
    batch_integrity: Option<bool>,
    one_at_a_time: Option<bool>,
    exponential_leakage: Option<bool>,
}

/// Parse a conveyor element and its children.
#[allow(dead_code)]
fn parse_conveyor_element<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start_event: &quick_xml::events::BytesStart,
) -> Result<ConveyorData, DeserializeError> {
    let mut data = ConveyorData::default();

    let attrs = Attrs::from_start(start_event, reader)?;
    data.discrete = attrs.get_opt_bool("discrete")?;
    data.batch_integrity = attrs.get_opt_bool("batch_integrity")?;
    data.one_at_a_time = attrs.get_opt_bool("one_at_a_time")?;
    data.exponential_leakage = attrs.get_opt_bool("exponential_leak")?;

    // Read child elements
    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"len" => {
                        data.length = Some(read_expression(reader, buf)?);
                    }
                    b"capacity" => {
                        data.capacity = Some(read_expression(reader, buf)?);
                    }
                    b"in_limit" => {
                        data.inflow_limit = Some(read_expression(reader, buf)?);
                    }
                    b"sample" => {
                        data.sample = Some(read_expression(reader, buf)?);
                    }
                    _ => {
                        // Skip unknown elements - copy name before clearing buf
                        let name = e.name().as_ref().to_vec();
                        buf.clear();
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == name.as_slice() => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                    }
                }
            }
            Event::End(end) if end.name().as_ref() == b"conveyor" => {
                break;
            }
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(data)
}

/// Internal implementation of stock deserialization that returns Stock enum.
pub(crate) fn deserialize_stock_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<Identifier>,
    access: Option<crate::model::vars::AccessType>,
    autoexport: Option<bool>,
    _is_empty_tag: bool,
) -> Result<Stock, DeserializeError> {
    let mut initial_equation: Option<Expression> = None;
    #[cfg(feature = "mathml")]
    let mut mathml_equation: Option<String> = None;
    let mut inflows: Vec<Identifier> = Vec::new();
    let mut outflows: Vec<Identifier> = Vec::new();
    let mut documentation: Option<Documentation> = None;
    let mut units: Option<UnitEquation> = None;
    let mut non_negative: Option<Option<bool>> = None;
    let mut range: Option<DeviceRange> = None;
    let mut scale: Option<DeviceScale> = None;
    let mut format: Option<FormatOptions> = None;
    let mut _event_poster: Option<EventPoster> = None;
    #[cfg(feature = "arrays")]
    let mut _dimensions: Option<VariableDimensions> = None;
    #[cfg(feature = "arrays")]
    let mut elements: Vec<ArrayElement> = Vec::new();

    // Stock type indicators
    let mut conveyor_data: Option<ConveyorData> = None;
    let mut is_queue: bool = false;

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"eqn" => {
                        initial_equation = Some(read_expression(reader, buf)?);
                    }
                    #[cfg(feature = "mathml")]
                    b"mathml" => {
                        mathml_equation = Some(read_text_content(reader, buf)?);
                    }
                    b"inflow" => {
                        let inflow_text = read_text_content(reader, buf)?;
                        let inflow_id =
                            Identifier::parse_from_attribute(&inflow_text).map_err(|e| {
                                DeserializeError::Custom(format!(
                                    "Invalid inflow identifier: {}",
                                    e
                                ))
                            })?;
                        inflows.push(inflow_id);
                    }
                    b"outflow" => {
                        let outflow_text = read_text_content(reader, buf)?;
                        let outflow_id =
                            Identifier::parse_from_attribute(&outflow_text).map_err(|e| {
                                DeserializeError::Custom(format!(
                                    "Invalid outflow identifier: {}",
                                    e
                                ))
                            })?;
                        outflows.push(outflow_id);
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
                    b"non_negative" => {
                        // Read the text content of the non_negative element
                        let text = read_text_content(reader, buf)?;
                        non_negative = Some(match text.trim() {
                            "true" => Some(true),
                            "false" => Some(false),
                            "" => None, // Empty content means default (true)
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid non_negative value: {}",
                                    text
                                )));
                            }
                        });
                    }
                    b"range" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
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
                        if let (Some(min_val), Some(max_val)) = (min, max) {
                            range = Some(DeviceRange {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"scale" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
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
                        } else if let (Some(min_val), Some(max_val)) = (min, max) {
                            scale = Some(DeviceScale::MinMax {
                                min: min_val,
                                max: max_val,
                            });
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
                    b"conveyor" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let mut cd = ConveyorData::default();
                        cd.discrete = attrs.get_opt_bool("discrete")?;
                        cd.batch_integrity = attrs.get_opt_bool("batch_integrity")?;
                        cd.one_at_a_time = attrs.get_opt_bool("one_at_a_time")?;
                        cd.exponential_leakage = attrs.get_opt_bool("exponential_leak")?;
                        buf.clear();
                        // Read conveyor child elements
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::Start(inner) => match inner.name().as_ref() {
                                    b"len" => cd.length = Some(read_expression(reader, buf)?),
                                    b"capacity" => {
                                        cd.capacity = Some(read_expression(reader, buf)?)
                                    }
                                    b"in_limit" => {
                                        cd.inflow_limit = Some(read_expression(reader, buf)?)
                                    }
                                    b"sample" => cd.sample = Some(read_expression(reader, buf)?),
                                    b"arrest" => {
                                        cd.arrest_value = Some(read_expression(reader, buf)?)
                                    }
                                    _ => {
                                        let name = inner.name().as_ref().to_vec();
                                        loop {
                                            match reader.read_event_into(buf)? {
                                                Event::End(e)
                                                    if e.name().as_ref() == name.as_slice() =>
                                                {
                                                    break;
                                                }
                                                Event::Eof => {
                                                    return Err(DeserializeError::UnexpectedEof);
                                                }
                                                _ => {}
                                            }
                                            buf.clear();
                                        }
                                    }
                                },
                                Event::End(end) if end.name().as_ref() == b"conveyor" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        conveyor_data = Some(cd);
                    }
                    b"queue" => {
                        is_queue = true;
                        buf.clear();
                        // Queue element may have content, skip it
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == b"queue" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                    }
                    #[cfg(feature = "arrays")]
                    b"element" => {
                        buf.clear();
                        let element = deserialize_array_element(reader, buf)?;
                        elements.push(element);
                    }
                    _ => {
                        // Skip unknown elements
                        let name = e.name().as_ref().to_vec();
                        buf.clear();
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == name.as_slice() => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                    }
                }
            }
            Event::Empty(e) => {
                // Handle empty tags
                match e.name().as_ref() {
                    b"non_negative" => {
                        // Empty tag <non_negative/> means default (true)
                        non_negative = Some(None);
                    }
                    b"queue" => {
                        // Empty <queue/> tag
                        is_queue = true;
                    }
                    b"conveyor" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        conveyor_data = Some(ConveyorData {
                            discrete: attrs.get_opt_bool("discrete")?,
                            ..Default::default()
                        });
                    }
                    b"range" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
                        if let (Some(min_val), Some(max_val)) = (min, max) {
                            range = Some(DeviceRange {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"scale" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
                        let auto = attrs.get_opt_bool("auto")?;
                        let group = attrs.get_opt_u32("group")?;
                        if let Some(auto_val) = auto {
                            scale = Some(DeviceScale::Auto(auto_val));
                        } else if let Some(group_val) = group {
                            scale = Some(DeviceScale::Group(group_val));
                        } else if let (Some(min_val), Some(max_val)) = (min, max) {
                            scale = Some(DeviceScale::MinMax {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"format" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let precision = attrs.get_opt_f64("precision")?;
                        let scale_by = attrs.get_opt_f64("scale_by")?;
                        let display_as = attrs.get_opt("display_as").and_then(|s| match s {
                            "number" => Some(DisplayAs::Number),
                            "currency" => Some(DisplayAs::Currency),
                            "percent" => Some(DisplayAs::Percent),
                            _ => None,
                        });
                        let delimit_000s = attrs.get_opt_bool("delimit_000s")?;
                        format = Some(FormatOptions {
                            precision,
                            scale_by,
                            display_as,
                            delimit_000s,
                        });
                    }
                    _ => {}
                }
            }
            Event::End(e) if e.name().as_ref() == b"stock" => {
                break;
            }
            Event::Eof => {
                return Err(DeserializeError::UnexpectedEof);
            }
            _ => {}
        }
        buf.clear();
    }

    let name = name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?;
    let initial_equation =
        initial_equation.ok_or_else(|| DeserializeError::MissingField("eqn".to_string()))?;

    // Determine the stock type based on what we parsed
    if let Some(conv) = conveyor_data {
        // It's a conveyor stock
        let length = conv.length.ok_or_else(|| {
            DeserializeError::Custom("Conveyor stock requires a length element".to_string())
        })?;

        Ok(Stock::Conveyor(ConveyorStock {
            name,
            access,
            autoexport,
            inflows,
            outflows,
            initial_equation,
            length,
            capacity: conv.capacity,
            inflow_limit: conv.inflow_limit,
            sample: conv.sample,
            arrest_value: conv.arrest_value,
            discrete: conv.discrete,
            batch_integrity: conv.batch_integrity,
            one_at_a_time: conv.one_at_a_time,
            exponential_leakage: conv.exponential_leakage,
            units,
            documentation,
            range,
            scale,
            format,
            #[cfg(feature = "arrays")]
            dimensions: _dimensions.map(|dims| dims.dims.into_iter().map(|d| d.name).collect()),
            #[cfg(feature = "arrays")]
            elements,
            event_poster: _event_poster,
            #[cfg(feature = "mathml")]
            mathml_equation,
        }))
    } else if is_queue {
        // It's a queue stock
        Ok(Stock::Queue(QueueStock {
            name,
            access,
            autoexport,
            inflows,
            outflows,
            initial_equation,
            units,
            documentation,
            range,
            scale,
            format,
            #[cfg(feature = "arrays")]
            dimensions: _dimensions.map(|dims| dims.dims.into_iter().map(|d| d.name).collect()),
            #[cfg(feature = "arrays")]
            elements,
            event_poster: _event_poster,
            #[cfg(feature = "mathml")]
            mathml_equation,
        }))
    } else {
        // It's a basic stock
        Ok(Stock::Basic(BasicStock {
            name,
            access,
            autoexport,
            inflows,
            outflows,
            initial_equation,
            #[cfg(feature = "mathml")]
            mathml_equation,
            non_negative,
            units,
            documentation,
            range,
            scale,
            format,
            #[cfg(feature = "arrays")]
            dimensions: _dimensions.map(|dims| dims.dims.into_iter().map(|d| d.name).collect()),
            #[cfg(feature = "arrays")]
            elements,
            event_poster: _event_poster,
        }))
    }
}

/// Internal implementation of basic stock deserialization.
/// This is kept for backwards compatibility but delegates to deserialize_stock_impl.
pub(crate) fn deserialize_basic_stock_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<Identifier>,
    access: Option<crate::model::vars::AccessType>,
    autoexport: Option<bool>,
    is_empty_tag: bool,
) -> Result<BasicStock, DeserializeError> {
    match deserialize_stock_impl(reader, buf, name, access, autoexport, is_empty_tag)? {
        Stock::Basic(basic) => Ok(basic),
        Stock::Conveyor(_) => Err(DeserializeError::Custom(
            "Expected BasicStock but got ConveyorStock".to_string(),
        )),
        Stock::Queue(_) => Err(DeserializeError::Custom(
            "Expected BasicStock but got QueueStock".to_string(),
        )),
    }
}

pub(crate) fn deserialize_group_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<Identifier>,
    is_empty_tag: bool,
) -> Result<Group, DeserializeError> {
    let mut doc: Option<Documentation> = None;
    let mut entities = Vec::new();

    if !is_empty_tag {
        loop {
            match reader.read_event_into(buf)? {
                Event::Start(e) if e.name().as_ref() == b"doc" => {
                    let doc_text = read_text_content(reader, buf)?;
                    // Determine if it's HTML or plain text
                    doc = Some(
                        if doc_text.trim().contains('<') && doc_text.trim().contains('>') {
                            Documentation::Html(doc_text)
                        } else {
                            Documentation::PlainText(doc_text)
                        },
                    );
                }
                Event::Start(e) if e.name().as_ref() == b"entity" => {
                    let attrs = Attrs::from_start(&e, reader)?;
                    let entity_name = attrs
                        .get_opt("name")
                        .map(|s| {
                            Identifier::parse_from_attribute(s).map_err(|err| {
                                DeserializeError::Custom(format!("Invalid entity name: {}", err))
                            })
                        })
                        .transpose()?;
                    let run = attrs.get_opt_bool("run")?;
                    buf.clear();

                    // Read until end of entity
                    loop {
                        match reader.read_event_into(buf)? {
                            Event::End(end) if end.name().as_ref() == b"entity" => break,
                            Event::Eof => return Err(DeserializeError::UnexpectedEof),
                            _ => {}
                        }
                        buf.clear();
                    }

                    if let Some(name) = entity_name {
                        entities.push(GroupEntity {
                            name,
                            run: run.unwrap_or(false),
                        });
                    }
                }
                Event::Empty(e) if e.name().as_ref() == b"entity" => {
                    let attrs = Attrs::from_start(&e, reader)?;
                    let entity_name = attrs
                        .get_opt("name")
                        .map(|s| {
                            Identifier::parse_from_attribute(s).map_err(|err| {
                                DeserializeError::Custom(format!("Invalid entity name: {}", err))
                            })
                        })
                        .transpose()?;
                    let run = attrs.get_opt_bool("run")?;

                    if let Some(name) = entity_name {
                        entities.push(GroupEntity {
                            name,
                            run: run.unwrap_or(false),
                        });
                    }
                }
                Event::End(e) if e.name().as_ref() == b"group" => break,
                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                _ => {}
            }
            buf.clear();
        }
    }

    Ok(Group {
        name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
        doc,
        entities,
        display: Vec::new(), // Display UIDs are handled separately in views
    })
}

/// Helper to read an optional expression from an element.
pub fn read_optional_expression<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    element_name: &str,
) -> Result<Option<Expression>, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == element_name.as_bytes() => {
            let expr = read_expression(reader, buf)?;
            Ok(Some(expr))
        }
        Event::Empty(e) if e.name().as_ref() == element_name.as_bytes() => Ok(None),
        _ => Ok(None),
    }
}
