//! Style deserialization module.
//!
//! This module handles deserialization of style structures and object styles.

use std::io::BufRead;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::{
    view::style::{ObjectStyle, Style},
    xml::deserialize::{
        DeserializeError,
        helpers::{
            parse_border_style as deserialize_border_style,
            parse_border_width as deserialize_border_width, parse_color as deserialize_color,
            parse_font_style as deserialize_font_style,
            parse_font_weight as deserialize_font_weight, parse_padding as deserialize_padding,
            parse_text_align as deserialize_text_align,
            parse_text_decoration as deserialize_text_decoration,
            parse_vertical_text_align as deserialize_vertical_text_align,
        },
    },
    xml::quick::de::Attrs,
};

pub fn deserialize_style<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Style, DeserializeError> {
    let event = reader.read_event_into(buf)?;
    let is_empty_tag = matches!(event, Event::Empty(_));

    match event {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"style" => {
            let attrs_obj = Attrs::from_start(&e, reader)?;
            buf.clear();
            deserialize_style_impl(reader, buf, attrs_obj.to_vec(), is_empty_tag)
        }
        _ => Err(DeserializeError::Custom(
            "Expected style element".to_string(),
        )),
    }
}

/// Internal implementation of style deserialization.
pub(crate) fn deserialize_style_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    attrs: Vec<(Vec<u8>, String)>,
    is_empty_tag: bool,
) -> Result<Style, DeserializeError> {
    let mut style = Style {
        color: None,
        background: None,
        z_index: None,
        border_width: None,
        border_color: None,
        border_style: None,
        font_family: None,
        font_style: None,
        font_weight: None,
        text_decoration: None,
        text_align: None,
        vertical_text_align: None,
        font_color: None,
        text_background: None,
        font_size: None,
        padding: None,
        stock: None,
        flow: None,
        aux: None,
        module: None,
        group: None,
        connector: None,
        alias: None,
        slider: None,
        knob: None,
        switch: None,
        options: None,
        numeric_input: None,
        list_input: None,
        graphical_input: None,
        numeric_display: None,
        lamp: None,
        gauge: None,
        graph: None,
        table: None,
        text_box: None,
        graphics_frame: None,
        button: None,
    };

    // Read attributes from the style element
    for (key, value) in attrs {
        match key.as_slice() {
            b"color" => style.color = Some(deserialize_color(&value)?),
            b"background" => style.background = Some(deserialize_color(&value)?),
            b"z_index" => {
                style.z_index = Some(
                    value
                        .parse::<i32>()
                        .map_err(|e| DeserializeError::Custom(format!("Invalid z_index: {}", e)))?,
                )
            }
            b"border_width" => style.border_width = Some(deserialize_border_width(&value)?),
            b"border_color" => style.border_color = Some(deserialize_color(&value)?),
            b"border_style" => style.border_style = Some(deserialize_border_style(&value)?),
            b"font_family" => style.font_family = Some(value),
            b"font_style" => style.font_style = Some(deserialize_font_style(&value)?),
            b"font_weight" => style.font_weight = Some(deserialize_font_weight(&value)?),
            b"text_decoration" => {
                style.text_decoration = Some(deserialize_text_decoration(&value)?)
            }
            b"text_align" => style.text_align = Some(deserialize_text_align(&value)?),
            b"vertical_text_align" => {
                style.vertical_text_align = Some(deserialize_vertical_text_align(&value)?)
            }
            b"font_color" => style.font_color = Some(deserialize_color(&value)?),
            b"text_background" => style.text_background = Some(deserialize_color(&value)?),
            b"font_size" => {
                let size_str = value.trim_end_matches("pt");
                style.font_size =
                    Some(size_str.parse::<f64>().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid font_size: {}", e))
                    })?);
            }
            b"padding" => style.padding = Some(deserialize_padding(&value)?),
            _ => {}
        }
    }

    if !is_empty_tag {
        loop {
            buf.clear();
            let event = reader.read_event_into(buf)?;
            match event {
                Event::Start(e) => {
                    let element_name = e.name().as_ref().to_vec();
                    let attrs_obj = Attrs::from_start(&e, reader)?;
                    let attrs = attrs_obj.to_vec();
                    buf.clear(); // Clear buf before calling deserialize_object_style
                    match element_name.as_slice() {
                        b"stock" => {
                            style.stock =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"flow" => {
                            style.flow =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"aux" => {
                            style.aux =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"module" => {
                            style.module =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"group" => {
                            style.group =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"connector" => {
                            style.connector =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"alias" => {
                            style.alias =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"slider" => {
                            style.slider =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"knob" => {
                            style.knob =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"switch" => {
                            style.switch =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"options" => {
                            style.options =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"numeric_input" => {
                            style.numeric_input =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"list_input" => {
                            style.list_input =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"graphical_input" => {
                            style.graphical_input =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"numeric_display" => {
                            style.numeric_display =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"lamp" => {
                            style.lamp =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"gauge" => {
                            style.gauge =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"graph" => {
                            style.graph =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"table" => {
                            style.table =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"text_box" => {
                            style.text_box =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"graphics_frame" => {
                            style.graphics_frame =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"button" => {
                            style.button =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        _ => {
                            // Unknown element, skip to end
                            loop {
                                buf.clear();
                                match reader.read_event_into(buf)? {
                                    Event::End(e)
                                        if e.name().as_ref() == element_name.as_slice() =>
                                    {
                                        break;
                                    }
                                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                Event::Empty(e) => {
                    let element_name = e.name().as_ref().to_vec();
                    let attrs_obj = Attrs::from_start(&e, reader)?;
                    let attrs = attrs_obj.to_vec();
                    buf.clear(); // Clear buf before calling deserialize_object_style
                    match element_name.as_slice() {
                        b"stock" => {
                            style.stock =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"flow" => {
                            style.flow =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"aux" => {
                            style.aux =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"module" => {
                            style.module =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"group" => {
                            style.group =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"connector" => {
                            style.connector =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"alias" => {
                            style.alias =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"slider" => {
                            style.slider =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"knob" => {
                            style.knob =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"switch" => {
                            style.switch =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"options" => {
                            style.options =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"numeric_input" => {
                            style.numeric_input =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"list_input" => {
                            style.list_input =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"graphical_input" => {
                            style.graphical_input =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"numeric_display" => {
                            style.numeric_display =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"lamp" => {
                            style.lamp =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"gauge" => {
                            style.gauge =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"graph" => {
                            style.graph =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"table" => {
                            style.table =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"text_box" => {
                            style.text_box =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"graphics_frame" => {
                            style.graphics_frame =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        b"button" => {
                            style.button =
                                Some(deserialize_object_style_from_attrs(reader, buf, &attrs)?)
                        }
                        _ => {}
                    }
                }
                Event::End(e) if e.name().as_ref() == b"style" => break,
                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                _ => {}
            }
        }
    }
    buf.clear();

    Ok(style)
}

/// Internal implementation of style deserialization with first element already read.
#[allow(dead_code)]
pub(crate) fn deserialize_style_impl_with_first_element<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    style_attrs: Vec<(Vec<u8>, String)>,
    element_name: Vec<u8>,
    element_attrs: Vec<(Vec<u8>, String)>,
) -> Result<Style, DeserializeError> {
    let mut style = Style {
        color: None,
        background: None,
        z_index: None,
        border_width: None,
        border_color: None,
        border_style: None,
        font_family: None,
        font_style: None,
        font_weight: None,
        text_decoration: None,
        text_align: None,
        vertical_text_align: None,
        font_color: None,
        text_background: None,
        font_size: None,
        padding: None,
        stock: None,
        flow: None,
        aux: None,
        module: None,
        group: None,
        connector: None,
        alias: None,
        slider: None,
        knob: None,
        switch: None,
        options: None,
        numeric_input: None,
        list_input: None,
        graphical_input: None,
        numeric_display: None,
        lamp: None,
        gauge: None,
        graph: None,
        table: None,
        text_box: None,
        graphics_frame: None,
        button: None,
    };

    // Read attributes from the style element
    for (key, value) in style_attrs {
        match key.as_slice() {
            b"color" => style.color = Some(deserialize_color(&value)?),
            b"background" => style.background = Some(deserialize_color(&value)?),
            b"z_index" => {
                style.z_index = Some(
                    value
                        .parse::<i32>()
                        .map_err(|e| DeserializeError::Custom(format!("Invalid z_index: {}", e)))?,
                )
            }
            b"border_width" => style.border_width = Some(deserialize_border_width(&value)?),
            b"border_color" => style.border_color = Some(deserialize_color(&value)?),
            b"border_style" => style.border_style = Some(deserialize_border_style(&value)?),
            b"font_family" => style.font_family = Some(value),
            b"font_style" => style.font_style = Some(deserialize_font_style(&value)?),
            b"font_weight" => style.font_weight = Some(deserialize_font_weight(&value)?),
            b"text_decoration" => {
                style.text_decoration = Some(deserialize_text_decoration(&value)?)
            }
            b"text_align" => style.text_align = Some(deserialize_text_align(&value)?),
            b"vertical_text_align" => {
                style.vertical_text_align = Some(deserialize_vertical_text_align(&value)?)
            }
            b"font_color" => style.font_color = Some(deserialize_color(&value)?),
            b"text_background" => style.text_background = Some(deserialize_color(&value)?),
            b"font_size" => {
                let size_str = value.trim_end_matches("pt");
                style.font_size =
                    Some(size_str.parse::<f64>().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid font_size: {}", e))
                    })?);
            }
            b"padding" => style.padding = Some(deserialize_padding(&value)?),
            _ => {}
        }
    }

    // Process the first element we already read
    match element_name.as_slice() {
        b"stock" => {
            style.stock = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"flow" => {
            style.flow = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"aux" => {
            style.aux = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"module" => {
            style.module = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"group" => {
            style.group = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"connector" => {
            style.connector = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"alias" => {
            style.alias = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"slider" => {
            style.slider = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"knob" => {
            style.knob = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"switch" => {
            style.switch = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"options" => {
            style.options = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"numeric_input" => {
            style.numeric_input = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"list_input" => {
            style.list_input = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"graphical_input" => {
            style.graphical_input = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"numeric_display" => {
            style.numeric_display = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"lamp" => {
            style.lamp = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"gauge" => {
            style.gauge = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"graph" => {
            style.graph = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"table" => {
            style.table = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"text_box" => {
            style.text_box = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"graphics_frame" => {
            style.graphics_frame = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        b"button" => {
            style.button = Some(deserialize_object_style_from_attrs(
                reader,
                buf,
                &element_attrs,
            )?)
        }
        _ => {
            // Skip unknown elements - read until end tag
            loop {
                match reader.read_event_into(buf)? {
                    Event::End(e) if e.name().as_ref() == element_name.as_slice() => break,
                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                    _ => {}
                }
                buf.clear();
            }
        }
    }
    buf.clear();

    // Continue processing remaining events
    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                let child_element_name = e.name().as_ref().to_vec();
                let attrs_obj = Attrs::from_start(&e, reader)?;
                let child_attrs = attrs_obj.to_vec();
                buf.clear();

                match child_element_name.as_slice() {
                    b"stock" => {
                        style.stock = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"flow" => {
                        style.flow = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"aux" => {
                        style.aux = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"module" => {
                        style.module = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"group" => {
                        style.group = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"connector" => {
                        style.connector = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"alias" => {
                        style.alias = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"slider" => {
                        style.slider = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"knob" => {
                        style.knob = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"switch" => {
                        style.switch = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"options" => {
                        style.options = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"numeric_input" => {
                        style.numeric_input = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"list_input" => {
                        style.list_input = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"graphical_input" => {
                        style.graphical_input = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"numeric_display" => {
                        style.numeric_display = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"lamp" => {
                        style.lamp = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"gauge" => {
                        style.gauge = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"graph" => {
                        style.graph = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"table" => {
                        style.table = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"text_box" => {
                        style.text_box = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"graphics_frame" => {
                        style.graphics_frame = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    b"button" => {
                        style.button = Some(deserialize_object_style_from_attrs(
                            reader,
                            buf,
                            &child_attrs,
                        )?)
                    }
                    _ => {
                        // Skip unknown elements - read until end tag
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(e)
                                    if e.name().as_ref() == child_element_name.as_slice() =>
                                {
                                    break;
                                }
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"style" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(style)
}

/// Deserialize an ObjectStyle from XML using pre-extracted attributes.
fn deserialize_object_style_from_attrs<R: BufRead>(
    _reader: &mut Reader<R>,
    _buf: &mut Vec<u8>,
    attrs: &[(Vec<u8>, String)],
) -> Result<ObjectStyle, DeserializeError> {
    let mut obj_style = ObjectStyle {
        color: None,
        background: None,
        z_index: None,
        border_width: None,
        border_color: None,
        border_style: None,
        font_family: None,
        font_style: None,
        font_weight: None,
        text_decoration: None,
        text_align: None,
        vertical_text_align: None,
        font_color: None,
        text_background: None,
        font_size: None,
        padding: None,
    };

    // Process attributes
    for (key, value) in attrs {
        match key.as_slice() {
            b"color" => obj_style.color = Some(deserialize_color(value)?),
            b"background" => obj_style.background = Some(deserialize_color(value)?),
            b"z_index" => {
                obj_style.z_index = Some(
                    value
                        .parse::<i32>()
                        .map_err(|e| DeserializeError::Custom(format!("Invalid z_index: {}", e)))?,
                )
            }
            b"border_width" => obj_style.border_width = Some(deserialize_border_width(value)?),
            b"border_color" => obj_style.border_color = Some(deserialize_color(value)?),
            b"border_style" => obj_style.border_style = Some(deserialize_border_style(value)?),
            b"font_family" => obj_style.font_family = Some(value.clone()),
            b"font_style" => obj_style.font_style = Some(deserialize_font_style(value)?),
            b"font_weight" => obj_style.font_weight = Some(deserialize_font_weight(value)?),
            b"text_decoration" => {
                obj_style.text_decoration = Some(deserialize_text_decoration(value)?)
            }
            b"text_align" => obj_style.text_align = Some(deserialize_text_align(value)?),
            b"vertical_text_align" => {
                obj_style.vertical_text_align = Some(deserialize_vertical_text_align(value)?)
            }
            b"font_color" => obj_style.font_color = Some(deserialize_color(value)?),
            b"text_background" => obj_style.text_background = Some(deserialize_color(value)?),
            b"font_size" => {
                let size_str = value.trim_end_matches("pt");
                obj_style.font_size =
                    Some(size_str.parse::<f64>().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid font_size: {}", e))
                    })?);
            }
            b"padding" => obj_style.padding = Some(deserialize_padding(value)?),
            _ => {}
        }
    }

    // ObjectStyle elements are always empty tags (attributes only), so we're done
    Ok(obj_style)
}
