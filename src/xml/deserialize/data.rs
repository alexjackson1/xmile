//! Data deserialization module.
//!
//! This module handles deserialization of data import and export definitions.

use std::io::BufRead;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::{
    data::{Data, DataExport, DataImport, TableExport},
    xml::{deserialize::DeserializeError, quick::de::Attrs},
};

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
                    let attrs = Attrs::from_start(&e, reader)?;
                    buf.clear();
                    imports.push(deserialize_data_import_from_attrs(reader, buf, &attrs)?);
                }
                Event::Empty(e) if e.name().as_ref() == b"import" => {
                    let attrs = Attrs::from_start(&e, reader)?;
                    buf.clear();
                    imports.push(deserialize_data_import_from_attrs(reader, buf, &attrs)?);
                }
                Event::Start(e) if e.name().as_ref() == b"export" => {
                    let attrs = Attrs::from_start(&e, reader)?;
                    buf.clear();
                    exports.push(deserialize_data_export_from_attrs(
                        reader, buf, &attrs, false,
                    )?);
                }
                Event::Empty(e) if e.name().as_ref() == b"export" => {
                    let attrs = Attrs::from_start(&e, reader)?;
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

/// Deserialize a DataImport from XML using pre-extracted attributes.
fn deserialize_data_import_from_attrs<R: BufRead>(
    _reader: &mut Reader<R>,
    _buf: &mut Vec<u8>,
    attrs: &Attrs,
) -> Result<DataImport, DeserializeError> {
    Ok(DataImport {
        data_type: attrs.get_opt_string("type"),
        enabled: attrs.get_opt_bool("enabled")?,
        frequency: attrs.get_opt_string("frequency"),
        orientation: attrs.get_opt_string("orientation"),
        resource: attrs.get_opt_string("resource"),
        worksheet: attrs.get_opt_string("worksheet"),
    })
}

/// Deserialize a DataExport from XML using pre-extracted attributes.
fn deserialize_data_export_from_attrs<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    attrs: &Attrs,
    is_empty_tag: bool,
) -> Result<DataExport, DeserializeError> {
    let mut export = DataExport {
        data_type: attrs.get_opt_string("type"),
        enabled: attrs.get_opt_bool("enabled")?,
        frequency: attrs.get_opt_string("frequency"),
        orientation: attrs.get_opt_string("orientation"),
        resource: attrs.get_opt_string("resource"),
        worksheet: attrs.get_opt_string("worksheet"),
        interval: attrs.get_opt_string("interval"),
        export_all: None,
        table_uid: None,
    };

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
                        let attrs = Attrs::from_start(&e, reader)?;
                        let uid = attrs.get_req_string("uid")?;
                        let use_settings = attrs.get_opt_bool("use_settings")?;
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
                        let attrs = Attrs::from_start(&e, reader)?;
                        let uid = attrs.get_req_string("uid")?;
                        let use_settings = attrs.get_opt_bool("use_settings")?;
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
