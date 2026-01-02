//! Variables deserialization module.
//!
//! This module handles deserialization of all variable types:
//! stocks, flows, auxiliaries, modules, groups, and graphical functions.

use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::BufRead;
use std::str::FromStr;

use crate::Expression;
use crate::equation::Identifier;
use crate::equation::units::UnitEquation;
use crate::model::events::{Event as ModelEvent, EventPoster, Threshold};
use crate::model::groups::{Group, GroupEntity};
use crate::model::object::{DeviceRange, DeviceScale, DisplayAs, Documentation, FormatOptions};
#[cfg(feature = "arrays")]
use crate::model::vars::array::ArrayElement;
#[cfg(feature = "arrays")]
use crate::model::vars::array::{Dimension, VariableDimensions};
use crate::model::vars::gf::{
    GraphicalFunction, GraphicalFunctionData, GraphicalFunctionPoints, GraphicalFunctionScale,
    GraphicalFunctionType,
};
#[cfg(feature = "submodels")]
use crate::model::vars::module::{Module, ModuleConnection};
use crate::model::vars::{
    aux::Auxiliary,
    flow::BasicFlow,
    stock::{BasicStock, ConveyorStock, QueueStock, Stock},
};
use crate::xml::deserialize::DeserializeError;
use crate::xml::deserialize::graphical_functions::{
    deserialize_graphical_function, deserialize_graphical_function_impl,
};
use crate::xml::deserialize::helpers::{read_number_content, read_text_content};
use crate::xml::quick::de::skip_element;
use crate::xml::schema::Variables;

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
        let mut attrs: Vec<(Vec<u8>, String)> = Vec::new();
        match &first_event {
            Event::Start(e) | Event::Empty(e) => {
                // Collect all attributes into a Vec - this iterator borrows from e which borrows from first_event
                for attr_result in e.attributes() {
                    let attr = attr_result?;
                    let key = attr.key.as_ref().to_vec();
                    let value = attr.decode_and_unescape_value(reader)?.to_string();
                    attrs.push((key, value));
                }
            }
            _ => {}
        }
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
            variables.push(crate::model::vars::Variable::GraphicalFunction(gf));
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
                        // Extract attributes from the already-read start event
                        let mut name: Option<Identifier> = None;
                        let mut access: Option<crate::model::vars::AccessType> = None;
                        let mut autoexport: Option<bool> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"name" => {
                                    let name_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    name =
                                        Some(Identifier::parse_from_attribute(&name_str).map_err(
                                            |e| {
                                                DeserializeError::Custom(format!(
                                                    "Invalid identifier: {}",
                                                    e
                                                ))
                                            },
                                        )?);
                                }
                                b"access" => {
                                    let access_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    access = Some(match access_str.as_str() {
                                        "input" => crate::model::vars::AccessType::Input,
                                        "output" => crate::model::vars::AccessType::Output,
                                        _ => {
                                            return Err(DeserializeError::Custom(format!(
                                                "Invalid access type: {}",
                                                access_str
                                            )));
                                        }
                                    });
                                }
                                b"autoexport" => {
                                    let autoexport_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    autoexport =
                                        Some(autoexport_str.parse::<bool>().map_err(|e| {
                                            DeserializeError::Custom(format!(
                                                "Invalid autoexport value: {}",
                                                e
                                            ))
                                        })?);
                                }
                                _ => {}
                            }
                        }
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
                        // Extract attributes from the already-read start event
                        let mut name: Option<Identifier> = None;
                        let mut access: Option<crate::model::vars::AccessType> = None;
                        let mut autoexport: Option<bool> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"name" => {
                                    let name_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    name =
                                        Some(Identifier::parse_from_attribute(&name_str).map_err(
                                            |e| {
                                                DeserializeError::Custom(format!(
                                                    "Invalid identifier: {}",
                                                    e
                                                ))
                                            },
                                        )?);
                                }
                                b"access" => {
                                    let access_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    access = Some(match access_str.as_str() {
                                        "input" => crate::model::vars::AccessType::Input,
                                        "output" => crate::model::vars::AccessType::Output,
                                        _ => {
                                            return Err(DeserializeError::Custom(format!(
                                                "Invalid access type: {}",
                                                access_str
                                            )));
                                        }
                                    });
                                }
                                b"autoexport" => {
                                    let autoexport_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    autoexport =
                                        Some(autoexport_str.parse::<bool>().map_err(|e| {
                                            DeserializeError::Custom(format!(
                                                "Invalid autoexport value: {}",
                                                e
                                            ))
                                        })?);
                                }
                                _ => {}
                            }
                        }
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
                        // Extract attributes from the already-read start event
                        let mut name: Option<Identifier> = None;
                        let mut access: Option<crate::model::vars::AccessType> = None;
                        let mut autoexport: Option<bool> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"name" => {
                                    let name_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    name =
                                        Some(Identifier::parse_from_attribute(&name_str).map_err(
                                            |e| {
                                                DeserializeError::Custom(format!(
                                                    "Invalid identifier: {}",
                                                    e
                                                ))
                                            },
                                        )?);
                                }
                                b"access" => {
                                    let access_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    access = Some(match access_str.as_str() {
                                        "input" => crate::model::vars::AccessType::Input,
                                        "output" => crate::model::vars::AccessType::Output,
                                        _ => {
                                            return Err(DeserializeError::Custom(format!(
                                                "Invalid access type: {}",
                                                access_str
                                            )));
                                        }
                                    });
                                }
                                b"autoexport" => {
                                    let autoexport_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    autoexport =
                                        Some(autoexport_str.parse::<bool>().map_err(|e| {
                                            DeserializeError::Custom(format!(
                                                "Invalid autoexport value: {}",
                                                e
                                            ))
                                        })?);
                                }
                                _ => {}
                            }
                        }
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
                        // Extract attributes from the already-read start event
                        let mut name: Option<Identifier> = None;
                        let mut gf_type: Option<GraphicalFunctionType> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"name" => {
                                    let name_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    name =
                                        Some(Identifier::parse_from_attribute(&name_str).map_err(
                                            |e| {
                                                DeserializeError::Custom(format!(
                                                    "Invalid identifier: {}",
                                                    e
                                                ))
                                            },
                                        )?);
                                }
                                b"type" => {
                                    let type_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    gf_type =
                                        Some(GraphicalFunctionType::from_str(&type_str).map_err(
                                            |e| {
                                                DeserializeError::Custom(format!(
                                                    "Invalid function type: {}",
                                                    e
                                                ))
                                            },
                                        )?);
                                }
                                _ => {}
                            }
                        }
                        buf.clear();
                        let gf = deserialize_graphical_function_impl(reader, buf, name, gf_type)?;
                        variables.push(crate::model::vars::Variable::GraphicalFunction(gf));
                    }
                    #[cfg(feature = "submodels")]
                    b"module" => {
                        // Extract name and resource attributes from already-read start event
                        let mut name: Option<Identifier> = None;
                        let mut resource: Option<String> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"name" => {
                                    let name_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    name =
                                        Some(Identifier::parse_from_attribute(&name_str).map_err(
                                            |err| {
                                                DeserializeError::Custom(format!(
                                                    "Invalid module name: {}",
                                                    err
                                                ))
                                            },
                                        )?);
                                }
                                b"resource" => {
                                    resource =
                                        Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                                _ => {}
                            }
                        }
                        buf.clear();
                        let module = deserialize_module_impl(reader, buf, name, resource, false)?;
                        variables.push(crate::model::vars::Variable::Module(module));
                    }
                    b"group" => {
                        // Extract name attribute
                        let mut name: Option<Identifier> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            if attr.key.as_ref() == b"name" {
                                let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                                name = Some(Identifier::parse_from_attribute(&name_str).map_err(
                                    |err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid group name: {}",
                                            err
                                        ))
                                    },
                                )?);
                            }
                        }
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
    let mut subscript: Option<String> = None;

    let is_empty = match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"element" => {
            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                if attr.key.as_ref() == b"subscript" {
                    subscript = Some(attr.decode_and_unescape_value(reader)?.to_string());
                }
            }
            false
        }
        Event::Empty(e) if e.name().as_ref() == b"element" => {
            // Read attributes from empty tag
            for attr in e.attributes() {
                let attr = attr?;
                if attr.key.as_ref() == b"subscript" {
                    subscript = Some(attr.decode_and_unescape_value(reader)?.to_string());
                }
            }
            true
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
            let mut min: Option<f64> = None;
            let mut max: Option<f64> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"min" => {
                        let min_str = attr.decode_and_unescape_value(reader)?.to_string();
                        min = Some(min_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid min value: {}", e))
                        })?);
                    }
                    b"max" => {
                        let max_str = attr.decode_and_unescape_value(reader)?.to_string();
                        max = Some(max_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid max value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

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

            Ok(DeviceRange {
                min: min.ok_or_else(|| DeserializeError::MissingField("min".to_string()))?,
                max: max.ok_or_else(|| DeserializeError::MissingField("max".to_string()))?,
            })
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
    let mut min: Option<f64> = None;
    let mut max: Option<f64> = None;

    for attr in start_event.attributes() {
        let attr = attr?;
        match attr.key.as_ref() {
            b"min" => {
                let min_str = attr.decode_and_unescape_value(reader)?.to_string();
                min =
                    Some(min_str.parse().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid min value: {}", e))
                    })?);
            }
            b"max" => {
                let max_str = attr.decode_and_unescape_value(reader)?.to_string();
                max =
                    Some(max_str.parse().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid max value: {}", e))
                    })?);
            }
            _ => {}
        }
    }

    // Read until end tag
    loop {
        match reader.read_event_into(buf)? {
            Event::End(e) if e.name().as_ref() == b"range" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(DeviceRange {
        min: min.ok_or_else(|| DeserializeError::MissingField("min".to_string()))?,
        max: max.ok_or_else(|| DeserializeError::MissingField("max".to_string()))?,
    })
}

/// Deserialize a DeviceScale from XML.
pub fn deserialize_scale<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<DeviceScale, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Empty(e) | Event::Start(e) if e.name().as_ref() == b"scale" => {
            let mut min: Option<f64> = None;
            let mut max: Option<f64> = None;
            let mut auto: Option<bool> = None;
            let mut group: Option<u32> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"min" => {
                        let min_str = attr.decode_and_unescape_value(reader)?.to_string();
                        min = Some(min_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid min value: {}", e))
                        })?);
                    }
                    b"max" => {
                        let max_str = attr.decode_and_unescape_value(reader)?.to_string();
                        max = Some(max_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid max value: {}", e))
                        })?);
                    }
                    b"auto" => {
                        let auto_str = attr.decode_and_unescape_value(reader)?.to_string();
                        auto = Some(match auto_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid auto value: {}",
                                    auto_str
                                )));
                            }
                        });
                    }
                    b"group" => {
                        let group_str = attr.decode_and_unescape_value(reader)?.to_string();
                        group = Some(group_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid group value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

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
    let mut min: Option<f64> = None;
    let mut max: Option<f64> = None;
    let mut auto: Option<bool> = None;
    let mut group: Option<u32> = None;

    for attr in start_event.attributes() {
        let attr = attr?;
        match attr.key.as_ref() {
            b"min" => {
                let min_str = attr.decode_and_unescape_value(reader)?.to_string();
                min =
                    Some(min_str.parse().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid min value: {}", e))
                    })?);
            }
            b"max" => {
                let max_str = attr.decode_and_unescape_value(reader)?.to_string();
                max =
                    Some(max_str.parse().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid max value: {}", e))
                    })?);
            }
            b"auto" => {
                let auto_str = attr.decode_and_unescape_value(reader)?.to_string();
                auto = Some(match auto_str.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => {
                        return Err(DeserializeError::Custom(format!(
                            "Invalid auto value: {}",
                            auto_str
                        )));
                    }
                });
            }
            b"group" => {
                let group_str = attr.decode_and_unescape_value(reader)?.to_string();
                group = Some(group_str.parse().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid group value: {}", e))
                })?);
            }
            _ => {}
        }
    }

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
            let mut precision: Option<f64> = None;
            let mut scale_by: Option<f64> = None;
            let mut display_as: Option<DisplayAs> = None;
            let mut delimit_000s: Option<bool> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"precision" => {
                        let prec_str = attr.decode_and_unescape_value(reader)?.to_string();
                        precision = Some(prec_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid precision value: {}", e))
                        })?);
                    }
                    b"scale_by" => {
                        let scale_str = attr.decode_and_unescape_value(reader)?.to_string();
                        scale_by = Some(scale_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid scale_by value: {}", e))
                        })?);
                    }
                    b"display_as" => {
                        let display_str = attr.decode_and_unescape_value(reader)?.to_string();
                        display_as = Some(match display_str.as_str() {
                            "number" => DisplayAs::Number,
                            "currency" => DisplayAs::Currency,
                            "percent" => DisplayAs::Percent,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid display_as value: {}",
                                    display_str
                                )));
                            }
                        });
                    }
                    b"delimit_000s" => {
                        let delim_str = attr.decode_and_unescape_value(reader)?.to_string();
                        delimit_000s = Some(match delim_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid delimit_000s value: {}",
                                    delim_str
                                )));
                            }
                        });
                    }
                    _ => {}
                }
            }

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
    let mut precision: Option<f64> = None;
    let mut scale_by: Option<f64> = None;
    let mut display_as: Option<DisplayAs> = None;
    let mut delimit_000s: Option<bool> = None;

    for attr in start_event.attributes() {
        let attr = attr?;
        match attr.key.as_ref() {
            b"precision" => {
                let prec_str = attr.decode_and_unescape_value(reader)?.to_string();
                precision = Some(prec_str.parse().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid precision value: {}", e))
                })?);
            }
            b"scale_by" => {
                let scale_str = attr.decode_and_unescape_value(reader)?.to_string();
                scale_by = Some(scale_str.parse().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid scale_by value: {}", e))
                })?);
            }
            b"display_as" => {
                let display_str = attr.decode_and_unescape_value(reader)?.to_string();
                display_as = Some(match display_str.as_str() {
                    "number" => DisplayAs::Number,
                    "currency" => DisplayAs::Currency,
                    "percent" => DisplayAs::Percent,
                    _ => {
                        return Err(DeserializeError::Custom(format!(
                            "Invalid display_as value: {}",
                            display_str
                        )));
                    }
                });
            }
            b"delimit_000s" => {
                let delim_str = attr.decode_and_unescape_value(reader)?.to_string();
                delimit_000s = Some(match delim_str.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => {
                        return Err(DeserializeError::Custom(format!(
                            "Invalid delimit_000s value: {}",
                            delim_str
                        )));
                    }
                });
            }
            _ => {}
        }
    }

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
                        let mut name: Option<String> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            if attr.key.as_ref() == b"name" {
                                name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                            }
                        }

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
            let mut min: Option<f64> = None;
            let mut max: Option<f64> = None;

            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"min" => {
                        let min_str = attr.decode_and_unescape_value(reader)?.to_string();
                        min = Some(min_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid min value: {}", e))
                        })?);
                    }
                    b"max" => {
                        let max_str = attr.decode_and_unescape_value(reader)?.to_string();
                        max = Some(max_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid max value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

            let mut thresholds = Vec::new();

            loop {
                let event = reader.read_event_into(buf)?;
                match &event {
                    Event::Start(e) if e.name().as_ref() == b"threshold" => {
                        // Clone the attributes we need before clearing buf
                        let mut value: Option<f64> = None;
                        let mut direction: Option<String> = None;
                        let mut repeat: Option<String> = None;
                        let mut interval: Option<f64> = None;

                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"value" => {
                                    let value_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    value = Some(value_str.parse().map_err(|e| {
                                        DeserializeError::Custom(format!("Invalid value: {}", e))
                                    })?);
                                }
                                b"direction" => {
                                    direction =
                                        Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                                b"repeat" => {
                                    repeat =
                                        Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                                b"interval" => {
                                    let interval_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    interval = Some(interval_str.parse().map_err(|e| {
                                        DeserializeError::Custom(format!("Invalid interval: {}", e))
                                    })?);
                                }
                                _ => {}
                            }
                        }
                        buf.clear();

                        // Now read events
                        let mut events = Vec::new();
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::Start(e) if e.name().as_ref() == b"event" => {
                                    let mut sim_action: Option<String> = None;
                                    for attr in e.attributes() {
                                        let attr = attr?;
                                        if attr.key.as_ref() == b"sim_action" {
                                            sim_action = Some(
                                                attr.decode_and_unescape_value(reader)?.to_string(),
                                            );
                                        }
                                    }
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

// Old graphical function implementation (replaced by module):
/// Deserialize GraphicalFunction from XML.
#[allow(dead_code)]
pub fn deserialize_graphical_function_old<R: BufRead>(
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
                        equation = Some(read_expression(reader, buf)?);
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
                        // Extract attributes before clearing buf - start tag already consumed
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
                        // Skip to end of xscale element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == b"xscale" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        if let (Some(min), Some(max)) = (min_val, max_val) {
                            x_scale = Some(GraphicalFunctionScale { min, max });
                        }
                    }
                    b"yscale" => {
                        // Extract attributes before clearing buf - start tag already consumed
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
                        // Skip to end of yscale element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == b"yscale" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        if let (Some(min), Some(max)) = (min_val, max_val) {
                            y_scale = Some(GraphicalFunctionScale { min, max });
                        }
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
                        // Need to skip to end as start tag already consumed
                        let element_name = e.name().as_ref().to_vec();
                        buf.clear();
                        let mut dims = Vec::new();
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::Empty(dim_e) if dim_e.name().as_ref() == b"dim" => {
                                    for attr in dim_e.attributes() {
                                        let attr = attr?;
                                        if attr.key.as_ref() == b"name" {
                                            let name =
                                                attr.decode_and_unescape_value(reader)?.to_string();
                                            dims.push(Dimension { name });
                                        }
                                    }
                                }
                                Event::End(end_e)
                                    if end_e.name().as_ref() == element_name.as_slice() =>
                                {
                                    break;
                                }
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        dimensions = Some(VariableDimensions { dims });
                    }
                    #[cfg(feature = "arrays")]
                    b"element" => {
                        // Extract subscript attribute
                        let mut subscript = String::new();
                        for attr in e.attributes() {
                            let attr = attr?;
                            if attr.key.as_ref() == b"subscript" {
                                subscript = attr.decode_and_unescape_value(reader)?.to_string();
                            }
                        }
                        // Read element content
                        buf.clear();
                        let mut elem_eqn: Option<Expression> = None;
                        let elem_gf: Option<GraphicalFunction> = None;
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::Start(inner_e) => {
                                    match inner_e.name().as_ref() {
                                        b"eqn" => {
                                            elem_eqn = Some(read_expression(reader, buf)?);
                                        }
                                        b"gf" => {
                                            // Skip gf content - complex to parse inline
                                            let gf_name = inner_e.name().as_ref().to_vec();
                                            loop {
                                                match reader.read_event_into(buf)? {
                                                    Event::End(end_e)
                                                        if end_e.name().as_ref()
                                                            == gf_name.as_slice() =>
                                                    {
                                                        break;
                                                    }
                                                    Event::Eof => {
                                                        return Err(
                                                            DeserializeError::UnexpectedEof,
                                                        );
                                                    }
                                                    _ => {}
                                                }
                                                buf.clear();
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                Event::End(end_e) if end_e.name().as_ref() == b"element" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        elements.push(ArrayElement {
                            subscript,
                            eqn: elem_eqn,
                            gf: elem_gf,
                        });
                    }
                    _ => {
                        // Skip unknown elements using the helper
                        let element_name = e.name().as_ref().to_vec();
                        buf.clear();
                        skip_element(reader, buf, &element_name)?;
                    }
                }
            }
            Event::Empty(e) => {
                match e.name().as_ref() {
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
                    b"xscale" => {
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
                            x_scale = Some(GraphicalFunctionScale { min, max });
                        }
                    }
                    b"yscale" => {
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
                            y_scale = Some(GraphicalFunctionScale { min, max });
                        }
                    }
                    _ => {} // Ignore other empty elements
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

    // Construct GraphicalFunctionData - first function
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

// Old impl (replaced by module):
/// Internal implementation of GraphicalFunction deserialization.
/// Used when the start tag has already been consumed.
#[allow(dead_code)]
pub(crate) fn deserialize_graphical_function_impl_old<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<Identifier>,
    r#type: Option<GraphicalFunctionType>,
) -> Result<GraphicalFunction, DeserializeError> {
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
                        equation = Some(read_expression(reader, buf)?);
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
                        dimensions = Some(deserialize_dimensions(reader, buf)?);
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

/// Deserialize GraphicalFunctionScale from XML.
#[allow(dead_code)]
fn deserialize_gf_scale<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<GraphicalFunctionScale, DeserializeError> {
    let event = reader.read_event_into(buf)?;
    match &event {
        Event::Empty(e) | Event::Start(e) => {
            let mut min: Option<f64> = None;
            let mut max: Option<f64> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"min" => {
                        let min_str = attr.decode_and_unescape_value(reader)?.to_string();
                        min = Some(min_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid min value: {}", e))
                        })?);
                    }
                    b"max" => {
                        let max_str = attr.decode_and_unescape_value(reader)?.to_string();
                        max = Some(max_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid max value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

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

            Ok(GraphicalFunctionScale {
                min: min.ok_or_else(|| DeserializeError::MissingField("min".to_string()))?,
                max: max.ok_or_else(|| DeserializeError::MissingField("max".to_string()))?,
            })
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
            let mut separator: Option<String> = None;

            for attr in e.attributes() {
                let attr = attr?;
                if attr.key.as_ref() == b"sep" {
                    separator = Some(attr.decode_and_unescape_value(reader)?.to_string());
                }
            }

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
            let mut name: Option<Identifier> = None;
            let mut resource: Option<String> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"name" => {
                        let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                        name =
                            Some(Identifier::parse_from_attribute(&name_str).map_err(|err| {
                                DeserializeError::Custom(format!("Invalid module name: {}", err))
                            })?);
                    }
                    b"resource" => {
                        resource = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }

            let mut connections = Vec::new();
            let mut documentation: Option<Documentation> = None;

            loop {
                match reader.read_event_into(buf)? {
                    Event::Start(e) if e.name().as_ref() == b"connect" => {
                        let mut to: Option<String> = None;
                        let mut from: Option<String> = None;

                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"to" => {
                                    to = Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                                b"from" => {
                                    from =
                                        Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                                _ => {}
                            }
                        }

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
                        let mut to: Option<String> = None;
                        let mut from: Option<String> = None;

                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"to" => {
                                    to = Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                                b"from" => {
                                    from =
                                        Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                                _ => {}
                            }
                        }

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
            let mut name: Option<Identifier> = None;
            let mut resource: Option<String> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"name" => {
                        let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                        name =
                            Some(Identifier::parse_from_attribute(&name_str).map_err(|err| {
                                DeserializeError::Custom(format!("Invalid module name: {}", err))
                            })?);
                    }
                    b"resource" => {
                        resource = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }

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
                    let mut to: Option<String> = None;
                    let mut from: Option<String> = None;

                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            b"to" => {
                                to = Some(attr.decode_and_unescape_value(reader)?.to_string());
                            }
                            b"from" => {
                                from = Some(attr.decode_and_unescape_value(reader)?.to_string());
                            }
                            _ => {}
                        }
                    }

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
                    let mut to: Option<String> = None;
                    let mut from: Option<String> = None;

                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            b"to" => {
                                to = Some(attr.decode_and_unescape_value(reader)?.to_string());
                            }
                            b"from" => {
                                from = Some(attr.decode_and_unescape_value(reader)?.to_string());
                            }
                            _ => {}
                        }
                    }

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
            let mut name: Option<Identifier> = None;

            for attr in e.attributes() {
                let attr = attr?;
                if attr.key.as_ref() == b"name" {
                    let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                    name = Some(Identifier::parse_from_attribute(&name_str).map_err(|e| {
                        DeserializeError::Custom(format!("Invalid group name: {}", e))
                    })?);
                }
            }

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
                            let mut entity_name: Option<Identifier> = None;
                            let mut run: Option<bool> = None;

                            for attr in e.attributes() {
                                let attr = attr?;
                                match attr.key.as_ref() {
                                    b"name" => {
                                        let name_str =
                                            attr.decode_and_unescape_value(reader)?.to_string();
                                        entity_name = Some(
                                            Identifier::parse_from_attribute(&name_str).map_err(
                                                |e| {
                                                    DeserializeError::Custom(format!(
                                                        "Invalid entity name: {}",
                                                        e
                                                    ))
                                                },
                                            )?,
                                        );
                                    }
                                    b"run" => {
                                        let run_str =
                                            attr.decode_and_unescape_value(reader)?.to_string();
                                        run = Some(match run_str.as_str() {
                                            "true" => true,
                                            "false" => false,
                                            _ => {
                                                return Err(DeserializeError::Custom(format!(
                                                    "Invalid run value: {}",
                                                    run_str
                                                )));
                                            }
                                        });
                                    }
                                    _ => {}
                                }
                            }

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
    let mut name: Option<Identifier> = None;
    let mut access: Option<crate::model::vars::AccessType> = None;
    let mut autoexport: Option<bool> = None;

    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"aux" => {
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
                    b"access" => {
                        let access_str = attr.decode_and_unescape_value(reader)?.to_string();
                        access = Some(match access_str.as_str() {
                            "input" => crate::model::vars::AccessType::Input,
                            "output" => crate::model::vars::AccessType::Output,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid access type: {}",
                                    access_str
                                )));
                            }
                        });
                    }
                    b"autoexport" => {
                        let autoexport_str = attr.decode_and_unescape_value(reader)?.to_string();
                        autoexport = Some(match autoexport_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid autoexport value: {}",
                                    autoexport_str
                                )));
                            }
                        });
                    }
                    _ => {}
                }
            }
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
    }
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
        dimensions: None, // TODO: Phase 3
        #[cfg(feature = "arrays")]
        elements,
        event_poster: None, // TODO: Phase 3
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
        dimensions: None, // TODO: Phase 3
        #[cfg(feature = "arrays")]
        elements,
        event_poster: None, // TODO: Phase 3
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
                        // Extract range attributes
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(s.parse().map_err(|err| {
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
                        if let (Some(min_val), Some(max_val)) = (min, max) {
                            range = Some(DeviceRange {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"scale" => {
                        // Extract scale attributes
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        let mut auto: Option<bool> = None;
                        let mut group: Option<u32> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(s.parse().map_err(|err| {
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
                        } else if let (Some(min_val), Some(max_val)) = (min, max) {
                            scale = Some(DeviceScale::MinMax {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"format" => {
                        // Extract format attributes
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
                        // Parse range from empty tag
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid max: {}", err))
                                    })?);
                                }
                                _ => {}
                            }
                        }
                        if let (Some(min_val), Some(max_val)) = (min, max) {
                            range = Some(DeviceRange {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"scale" => {
                        // Parse scale from empty tag
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        let mut auto: Option<bool> = None;
                        let mut group: Option<u32> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(s.parse().unwrap_or(0.0));
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(s.parse().unwrap_or(0.0));
                                }
                                b"auto" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    auto = Some(s == "true");
                                }
                                b"group" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    group = Some(s.parse().unwrap_or(0));
                                }
                                _ => {}
                            }
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
                        // Parse format from empty tag
                        let mut precision: Option<f64> = None;
                        let mut scale_by: Option<f64> = None;
                        let mut display_as: Option<DisplayAs> = None;
                        let mut delimit_000s: Option<bool> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"precision" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    precision = Some(s.parse().unwrap_or(0.0));
                                }
                                b"scale_by" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    scale_by = Some(s.parse().unwrap_or(0.0));
                                }
                                b"display_as" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    display_as = match s.as_str() {
                                        "number" => Some(DisplayAs::Number),
                                        "currency" => Some(DisplayAs::Currency),
                                        "percent" => Some(DisplayAs::Percent),
                                        _ => None,
                                    };
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
        dimensions: None, // TODO: Phase 3
        #[cfg(feature = "arrays")]
        elements,
        event_poster: None, // TODO: Phase 3
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
    let mut name: Option<Identifier> = None;
    let mut access: Option<crate::model::vars::AccessType> = None;
    let mut autoexport: Option<bool> = None;

    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"stock" => {
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
                    b"access" => {
                        let access_str = attr.decode_and_unescape_value(reader)?.to_string();
                        access = Some(match access_str.as_str() {
                            "input" => crate::model::vars::AccessType::Input,
                            "output" => crate::model::vars::AccessType::Output,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid access type: {}",
                                    access_str
                                )));
                            }
                        });
                    }
                    b"autoexport" => {
                        let autoexport_str = attr.decode_and_unescape_value(reader)?.to_string();
                        autoexport = Some(match autoexport_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid autoexport value: {}",
                                    autoexport_str
                                )));
                            }
                        });
                    }
                    _ => {}
                }
            }
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
    }
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

    // Read attributes
    for attr in start_event.attributes() {
        let attr = attr?;
        match attr.key.as_ref() {
            b"discrete" => {
                let value = attr.decode_and_unescape_value(reader)?.to_string();
                data.discrete = Some(value.parse::<bool>().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid discrete value: {}", e))
                })?);
            }
            b"batch_integrity" => {
                let value = attr.decode_and_unescape_value(reader)?.to_string();
                data.batch_integrity = Some(value.parse::<bool>().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid batch_integrity value: {}", e))
                })?);
            }
            b"one_at_a_time" => {
                let value = attr.decode_and_unescape_value(reader)?.to_string();
                data.one_at_a_time = Some(value.parse::<bool>().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid one_at_a_time value: {}", e))
                })?);
            }
            b"exponential_leak" => {
                let value = attr.decode_and_unescape_value(reader)?.to_string();
                data.exponential_leakage = Some(value.parse::<bool>().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid exponential_leak value: {}", e))
                })?);
            }
            _ => {}
        }
    }

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
                        // Extract attributes before clearing buf
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let min_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(min_str.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let max_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(max_str.parse().map_err(|err| {
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
                        if let (Some(min_val), Some(max_val)) = (min, max) {
                            range = Some(DeviceRange {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"scale" => {
                        // Extract attributes before clearing buf
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        let mut auto: Option<bool> = None;
                        let mut group: Option<u32> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(s.parse().map_err(|err| {
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
                        } else if let (Some(min_val), Some(max_val)) = (min, max) {
                            scale = Some(DeviceScale::MinMax {
                                min: min_val,
                                max: max_val,
                            });
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
                    b"conveyor" => {
                        // Extract attributes first
                        let mut cd = ConveyorData::default();
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"discrete" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    cd.discrete = Some(s == "true");
                                }
                                b"batch_integrity" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    cd.batch_integrity = Some(s == "true");
                                }
                                b"one_at_a_time" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    cd.one_at_a_time = Some(s == "true");
                                }
                                b"exponential_leak" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    cd.exponential_leakage = Some(s == "true");
                                }
                                _ => {}
                            }
                        }
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
                        // Empty <conveyor/> tag - parse attributes
                        conveyor_data = Some(ConveyorData::default());
                        // Read attributes from empty conveyor tag
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"discrete" => {
                                    let value = attr.decode_and_unescape_value(reader)?.to_string();
                                    if let Some(ref mut cd) = conveyor_data {
                                        cd.discrete = Some(value.parse::<bool>().unwrap_or(false));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    b"range" => {
                        // Parse range from empty tag
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(s.parse().unwrap_or(0.0));
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(s.parse().unwrap_or(0.0));
                                }
                                _ => {}
                            }
                        }
                        if let (Some(min_val), Some(max_val)) = (min, max) {
                            range = Some(DeviceRange {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"scale" => {
                        // Parse scale from empty tag
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        let mut auto: Option<bool> = None;
                        let mut group: Option<u32> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(s.parse().unwrap_or(0.0));
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(s.parse().unwrap_or(0.0));
                                }
                                b"auto" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    auto = Some(s == "true");
                                }
                                b"group" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    group = Some(s.parse().unwrap_or(0));
                                }
                                _ => {}
                            }
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
                        // Parse format from empty tag
                        let mut precision: Option<f64> = None;
                        let mut scale_by: Option<f64> = None;
                        let mut display_as: Option<DisplayAs> = None;
                        let mut delimit_000s: Option<bool> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"precision" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    precision = Some(s.parse().unwrap_or(0.0));
                                }
                                b"scale_by" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    scale_by = Some(s.parse().unwrap_or(0.0));
                                }
                                b"display_as" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    display_as = match s.as_str() {
                                        "number" => Some(DisplayAs::Number),
                                        "currency" => Some(DisplayAs::Currency),
                                        "percent" => Some(DisplayAs::Percent),
                                        _ => None,
                                    };
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
                    let mut entity_name: Option<Identifier> = None;
                    let mut run: Option<bool> = None;

                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            b"name" => {
                                let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                                entity_name = Some(
                                    Identifier::parse_from_attribute(&name_str).map_err(|err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid entity name: {}",
                                            err
                                        ))
                                    })?,
                                );
                            }
                            b"run" => {
                                let run_str = attr.decode_and_unescape_value(reader)?.to_string();
                                run = Some(run_str == "true");
                            }
                            _ => {}
                        }
                    }
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
                    let mut entity_name: Option<Identifier> = None;
                    let mut run: Option<bool> = None;

                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            b"name" => {
                                let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                                entity_name = Some(
                                    Identifier::parse_from_attribute(&name_str).map_err(|err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid entity name: {}",
                                            err
                                        ))
                                    })?,
                                );
                            }
                            b"run" => {
                                let run_str = attr.decode_and_unescape_value(reader)?.to_string();
                                run = Some(run_str == "true");
                            }
                            _ => {}
                        }
                    }

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
