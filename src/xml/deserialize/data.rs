//! Data deserialization module.
//!
//! This module handles deserialization of data import and export definitions.

use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::BufRead;

use crate::data::{Data, DataExport, DataImport, TableExport};
use crate::xml::deserialize::DeserializeError;

/// Deserialize Data from XML.
pub fn deserialize_data<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Data, DeserializeError> {
    let event = reader.read_event_into(buf)?;
    let is_empty_tag = matches!(event, Event::Empty(_));

    match event {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"data" => {
            buf.clear();
            deserialize_data_impl(reader, buf, is_empty_tag)
        }
        _ => Err(DeserializeError::Custom(
            "Expected data element".to_string(),
        )),
    }
}

/// Internal implementation of data deserialization.
pub(crate) fn deserialize_data_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    is_empty_tag: bool,
) -> Result<Data, DeserializeError> {
    let mut imports = Vec::new();
    let mut exports = Vec::new();

    if !is_empty_tag {
        loop {
            let event = reader.read_event_into(buf)?;
            match event {
                Event::Start(e) if e.name().as_ref() == b"import" => {
                    // Extract attributes before clearing buf
                    let mut attrs = Vec::new();
                    for attr in e.attributes() {
                        let attr = attr?;
                        let key = attr.key.as_ref().to_vec();
                        let value = attr.decode_and_unescape_value(reader)?.to_string();
                        attrs.push((key, value));
                    }
                    buf.clear();
                    imports.push(deserialize_data_import_from_attrs(reader, buf, &attrs)?);
                }
                Event::Empty(e) if e.name().as_ref() == b"import" => {
                    // Extract attributes before clearing buf
                    let mut attrs = Vec::new();
                    for attr in e.attributes() {
                        let attr = attr?;
                        let key = attr.key.as_ref().to_vec();
                        let value = attr.decode_and_unescape_value(reader)?.to_string();
                        attrs.push((key, value));
                    }
                    buf.clear();
                    imports.push(deserialize_data_import_from_attrs(reader, buf, &attrs)?);
                }
                Event::Start(e) if e.name().as_ref() == b"export" => {
                    // Extract attributes before clearing buf
                    let mut attrs = Vec::new();
                    for attr in e.attributes() {
                        let attr = attr?;
                        let key = attr.key.as_ref().to_vec();
                        let value = attr.decode_and_unescape_value(reader)?.to_string();
                        attrs.push((key, value));
                    }
                    buf.clear();
                    exports.push(deserialize_data_export_from_attrs(
                        reader, buf, &attrs, false,
                    )?);
                }
                Event::Empty(e) if e.name().as_ref() == b"export" => {
                    // Extract attributes before clearing buf
                    let mut attrs = Vec::new();
                    for attr in e.attributes() {
                        let attr = attr?;
                        let key = attr.key.as_ref().to_vec();
                        let value = attr.decode_and_unescape_value(reader)?.to_string();
                        attrs.push((key, value));
                    }
                    buf.clear();
                    exports.push(deserialize_data_export_from_attrs(
                        reader, buf, &attrs, true,
                    )?);
                }
                Event::End(e) if e.name().as_ref() == b"data" => break,
                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                _ => {}
            }
            buf.clear();
        }
    }
    buf.clear();

    Ok(Data { imports, exports })
}

/// Internal implementation of data deserialization with first element already read.
#[allow(dead_code)]
pub(crate) fn deserialize_data_impl_with_first_element<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    element_name: Vec<u8>,
    element_attrs: Vec<(Vec<u8>, String)>,
    is_empty_element: bool,
) -> Result<Data, DeserializeError> {
    let mut imports = Vec::new();
    let mut exports = Vec::new();

    // Process the first element we already read
    match element_name.as_slice() {
        b"import" => {
            imports.push(deserialize_data_import_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?);
        }
        b"export" => {
            exports.push(deserialize_data_export_from_attrs(
                reader,
                buf,
                &element_attrs,
                is_empty_element,
            )?);
        }
        _ => {}
    }
    buf.clear();

    // Continue processing remaining events
    loop {
        let event = reader.read_event_into(buf)?;
        match event {
            Event::Start(e) if e.name().as_ref() == b"import" => {
                // Extract attributes before clearing buf
                let mut attrs = Vec::new();
                for attr in e.attributes() {
                    let attr = attr?;
                    let key = attr.key.as_ref().to_vec();
                    let value = attr.decode_and_unescape_value(reader)?.to_string();
                    attrs.push((key, value));
                }
                buf.clear();
                imports.push(deserialize_data_import_from_attrs(reader, buf, &attrs)?);
            }
            Event::Empty(e) if e.name().as_ref() == b"import" => {
                // Extract attributes before clearing buf
                let mut attrs = Vec::new();
                for attr in e.attributes() {
                    let attr = attr?;
                    let key = attr.key.as_ref().to_vec();
                    let value = attr.decode_and_unescape_value(reader)?.to_string();
                    attrs.push((key, value));
                }
                buf.clear();
                imports.push(deserialize_data_import_from_attrs(reader, buf, &attrs)?);
            }
            Event::Start(e) if e.name().as_ref() == b"export" => {
                // Extract attributes before clearing buf
                let mut attrs = Vec::new();
                for attr in e.attributes() {
                    let attr = attr?;
                    let key = attr.key.as_ref().to_vec();
                    let value = attr.decode_and_unescape_value(reader)?.to_string();
                    attrs.push((key, value));
                }
                buf.clear();
                exports.push(deserialize_data_export_from_attrs(
                    reader, buf, &attrs, false,
                )?);
            }
            Event::Empty(e) if e.name().as_ref() == b"export" => {
                // Extract attributes before clearing buf
                let mut attrs = Vec::new();
                for attr in e.attributes() {
                    let attr = attr?;
                    let key = attr.key.as_ref().to_vec();
                    let value = attr.decode_and_unescape_value(reader)?.to_string();
                    attrs.push((key, value));
                }
                buf.clear();
                exports.push(deserialize_data_export_from_attrs(
                    reader, buf, &attrs, true,
                )?);
            }
            Event::End(e) if e.name().as_ref() == b"data" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(Data { imports, exports })
}

/// Deserialize a DataImport from XML using pre-extracted attributes.
fn deserialize_data_import_from_attrs<R: BufRead>(
    _reader: &mut Reader<R>,
    _buf: &mut Vec<u8>,
    attrs: &[(Vec<u8>, String)],
) -> Result<DataImport, DeserializeError> {
    let mut import = DataImport {
        data_type: None,
        enabled: None,
        frequency: None,
        orientation: None,
        resource: None,
        worksheet: None,
    };

    // Process attributes
    for (key, value) in attrs {
        match key.as_slice() {
            b"type" => import.data_type = Some(value.clone()),
            b"enabled" => {
                import.enabled = Some(
                    value
                        .parse::<bool>()
                        .map_err(|e| DeserializeError::Custom(format!("Invalid enabled: {}", e)))?,
                )
            }
            b"frequency" => import.frequency = Some(value.clone()),
            b"orientation" => import.orientation = Some(value.clone()),
            b"resource" => import.resource = Some(value.clone()),
            b"worksheet" => import.worksheet = Some(value.clone()),
            _ => {}
        }
    }

    Ok(import)
}

/// Deserialize a DataExport from XML using pre-extracted attributes.
fn deserialize_data_export_from_attrs<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    attrs: &[(Vec<u8>, String)],
    is_empty_tag: bool,
) -> Result<DataExport, DeserializeError> {
    let mut export = DataExport {
        data_type: None,
        enabled: None,
        frequency: None,
        orientation: None,
        resource: None,
        worksheet: None,
        interval: None,
        export_all: None,
        table_uid: None,
    };

    // Process attributes
    for (key, value) in attrs {
        match key.as_slice() {
            b"type" => export.data_type = Some(value.clone()),
            b"enabled" => {
                export.enabled = Some(
                    value
                        .parse::<bool>()
                        .map_err(|e| DeserializeError::Custom(format!("Invalid enabled: {}", e)))?,
                )
            }
            b"frequency" => export.frequency = Some(value.clone()),
            b"orientation" => export.orientation = Some(value.clone()),
            b"resource" => export.resource = Some(value.clone()),
            b"worksheet" => export.worksheet = Some(value.clone()),
            b"interval" => export.interval = Some(value.clone()),
            _ => {}
        }
    }

    // Read child elements (all or table)
    if !is_empty_tag {
        buf.clear();
        let next_event = reader.read_event_into(buf)?;

        match next_event {
            Event::End(_) => {
                // It was an empty tag, we're done
            }
            Event::Start(e) => {
                // It has content, process it
                match e.name().as_ref() {
                    b"all" => {
                        export.export_all = Some(());
                        // Skip to end of all element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(e) if e.name().as_ref() == b"all" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                    }
                    b"table" => {
                        let mut uid = String::new();
                        let mut use_settings = None;

                        for attr in e.attributes() {
                            let attr = attr?;
                            let value = attr.decode_and_unescape_value(reader)?.to_string();
                            match attr.key.as_ref() {
                                b"uid" => uid = value,
                                b"use_settings" => {
                                    use_settings = Some(value.parse::<bool>().map_err(|e| {
                                        DeserializeError::Custom(format!(
                                            "Invalid use_settings: {}",
                                            e
                                        ))
                                    })?)
                                }
                                _ => {}
                            }
                        }

                        if uid.is_empty() {
                            return Err(DeserializeError::MissingField("table.uid".to_string()));
                        }

                        export.table_uid = Some(TableExport { uid, use_settings });

                        // Skip to end of table element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(e) if e.name().as_ref() == b"table" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                    }
                    _ => {
                        return Err(DeserializeError::Custom(
                            "Unexpected element in export".to_string(),
                        ));
                    }
                }
            }
            Event::Empty(e) => {
                // Empty child element
                match e.name().as_ref() {
                    b"all" => {
                        export.export_all = Some(());
                    }
                    b"table" => {
                        let mut uid = String::new();
                        let mut use_settings = None;

                        for attr in e.attributes() {
                            let attr = attr?;
                            let value = attr.decode_and_unescape_value(reader)?.to_string();
                            match attr.key.as_ref() {
                                b"uid" => uid = value,
                                b"use_settings" => {
                                    use_settings = Some(value.parse::<bool>().map_err(|e| {
                                        DeserializeError::Custom(format!(
                                            "Invalid use_settings: {}",
                                            e
                                        ))
                                    })?)
                                }
                                _ => {}
                            }
                        }

                        if uid.is_empty() {
                            return Err(DeserializeError::MissingField("table.uid".to_string()));
                        }

                        export.table_uid = Some(TableExport { uid, use_settings });
                    }
                    _ => {
                        return Err(DeserializeError::Custom(
                            "Unexpected element in export".to_string(),
                        ));
                    }
                }
            }
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {
                return Err(DeserializeError::Custom(
                    "Unexpected event in export".to_string(),
                ));
            }
        }
    }

    Ok(export)
}
