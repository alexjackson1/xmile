//! Macro deserialization module.
//!
//! This module handles deserialization of macro definitions (feature-gated).

#[cfg(feature = "macros")]
use quick_xml::Reader;
#[cfg(feature = "macros")]
use quick_xml::events::Event;
#[cfg(feature = "macros")]
use std::io::BufRead;

#[cfg(feature = "macros")]
use crate::Expression;
#[cfg(feature = "macros")]
use crate::equation::Identifier;
#[cfg(feature = "macros")]
use crate::r#macro::{Macro, MacroParameter};
#[cfg(feature = "macros")]
use crate::model::object::Documentation;
#[cfg(feature = "macros")]
use crate::xml::deserialize::DeserializeError;
#[cfg(feature = "macros")]
use crate::xml::deserialize::helpers::read_text_content;

/// Deserialize a Macro from XML.
#[cfg(feature = "macros")]
pub fn deserialize_macro<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Macro, DeserializeError> {
    let event = reader.read_event_into(buf)?;
    let is_empty_tag = matches!(event, Event::Empty(_));

    match event {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"macro" => {
            let mut name: Option<Identifier> = None;
            let mut namespace: Option<String> = None;

            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"name" => {
                        let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                        name = Some(Identifier::parse_from_attribute(&name_str).map_err(|e| {
                            DeserializeError::Custom(format!("Invalid macro name: {}", e))
                        })?);
                    }
                    b"namespace" => {
                        namespace = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }

            let mut parameters: Vec<MacroParameter> = Vec::new();
            let mut eqn: Option<Expression> = None;
            let mut format: Option<String> = None;
            let mut doc: Option<Documentation> = None;
            let mut sim_specs: Option<crate::specs::SimulationSpecs> = None;
            let mut variables: Option<Vec<crate::model::vars::Variable>> = None;
            let mut views: Option<crate::view::View> = None;

            // If it's a start tag, read child elements
            if !is_empty_tag {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"parm" => {
                            let mut param_default: Option<Expression> = None;

                            // Read attributes
                            for attr in e.attributes() {
                                let attr = attr?;
                                if attr.key.as_ref() == b"default" {
                                    let default_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    use crate::equation::parse::expression;
                                    let (remaining, expr) =
                                        expression(&default_str).map_err(|err| {
                                            DeserializeError::Custom(format!(
                                                "Invalid default expression: {}",
                                                err
                                            ))
                                        })?;
                                    if !remaining.is_empty() {
                                        return Err(DeserializeError::Custom(format!(
                                            "Unexpected trailing characters after default expression: '{}'",
                                            remaining
                                        )));
                                    }
                                    param_default = Some(expr);
                                }
                            }

                            // Read parameter name from text content
                            let parm_name_str = read_text_content(reader, buf)?;
                            let parm_name = Identifier::parse_from_attribute(&parm_name_str)
                                .map_err(|err| {
                                    DeserializeError::Custom(format!(
                                        "Invalid parameter name: {}",
                                        err
                                    ))
                                })?;
                            parameters.push(MacroParameter {
                                name: parm_name,
                                default: param_default,
                            });
                        }
                        Event::Start(e) if e.name().as_ref() == b"eqn" => {
                            use crate::xml::deserialize::read_expression;
                            eqn = Some(read_expression(reader, buf)?);
                        }
                        Event::Start(e) if e.name().as_ref() == b"format" => {
                            format = Some(read_text_content(reader, buf)?);
                        }
                        Event::Start(e) if e.name().as_ref() == b"doc" => {
                            let doc_text = read_text_content(reader, buf)?;
                            doc = Some(
                                if doc_text.trim().contains('<') && doc_text.trim().contains('>') {
                                    Documentation::Html(doc_text)
                                } else {
                                    Documentation::PlainText(doc_text)
                                },
                            );
                        }
                        Event::Start(e) if e.name().as_ref() == b"sim_specs" => {
                            use crate::xml::deserialize::specs::deserialize_sim_specs_impl;
                            let mut method: Option<String> = None;
                            let mut time_units: Option<String> = None;
                            for attr in e.attributes() {
                                let attr = attr?;
                                match attr.key.as_ref() {
                                    b"method" => {
                                        method = Some(
                                            attr.decode_and_unescape_value(reader)?.to_string(),
                                        );
                                    }
                                    b"time_units" => {
                                        time_units = Some(
                                            attr.decode_and_unescape_value(reader)?.to_string(),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                            buf.clear();
                            sim_specs =
                                Some(deserialize_sim_specs_impl(reader, buf, method, time_units)?);
                        }
                        Event::Start(e) if e.name().as_ref() == b"variables" => {
                            use crate::xml::deserialize::variables::deserialize_variables_impl;
                            buf.clear();
                            let vars = deserialize_variables_impl(reader, buf)?;
                            variables = Some(vars.variables);
                        }
                        Event::Start(e) if e.name().as_ref() == b"view" => {
                            use crate::xml::deserialize::views::deserialize_view_impl;
                            let attrs: Vec<_> = e
                                .attributes()
                                .filter_map(|a| a.ok())
                                .map(|a| {
                                    (
                                        a.key.as_ref().to_vec(),
                                        String::from_utf8_lossy(&a.value).to_string(),
                                    )
                                })
                                .collect();
                            buf.clear();
                            views = Some(deserialize_view_impl(reader, buf, attrs)?);
                        }
                        Event::End(e) if e.name().as_ref() == b"macro" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }

            let name_id =
                name.ok_or_else(|| DeserializeError::MissingField("macro name".to_string()))?;

            // Parse namespace string into Vec<Namespace>
            let namespace_vec =
                namespace.map(|ns_str| crate::namespace::Namespace::from_str(&ns_str));

            Ok(Macro {
                name: name_id,
                eqn: eqn.ok_or_else(|| DeserializeError::MissingField("macro eqn".to_string()))?,
                parameters,
                format,
                doc,
                sim_specs,
                namespace: namespace_vec,
                variables,
                views,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected macro element".to_string(),
        )),
    }
}

/// Deserialize a Macro from XML.
/// Used when the start tag has already been consumed.
#[cfg(feature = "macros")]
pub(crate) fn deserialize_macro_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<String>,
    _namespace: Option<String>,
) -> Result<Macro, DeserializeError> {
    let name = name.ok_or_else(|| DeserializeError::MissingField("macro name".to_string()))?;
    let name_id = Identifier::parse_from_attribute(&name)
        .map_err(|e| DeserializeError::Custom(format!("Invalid macro name: {}", e)))?;

    let mut eqn: Option<Expression> = None;
    let mut parameters: Vec<MacroParameter> = Vec::new();
    let mut doc: Option<Documentation> = None;

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"eqn" => {
                        use crate::xml::deserialize::read_expression;
                        eqn = Some(read_expression(reader, buf)?);
                    }
                    b"parm" => {
                        // Extract default attribute
                        let mut default: Option<Expression> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            if attr.key.as_ref() == b"default" {
                                let default_str =
                                    attr.decode_and_unescape_value(reader)?.to_string();
                                use crate::equation::parse::expression;
                                let (remaining, expr) =
                                    expression(&default_str).map_err(|err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid default expression: {}",
                                            err
                                        ))
                                    })?;
                                if !remaining.is_empty() {
                                    return Err(DeserializeError::Custom(format!(
                                        "Unexpected trailing characters after default expression: '{}'",
                                        remaining
                                    )));
                                }
                                default = Some(expr);
                            }
                        }
                        // Read parameter name from text content
                        let parm_name_str = read_text_content(reader, buf)?;
                        let parm_name =
                            Identifier::parse_from_attribute(&parm_name_str).map_err(|err| {
                                DeserializeError::Custom(format!("Invalid parameter name: {}", err))
                            })?;
                        parameters.push(MacroParameter {
                            name: parm_name,
                            default,
                        });
                    }
                    b"doc" => {
                        let doc_text = read_text_content(reader, buf)?;
                        doc = Some(
                            if doc_text.trim().contains('<') && doc_text.trim().contains('>') {
                                Documentation::Html(doc_text)
                            } else {
                                Documentation::PlainText(doc_text)
                            },
                        );
                    }
                    _ => {
                        // Skip unknown elements
                        let element_name = e.name().as_ref().to_vec();
                        use crate::xml::quick::de::skip_element;
                        skip_element(reader, buf, &element_name)?;
                    }
                }
            }
            Event::Empty(_) => {
                // Ignore empty elements
            }
            Event::End(e) if e.name().as_ref() == b"macro" => {
                break;
            }
            Event::Eof => {
                return Err(DeserializeError::UnexpectedEof);
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(Macro {
        name: name_id,
        eqn: eqn.ok_or_else(|| DeserializeError::MissingField("macro eqn".to_string()))?,
        parameters,
        format: None,
        doc,
        sim_specs: None,
        namespace: None,
        variables: None,
        views: None,
    })
}
