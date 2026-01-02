//! Simulation specs deserialization module.
//!
//! This module handles deserialization of simulation specifications,
//! including start/stop times, time step, method, and other simulation parameters.

use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::BufRead;

use crate::specs::SimulationSpecs;
use crate::xml::deserialize::DeserializeError;
use crate::xml::deserialize::helpers::read_number_content;

/// Deserialize a SimulationSpecs structure from XML.
///
/// This function expects the reader to be positioned at the start of a <sim_specs> element.
pub fn deserialize_sim_specs<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<SimulationSpecs, DeserializeError> {
    // Expect <sim_specs> start tag
    let event = reader.read_event_into(buf)?;
    let mut method: Option<String> = None;
    let mut time_units: Option<String> = None;

    match event {
        Event::Start(e) if e.name().as_ref() == b"sim_specs" => {
            // Read attributes from sim_specs element
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"method" => {
                        method = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"time_units" => {
                        time_units = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }
        }
        Event::Start(e) => {
            return Err(DeserializeError::UnexpectedElement {
                expected: "sim_specs".to_string(),
                found: String::from_utf8_lossy(e.name().as_ref()).to_string(),
            });
        }
        _ => {
            return Err(DeserializeError::Custom(
                "Expected sim_specs start tag".to_string(),
            ));
        }
    }
    buf.clear();
    deserialize_sim_specs_impl(reader, buf, method, time_units)
}

/// Internal implementation of sim_specs deserialization.
pub(crate) fn deserialize_sim_specs_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    method: Option<String>,
    time_units: Option<String>,
) -> Result<SimulationSpecs, DeserializeError> {
    let mut start: Option<f64> = None;
    let mut stop: Option<f64> = None;
    let mut dt: Option<f64> = None;
    let mut pause: Option<f64> = None;
    let mut run_by: Option<String> = None;

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"start" => {
                        start = Some(read_number_content(reader, buf)?);
                    }
                    b"stop" => {
                        stop = Some(read_number_content(reader, buf)?);
                    }
                    b"dt" => {
                        dt = Some(read_number_content(reader, buf)?);
                    }
                    b"pause" => {
                        pause = Some(read_number_content(reader, buf)?);
                    }
                    b"run" => {
                        // Check for "by" attribute
                        for attr in e.attributes() {
                            let attr = attr?;
                            if attr.key.as_ref() == b"by" {
                                run_by = Some(attr.decode_and_unescape_value(reader)?.to_string());
                            }
                        }
                        // May also have text content, but typically it's just the attribute
                        // Read until end tag
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(e) if e.name().as_ref() == b"run" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                    }
                    _ => {}
                }
            }
            Event::End(e) if e.name().as_ref() == b"sim_specs" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(SimulationSpecs {
        start: start.ok_or_else(|| DeserializeError::MissingField("start".to_string()))?,
        stop: stop.ok_or_else(|| DeserializeError::MissingField("stop".to_string()))?,
        dt,
        method,
        time_units,
        pause,
        run_by,
    })
}
