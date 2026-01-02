//! File-level dimensions deserialization module.
//!
//! This module handles deserialization of file-level dimension definitions,
//! which are used for array variables.

use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::BufRead;

use crate::dimensions::{Dimension as FileDimension, DimensionElement, Dimensions};
use crate::xml::deserialize::DeserializeError;

/// Deserialize file-level Dimensions from XML.
pub fn deserialize_file_dimensions<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Dimensions, DeserializeError> {
    let event = reader.read_event_into(buf)?;
    match event {
        Event::Start(e) if e.name().as_ref() == b"dimensions" => {
            buf.clear();
            deserialize_file_dimensions_impl(reader, buf)
        }
        _ => Err(DeserializeError::Custom(
            "Expected dimensions element".to_string(),
        )),
    }
}

/// Internal implementation of file dimensions deserialization.
pub(crate) fn deserialize_file_dimensions_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Dimensions, DeserializeError> {
    let mut dims = Vec::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) if e.name().as_ref() == b"dim" => {
                dims.push(deserialize_dimension(reader, buf)?);
            }
            Event::End(e) if e.name().as_ref() == b"dimensions" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(Dimensions { dims })
}

/// Deserialize a file-level Dimension from XML.
fn deserialize_dimension<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<FileDimension, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"dim" => {
            let mut name: Option<String> = None;
            let mut size: Option<usize> = None;

            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"name" => {
                        name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"size" => {
                        let size_str = attr.decode_and_unescape_value(reader)?.to_string();
                        size = Some(size_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid size value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

            let mut elements = Vec::new();

            // If it's a start tag (not empty), read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"elem" => {
                            let mut elem_name: Option<String> = None;
                            for attr in e.attributes() {
                                let attr = attr?;
                                if attr.key.as_ref() == b"name" {
                                    elem_name =
                                        Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                            }
                            if let Some(name) = elem_name {
                                elements.push(DimensionElement { name });
                            }

                            // If it's a start tag, read until end
                            if matches!(reader.read_event_into(buf)?, Event::Start(_)) {
                                loop {
                                    match reader.read_event_into(buf)? {
                                        Event::End(e) if e.name().as_ref() == b"elem" => break,
                                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                        _ => {}
                                    }
                                    buf.clear();
                                }
                            }
                        }
                        Event::End(e) if e.name().as_ref() == b"dim" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(FileDimension {
                name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
                size,
                elements,
            })
        }
        _ => Err(DeserializeError::Custom("Expected dim element".to_string())),
    }
}
