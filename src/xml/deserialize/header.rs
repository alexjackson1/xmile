//! Header deserialization module.
//!
//! This module handles deserialization of XMILE file headers, including
//! vendor information, product details, contact information, and includes.

use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::BufRead;

use crate::header::{Contact, Header, Include, Includes, Product};
use crate::xml::deserialize::DeserializeError;
use crate::xml::deserialize::helpers::read_text_content;
use crate::xml::quick::de::skip_element;

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
    // TODO: options in Phase 2

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                let tag_name = e.name().as_ref().to_vec();
                match tag_name.as_slice() {
                    b"vendor" => {
                        vendor = Some(read_text_content(reader, buf)?);
                    }
                    b"product" => {
                        // Use Attrs helper for cleaner attribute parsing
                        use crate::xml::quick::de::Attrs;
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
                    _ => {
                        // Skip unknown elements properly using the helper
                        skip_element(reader, buf, &tag_name)?;
                    }
                }
            }
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
        options: None, // TODO: implement options deserialization
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
                    let mut resource: Option<String> = None;

                    // Read attributes
                    for attr in e.attributes() {
                        let attr = attr?;
                        if attr.key.as_ref() == b"resource" {
                            resource = Some(attr.decode_and_unescape_value(reader)?.to_string());
                        }
                    }

                    if let Some(resource) = resource {
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
                    let mut resource: Option<String> = None;

                    // Read attributes
                    for attr in e.attributes() {
                        let attr = attr?;
                        if attr.key.as_ref() == b"resource" {
                            resource = Some(attr.decode_and_unescape_value(reader)?.to_string());
                        }
                    }

                    if let Some(resource) = resource {
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
