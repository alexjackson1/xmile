//! Behavior deserialization module.
//!
//! This module handles deserialization of behavior definitions,
//! including global and entity-specific behavior settings.

use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::BufRead;

use crate::behavior::{Behavior, EntityBehavior, EntityBehaviorEntry};
use crate::xml::deserialize::DeserializeError;

/// Deserialize Behavior from XML.
pub fn deserialize_behavior<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Behavior, DeserializeError> {
    let event = reader.read_event_into(buf)?;
    let is_empty_tag = matches!(event, Event::Empty(_));

    match event {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"behavior" => {
            buf.clear();
            deserialize_behavior_impl(reader, buf, is_empty_tag)
        }
        _ => Err(DeserializeError::Custom(
            "Expected behavior element".to_string(),
        )),
    }
}

/// Internal implementation of behavior deserialization.
pub(crate) fn deserialize_behavior_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    is_empty_tag: bool,
) -> Result<Behavior, DeserializeError> {
    let mut global = EntityBehavior::default();
    let mut entities = Vec::new();

    if !is_empty_tag {
        loop {
            match reader.read_event_into(buf)? {
                Event::Start(e) => {
                    match e.name().as_ref() {
                        b"non_negative" => {
                            // Global non_negative
                            global.non_negative = Some(true);
                            // Skip to end of element
                            loop {
                                match reader.read_event_into(buf)? {
                                    Event::End(e) if e.name().as_ref() == b"non_negative" => break,
                                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                    _ => {}
                                }
                                buf.clear();
                            }
                        }
                        b"stock" | b"flow" | b"aux" | b"gf" => {
                            let entity_type =
                                String::from_utf8_lossy(e.name().as_ref()).to_string();
                            let mut entity_behavior = EntityBehavior::default();

                            loop {
                                match reader.read_event_into(buf)? {
                                    Event::Start(inner)
                                        if inner.name().as_ref() == b"non_negative" =>
                                    {
                                        entity_behavior.non_negative = Some(true);
                                        // Read to end of non_negative
                                        loop {
                                            match reader.read_event_into(buf)? {
                                                Event::End(end)
                                                    if end.name().as_ref() == b"non_negative" =>
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
                                    Event::Empty(inner)
                                        if inner.name().as_ref() == b"non_negative" =>
                                    {
                                        entity_behavior.non_negative = Some(true);
                                        // Empty tag - nothing more to read
                                    }
                                    Event::End(end)
                                        if end.name().as_ref() == entity_type.as_bytes() =>
                                    {
                                        break;
                                    }
                                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                    _ => {}
                                }
                                buf.clear();
                            }

                            entities.push(EntityBehaviorEntry {
                                entity_type,
                                behavior: entity_behavior,
                            });
                        }
                        _ => {}
                    }
                }
                Event::Empty(e) => {
                    match e.name().as_ref() {
                        b"non_negative" => {
                            // Global non_negative
                            global.non_negative = Some(true);
                        }
                        _ => {}
                    }
                }
                Event::End(e) if e.name().as_ref() == b"behavior" => break,
                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                _ => {}
            }
            buf.clear();
        }
    }
    buf.clear();

    Ok(Behavior { global, entities })
}

/// Internal implementation of behavior deserialization with first element already read.
#[allow(dead_code)]
pub(crate) fn deserialize_behavior_impl_with_first_element<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    element_name: Vec<u8>,
    is_first_element_empty: bool,
) -> Result<Behavior, DeserializeError> {
    let mut global = EntityBehavior::default();
    let mut entities = Vec::new();

    // Process the first element we already read
    match element_name.as_slice() {
        b"non_negative" => {
            global.non_negative = Some(true);
            // Only read to end tag if it wasn't an empty element
            if !is_first_element_empty {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"non_negative" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
        }
        b"stock" | b"flow" | b"aux" | b"gf" => {
            let entity_type = String::from_utf8_lossy(element_name.as_slice());
            let mut entity_behavior = EntityBehavior::default();

            if !is_first_element_empty {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(inner) if inner.name().as_ref() == b"non_negative" => {
                            entity_behavior.non_negative = Some(true);
                            loop {
                                match reader.read_event_into(buf)? {
                                    Event::End(end) if end.name().as_ref() == b"non_negative" => {
                                        break;
                                    }
                                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                    _ => {}
                                }
                                buf.clear();
                            }
                        }
                        Event::Empty(inner) if inner.name().as_ref() == b"non_negative" => {
                            entity_behavior.non_negative = Some(true);
                        }
                        Event::End(end) if end.name().as_ref() == element_name.as_slice() => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }

            entities.push(EntityBehaviorEntry {
                entity_type: entity_type.to_string(),
                behavior: entity_behavior,
            });
        }
        _ => {}
    }
    buf.clear();

    // Continue processing remaining elements
    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => match e.name().as_ref() {
                b"non_negative" => {
                    global.non_negative = Some(true);
                    loop {
                        match reader.read_event_into(buf)? {
                            Event::End(e) if e.name().as_ref() == b"non_negative" => break,
                            Event::Eof => return Err(DeserializeError::UnexpectedEof),
                            _ => {}
                        }
                        buf.clear();
                    }
                }
                b"stock" | b"flow" | b"aux" | b"gf" => {
                    let entity_type = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut entity_behavior = EntityBehavior::default();

                    loop {
                        match reader.read_event_into(buf)? {
                            Event::Start(inner) if inner.name().as_ref() == b"non_negative" => {
                                entity_behavior.non_negative = Some(true);
                                loop {
                                    match reader.read_event_into(buf)? {
                                        Event::End(end)
                                            if end.name().as_ref() == b"non_negative" =>
                                        {
                                            break;
                                        }
                                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                        _ => {}
                                    }
                                    buf.clear();
                                }
                            }
                            Event::Empty(inner) if inner.name().as_ref() == b"non_negative" => {
                                entity_behavior.non_negative = Some(true);
                            }
                            Event::End(end) if end.name().as_ref() == entity_type.as_bytes() => {
                                break;
                            }
                            Event::Eof => return Err(DeserializeError::UnexpectedEof),
                            _ => {}
                        }
                        buf.clear();
                    }

                    entities.push(EntityBehaviorEntry {
                        entity_type,
                        behavior: entity_behavior,
                    });
                }
                _ => {}
            },
            Event::Empty(e) => match e.name().as_ref() {
                b"non_negative" => {
                    global.non_negative = Some(true);
                }
                _ => {}
            },
            Event::End(e) if e.name().as_ref() == b"behavior" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(Behavior { global, entities })
}
