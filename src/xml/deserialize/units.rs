//! Model units deserialization module.
//!
//! This module handles deserialization of model unit definitions,
//! including unit names, equations, and aliases.

use std::io::BufRead;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::{
    units::{ModelUnits, UnitDefinition},
    xml::deserialize::{DeserializeError, helpers::read_text_content},
    xml::quick::de::{Attrs, skip_element},
};

/// Deserialize ModelUnits from XML.
pub fn deserialize_model_units<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<ModelUnits, DeserializeError> {
    let event = reader.read_event_into(buf)?;
    match event {
        Event::Start(e) if e.name().as_ref() == b"model_units" => {
            buf.clear();
            deserialize_model_units_impl(reader, buf)
        }
        _ => Err(DeserializeError::Custom(
            "Expected model_units element".to_string(),
        )),
    }
}

/// Internal implementation of model_units deserialization.
pub(crate) fn deserialize_model_units_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<ModelUnits, DeserializeError> {
    let mut units = Vec::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) if e.name().as_ref() == b"unit" => {
                units.push(deserialize_unit_definition(reader, buf)?);
            }
            Event::End(e) if e.name().as_ref() == b"model_units" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(ModelUnits { units })
}

/// Deserialize a UnitDefinition from XML.
fn deserialize_unit_definition<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<UnitDefinition, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"unit" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let name = attrs.get_opt_string("name");
            let disabled = attrs.get_opt_bool("disabled")?;

            let mut eqn: Option<String> = None;
            let mut aliases = Vec::new();

            loop {
                match reader.read_event_into(buf)? {
                    Event::Start(e) => {
                        match e.name().as_ref() {
                            b"eqn" => {
                                eqn = Some(read_text_content(reader, buf)?);
                            }
                            b"alias" => {
                                let alias_text = read_text_content(reader, buf)?;
                                aliases.push(alias_text);
                            }
                            _ => {
                                // Skip unknown elements using the helper
                                let element_name = e.name().as_ref().to_vec();
                                skip_element(reader, buf, &element_name)?;
                            }
                        }
                    }
                    Event::End(e) if e.name().as_ref() == b"unit" => break,
                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                    _ => {}
                }
                buf.clear();
            }

            Ok(UnitDefinition {
                name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
                eqn,
                aliases,
                disabled,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected unit element".to_string(),
        )),
    }
}
