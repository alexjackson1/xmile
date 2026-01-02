//! Header deserialization module.
//!
//! This module handles deserialization of XMILE file headers, including
//! vendor information, product details, contact information, and includes.

use std::io::BufRead;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::{
    header::{
        Contact, Header, Include, Includes, Options, Product, UsesArrays, UsesConveyor,
        UsesEventPosters, UsesInputs, UsesMacros, UsesOutputs, UsesQueue,
    },
    xml::{
        deserialize::{DeserializeError, helpers::read_text_content},
        quick::de::{Attrs, skip_element},
    },
};

/// Deserialize header from an already-read start tag.
/// This function is called when the start tag has already been consumed by the caller.
pub fn deserialize_header_from_start<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    _start_event: &quick_xml::events::BytesStart,
) -> Result<Header, DeserializeError> {
    deserialize_header_impl(reader, buf)
}

pub(crate) fn deserialize_header_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Header, DeserializeError> {
    buf.clear();

    let mut vendor: Option<String> = None;
    let mut product: Option<Product> = None;
    let mut name: Option<String> = None;
    let mut version_info: Option<String> = None;
    let mut caption: Option<String> = None;
    let mut image: Option<String> = None;
    let mut author: Option<String> = None;
    let mut affiliation: Option<String> = None;
    let mut client: Option<String> = None;
    let mut copyright: Option<String> = None;
    let mut contact: Option<Contact> = None;
    let mut created: Option<String> = None;
    let mut modified: Option<String> = None;
    let mut uuid: Option<String> = None;
    let mut includes: Option<Includes> = None;
    let mut options: Option<Options> = None;

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                let tag_name = e.name().as_ref().to_vec();
                match tag_name.as_slice() {
                    b"vendor" => {
                        vendor = Some(read_text_content(reader, buf)?);
                    }
                    b"product" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let version = attrs.get_req_string("version")?;
                        let lang = attrs.get_opt_string("lang");
                        let product_name = read_text_content(reader, buf)?;
                        product = Some(Product {
                            version,
                            lang,
                            name: product_name,
                        });
                    }
                    b"name" => {
                        name = Some(read_text_content(reader, buf)?);
                    }
                    b"version" => {
                        version_info = Some(read_text_content(reader, buf)?);
                    }
                    b"caption" => {
                        caption = Some(read_text_content(reader, buf)?);
                    }
                    b"image" => {
                        image = Some(read_text_content(reader, buf)?);
                    }
                    b"author" => {
                        author = Some(read_text_content(reader, buf)?);
                    }
                    b"affiliation" => {
                        affiliation = Some(read_text_content(reader, buf)?);
                    }
                    b"client" => {
                        client = Some(read_text_content(reader, buf)?);
                    }
                    b"copyright" => {
                        copyright = Some(read_text_content(reader, buf)?);
                    }
                    b"contact" => {
                        contact = Some(deserialize_contact(reader, buf)?);
                    }
                    b"created" => {
                        created = Some(read_text_content(reader, buf)?);
                    }
                    b"modified" => {
                        modified = Some(read_text_content(reader, buf)?);
                    }
                    b"uuid" => {
                        uuid = Some(read_text_content(reader, buf)?);
                    }
                    b"includes" => {
                        includes = Some(deserialize_includes(reader, buf)?);
                    }
                    b"options" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let namespace = attrs.get_opt_string("namespace");
                        buf.clear();
                        options = Some(deserialize_options(reader, buf, namespace)?);
                    }
                    _ => {
                        // Skip unknown elements properly using the helper
                        skip_element(reader, buf, &tag_name)?;
                    }
                }
            }
            Event::Empty(e) => match e.name().as_ref() {
                b"options" => {
                    let attrs = Attrs::from_start(&e, reader)?;
                    let namespace = attrs.get_opt_string("namespace");
                    buf.clear();
                    options = Some(deserialize_options(reader, buf, namespace)?);
                }
                _ => {}
            },
            Event::End(e) if e.name().as_ref() == b"header" => {
                break;
            }
            Event::Eof => {
                return Err(DeserializeError::UnexpectedEof);
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(Header {
        vendor: vendor
            .ok_or_else(|| DeserializeError::MissingField("header/vendor".to_string()))?,
        product: product
            .ok_or_else(|| DeserializeError::MissingField("header/product".to_string()))?,
        options,
        name,
        version_info,
        caption,
        image,
        author,
        affiliation,
        client,
        copyright,
        contact,
        created,
        modified,
        uuid,
        includes,
    })
}

/// Deserialize header from XML.
/// This function reads the header start tag itself.
pub fn deserialize_header<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Header, DeserializeError> {
    // Expect <header> start tag
    let event = reader.read_event_into(buf)?;
    match event {
        Event::Start(e) if e.name().as_ref() == b"header" => {
            // Clone the BytesStart to avoid lifetime issues
            let _e_name = e.name().as_ref().to_vec();
            buf.clear();
            // Reconstruct a minimal BytesStart for deserialize_header_from_start
            // Actually, we can just call deserialize_header_impl directly
            deserialize_header_impl(reader, buf)
        }
        Event::Start(e) => {
            return Err(DeserializeError::UnexpectedElement {
                expected: "header".to_string(),
                found: String::from_utf8_lossy(e.name().as_ref()).to_string(),
            });
        }
        _ => {
            return Err(DeserializeError::Custom(
                "Expected header start tag".to_string(),
            ));
        }
    }
}

/// Deserialize a Contact structure from XML.
fn deserialize_contact<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Contact, DeserializeError> {
    let mut address: Option<String> = None;
    let mut phone: Option<String> = None;
    let mut fax: Option<String> = None;
    let mut email: Option<String> = None;
    let mut website: Option<String> = None;

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => match e.name().as_ref() {
                b"address" => address = Some(read_text_content(reader, buf)?),
                b"phone" => phone = Some(read_text_content(reader, buf)?),
                b"fax" => fax = Some(read_text_content(reader, buf)?),
                b"email" => email = Some(read_text_content(reader, buf)?),
                b"website" => website = Some(read_text_content(reader, buf)?),
                _ => {}
            },
            Event::End(e) if e.name().as_ref() == b"contact" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(Contact {
        address,
        phone,
        fax,
        email,
        website,
    })
}

/// Deserialize an Includes structure from XML.
fn deserialize_includes<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Includes, DeserializeError> {
    let mut includes = Vec::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                if e.name().as_ref() == b"include" {
                    let attrs = Attrs::from_start(&e, reader)?;
                    if let Some(resource) = attrs.get_opt_string("resource") {
                        includes.push(Include { resource });
                    }

                    // Read until end tag
                    loop {
                        match reader.read_event_into(buf)? {
                            Event::End(e) if e.name().as_ref() == b"include" => break,
                            Event::Eof => return Err(DeserializeError::UnexpectedEof),
                            _ => {}
                        }
                        buf.clear();
                    }
                }
            }
            Event::Empty(e) => {
                if e.name().as_ref() == b"include" {
                    let attrs = Attrs::from_start(&e, reader)?;
                    if let Some(resource) = attrs.get_opt_string("resource") {
                        includes.push(Include { resource });
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"includes" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(Includes { includes })
}

/// Deserialize Options from XML.
/// The start tag has already been read and attributes extracted.
fn deserialize_options<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    namespace: Option<String>,
) -> Result<Options, DeserializeError> {
    let mut uses_conveyor: Option<UsesConveyor> = None;
    let mut uses_queue: Option<UsesQueue> = None;
    let mut uses_arrays: Option<UsesArrays> = None;
    let mut uses_submodels: Option<bool> = None;
    let mut uses_macros: Option<UsesMacros> = None;
    let mut uses_event_posters: Option<UsesEventPosters> = None;
    let mut has_model_view: Option<bool> = None;
    let mut uses_outputs: Option<UsesOutputs> = None;
    let mut uses_inputs: Option<UsesInputs> = None;
    let mut uses_annotation: Option<bool> = None;

    // Process child elements
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                let element_name = e.name().as_ref().to_vec();
                let element_attrs = Attrs::from_start(&e, reader)?;

                match element_name.as_slice() {
                    b"uses_conveyor" => {
                        let arrest = element_attrs.get_opt_bool("arrest")?;
                        let leak = element_attrs.get_opt_bool("leak")?;
                        uses_conveyor = Some(UsesConveyor { arrest, leak });
                        skip_element(reader, buf, b"uses_conveyor")?;
                    }
                    b"uses_queue" => {
                        let overflow = element_attrs.get_opt_bool("overflow")?;
                        uses_queue = Some(UsesQueue { overflow });
                        skip_element(reader, buf, b"uses_queue")?;
                    }
                    b"uses_arrays" => {
                        let maximum_dimensions =
                            element_attrs.get_req_u32("maximum_dimensions")? as usize;
                        let invalid_index_value =
                            element_attrs.get_opt_string("invalid_index_value");
                        uses_arrays = Some(UsesArrays {
                            maximum_dimensions,
                            invalid_index_value,
                        });
                        skip_element(reader, buf, b"uses_arrays")?;
                    }
                    b"uses_submodels" => {
                        uses_submodels = Some(true);
                        skip_element(reader, buf, b"uses_submodels")?;
                    }
                    b"uses_macros" => {
                        let recursive_macros = element_attrs
                            .get_opt_bool("recursive_macros")?
                            .ok_or_else(|| {
                                DeserializeError::MissingField(
                                    "uses_macros@recursive_macros".to_string(),
                                )
                            })?;
                        let option_filters = element_attrs
                            .get_opt_bool("option_filters")?
                            .ok_or_else(|| {
                                DeserializeError::MissingField(
                                    "uses_macros@option_filters".to_string(),
                                )
                            })?;
                        uses_macros = Some(UsesMacros {
                            recursive_macros,
                            option_filters,
                        });
                        skip_element(reader, buf, b"uses_macros")?;
                    }
                    b"uses_event_posters" => {
                        let messages = element_attrs.get_opt_bool("messages")?;
                        uses_event_posters = Some(UsesEventPosters { messages });
                        skip_element(reader, buf, b"uses_event_posters")?;
                    }
                    b"has_model_view" => {
                        has_model_view = Some(true);
                        skip_element(reader, buf, b"has_model_view")?;
                    }
                    b"uses_outputs" => {
                        let numeric_display = element_attrs.get_opt_bool("numeric_display")?;
                        let lamp = element_attrs.get_opt_bool("lamp")?;
                        let gauge = element_attrs.get_opt_bool("gauge")?;
                        uses_outputs = Some(UsesOutputs {
                            numeric_display,
                            lamp,
                            gauge,
                        });
                        skip_element(reader, buf, b"uses_outputs")?;
                    }
                    b"uses_inputs" => {
                        let numeric_input = element_attrs.get_opt_bool("numeric_input")?;
                        let list = element_attrs.get_opt_bool("list")?;
                        let graphical_input = element_attrs.get_opt_bool("graphical_input")?;
                        uses_inputs = Some(UsesInputs {
                            numeric_input,
                            list,
                            graphical_input,
                        });
                        skip_element(reader, buf, b"uses_inputs")?;
                    }
                    b"uses_annotation" => {
                        uses_annotation = Some(true);
                        skip_element(reader, buf, b"uses_annotation")?;
                    }
                    _ => {
                        // Unknown option element, skip it
                        skip_element(reader, buf, &element_name)?;
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"options" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(Options {
        namespace,
        uses_conveyor,
        uses_queue,
        uses_arrays,
        uses_submodels,
        uses_macros,
        uses_event_posters,
        has_model_view,
        uses_outputs,
        uses_inputs,
        uses_annotation,
    })
}
