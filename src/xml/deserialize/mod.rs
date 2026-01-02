//! XML deserialization module.
//!
//! This module provides manual XML deserialization for XMILE structures using quick-xml.
//! It handles edge cases (empty tags, optional fields, CDATA sections, etc.) naturally
//! and enables reliable round-trip testing with the serialization module.

pub mod behavior;
pub mod data;
pub mod dimensions;
pub mod graphical_functions;
pub mod header;
pub mod helpers;
#[cfg(feature = "macros")]
pub mod macros;
pub mod specs;
pub mod style;
pub mod units;
pub mod variables;
pub mod views;

// Re-export submodule functions for convenience
pub use behavior::deserialize_behavior;
pub use data::deserialize_data;
pub use dimensions::deserialize_file_dimensions;
pub use header::deserialize_header;
#[cfg(feature = "macros")]
pub use macros::deserialize_macro;
pub use specs::deserialize_sim_specs;
pub use style::deserialize_style;
pub use units::deserialize_model_units;
pub use variables::deserialize_variables;
pub use views::{deserialize_view, deserialize_views};

// Main deserialization types and functions
use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::BufRead;
use thiserror::Error;

use crate::behavior::Behavior;
use crate::specs::SimulationSpecs;
use crate::xml::quick::de::{Attrs, skip_element};
use crate::xml::schema::{Model, Variables};

// Import internal implementations for use within this module
use specs::deserialize_sim_specs_impl;
use variables::deserialize_variables_impl;
use views::deserialize_views_impl;

// Re-export helper functions from variables module
#[cfg(feature = "arrays")]
pub use variables::{deserialize_array_element, deserialize_dimensions};
pub use variables::{
    deserialize_event_poster, deserialize_format, deserialize_format_from_start, deserialize_range,
    deserialize_range_from_start, deserialize_scale, deserialize_scale_from_start, read_expression,
    read_non_negative,
};

/// Errors that can occur during XML deserialization.
#[derive(Debug, Error)]
pub enum DeserializeError {
    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("XML attribute error: {0}")]
    Attr(#[from] quick_xml::events::attributes::AttrError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Deserialization error: {0}")]
    Custom(String),
    #[error("Unexpected end of XML")]
    UnexpectedEof,
    #[error("Unexpected element: expected {expected}, found {found}")]
    UnexpectedElement { expected: String, found: String },
    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Deserialize a Model structure from XML.
///
/// This function expects the reader to be positioned at the start of a <model> element.
pub fn deserialize_model<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Model, DeserializeError> {
    // Expect <model> start tag
    let event = reader.read_event_into(buf)?;

    let (name, resource) = match event {
        Event::Start(e) if e.name().as_ref() == b"model" => {
            // Use Attrs helper for cleaner attribute parsing
            let attrs = Attrs::from_start(&e, reader)?;
            (
                attrs.get_opt_string("name"),
                attrs.get_opt_string("resource"),
            )
        }
        Event::Start(e) => {
            return Err(DeserializeError::UnexpectedElement {
                expected: "model".to_string(),
                found: String::from_utf8_lossy(e.name().as_ref()).to_string(),
            });
        }
        _ => {
            return Err(DeserializeError::Custom(
                "Expected model start tag".to_string(),
            ));
        }
    };
    buf.clear();
    deserialize_model_impl(reader, buf, name, resource)
}

/// Internal implementation of model deserialization.
pub(crate) fn deserialize_model_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<String>,
    resource: Option<String>,
) -> Result<Model, DeserializeError> {
    let mut sim_specs: Option<SimulationSpecs> = None;
    let mut behavior: Option<Behavior> = None;
    let mut variables: Option<Variables> = None;
    let mut views: Option<crate::xml::schema::Views> = None;

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"sim_specs" => {
                        // Extract attributes from the already-read start event
                        let mut method: Option<String> = None;
                        let mut time_units: Option<String> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"method" => {
                                    method =
                                        Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                                b"time_units" => {
                                    time_units =
                                        Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                                _ => {}
                            }
                        }
                        buf.clear();
                        sim_specs =
                            Some(deserialize_sim_specs_impl(reader, buf, method, time_units)?);
                    }
                    b"variables" => {
                        // deserialize_variables expects to read the start tag itself
                        // but we've already consumed it, so call the impl directly
                        buf.clear();
                        variables = Some(deserialize_variables_impl(reader, buf)?);
                    }
                    b"behavior" => {
                        // deserialize_behavior already reads its own event, so it's fine
                        behavior = Some(deserialize_behavior(reader, buf)?);
                    }
                    b"views" => {
                        // Extract attributes from the already-read start event
                        let mut visible_view: Option<u32> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            if attr.key.as_ref() == b"visible_view" {
                                let visible_str =
                                    attr.decode_and_unescape_value(reader)?.to_string();
                                visible_view = Some(visible_str.parse().map_err(|e| {
                                    DeserializeError::Custom(format!(
                                        "Invalid visible_view value: {}",
                                        e
                                    ))
                                })?);
                            }
                        }
                        buf.clear();
                        views = Some(deserialize_views_impl(reader, buf, visible_view)?);
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
                    b"variables" => {
                        // Empty variables element: <variables/>
                        variables = Some(Variables {
                            variables: Vec::new(),
                        });
                    }
                    b"views" => {
                        // Empty views element: <views/>
                        views = Some(crate::xml::schema::Views {
                            visible_view: None,
                            views: Vec::new(),
                            style: None,
                        });
                    }
                    // Other empty elements can be ignored
                    _ => {}
                }
            }
            Event::End(e) if e.name().as_ref() == b"model" => {
                break;
            }
            Event::Eof => {
                return Err(DeserializeError::UnexpectedEof);
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(Model {
        name,
        resource,
        sim_specs,
        behavior,
        variables: variables
            .ok_or_else(|| DeserializeError::MissingField("variables".to_string()))?,
        views,
    })
}
