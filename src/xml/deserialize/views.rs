//! Views deserialization module.
//!
//! This module handles deserialization of views and all view objects:
//! stocks, flows, auxes, modules, groups, connectors, aliases, and UI elements.

use std::io::BufRead;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::{
    Expression,
    equation::{Identifier, parse::unit_equation, units::UnitEquation},
    model::{
        events::EventPoster,
        groups::{Group, GroupEntity},
        object::{DeviceRange, DeviceScale, DisplayAs, Documentation, FormatOptions},
        vars::{
            aux::Auxiliary,
            flow::BasicFlow,
            stock::{BasicStock, ConveyorStock, QueueStock, Stock},
        },
    },
    view::{
        PageOrientation, PageSequence, View, ViewType,
        objects::{
            AliasObject, AuxObject, ButtonAppearance, ButtonObject, ButtonStyle, ConnectorObject,
            DataAction, FileAction, FlowObject, GaugeObject, GraphObject, GraphType,
            GraphicalInputObject, GraphicsFrameContent, GraphicsFrameObject, GroupObject,
            ImageContent, KnobObject, LampObject, LineStyle, Link, LinkEffect, LinkTarget,
            ListInputObject, MenuAction, MiscellaneousAction, ModuleObject, NumericDisplayObject,
            NumericInputObject, OptionEntity, OptionsLayout, OptionsObject, PenStyle, Plot,
            PlotScale, Point, Pointer, Polarity, PopupContent, PrintingAction, ReportBalances,
            ReportFlows, RestoreAction, Shape, SimulationAction, SliderObject,
            StackedContainerObject, StockObject, SwitchAction, SwitchObject, SwitchStyle,
            TableItem, TableItemType, TableObject, TableOrientation, TextBoxAppearance,
            TextBoxObject, VideoContent, Zone, ZoneType,
        },
        style::{
            BorderStyle, BorderWidth, Color, FontStyle, FontWeight, Style, TextAlign,
            TextDecoration, VerticalTextAlign,
        },
    },
    xml::{
        deserialize::{
            DeserializeError,
            helpers::{
                parse_border_width, parse_color, parse_font_style, parse_font_weight,
                parse_text_align, parse_text_decoration, parse_text_padding,
                parse_vertical_text_align, read_number_content, read_text_content,
            },
            style::deserialize_style_impl,
            variables::{
                deserialize_event_poster, deserialize_format, deserialize_range, deserialize_scale,
                read_expression,
            },
        },
        quick::de::{Attrs, skip_element},
    },
};

#[cfg(feature = "arrays")]
use crate::{
    model::vars::array::{ArrayElement, VariableDimensions},
    xml::deserialize::variables::deserialize_array_element,
    xml::deserialize::variables::deserialize_dimensions,
};

#[cfg(feature = "submodels")]
use crate::model::vars::module::{Module, ModuleConnection};

/// Helper to parse common object attributes (uid, name, x, y, width, height) from Attrs.
fn parse_object_attrs(
    attrs: &Attrs,
) -> Result<
    (
        Option<i32>,
        Option<String>,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        Option<f64>,
    ),
    DeserializeError,
> {
    let uid = attrs.get_opt_i32("uid")?;
    let name = attrs.get_opt_string("name");
    let x = attrs.get_opt_f64("x")?;
    let y = attrs.get_opt_f64("y")?;
    let width = attrs.get_opt_f64("width")?;
    let height = attrs.get_opt_f64("height")?;
    Ok((uid, name, x, y, width, height))
}
pub fn deserialize_views<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<crate::xml::schema::Views, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"views" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let visible_view = attrs.get_opt_u32("visible_view")?;

            buf.clear();
            deserialize_views_impl(reader, buf, visible_view)
        }
        Event::Start(e) => {
            return Err(DeserializeError::UnexpectedElement {
                expected: "views".to_string(),
                found: String::from_utf8_lossy(e.name().as_ref()).to_string(),
            });
        }
        _ => {
            return Err(DeserializeError::Custom(
                "Expected views start tag".to_string(),
            ));
        }
    }
}

/// Internal implementation of views deserialization.
pub(crate) fn deserialize_views_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    visible_view: Option<u32>,
) -> Result<crate::xml::schema::Views, DeserializeError> {
    let mut style: Option<Style> = None;
    let mut views = Vec::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) if e.name().as_ref() == b"view" => {
                let attrs_obj = Attrs::from_start(&e, reader)?;
                let attrs = attrs_obj.to_vec();
                buf.clear();
                views.push(deserialize_view_impl(reader, buf, attrs)?);
            }
            Event::Start(e) if e.name().as_ref() == b"style" => {
                let attrs_obj = Attrs::from_start(&e, reader)?;
                buf.clear();
                style = Some(deserialize_style_impl(
                    reader,
                    buf,
                    attrs_obj.to_vec(),
                    false,
                )?);
            }
            Event::Empty(e) if e.name().as_ref() == b"style" => {
                let attrs_obj = Attrs::from_start(&e, reader)?;
                buf.clear();
                style = Some(deserialize_style_impl(
                    reader,
                    buf,
                    attrs_obj.to_vec(),
                    true,
                )?);
            }
            Event::End(e) if e.name().as_ref() == b"views" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(crate::xml::schema::Views {
        visible_view,
        views,
        style,
    })
}

/// Deserialize a View from XML.
pub fn deserialize_view<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<View, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"view" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let view_type = attrs
                .get_opt("type")
                .map(|s| parse_view_type(s))
                .transpose()?;
            let order = attrs.get_opt_u32("order")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let zoom = attrs.get_opt_f64("zoom")?;
            let scroll_x = attrs.get_opt_f64("scroll_x")?;
            let scroll_y = attrs.get_opt_f64("scroll_y")?;
            let background = attrs.get_opt_string("background");
            let page_width = attrs.get_opt_f64("page_width")?;
            let page_height = attrs.get_opt_f64("page_height")?;
            let page_sequence = attrs
                .get_opt("page_sequence")
                .map(|s| match s {
                    "row" => Ok(PageSequence::Row),
                    "column" => Ok(PageSequence::Column),
                    _ => Err(DeserializeError::Custom(format!(
                        "Invalid page_sequence: {}",
                        s
                    ))),
                })
                .transpose()?;
            let page_orientation = attrs
                .get_opt("page_orientation")
                .map(|s| match s {
                    "landscape" => Ok(PageOrientation::Landscape),
                    "portrait" => Ok(PageOrientation::Portrait),
                    _ => Err(DeserializeError::Custom(format!(
                        "Invalid page_orientation: {}",
                        s
                    ))),
                })
                .transpose()?;
            let show_pages = attrs.get_opt_bool("show_pages")?;
            let home_page = attrs.get_opt_u32("home_page")?;
            let home_view = attrs.get_opt_bool("home_view")?;

            // Read child elements
            let mut style: Option<Style> = None;
            let mut stocks = Vec::new();
            let mut flows = Vec::new();
            let mut auxes = Vec::new();
            let mut modules = Vec::new();
            let mut groups = Vec::new();
            let mut connectors = Vec::new();
            let mut aliases = Vec::new();
            let mut stacked_containers = Vec::new();
            let mut sliders = Vec::new();
            let mut knobs = Vec::new();
            let mut switches = Vec::new();
            let mut options = Vec::new();
            let mut numeric_inputs = Vec::new();
            let mut list_inputs = Vec::new();
            let mut graphical_inputs = Vec::new();
            let mut numeric_displays = Vec::new();
            let mut lamps = Vec::new();
            let mut gauges = Vec::new();
            let mut graphs = Vec::new();
            let mut tables = Vec::new();
            let mut text_boxes = Vec::new();
            let mut graphics_frames = Vec::new();
            let mut buttons = Vec::new();

            loop {
                match reader.read_event_into(buf)? {
                    Event::Start(e) => {
                        match e.name().as_ref() {
                            b"style" => {
                                let attrs_obj = Attrs::from_start(&e, reader)?;
                                buf.clear();
                                style = Some(deserialize_style_impl(
                                    reader,
                                    buf,
                                    attrs_obj.to_vec(),
                                    false,
                                )?);
                            }
                            b"stock" => {
                                stocks.push(deserialize_stock_object(reader, buf)?);
                            }
                            b"flow" => {
                                flows.push(deserialize_flow_object(reader, buf)?);
                            }
                            b"aux" => {
                                auxes.push(deserialize_aux_object(reader, buf)?);
                            }
                            b"module" => {
                                // Extract attributes from the start event
                                let attrs = Attrs::from_start(&e, reader)?;
                                let (uid, name, x, y, width, height) = parse_object_attrs(&attrs)?;
                                let display_attrs = read_display_attributes(&e, reader)?;

                                // Read child elements (shape)
                                let mut shape: Option<Shape> = None;
                                buf.clear();
                                loop {
                                    match reader.read_event_into(buf)? {
                                        Event::Start(inner_e)
                                            if inner_e.name().as_ref() == b"shape" =>
                                        {
                                            shape = Some(deserialize_shape(reader, buf)?);
                                        }
                                        Event::End(end_e) if end_e.name().as_ref() == b"module" => {
                                            break;
                                        }
                                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                        _ => {}
                                    }
                                    buf.clear();
                                }

                                modules.push(ModuleObject {
                                    uid: crate::Uid::new(uid.ok_or_else(|| {
                                        DeserializeError::MissingField("uid".to_string())
                                    })?),
                                    name: name.ok_or_else(|| {
                                        DeserializeError::MissingField("name".to_string())
                                    })?,
                                    x: x.ok_or_else(|| {
                                        DeserializeError::MissingField("x".to_string())
                                    })?,
                                    y: y.ok_or_else(|| {
                                        DeserializeError::MissingField("y".to_string())
                                    })?,
                                    width: width.ok_or_else(|| {
                                        DeserializeError::MissingField("width".to_string())
                                    })?,
                                    height: height.ok_or_else(|| {
                                        DeserializeError::MissingField("height".to_string())
                                    })?,
                                    shape,
                                    color: display_attrs.color,
                                    background: display_attrs.background,
                                    z_index: display_attrs.z_index,
                                    font_family: display_attrs.font_family,
                                    font_size: display_attrs.font_size,
                                    font_weight: display_attrs.font_weight,
                                    font_style: display_attrs.font_style,
                                    text_decoration: display_attrs.text_decoration,
                                    text_align: display_attrs.text_align,
                                    text_background: display_attrs.text_background,
                                    vertical_text_align: display_attrs.vertical_text_align,
                                    text_padding: display_attrs.text_padding,
                                    font_color: display_attrs.font_color,
                                    text_border_color: display_attrs.text_border_color,
                                    text_border_width: display_attrs.text_border_width,
                                    text_border_style: display_attrs.text_border_style,
                                    label_side: display_attrs.label_side,
                                    label_angle: display_attrs.label_angle,
                                });
                            }
                            b"group" => {
                                groups.push(deserialize_group_object(reader, buf)?);
                            }
                            b"connector" => {
                                connectors.push(deserialize_connector_object(reader, buf)?);
                            }
                            b"alias" => {
                                aliases.push(deserialize_alias_object(reader, buf)?);
                            }
                            b"stacked_container" => {
                                stacked_containers
                                    .push(deserialize_stacked_container_object(reader, buf)?);
                            }
                            b"slider" => {
                                sliders.push(deserialize_slider_object(reader, buf)?);
                            }
                            b"knob" => {
                                knobs.push(deserialize_knob_object(reader, buf)?);
                            }
                            b"switch" => {
                                switches.push(deserialize_switch_object(reader, buf)?);
                            }
                            b"options" => {
                                options.push(deserialize_options_object(reader, buf)?);
                            }
                            b"numeric_input" => {
                                numeric_inputs.push(deserialize_numeric_input_object(reader, buf)?);
                            }
                            b"list_input" => {
                                list_inputs.push(deserialize_list_input_object(reader, buf)?);
                            }
                            b"graphical_input" => {
                                graphical_inputs
                                    .push(deserialize_graphical_input_object(reader, buf)?);
                            }
                            b"numeric_display" => {
                                numeric_displays
                                    .push(deserialize_numeric_display_object(reader, buf)?);
                            }
                            b"lamp" => {
                                lamps.push(deserialize_lamp_object(reader, buf)?);
                            }
                            b"gauge" => {
                                gauges.push(deserialize_gauge_object(reader, buf)?);
                            }
                            b"graph" => {
                                graphs.push(deserialize_graph_object(reader, buf)?);
                            }
                            b"table" => {
                                tables.push(deserialize_table_object(reader, buf)?);
                            }
                            b"text_box" => {
                                text_boxes.push(deserialize_text_box_object(reader, buf)?);
                            }
                            b"graphics_frame" => {
                                graphics_frames
                                    .push(deserialize_graphics_frame_object(reader, buf)?);
                            }
                            b"button" => {
                                buttons.push(deserialize_button_object(reader, buf)?);
                            }
                            _ => {
                                // Skip unknown elements using the helper
                                let element_name = e.name().as_ref().to_vec();
                                skip_element(reader, buf, &element_name)?;
                            }
                        }
                    }
                    Event::End(e) if e.name().as_ref() == b"view" => break,
                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                    _ => {}
                }
                buf.clear();
            }

            Ok(View {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                view_type: view_type.unwrap_or(ViewType::StockFlow),
                order,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                zoom,
                scroll_x,
                scroll_y,
                background,
                page_width: page_width
                    .ok_or_else(|| DeserializeError::MissingField("page_width".to_string()))?,
                page_height: page_height
                    .ok_or_else(|| DeserializeError::MissingField("page_height".to_string()))?,
                page_sequence: page_sequence.unwrap_or(PageSequence::Row),
                page_orientation: page_orientation.unwrap_or(PageOrientation::Landscape),
                show_pages: show_pages.unwrap_or(false),
                home_page: home_page.unwrap_or(0),
                home_view: home_view.unwrap_or(false),
                style,
                stocks,
                flows,
                auxes,
                modules,
                groups,
                connectors,
                aliases,
                stacked_containers,
                sliders,
                knobs,
                switches,
                options,
                numeric_inputs,
                list_inputs,
                graphical_inputs,
                numeric_displays,
                lamps,
                gauges,
                graphs,
                tables,
                text_boxes,
                graphics_frames,
                buttons,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected view element".to_string(),
        )),
    }
}

/// Internal implementation of View deserialization when start tag is already read.
pub(crate) fn deserialize_view_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    attrs: Vec<(Vec<u8>, String)>,
) -> Result<View, DeserializeError> {
    let mut uid: Option<i32> = None;
    let mut view_type: Option<ViewType> = None;
    let mut order: Option<u32> = None;
    let mut width: Option<f64> = None;
    let mut height: Option<f64> = None;
    let mut zoom: Option<f64> = None;
    let mut scroll_x: Option<f64> = None;
    let mut scroll_y: Option<f64> = None;
    let mut background: Option<String> = None;
    let mut page_width: Option<f64> = None;
    let mut page_height: Option<f64> = None;
    let mut page_sequence: Option<PageSequence> = None;
    let mut page_orientation: Option<PageOrientation> = None;
    let mut show_pages: Option<bool> = None;
    let mut home_page: Option<u32> = None;
    let mut home_view: Option<bool> = None;

    // Parse attributes from vec
    for (key, value) in attrs {
        match key.as_slice() {
            b"uid" => {
                uid = Some(
                    value
                        .parse()
                        .map_err(|e| DeserializeError::Custom(format!("Invalid uid: {}", e)))?,
                )
            }
            b"type" => view_type = Some(parse_view_type(&value)?),
            b"order" => {
                order = Some(
                    value
                        .parse()
                        .map_err(|e| DeserializeError::Custom(format!("Invalid order: {}", e)))?,
                )
            }
            b"width" => {
                width = Some(
                    value
                        .parse()
                        .map_err(|e| DeserializeError::Custom(format!("Invalid width: {}", e)))?,
                )
            }
            b"height" => {
                height = Some(
                    value
                        .parse()
                        .map_err(|e| DeserializeError::Custom(format!("Invalid height: {}", e)))?,
                )
            }
            b"zoom" => {
                zoom = Some(
                    value
                        .parse()
                        .map_err(|e| DeserializeError::Custom(format!("Invalid zoom: {}", e)))?,
                )
            }
            b"scroll_x" => {
                scroll_x =
                    Some(value.parse().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid scroll_x: {}", e))
                    })?)
            }
            b"scroll_y" => {
                scroll_y =
                    Some(value.parse().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid scroll_y: {}", e))
                    })?)
            }
            b"background" => background = Some(value),
            b"page_width" => {
                page_width =
                    Some(value.parse().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid page_width: {}", e))
                    })?)
            }
            b"page_height" => {
                page_height =
                    Some(value.parse().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid page_height: {}", e))
                    })?)
            }
            b"page_sequence" => {
                page_sequence = Some(match value.as_str() {
                    "row" => PageSequence::Row,
                    "column" => PageSequence::Column,
                    _ => {
                        return Err(DeserializeError::Custom(format!(
                            "Invalid page_sequence: {}",
                            value
                        )));
                    }
                })
            }
            b"page_orientation" => {
                page_orientation = Some(match value.as_str() {
                    "landscape" => PageOrientation::Landscape,
                    "portrait" => PageOrientation::Portrait,
                    _ => {
                        return Err(DeserializeError::Custom(format!(
                            "Invalid page_orientation: {}",
                            value
                        )));
                    }
                })
            }
            b"show_pages" => show_pages = Some(value == "true"),
            b"home_page" => {
                home_page =
                    Some(value.parse().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid home_page: {}", e))
                    })?)
            }
            b"home_view" => home_view = Some(value == "true"),
            _ => {}
        }
    }

    // Read child elements
    #[allow(unused_mut)]
    let mut style: Option<Style> = None;
    let mut stocks = Vec::new();
    let flows = Vec::new();
    let auxes = Vec::new();
    let mut modules = Vec::new();
    let groups = Vec::new();
    let connectors = Vec::new();
    let aliases = Vec::new();
    let stacked_containers = Vec::new();
    let sliders = Vec::new();
    let knobs = Vec::new();
    let switches = Vec::new();
    let options = Vec::new();
    let numeric_inputs = Vec::new();
    let list_inputs = Vec::new();
    let graphical_inputs = Vec::new();
    let numeric_displays = Vec::new();
    let lamps = Vec::new();
    let gauges = Vec::new();
    let graphs = Vec::new();
    let tables = Vec::new();
    let text_boxes = Vec::new();
    let graphics_frames = Vec::new();
    let buttons = Vec::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                let element_name = e.name().as_ref().to_vec();

                match element_name.as_slice() {
                    b"stock" => {
                        // Extract attributes and create StockObject
                        let attrs = Attrs::from_start(&e, reader)?;
                        let (uid, name, x, y, width, height) = parse_object_attrs(&attrs)?;
                        buf.clear();

                        // Skip to end of stock element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == b"stock" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }

                        stocks.push(StockObject {
                            uid: crate::Uid::new(uid.unwrap_or(0)),
                            name: name.unwrap_or_default(),
                            x,
                            y,
                            width: width.unwrap_or(50.0),
                            height: height.unwrap_or(35.0),
                            shape: None,
                            color: None,
                            background: None,
                            z_index: None,
                            font_family: None,
                            font_size: None,
                            font_weight: None,
                            font_style: None,
                            text_decoration: None,
                            text_align: None,
                            text_background: None,
                            vertical_text_align: None,
                            text_padding: None,
                            font_color: None,
                            text_border_color: None,
                            text_border_width: None,
                            text_border_style: None,
                            label_side: None,
                            label_angle: None,
                        });
                    }
                    b"module" => {
                        // Extract attributes and create ModuleObject
                        let attrs = Attrs::from_start(&e, reader)?;
                        let (uid, name, x, y, width, height) = parse_object_attrs(&attrs)?;
                        let display_attrs = read_display_attributes(&e, reader)?;

                        // Read child elements (shape)
                        let mut shape: Option<Shape> = None;
                        buf.clear();
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::Start(inner_e) if inner_e.name().as_ref() == b"shape" => {
                                    let shape_attrs = Attrs::from_start(&inner_e, reader)?;
                                    let shape_type = shape_attrs.get_opt_string("type");
                                    let shape_width = shape_attrs.get_opt_f64("width")?;
                                    let shape_height = shape_attrs.get_opt_f64("height")?;
                                    let corner_radius = shape_attrs.get_opt_f64("corner_radius")?;
                                    let radius = shape_attrs.get_opt_f64("radius")?;

                                    // Skip to end of shape element
                                    buf.clear();
                                    loop {
                                        match reader.read_event_into(buf)? {
                                            Event::End(e) if e.name().as_ref() == b"shape" => break,
                                            Event::Eof => {
                                                return Err(DeserializeError::UnexpectedEof);
                                            }
                                            _ => {}
                                        }
                                        buf.clear();
                                    }

                                    // Create the shape based on type
                                    shape = shape_type.map(|t| match t.as_str() {
                                        "rectangle" => Shape::Rectangle {
                                            width: shape_width.unwrap_or(0.0),
                                            height: shape_height.unwrap_or(0.0),
                                            corner_radius,
                                        },
                                        "circle" => Shape::Circle {
                                            radius: radius.unwrap_or(0.0),
                                        },
                                        "name_only" => Shape::NameOnly {
                                            width: shape_width,
                                            height: shape_height,
                                        },
                                        _ => Shape::Rectangle {
                                            width: shape_width.unwrap_or(0.0),
                                            height: shape_height.unwrap_or(0.0),
                                            corner_radius,
                                        },
                                    });
                                }
                                Event::Empty(inner_e) if inner_e.name().as_ref() == b"shape" => {
                                    let shape_attrs = Attrs::from_start(&inner_e, reader)?;
                                    let shape_type = shape_attrs.get_opt_string("type");
                                    let shape_width = shape_attrs.get_opt_f64("width")?;
                                    let shape_height = shape_attrs.get_opt_f64("height")?;
                                    let corner_radius = shape_attrs.get_opt_f64("corner_radius")?;
                                    let radius = shape_attrs.get_opt_f64("radius")?;

                                    // Create the shape based on type
                                    shape = shape_type.map(|t| match t.as_str() {
                                        "rectangle" => Shape::Rectangle {
                                            width: shape_width.unwrap_or(0.0),
                                            height: shape_height.unwrap_or(0.0),
                                            corner_radius,
                                        },
                                        "circle" => Shape::Circle {
                                            radius: radius.unwrap_or(0.0),
                                        },
                                        "name_only" => Shape::NameOnly {
                                            width: shape_width,
                                            height: shape_height,
                                        },
                                        _ => Shape::Rectangle {
                                            width: shape_width.unwrap_or(0.0),
                                            height: shape_height.unwrap_or(0.0),
                                            corner_radius,
                                        },
                                    });
                                }
                                Event::End(end_e) if end_e.name().as_ref() == b"module" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }

                        modules.push(ModuleObject {
                            uid: crate::Uid::new(uid.unwrap_or(0)),
                            name: name.unwrap_or_default(),
                            x: x.unwrap_or(0.0),
                            y: y.unwrap_or(0.0),
                            width: width.unwrap_or(100.0),
                            height: height.unwrap_or(50.0),
                            shape,
                            color: display_attrs.color,
                            background: display_attrs.background,
                            z_index: display_attrs.z_index,
                            font_family: display_attrs.font_family,
                            font_size: display_attrs.font_size,
                            font_weight: display_attrs.font_weight,
                            font_style: display_attrs.font_style,
                            text_decoration: display_attrs.text_decoration,
                            text_align: display_attrs.text_align,
                            text_background: display_attrs.text_background,
                            vertical_text_align: display_attrs.vertical_text_align,
                            text_padding: display_attrs.text_padding,
                            font_color: display_attrs.font_color,
                            text_border_color: display_attrs.text_border_color,
                            text_border_width: display_attrs.text_border_width,
                            text_border_style: display_attrs.text_border_style,
                            label_side: display_attrs.label_side,
                            label_angle: display_attrs.label_angle,
                        });
                    }
                    _ => {
                        // Skip this element
                        buf.clear();
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end)
                                    if end.name().as_ref() == element_name.as_slice() =>
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
            Event::Empty(e) => {
                // Handle empty elements (self-closing tags like <stock ... />)
                match e.name().as_ref() {
                    b"stock" => {
                        // Extract attributes and create StockObject
                        let attrs = Attrs::from_start(&e, reader)?;
                        let (uid, name, x, y, width, height) = parse_object_attrs(&attrs)?;

                        stocks.push(StockObject {
                            uid: crate::Uid::new(uid.unwrap_or(0)),
                            name: name.unwrap_or_default(),
                            x,
                            y,
                            width: width.unwrap_or(50.0),
                            height: height.unwrap_or(35.0),
                            shape: None,
                            color: None,
                            background: None,
                            z_index: None,
                            font_family: None,
                            font_size: None,
                            font_weight: None,
                            font_style: None,
                            text_decoration: None,
                            text_align: None,
                            text_background: None,
                            vertical_text_align: None,
                            text_padding: None,
                            font_color: None,
                            text_border_color: None,
                            text_border_width: None,
                            text_border_style: None,
                            label_side: None,
                            label_angle: None,
                        });
                    }
                    _ => {} // Skip other empty elements
                }
            }
            Event::End(e) if e.name().as_ref() == b"view" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(View {
        uid: crate::Uid::new(uid.unwrap_or(0)),
        view_type: view_type.unwrap_or(ViewType::StockFlow),
        order,
        width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
        height: height.ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
        zoom,
        scroll_x,
        scroll_y,
        background,
        page_width: page_width
            .ok_or_else(|| DeserializeError::MissingField("page_width".to_string()))?,
        page_height: page_height
            .ok_or_else(|| DeserializeError::MissingField("page_height".to_string()))?,
        page_sequence: page_sequence.unwrap_or(PageSequence::Row),
        page_orientation: page_orientation.unwrap_or(PageOrientation::Landscape),
        show_pages: show_pages.unwrap_or(false),
        home_page: home_page.unwrap_or(0),
        home_view: home_view.unwrap_or(false),
        style,
        stocks,
        flows,
        auxes,
        modules,
        groups,
        connectors,
        aliases,
        stacked_containers,
        sliders,
        knobs,
        switches,
        options,
        numeric_inputs,
        list_inputs,
        graphical_inputs,
        numeric_displays,
        lamps,
        gauges,
        graphs,
        tables,
        text_boxes,
        graphics_frames,
        buttons,
    })
}

/// Deserialize a StockObject from XML.
pub fn deserialize_stock_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<StockObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"stock" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let (uid, name, x, y, width, height) = parse_object_attrs(&attrs)?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut shape: Option<Shape> = None;

            // If it's a start tag (not empty), read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"shape" => {
                            shape = Some(deserialize_shape(reader, buf)?);
                        }
                        Event::End(e) if e.name().as_ref() == b"stock" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(StockObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
                x,
                y,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                shape,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
                label_side: display_attrs.label_side,
                label_angle: display_attrs.label_angle,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected stock element".to_string(),
        )),
    }
}

/// Deserialize a FlowObject from XML.
pub fn deserialize_flow_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<FlowObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"flow" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let (uid, name, x, y, width, height) = parse_object_attrs(&attrs)?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut pts = Vec::new();

            loop {
                match reader.read_event_into(buf)? {
                    Event::Start(e) if e.name().as_ref() == b"pts" => {
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"pt" => {
                                    let attrs = Attrs::from_start(&e, reader)?;
                                    let pt_x = attrs.get_opt_f64("x")?;
                                    let pt_y = attrs.get_opt_f64("y")?;

                                    if let (Some(x), Some(y)) = (pt_x, pt_y) {
                                        pts.push(Point { x, y });
                                    }

                                    // If it's a start tag (not empty), read until end
                                    if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                                        loop {
                                            match reader.read_event_into(buf)? {
                                                Event::End(e) if e.name().as_ref() == b"pt" => {
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
                                }
                                Event::End(e) if e.name().as_ref() == b"pts" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                    }
                    Event::End(e) if e.name().as_ref() == b"flow" => break,
                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                    _ => {}
                }
                buf.clear();
            }

            Ok(FlowObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
                x,
                y,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                pts,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
                label_side: display_attrs.label_side,
                label_angle: display_attrs.label_angle,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected flow element".to_string(),
        )),
    }
}

/// Deserialize an AuxObject from XML.
pub fn deserialize_aux_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<AuxObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"aux" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let (uid, name, x, y, width, height) = parse_object_attrs(&attrs)?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut shape: Option<Shape> = None;

            // If it's a start tag (not empty), read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"shape" => {
                            shape = Some(deserialize_shape(reader, buf)?);
                        }
                        Event::End(e) if e.name().as_ref() == b"aux" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(AuxObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
                x,
                y,
                width,
                height,
                shape,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
                label_side: display_attrs.label_side,
                label_angle: display_attrs.label_angle,
            })
        }
        _ => Err(DeserializeError::Custom("Expected aux element".to_string())),
    }
}

/// Helper struct for reading display attributes
struct DisplayAttributes {
    color: Option<Color>,
    background: Option<Color>,
    z_index: Option<i32>,
    font_family: Option<String>,
    font_size: Option<f64>,
    font_weight: Option<FontWeight>,
    font_style: Option<FontStyle>,
    text_decoration: Option<TextDecoration>,
    text_align: Option<TextAlign>,
    text_background: Option<Color>,
    vertical_text_align: Option<VerticalTextAlign>,
    text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    font_color: Option<Color>,
    text_border_color: Option<Color>,
    text_border_width: Option<BorderWidth>,
    text_border_style: Option<BorderStyle>,
    label_side: Option<String>,
    label_angle: Option<f64>,
}

/// Read display attributes from an element's attributes.
fn read_display_attributes<R: BufRead>(
    e: &quick_xml::events::BytesStart<'_>,
    reader: &Reader<R>,
) -> Result<DisplayAttributes, DeserializeError> {
    let attrs = Attrs::from_start(e, reader)?;

    let color = attrs.get_opt("color").map(|s| parse_color(s)).transpose()?;
    let background = attrs
        .get_opt("background")
        .map(|s| parse_color(s))
        .transpose()?;
    let z_index = attrs.get_opt_i32("z_index")?;
    let font_family = attrs.get_opt_string("font_family");
    let font_size = attrs
        .get_opt("font_size")
        .map(|s| {
            let font_size_clean = s.trim_end_matches("pt").trim();
            font_size_clean
                .parse()
                .map_err(|e| DeserializeError::Custom(format!("Invalid font_size value: {}", e)))
        })
        .transpose()?;
    let font_weight = attrs
        .get_opt("font_weight")
        .map(|s| match s {
            "normal" => Ok(FontWeight::Normal),
            "bold" => Ok(FontWeight::Bold),
            _ => Err(DeserializeError::Custom(format!(
                "Invalid font_weight: {}",
                s
            ))),
        })
        .transpose()?;
    let font_style = attrs
        .get_opt("font_style")
        .map(|s| match s {
            "normal" => Ok(FontStyle::Normal),
            "italic" => Ok(FontStyle::Italic),
            _ => Err(DeserializeError::Custom(format!(
                "Invalid font_style: {}",
                s
            ))),
        })
        .transpose()?;
    let text_decoration = attrs
        .get_opt("text_decoration")
        .map(|s| match s {
            "normal" => Ok(TextDecoration::Normal),
            "underline" => Ok(TextDecoration::Underline),
            _ => Err(DeserializeError::Custom(format!(
                "Invalid text_decoration: {}",
                s
            ))),
        })
        .transpose()?;
    let text_align = attrs
        .get_opt("text_align")
        .map(|s| match s {
            "left" => Ok(TextAlign::Left),
            "right" => Ok(TextAlign::Right),
            "center" => Ok(TextAlign::Center),
            _ => Err(DeserializeError::Custom(format!(
                "Invalid text_align: {}",
                s
            ))),
        })
        .transpose()?;
    let text_background = attrs
        .get_opt("text_background")
        .map(|s| parse_color(s))
        .transpose()?;
    let vertical_text_align = attrs
        .get_opt("vertical_text_align")
        .map(|s| match s {
            "top" => Ok(VerticalTextAlign::Top),
            "bottom" => Ok(VerticalTextAlign::Bottom),
            "center" => Ok(VerticalTextAlign::Center),
            _ => Err(DeserializeError::Custom(format!(
                "Invalid vertical_text_align: {}",
                s
            ))),
        })
        .transpose()?;
    let text_padding = attrs
        .get_opt("text_padding")
        .map(|s| parse_text_padding(s))
        .transpose()?;
    let font_color = attrs
        .get_opt("font_color")
        .map(|s| parse_color(s))
        .transpose()?;
    let text_border_color = attrs
        .get_opt("text_border_color")
        .map(|s| parse_color(s))
        .transpose()?;
    let text_border_width = attrs
        .get_opt("text_border_width")
        .map(|s| parse_border_width(s))
        .transpose()?;
    let text_border_style = attrs
        .get_opt("text_border_style")
        .map(|s| match s {
            "none" => Ok(BorderStyle::None),
            "solid" => Ok(BorderStyle::Solid),
            _ => Err(DeserializeError::Custom(format!(
                "Invalid text_border_style: {}",
                s
            ))),
        })
        .transpose()?;
    let label_side = attrs.get_opt_string("label_side");
    let label_angle = attrs.get_opt_f64("label_angle")?;

    Ok(DisplayAttributes {
        color,
        background,
        z_index,
        font_family,
        font_size,
        font_weight,
        font_style,
        text_decoration,
        text_align,
        text_background,
        vertical_text_align,
        text_padding,
        font_color,
        text_border_color,
        text_border_width,
        text_border_style,
        label_side,
        label_angle,
    })
}

/// Deserialize a Shape from XML.
fn deserialize_shape<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Shape, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"shape" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let shape_type = attrs.get_opt_string("type");
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let corner_radius = attrs.get_opt_f64("corner_radius")?;
            let radius = attrs.get_opt_f64("radius")?;

            // If it's a start tag, read until end
            if matches!(reader.read_event_into(buf)?, Event::Start(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"shape" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            match shape_type.as_deref() {
                Some("rectangle") => Ok(Shape::Rectangle {
                    width: width
                        .ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                    height: height
                        .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                    corner_radius,
                }),
                Some("circle") => Ok(Shape::Circle {
                    radius: radius
                        .ok_or_else(|| DeserializeError::MissingField("radius".to_string()))?,
                }),
                Some("name_only") => Ok(Shape::NameOnly { width, height }),
                _ => Err(DeserializeError::Custom(format!(
                    "Invalid shape type: {:?}",
                    shape_type
                ))),
            }
        }
        _ => Err(DeserializeError::Custom(
            "Expected shape element".to_string(),
        )),
    }
}

/// Deserialize a ModuleObject from XML.
pub fn deserialize_module_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<ModuleObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"module" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let (uid, name, x, y, width, height) = parse_object_attrs(&attrs)?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut shape: Option<Shape> = None;

            loop {
                match reader.read_event_into(buf)? {
                    Event::Start(e) if e.name().as_ref() == b"shape" => {
                        shape = Some(deserialize_shape(reader, buf)?);
                    }
                    Event::End(e) if e.name().as_ref() == b"module" => break,
                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                    _ => {}
                }
                buf.clear();
            }
            buf.clear();

            Ok(ModuleObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                shape,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
                label_side: display_attrs.label_side,
                label_angle: display_attrs.label_angle,
            })
        }
        Event::Empty(e) if e.name().as_ref() == b"module" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let (uid, name, x, y, width, height) = parse_object_attrs(&attrs)?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;
            buf.clear();

            Ok(ModuleObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                shape: None,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
                label_side: display_attrs.label_side,
                label_angle: display_attrs.label_angle,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected module element".to_string(),
        )),
    }
}

/// Deserialize a ModuleObject from a start event that has already been consumed.
pub fn deserialize_module_object_from_start<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start_event: &quick_xml::events::BytesStart,
) -> Result<ModuleObject, DeserializeError> {
    let attrs = Attrs::from_start(start_event, reader)?;
    let uid = attrs.get_opt_i32("uid")?;
    let name = attrs.get_opt_string("name");
    let x = attrs.get_opt_f64("x")?;
    let y = attrs.get_opt_f64("y")?;
    let width = attrs.get_opt_f64("width")?;
    let height = attrs.get_opt_f64("height")?;

    // Read common display attributes
    let display_attrs = read_display_attributes(start_event, reader)?;

    let mut shape: Option<Shape> = None;

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) if e.name().as_ref() == b"shape" => {
                shape = Some(deserialize_shape(reader, buf)?);
            }
            Event::End(e) if e.name().as_ref() == b"module" => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }
    buf.clear();

    Ok(ModuleObject {
        uid: crate::Uid::new(uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?),
        name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
        x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
        y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
        width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
        height: height.ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
        shape,
        color: display_attrs.color,
        background: display_attrs.background,
        z_index: display_attrs.z_index,
        font_family: display_attrs.font_family,
        font_size: display_attrs.font_size,
        font_weight: display_attrs.font_weight,
        font_style: display_attrs.font_style,
        text_decoration: display_attrs.text_decoration,
        text_align: display_attrs.text_align,
        text_background: display_attrs.text_background,
        vertical_text_align: display_attrs.vertical_text_align,
        text_padding: display_attrs.text_padding,
        font_color: display_attrs.font_color,
        text_border_color: display_attrs.text_border_color,
        text_border_width: display_attrs.text_border_width,
        text_border_style: display_attrs.text_border_style,
        label_side: display_attrs.label_side,
        label_angle: display_attrs.label_angle,
    })
}

/// Deserialize a GroupObject from XML.
pub fn deserialize_group_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<GroupObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"group" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let name = attrs.get_opt_string("name");
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let locked = attrs.get_opt_bool("locked")?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut items = Vec::new();

            // If it's a start tag (not empty), read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"item" => {
                            let attrs = Attrs::from_start(&e, reader)?;
                            let item_uid = attrs.get_opt_i32("uid")?;
                            if let Some(uid) = item_uid {
                                items.push(crate::Uid::new(uid));
                            }

                            // If it's a start tag, read until end
                            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                                loop {
                                    match reader.read_event_into(buf)? {
                                        Event::End(e) if e.name().as_ref() == b"item" => break,
                                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                        _ => {}
                                    }
                                    buf.clear();
                                }
                            }
                        }
                        Event::End(e) if e.name().as_ref() == b"group" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(GroupObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                locked: locked.unwrap_or(false),
                items,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected group element".to_string(),
        )),
    }
}

/// Deserialize a ConnectorObject from XML.
pub fn deserialize_connector_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<ConnectorObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"connector" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let angle = attrs.get_opt_f64("angle")?;
            let line_style = attrs
                .get_opt("line_style")
                .map(|s| parse_line_style(s))
                .transpose()?;
            let delay_mark = attrs.get_opt_bool("delay_mark")?;
            let polarity = attrs
                .get_opt("polarity")
                .map(|s| parse_polarity(s))
                .transpose()?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut from: Option<Pointer> = None;
            let mut to: Option<Pointer> = None;
            let mut pts = Vec::new();

            loop {
                match reader.read_event_into(buf)? {
                    Event::Start(e) if e.name().as_ref() == b"from" => {
                        from = Some(deserialize_pointer(reader, buf)?);
                    }
                    Event::Start(e) if e.name().as_ref() == b"to" => {
                        to = Some(deserialize_pointer(reader, buf)?);
                    }
                    Event::Start(e) if e.name().as_ref() == b"pts" => {
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"pt" => {
                                    let attrs = Attrs::from_start(&e, reader)?;
                                    let pt_x = attrs.get_opt_f64("x")?;
                                    let pt_y = attrs.get_opt_f64("y")?;

                                    if let (Some(x), Some(y)) = (pt_x, pt_y) {
                                        pts.push(Point { x, y });
                                    }

                                    // If it's a start tag, read until end
                                    if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                                        loop {
                                            match reader.read_event_into(buf)? {
                                                Event::End(e) if e.name().as_ref() == b"pt" => {
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
                                }
                                Event::End(e) if e.name().as_ref() == b"pts" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                    }
                    Event::End(e) if e.name().as_ref() == b"connector" => break,
                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                    _ => {}
                }
                buf.clear();
            }

            Ok(ConnectorObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                angle: angle.ok_or_else(|| DeserializeError::MissingField("angle".to_string()))?,
                line_style,
                delay_mark: delay_mark.unwrap_or(false),
                polarity,
                from: from.ok_or_else(|| DeserializeError::MissingField("from".to_string()))?,
                to: to.ok_or_else(|| DeserializeError::MissingField("to".to_string()))?,
                pts,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected connector element".to_string(),
        )),
    }
}

/// Deserialize an AliasObject from XML.
pub fn deserialize_alias_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<AliasObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"alias" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut of: Option<String> = None;
            let mut shape: Option<Shape> = None;

            // If it's a start tag (not empty), read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"of" => {
                            of = Some(read_text_content(reader, buf)?);
                        }
                        Event::Start(e) if e.name().as_ref() == b"shape" => {
                            shape = Some(deserialize_shape(reader, buf)?);
                        }
                        Event::End(e) if e.name().as_ref() == b"alias" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(AliasObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                of: of.ok_or_else(|| DeserializeError::MissingField("of".to_string()))?,
                shape,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
                label_side: display_attrs.label_side,
                label_angle: display_attrs.label_angle,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected alias element".to_string(),
        )),
    }
}

/// Deserialize a Pointer from XML.
/// This is called when we're already inside a <from> or <to> element.
fn deserialize_pointer<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Pointer, DeserializeError> {
    // Peek at the next event to see if it's an alias tag or text
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"alias" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;

            // If it's a start tag (not empty), read until end
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"alias" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(Pointer::Alias(crate::Uid::new(uid.ok_or_else(|| {
                DeserializeError::MissingField("uid".to_string())
            })?)))
        }
        Event::Text(e) => {
            let name = e.unescape()?.to_string();
            buf.clear();
            Ok(Pointer::Name(name))
        }
        _ => {
            // Try reading as text content
            let name = read_text_content(reader, buf)?;
            Ok(Pointer::Name(name))
        }
    }
}

/// Parse Polarity from string.
fn parse_polarity(s: &str) -> Result<Polarity, DeserializeError> {
    match s {
        "+" => Ok(Polarity::Positive),
        "-" => Ok(Polarity::Negative),
        "none" => Ok(Polarity::None),
        _ => Err(DeserializeError::Custom(format!("Invalid polarity: {}", s))),
    }
}

/// Parse LineStyle from string.
fn parse_line_style(s: &str) -> Result<LineStyle, DeserializeError> {
    match s {
        "solid" => Ok(LineStyle::Solid),
        "dashed" => Ok(LineStyle::Dashed),
        _ => Ok(LineStyle::VendorSpecific(s.to_string())),
    }
}

/// Deserialize a StackedContainerObject from XML.
pub fn deserialize_stacked_container_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<StackedContainerObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"stacked_container" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let visible_index = attrs.get_opt_usize("visible_index")?;

            // If it's a start tag, read until end
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"stacked_container" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(StackedContainerObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                visible_index: visible_index
                    .ok_or_else(|| DeserializeError::MissingField("visible_index".to_string()))?,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected stacked_container element".to_string(),
        )),
    }
}

/// Deserialize a SliderObject from XML.
pub fn deserialize_slider_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<SliderObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"slider" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let min = attrs.get_opt_f64("min")?;
            let max = attrs.get_opt_f64("max")?;
            let show_name = attrs.get_opt_bool("show_name")?;
            let show_number = attrs.get_opt_bool("show_number")?;
            let show_min_max = attrs.get_opt_bool("show_min_max")?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut entity_name: Option<String> = None;
            let mut reset_to: Option<(f64, String)> = None;

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"entity" => {
                            let attrs = Attrs::from_start(&e, reader)?;
                            entity_name = attrs.get_opt_string("name");

                            // If it's a start tag, read until end
                            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                                loop {
                                    match reader.read_event_into(buf)? {
                                        Event::End(e) if e.name().as_ref() == b"entity" => break,
                                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                        _ => {}
                                    }
                                    buf.clear();
                                }
                            }
                        }
                        Event::Start(e) if e.name().as_ref() == b"reset_to" => {
                            let attrs = Attrs::from_start(&e, reader)?;
                            let after = attrs.get_opt_string("after");
                            let value_str = read_text_content(reader, buf)?;
                            let value = value_str.parse::<f64>().map_err(|e| {
                                DeserializeError::Custom(format!("Invalid reset_to value: {}", e))
                            })?;
                            if let Some(after_val) = after {
                                reset_to = Some((value, after_val));
                            }
                        }
                        Event::End(e) if e.name().as_ref() == b"slider" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(SliderObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                min: min.ok_or_else(|| DeserializeError::MissingField("min".to_string()))?,
                max: max.ok_or_else(|| DeserializeError::MissingField("max".to_string()))?,
                show_name: show_name.unwrap_or(true),
                show_number: show_number.unwrap_or(true),
                show_min_max: show_min_max.unwrap_or(true),
                entity_name: entity_name.unwrap_or_default(),
                reset_to,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected slider element".to_string(),
        )),
    }
}

/// Deserialize a KnobObject from XML (same as SliderObject but with <knob> tag).
pub fn deserialize_knob_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<KnobObject, DeserializeError> {
    // KnobObject is a type alias for SliderObject, but uses <knob> tag
    // We can reuse the same logic but check for "knob" instead
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"knob" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let min = attrs.get_opt_f64("min")?;
            let max = attrs.get_opt_f64("max")?;
            let show_name = attrs.get_opt_bool("show_name")?;
            let show_number = attrs.get_opt_bool("show_number")?;
            let show_min_max = attrs.get_opt_bool("show_min_max")?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut entity_name: Option<String> = None;

            // If it's a start tag, read child elements (knobs don't have reset_to)
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"entity" => {
                            let attrs = Attrs::from_start(&e, reader)?;
                            entity_name = attrs.get_opt_string("name");

                            // If it's a start tag, read until end
                            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                                loop {
                                    match reader.read_event_into(buf)? {
                                        Event::End(e) if e.name().as_ref() == b"entity" => break,
                                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                        _ => {}
                                    }
                                    buf.clear();
                                }
                            }
                        }
                        Event::End(e) if e.name().as_ref() == b"knob" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(SliderObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                min: min.ok_or_else(|| DeserializeError::MissingField("min".to_string()))?,
                max: max.ok_or_else(|| DeserializeError::MissingField("max".to_string()))?,
                show_name: show_name.unwrap_or(true),
                show_number: show_number.unwrap_or(true),
                show_min_max: show_min_max.unwrap_or(true),
                entity_name: entity_name.unwrap_or_default(),
                reset_to: None, // Knobs don't have reset_to
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected knob element".to_string(),
        )),
    }
}

/// Parse SwitchStyle from string.
fn parse_switch_style(s: &str) -> Result<SwitchStyle, DeserializeError> {
    match s {
        "toggle" => Ok(SwitchStyle::Toggle),
        "push_button" => Ok(SwitchStyle::PushButton),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid switch_style: {}",
            s
        ))),
    }
}

/// Deserialize a SwitchObject from XML.
pub fn deserialize_switch_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<SwitchObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"switch" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let show_name = attrs.get_opt_bool("show_name")?;
            let switch_style = attrs
                .get_opt("switch_style")
                .map(|s| parse_switch_style(s))
                .transpose()?;
            let clicking_sound = attrs.get_opt_bool("clicking_sound")?;
            let entity_name = attrs.get_opt_string("entity_name");
            let entity_value = attrs.get_opt_f64("entity_value")?;
            let group_name = attrs.get_opt_string("group_name");
            let module_name = attrs.get_opt_string("module_name");

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut reset_to: Option<(f64, String)> = None;

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"reset_to" => {
                            let attrs = Attrs::from_start(&e, reader)?;
                            let after = attrs.get_opt_string("after");
                            let value_str = read_text_content(reader, buf)?;
                            let value = value_str.parse::<f64>().map_err(|e| {
                                DeserializeError::Custom(format!("Invalid reset_to value: {}", e))
                            })?;
                            if let Some(after_val) = after {
                                reset_to = Some((value, after_val));
                            }
                        }
                        Event::End(e) if e.name().as_ref() == b"switch" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(SwitchObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                show_name: show_name.unwrap_or(true),
                switch_style: switch_style
                    .ok_or_else(|| DeserializeError::MissingField("switch_style".to_string()))?,
                clicking_sound: clicking_sound.unwrap_or(false),
                entity_name,
                entity_value,
                group_name,
                module_name,
                reset_to,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
                label_side: display_attrs.label_side,
                label_angle: display_attrs.label_angle,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected switch element".to_string(),
        )),
    }
}

/// Parse OptionsLayout from string.
fn parse_options_layout(s: &str) -> Result<OptionsLayout, DeserializeError> {
    match s {
        "vertical" => Ok(OptionsLayout::Vertical),
        "horizontal" => Ok(OptionsLayout::Horizontal),
        "grid" => Ok(OptionsLayout::Grid),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid options layout: {}",
            s
        ))),
    }
}

/// Deserialize an OptionsObject from XML.
pub fn deserialize_options_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<OptionsObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"options" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let layout = attrs
                .get_opt("layout")
                .map(|s| parse_options_layout(s))
                .transpose()?;
            let horizontal_spacing = attrs.get_opt_f64("horizontal_spacing")?;
            let vertical_spacing = attrs.get_opt_f64("vertical_spacing")?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut entities = Vec::new();

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"entity" => {
                            let attrs = Attrs::from_start(&e, reader)?;
                            let entity_name = attrs.get_opt_string("name");
                            let index = attrs.get_opt_string("index");
                            let mut value: Option<f64> = None;

                            // Read text content for value
                            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                                let value_str = read_text_content(reader, buf)?;
                                value = Some(value_str.parse::<f64>().map_err(|e| {
                                    DeserializeError::Custom(format!("Invalid entity value: {}", e))
                                })?);
                            }

                            if let Some(name) = entity_name {
                                if let Some(val) = value {
                                    entities.push(OptionEntity {
                                        entity_name: name,
                                        index,
                                        value: val,
                                    });
                                }
                            }
                        }
                        Event::End(e) if e.name().as_ref() == b"options" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(OptionsObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                layout: layout
                    .ok_or_else(|| DeserializeError::MissingField("layout".to_string()))?,
                horizontal_spacing: horizontal_spacing.ok_or_else(|| {
                    DeserializeError::MissingField("horizontal_spacing".to_string())
                })?,
                vertical_spacing: vertical_spacing.ok_or_else(|| {
                    DeserializeError::MissingField("vertical_spacing".to_string())
                })?,
                entities,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected options element".to_string(),
        )),
    }
}

/// Deserialize a NumericInputObject from XML.
pub fn deserialize_numeric_input_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<NumericInputObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"numeric_input" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let entity_name = attrs.get_opt_string("entity_name");
            let entity_index = attrs.get_opt_string("entity_index");
            let min = attrs.get_opt_f64("min")?;
            let max = attrs.get_opt_f64("max")?;
            let precision = attrs.get_opt_f64("precision")?;
            let value = attrs.get_opt_f64("value")?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            // If it's a start tag, read until end
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"numeric_input" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(NumericInputObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                entity_name: entity_name
                    .ok_or_else(|| DeserializeError::MissingField("entity_name".to_string()))?,
                entity_index,
                min: min.ok_or_else(|| DeserializeError::MissingField("min".to_string()))?,
                max: max.ok_or_else(|| DeserializeError::MissingField("max".to_string()))?,
                precision,
                value: value.ok_or_else(|| DeserializeError::MissingField("value".to_string()))?,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected numeric_input element".to_string(),
        )),
    }
}

/// Deserialize a ListInputObject from XML.
pub fn deserialize_list_input_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<ListInputObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"list_input" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let name = attrs.get_opt_string("name");
            let column_width = attrs.get_opt_f64("column_width")?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut numeric_inputs = Vec::new();

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"numeric_input" => {
                            numeric_inputs.push(deserialize_numeric_input_object(reader, buf)?);
                        }
                        Event::End(e) if e.name().as_ref() == b"list_input" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(ListInputObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
                column_width: column_width
                    .ok_or_else(|| DeserializeError::MissingField("column_width".to_string()))?,
                numeric_inputs,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected list_input element".to_string(),
        )),
    }
}

/// Deserialize GraphicalFunctionData from XML (view objects version).
fn deserialize_view_graphical_function_data<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<crate::view::objects::GraphicalFunctionData, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"gf" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let xscale_min = attrs.get_opt_f64("xscale_min")?;
            let xscale_max = attrs.get_opt_f64("xscale_max")?;
            let mut ypts: Vec<f64> = Vec::new();

            // Read ypts from text content
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                let ypts_str = read_text_content(reader, buf)?;
                ypts = ypts_str
                    .split_whitespace()
                    .map(|s| {
                        s.parse::<f64>().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid ypts value: {}", e))
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
            }

            // If it's a start tag, read until end
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"gf" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(crate::view::objects::GraphicalFunctionData {
                xscale_min: xscale_min
                    .ok_or_else(|| DeserializeError::MissingField("xscale_min".to_string()))?,
                xscale_max: xscale_max
                    .ok_or_else(|| DeserializeError::MissingField("xscale_max".to_string()))?,
                ypts,
            })
        }
        _ => Err(DeserializeError::Custom("Expected gf element".to_string())),
    }
}

/// Deserialize a GraphicalInputObject from XML.
pub fn deserialize_graphical_input_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<GraphicalInputObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"graphical_input" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let entity_name = attrs.get_opt_string("entity_name");

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut graphical_function: Option<crate::view::objects::GraphicalFunctionData> = None;

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"gf" => {
                            graphical_function =
                                Some(deserialize_view_graphical_function_data(reader, buf)?);
                        }
                        Event::End(e) if e.name().as_ref() == b"graphical_input" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(GraphicalInputObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                entity_name: entity_name
                    .ok_or_else(|| DeserializeError::MissingField("entity_name".to_string()))?,
                graphical_function,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected graphical_input element".to_string(),
        )),
    }
}

/// Deserialize a NumericDisplayObject from XML.
pub fn deserialize_numeric_display_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<NumericDisplayObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"numeric_display" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let entity_name = attrs.get_opt_string("entity_name");
            let show_name = attrs.get_opt_bool("show_name")?;
            let retain_ending_value = attrs.get_opt_bool("retain_ending_value")?;
            let precision = attrs.get_opt_f64("precision")?;
            let delimit_000s = attrs.get_opt_bool("delimit_000s")?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            // If it's a start tag, read until end
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"numeric_display" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(NumericDisplayObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                entity_name: entity_name
                    .ok_or_else(|| DeserializeError::MissingField("entity_name".to_string()))?,
                show_name: show_name.unwrap_or(true),
                retain_ending_value: retain_ending_value.unwrap_or(false),
                precision,
                delimit_000s: delimit_000s.unwrap_or(false),
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected numeric_display element".to_string(),
        )),
    }
}

/// Parse ZoneType from string.
fn parse_zone_type(s: &str) -> Result<ZoneType, DeserializeError> {
    match s {
        "normal" => Ok(ZoneType::Normal),
        "caution" => Ok(ZoneType::Caution),
        "panic" => Ok(ZoneType::Panic),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid zone type: {}",
            s
        ))),
    }
}

/// Deserialize a Zone from XML.
fn deserialize_zone<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Zone, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"zone" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let zone_type = attrs
                .get_opt("type")
                .map(|s| parse_zone_type(s))
                .transpose()?;
            let color = attrs.get_opt("color").map(|s| parse_color(s)).transpose()?;
            let min = attrs.get_opt_f64("min")?;
            let max = attrs.get_opt_f64("max")?;
            let sound = attrs.get_opt_string("sound");

            // If it's a start tag, read until end
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"zone" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(Zone {
                zone_type: zone_type
                    .ok_or_else(|| DeserializeError::MissingField("type".to_string()))?,
                color: color.ok_or_else(|| DeserializeError::MissingField("color".to_string()))?,
                min: min.ok_or_else(|| DeserializeError::MissingField("min".to_string()))?,
                max: max.ok_or_else(|| DeserializeError::MissingField("max".to_string()))?,
                sound,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected zone element".to_string(),
        )),
    }
}

/// Deserialize a LampObject from XML.
pub fn deserialize_lamp_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<LampObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"lamp" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let entity_name = attrs.get_opt_string("entity_name");
            let show_name = attrs.get_opt_bool("show_name")?;
            let retain_ending_value = attrs.get_opt_bool("retain_ending_value")?;
            let flash_on_panic = attrs.get_opt_bool("flash_on_panic")?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut zones = Vec::new();

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"zone" => {
                            zones.push(deserialize_zone(reader, buf)?);
                        }
                        Event::End(e) if e.name().as_ref() == b"lamp" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(LampObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                entity_name: entity_name
                    .ok_or_else(|| DeserializeError::MissingField("entity_name".to_string()))?,
                show_name: show_name.unwrap_or(true),
                retain_ending_value: retain_ending_value.unwrap_or(false),
                flash_on_panic: flash_on_panic.unwrap_or(false),
                zones,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected lamp element".to_string(),
        )),
    }
}

/// Deserialize a GaugeObject from XML.
pub fn deserialize_gauge_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<GaugeObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"gauge" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let entity_name = attrs.get_opt_string("entity_name");
            let show_name = attrs.get_opt_bool("show_name")?;
            let show_number = attrs.get_opt_bool("show_number")?;
            let retain_ending_value = attrs.get_opt_bool("retain_ending_value")?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut zones = Vec::new();

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"zone" => {
                            zones.push(deserialize_zone(reader, buf)?);
                        }
                        Event::End(e) if e.name().as_ref() == b"gauge" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(GaugeObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                entity_name: entity_name
                    .ok_or_else(|| DeserializeError::MissingField("entity_name".to_string()))?,
                show_name: show_name.unwrap_or(true),
                show_number: show_number.unwrap_or(true),
                retain_ending_value: retain_ending_value.unwrap_or(false),
                zones,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected gauge element".to_string(),
        )),
    }
}

/// Parse GraphType from string.
fn parse_graph_type(s: &str) -> Result<GraphType, DeserializeError> {
    match s {
        "time_series" => Ok(GraphType::TimeSeries),
        "scatter" => Ok(GraphType::Scatter),
        "bar" => Ok(GraphType::Bar),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid graph type: {}",
            s
        ))),
    }
}

/// Parse PenStyle from string.
fn parse_pen_style(s: &str) -> Result<PenStyle, DeserializeError> {
    match s {
        "solid" => Ok(PenStyle::Solid),
        "dotted" => Ok(PenStyle::Dotted),
        "dashed" => Ok(PenStyle::Dashed),
        "dot_dashed" => Ok(PenStyle::DotDashed),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid pen style: {}",
            s
        ))),
    }
}

/// Deserialize a PlotScale from XML.
fn deserialize_plot_scale<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<PlotScale, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"scale" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let min = attrs.get_opt_f64("min")?;
            let max = attrs.get_opt_f64("max")?;

            // If it's a start tag, read until end
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"scale" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(PlotScale {
                min: min.ok_or_else(|| DeserializeError::MissingField("min".to_string()))?,
                max: max.ok_or_else(|| DeserializeError::MissingField("max".to_string()))?,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected scale element".to_string(),
        )),
    }
}

/// Deserialize a Plot from XML.
fn deserialize_plot<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Plot, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"plot" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let index = attrs.get_opt_u32("index")?;
            let pen_width = attrs.get_opt_f64("pen_width")?;
            let pen_style = attrs
                .get_opt("pen_style")
                .map(|s| parse_pen_style(s))
                .transpose()?;
            let show_y_axis = attrs.get_opt_bool("show_y_axis")?;
            let title = attrs.get_opt_string("title");
            let right_axis = attrs.get_opt_bool("right_axis")?;
            let entity_name = attrs.get_opt_string("entity_name");
            let precision = attrs.get_opt_f64("precision")?;
            let color = attrs.get_opt("color").map(|s| parse_color(s)).transpose()?;

            let mut scale: Option<PlotScale> = None;

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"scale" => {
                            scale = Some(deserialize_plot_scale(reader, buf)?);
                        }
                        Event::End(e) if e.name().as_ref() == b"plot" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(Plot {
                index: index.ok_or_else(|| DeserializeError::MissingField("index".to_string()))?,
                pen_width: pen_width
                    .ok_or_else(|| DeserializeError::MissingField("pen_width".to_string()))?,
                pen_style: pen_style
                    .ok_or_else(|| DeserializeError::MissingField("pen_style".to_string()))?,
                show_y_axis: show_y_axis.unwrap_or(false),
                title: title.ok_or_else(|| DeserializeError::MissingField("title".to_string()))?,
                right_axis: right_axis.unwrap_or(false),
                entity_name: entity_name
                    .ok_or_else(|| DeserializeError::MissingField("entity_name".to_string()))?,
                precision,
                scale,
                color,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected plot element".to_string(),
        )),
    }
}

/// Deserialize a GraphObject from XML.
pub fn deserialize_graph_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<GraphObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"graph" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let graph_type = attrs
                .get_opt("graph_type")
                .map(|s| parse_graph_type(s))
                .transpose()?;
            let title = attrs.get_opt_string("title");
            let doc = attrs.get_opt_string("doc");
            let show_grid = attrs.get_opt_bool("show_grid")?;
            let num_x_grid_lines = attrs.get_opt_u32("num_x_grid_lines")?;
            let num_y_grid_lines = attrs.get_opt_u32("num_y_grid_lines")?;
            let num_x_labels = attrs.get_opt_u32("num_x_labels")?;
            let num_y_labels = attrs.get_opt_u32("num_y_labels")?;
            let x_axis_title = attrs.get_opt_string("x_axis_title");
            let right_axis_title = attrs.get_opt_string("right_axis_title");
            let right_axis_auto_scale = attrs.get_opt_bool("right_axis_auto_scale")?;
            let right_axis_multi_scale = attrs.get_opt_bool("right_axis_multi_scale")?;
            let left_axis_title = attrs.get_opt_string("left_axis_title");
            let left_axis_auto_scale = attrs.get_opt_bool("left_axis_auto_scale")?;
            let left_axis_multi_scale = attrs.get_opt_bool("left_axis_multi_scale")?;
            let plot_numbers = attrs.get_opt_bool("plot_numbers")?;
            let comparative = attrs.get_opt_bool("comparative")?;
            let from = attrs.get_opt_f64("from")?;
            let to = attrs.get_opt_f64("to")?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut plots = Vec::new();

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"plot" => {
                            plots.push(deserialize_plot(reader, buf)?);
                        }
                        Event::End(e) if e.name().as_ref() == b"graph" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(GraphObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                graph_type: graph_type
                    .ok_or_else(|| DeserializeError::MissingField("graph_type".to_string()))?,
                title,
                doc,
                show_grid: show_grid.unwrap_or(false),
                num_x_grid_lines: num_x_grid_lines.unwrap_or(0),
                num_y_grid_lines: num_y_grid_lines.unwrap_or(0),
                num_x_labels: num_x_labels.unwrap_or(0),
                num_y_labels: num_y_labels.unwrap_or(0),
                x_axis_title,
                right_axis_title,
                right_axis_auto_scale: right_axis_auto_scale.unwrap_or(false),
                right_axis_multi_scale: right_axis_multi_scale.unwrap_or(false),
                left_axis_title,
                left_axis_auto_scale: left_axis_auto_scale.unwrap_or(false),
                left_axis_multi_scale: left_axis_multi_scale.unwrap_or(false),
                plot_numbers: plot_numbers.unwrap_or(false),
                comparative: comparative.unwrap_or(false),
                from,
                to,
                plots,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected graph element".to_string(),
        )),
    }
}

/// Parse TableItemType from string.
fn parse_table_item_type(s: &str) -> Result<TableItemType, DeserializeError> {
    match s {
        "time" => Ok(TableItemType::Time),
        "variable" => Ok(TableItemType::Variable),
        "blank" => Ok(TableItemType::Blank),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid table item type: {}",
            s
        ))),
    }
}

/// Parse TableOrientation from string.
fn parse_table_orientation(s: &str) -> Result<TableOrientation, DeserializeError> {
    match s {
        "horizontal" => Ok(TableOrientation::Horizontal),
        "vertical" => Ok(TableOrientation::Vertical),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid table orientation: {}",
            s
        ))),
    }
}

/// Parse ReportBalances from string.
fn parse_report_balances(s: &str) -> Result<ReportBalances, DeserializeError> {
    match s {
        "beginning" => Ok(ReportBalances::Beginning),
        "ending" => Ok(ReportBalances::Ending),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid report balances: {}",
            s
        ))),
    }
}

/// Parse ReportFlows from string.
fn parse_report_flows(s: &str) -> Result<ReportFlows, DeserializeError> {
    match s {
        "instantaneous" => Ok(ReportFlows::Instantaneous),
        "summed" => Ok(ReportFlows::Summed),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid report flows: {}",
            s
        ))),
    }
}

/// Deserialize a TableItem from XML.
fn deserialize_table_item<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<TableItem, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"item" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let item_type = attrs
                .get_opt("type")
                .map(|s| parse_table_item_type(s))
                .transpose()?;
            let entity_name = attrs.get_opt_string("entity_name");
            let precision = attrs.get_opt_f64("precision")?;
            let delimit_000s = attrs.get_opt_bool("delimit_000s")?;
            let column_width = attrs.get_opt_f64("column_width")?;

            // If it's a start tag, read until end
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"item" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(TableItem {
                item_type: item_type
                    .ok_or_else(|| DeserializeError::MissingField("type".to_string()))?,
                entity_name,
                precision,
                delimit_000s: delimit_000s.unwrap_or(false),
                column_width,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected item element".to_string(),
        )),
    }
}

/// Deserialize a TableObject from XML.
pub fn deserialize_table_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<TableObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"table" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let title = attrs.get_opt_string("title");
            let doc = attrs.get_opt_string("doc");
            let orientation = attrs
                .get_opt("orientation")
                .map(|s| parse_table_orientation(s))
                .transpose()?;
            let column_width = attrs.get_opt_f64("column_width")?;
            let blank_column_width = attrs.get_opt_f64("blank_column_width")?;
            let interval = attrs.get_opt_string("interval");
            let report_balances = attrs
                .get_opt("report_balances")
                .map(|s| parse_report_balances(s))
                .transpose()?;
            let report_flows = attrs
                .get_opt("report_flows")
                .map(|s| parse_report_flows(s))
                .transpose()?;
            let comparative = attrs.get_opt_bool("comparative")?;
            let wrap_text = attrs.get_opt_bool("wrap_text")?;

            // Header style attributes
            let header_font_family = attrs.get_opt_string("header_font_family");
            let header_font_size = attrs
                .get_opt("header_font_size")
                .map(|s| {
                    let size_str = s.trim_end_matches("pt");
                    size_str.parse().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid header_font_size value: {}", e))
                    })
                })
                .transpose()?;
            let header_font_weight = attrs
                .get_opt("header_font_weight")
                .map(|s| parse_font_weight(s))
                .transpose()?;
            let header_font_style = attrs
                .get_opt("header_font_style")
                .map(|s| parse_font_style(s))
                .transpose()?;
            let header_text_decoration = attrs
                .get_opt("header_text_decoration")
                .map(|s| parse_text_decoration(s))
                .transpose()?;
            let header_text_align = attrs
                .get_opt("header_text_align")
                .map(|s| parse_text_align(s))
                .transpose()?;
            let header_vertical_text_align = attrs
                .get_opt("header_vertical_text_align")
                .map(|s| parse_vertical_text_align(s))
                .transpose()?;
            let header_text_background = attrs
                .get_opt("header_text_background")
                .map(|s| parse_color(s))
                .transpose()?;
            let header_text_padding = attrs
                .get_opt("header_text_padding")
                .map(|s| parse_text_padding(s))
                .transpose()?;
            let header_font_color = attrs
                .get_opt("header_font_color")
                .map(|s| parse_color(s))
                .transpose()?;
            let header_text_border_color = attrs
                .get_opt("header_text_border_color")
                .map(|s| parse_color(s))
                .transpose()?;
            let header_text_border_width = attrs
                .get_opt("header_text_border_width")
                .map(|s| parse_border_width(s))
                .transpose()?;
            let header_text_border_style = attrs
                .get_opt("header_text_border_style")
                .map(|s| match s {
                    "none" => Ok(BorderStyle::None),
                    "solid" => Ok(BorderStyle::Solid),
                    _ => Err(DeserializeError::Custom(format!(
                        "Invalid header_text_border_style: {}",
                        s
                    ))),
                })
                .transpose()?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut items = Vec::new();

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"item" => {
                            items.push(deserialize_table_item(reader, buf)?);
                        }
                        Event::End(e) if e.name().as_ref() == b"table" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(TableObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                title,
                doc,
                orientation: orientation
                    .ok_or_else(|| DeserializeError::MissingField("orientation".to_string()))?,
                column_width: column_width
                    .ok_or_else(|| DeserializeError::MissingField("column_width".to_string()))?,
                blank_column_width,
                interval: interval
                    .ok_or_else(|| DeserializeError::MissingField("interval".to_string()))?,
                report_balances: report_balances
                    .ok_or_else(|| DeserializeError::MissingField("report_balances".to_string()))?,
                report_flows: report_flows
                    .ok_or_else(|| DeserializeError::MissingField("report_flows".to_string()))?,
                comparative: comparative.unwrap_or(false),
                wrap_text: wrap_text.unwrap_or(false),
                items,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
                header_font_family,
                header_font_size,
                header_font_weight,
                header_font_style,
                header_text_decoration,
                header_text_align,
                header_vertical_text_align,
                header_text_background,
                header_text_padding,
                header_font_color,
                header_text_border_color,
                header_text_border_width,
                header_text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected table element".to_string(),
        )),
    }
}

/// Parse TextBoxAppearance from string.
fn parse_text_box_appearance(s: &str) -> Result<TextBoxAppearance, DeserializeError> {
    match s {
        "transparent" => Ok(TextBoxAppearance::Transparent),
        "normal" => Ok(TextBoxAppearance::Normal),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid text box appearance: {}",
            s
        ))),
    }
}

/// Deserialize a TextBoxObject from XML.
pub fn deserialize_text_box_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<TextBoxObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"text_box" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let appearance = attrs
                .get_opt("appearance")
                .map(|s| parse_text_box_appearance(s))
                .transpose()?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            // Read text content
            let content = read_text_content(reader, buf)?;

            Ok(TextBoxObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                appearance: appearance
                    .ok_or_else(|| DeserializeError::MissingField("appearance".to_string()))?,
                content,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected text_box element".to_string(),
        )),
    }
}

/// Deserialize GraphicsFrameContent from XML.
fn deserialize_graphics_frame_content<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<GraphicsFrameContent, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"image" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let size_to_parent = attrs.get_opt_bool("size_to_parent")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let resource = attrs.get_opt_string("resource");
            let data = attrs.get_opt_string("data");

            // If it's a start tag, read until end
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"image" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(GraphicsFrameContent::Image(ImageContent {
                size_to_parent: size_to_parent.unwrap_or(false),
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                resource,
                data,
            }))
        }
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"video" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let size_to_parent = attrs.get_opt_bool("size_to_parent")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let resource = attrs.get_opt_string("resource");

            // If it's a start tag, read until end
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"video" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(GraphicsFrameContent::Video(VideoContent {
                size_to_parent: size_to_parent.unwrap_or(false),
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                resource: resource
                    .ok_or_else(|| DeserializeError::MissingField("resource".to_string()))?,
            }))
        }
        _ => Err(DeserializeError::Custom(
            "Expected image or video element".to_string(),
        )),
    }
}

/// Deserialize a GraphicsFrameObject from XML.
pub fn deserialize_graphics_frame_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<GraphicsFrameObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"graphics_frame" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let border_color = attrs
                .get_opt("border_color")
                .map(|s| parse_color(s))
                .transpose()?;
            let border_style = attrs
                .get_opt("border_style")
                .map(|s| match s {
                    "none" => Ok(BorderStyle::None),
                    "solid" => Ok(BorderStyle::Solid),
                    _ => Err(DeserializeError::Custom(format!(
                        "Invalid border_style: {}",
                        s
                    ))),
                })
                .transpose()?;
            let border_width = attrs
                .get_opt("border_width")
                .map(|s| parse_border_width(s))
                .transpose()?;

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut content: Option<GraphicsFrameContent> = None;

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e)
                            if e.name().as_ref() == b"image" || e.name().as_ref() == b"video" =>
                        {
                            content = Some(deserialize_graphics_frame_content(reader, buf)?);
                        }
                        Event::End(e) if e.name().as_ref() == b"graphics_frame" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(GraphicsFrameObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                border_color,
                border_style,
                border_width,
                content: content
                    .ok_or_else(|| DeserializeError::MissingField("content".to_string()))?,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected graphics_frame element".to_string(),
        )),
    }
}

/// Parse ButtonAppearance from string.
fn parse_button_appearance(s: &str) -> Result<ButtonAppearance, DeserializeError> {
    match s {
        "opaque" => Ok(ButtonAppearance::Opaque),
        "transparent" => Ok(ButtonAppearance::Transparent),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid button appearance: {}",
            s
        ))),
    }
}

/// Parse ButtonStyle from string.
fn parse_button_style(s: &str) -> Result<ButtonStyle, DeserializeError> {
    match s {
        "square" => Ok(ButtonStyle::Square),
        "rounded" => Ok(ButtonStyle::Rounded),
        "capsule" => Ok(ButtonStyle::Capsule),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid button style: {}",
            s
        ))),
    }
}

/// Parse LinkEffect from string.
fn parse_link_effect(s: &str) -> Result<LinkEffect, DeserializeError> {
    match s {
        "dissolve" => Ok(LinkEffect::Dissolve),
        "checkerboard" => Ok(LinkEffect::Checkerboard),
        "bars" => Ok(LinkEffect::Bars),
        "wipe_left" => Ok(LinkEffect::WipeLeft),
        "wipe_right" => Ok(LinkEffect::WipeRight),
        "wipe_top" => Ok(LinkEffect::WipeTop),
        "wipe_bottom" => Ok(LinkEffect::WipeBottom),
        "wipe_clockwise" => Ok(LinkEffect::WipeClockwise),
        "wipe_counterclockwise" => Ok(LinkEffect::WipeCounterclockwise),
        "iris_in" => Ok(LinkEffect::IrisIn),
        "iris_out" => Ok(LinkEffect::IrisOut),
        "doors_close" => Ok(LinkEffect::DoorsClose),
        "doors_open" => Ok(LinkEffect::DoorsOpen),
        "venetian_left" => Ok(LinkEffect::VenetianLeft),
        "venetian_right" => Ok(LinkEffect::VenetianRight),
        "venetian_top" => Ok(LinkEffect::VenetianTop),
        "venetian_bottom" => Ok(LinkEffect::VenetianBottom),
        "push_bottom" => Ok(LinkEffect::PushBottom),
        "push_top" => Ok(LinkEffect::PushTop),
        "push_left" => Ok(LinkEffect::PushLeft),
        "push_right" => Ok(LinkEffect::PushRight),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid link effect: {}",
            s
        ))),
    }
}

/// Deserialize a Link from XML.
fn deserialize_link<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Link, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"link" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let zoom = attrs.get_opt_f64("zoom")?;
            let effect = attrs
                .get_opt("effect")
                .map(|s| parse_link_effect(s))
                .transpose()?;
            let to_black = attrs.get_opt_bool("to_black")?;
            let target_type = attrs.get_opt_string("target");
            let view_type = attrs.get_opt_string("view_type");
            let order = attrs.get_opt_string("order");
            let page = attrs.get_opt_string("page");
            let url = attrs.get_opt_string("url");

            // Build LinkTarget from parsed attributes
            let target = match target_type.as_deref() {
                Some("view") => LinkTarget::View {
                    view_type: view_type
                        .ok_or_else(|| DeserializeError::MissingField("view_type".to_string()))?,
                    order: order
                        .ok_or_else(|| DeserializeError::MissingField("order".to_string()))?,
                },
                Some("page") => LinkTarget::Page {
                    view_type: view_type
                        .ok_or_else(|| DeserializeError::MissingField("view_type".to_string()))?,
                    order: order
                        .ok_or_else(|| DeserializeError::MissingField("order".to_string()))?,
                    page: page.ok_or_else(|| DeserializeError::MissingField("page".to_string()))?,
                },
                Some("next_page") => LinkTarget::NextPage,
                Some("previous_page") => LinkTarget::PreviousPage,
                Some("home_page") => LinkTarget::HomePage,
                Some("next_view") => LinkTarget::NextView,
                Some("previous_view") => LinkTarget::PreviousView,
                Some("home_view") => LinkTarget::HomeView,
                Some("back_page") => LinkTarget::BackPage,
                Some("back_view") => LinkTarget::BackView,
                Some("url") => LinkTarget::Url(
                    url.ok_or_else(|| DeserializeError::MissingField("url".to_string()))?,
                ),
                _ => return Err(DeserializeError::MissingField("target".to_string())),
            };

            // If it's a start tag, read until end
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"link" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(Link {
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                zoom: zoom.ok_or_else(|| DeserializeError::MissingField("zoom".to_string()))?,
                effect,
                to_black: to_black.unwrap_or(false),
                target,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected link element".to_string(),
        )),
    }
}

/// Parse FileAction from string.
fn parse_file_action(s: &str) -> Result<FileAction, DeserializeError> {
    match s {
        "open" => Ok(FileAction::Open),
        "close" => Ok(FileAction::Close),
        "save" => Ok(FileAction::Save),
        "save_as" => Ok(FileAction::SaveAs),
        "save_as_image" => Ok(FileAction::SaveAsImage),
        "revert" => Ok(FileAction::Revert),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid file action: {}",
            s
        ))),
    }
}

/// Parse PrintingAction from string.
fn parse_printing_action(s: &str) -> Result<PrintingAction, DeserializeError> {
    match s {
        "print_setup" => Ok(PrintingAction::PrintSetup),
        "print" => Ok(PrintingAction::Print),
        "print_screen" => Ok(PrintingAction::PrintScreen),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid printing action: {}",
            s
        ))),
    }
}

/// Parse SimulationAction from string.
fn parse_simulation_action(s: &str) -> Result<SimulationAction, DeserializeError> {
    match s {
        "run" => Ok(SimulationAction::Run),
        "pause" => Ok(SimulationAction::Pause),
        "resume" => Ok(SimulationAction::Resume),
        "stop" => Ok(SimulationAction::Stop),
        "run_restore" => Ok(SimulationAction::RunRestore),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid simulation action: {}",
            s
        ))),
    }
}

/// Parse RestoreAction from string.
fn parse_restore_action(s: &str) -> Result<RestoreAction, DeserializeError> {
    match s {
        "restore_all" => Ok(RestoreAction::RestoreAll),
        "restore_sliders" => Ok(RestoreAction::RestoreSliders),
        "restore_knobs" => Ok(RestoreAction::RestoreKnobs),
        "restore_list_inputs" => Ok(RestoreAction::RestoreListInputs),
        "restore_graphical_inputs" => Ok(RestoreAction::RestoreGraphicalInputs),
        "restore_switches" => Ok(RestoreAction::RestoreSwitches),
        "restore_numeric_displays" => Ok(RestoreAction::RestoreNumericDisplays),
        "restore_graphs_tables" => Ok(RestoreAction::RestoreGraphsTables),
        "restore_lamps_gauges" => Ok(RestoreAction::RestoreLampsGauges),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid restore action: {}",
            s
        ))),
    }
}

/// Deserialize DataAction from XML.
fn deserialize_data_action<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<DataAction, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"data_action" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let action = attrs.get_opt_string("action");
            let run_name = attrs.get_opt_string("run_name");
            let resource = attrs.get_opt_string("resource");
            let worksheet = attrs.get_opt_string("worksheet");
            let all = attrs.get_opt_bool("all")?;

            // If it's a start tag, read until end
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"data_action" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            match action.as_deref() {
                Some("data_manager") => Ok(DataAction::DataManager),
                Some("save_data_now") => Ok(DataAction::SaveDataNow {
                    run_name: run_name
                        .ok_or_else(|| DeserializeError::MissingField("run_name".to_string()))?,
                }),
                Some("import_now") => Ok(DataAction::ImportNow {
                    resource: resource
                        .ok_or_else(|| DeserializeError::MissingField("resource".to_string()))?,
                    worksheet,
                    all: all.unwrap_or(false),
                }),
                Some("export_now") => Ok(DataAction::ExportNow {
                    resource: resource
                        .ok_or_else(|| DeserializeError::MissingField("resource".to_string()))?,
                    worksheet,
                    all: all.unwrap_or(false),
                }),
                _ => Err(DeserializeError::MissingField("action".to_string())),
            }
        }
        _ => Err(DeserializeError::Custom(
            "Expected data_action element".to_string(),
        )),
    }
}

/// Parse MiscellaneousAction from string.
fn parse_miscellaneous_action(s: &str) -> Result<MiscellaneousAction, DeserializeError> {
    match s {
        "exit" => Ok(MiscellaneousAction::Exit),
        "find" => Ok(MiscellaneousAction::Find),
        "run_specs" => Ok(MiscellaneousAction::RunSpecs),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid miscellaneous action: {}",
            s
        ))),
    }
}

/// Deserialize MenuAction from XML.
fn deserialize_menu_action<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<MenuAction, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"menu_action" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let action_type = attrs.get_opt_string("type");
            let action = attrs.get_opt_string("action");

            // If it's a start tag, read child elements (for DataAction)
            let mut data_action: Option<DataAction> = None;
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"data_action" => {
                            data_action = Some(deserialize_data_action(reader, buf)?);
                        }
                        Event::End(e) if e.name().as_ref() == b"menu_action" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            match action_type.as_deref() {
                Some("file") => Ok(MenuAction::File(parse_file_action(
                    action
                        .ok_or_else(|| DeserializeError::MissingField("action".to_string()))?
                        .as_str(),
                )?)),
                Some("printing") => Ok(MenuAction::Printing(parse_printing_action(
                    action
                        .ok_or_else(|| DeserializeError::MissingField("action".to_string()))?
                        .as_str(),
                )?)),
                Some("simulation") => Ok(MenuAction::Simulation(parse_simulation_action(
                    action
                        .ok_or_else(|| DeserializeError::MissingField("action".to_string()))?
                        .as_str(),
                )?)),
                Some("restore") => Ok(MenuAction::Restore(parse_restore_action(
                    action
                        .ok_or_else(|| DeserializeError::MissingField("action".to_string()))?
                        .as_str(),
                )?)),
                Some("data") => Ok(MenuAction::Data(data_action.ok_or_else(|| {
                    DeserializeError::MissingField("data_action".to_string())
                })?)),
                Some("miscellaneous") => Ok(MenuAction::Miscellaneous(parse_miscellaneous_action(
                    action
                        .ok_or_else(|| DeserializeError::MissingField("action".to_string()))?
                        .as_str(),
                )?)),
                _ => Err(DeserializeError::MissingField("type".to_string())),
            }
        }
        _ => Err(DeserializeError::Custom(
            "Expected menu_action element".to_string(),
        )),
    }
}

/// Deserialize PopupContent from XML.
fn deserialize_popup_content<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<PopupContent, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"text_box" => Ok(PopupContent::TextBox(
            deserialize_text_box_object(reader, buf)?,
        )),
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"image" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let size_to_parent = attrs.get_opt_bool("size_to_parent")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let resource = attrs.get_opt_string("resource");
            let data = attrs.get_opt_string("data");

            // If it's a start tag, read until end
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"image" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(PopupContent::Image(ImageContent {
                size_to_parent: size_to_parent.unwrap_or(false),
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                resource,
                data,
            }))
        }
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"video" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let size_to_parent = attrs.get_opt_bool("size_to_parent")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let resource = attrs.get_opt_string("resource");

            // If it's a start tag, read until end
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == b"video" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(PopupContent::Video(VideoContent {
                size_to_parent: size_to_parent.unwrap_or(false),
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                resource: resource
                    .ok_or_else(|| DeserializeError::MissingField("resource".to_string()))?,
            }))
        }
        _ => Err(DeserializeError::Custom(
            "Expected text_box, image, or video element".to_string(),
        )),
    }
}

/// Deserialize a ButtonObject from XML.
pub fn deserialize_button_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<ButtonObject, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"button" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let uid = attrs.get_opt_i32("uid")?;
            let x = attrs.get_opt_f64("x")?;
            let y = attrs.get_opt_f64("y")?;
            let width = attrs.get_opt_f64("width")?;
            let height = attrs.get_opt_f64("height")?;
            let appearance = attrs
                .get_opt("appearance")
                .map(|s| parse_button_appearance(s))
                .transpose()?;
            let style = attrs
                .get_opt("style")
                .map(|s| parse_button_style(s))
                .transpose()?;
            let label = attrs.get_opt_string("label");
            let clicking_sound = attrs.get_opt_bool("clicking_sound")?;
            let sound = attrs.get_opt_string("sound");

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut image: Option<ImageContent> = None;
            let mut popup: Option<PopupContent> = None;
            let mut link: Option<Link> = None;
            let mut menu_action: Option<MenuAction> = None;
            let mut switch_action: Option<SwitchAction> = None;

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"image" => {
                            let attrs = Attrs::from_start(&e, reader)?;
                            let size_to_parent = attrs.get_opt_bool("size_to_parent")?;
                            let width = attrs.get_opt_f64("width")?;
                            let height = attrs.get_opt_f64("height")?;
                            let resource = attrs.get_opt_string("resource");
                            let data = attrs.get_opt_string("data");

                            // If it's a start tag, read until end
                            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                                loop {
                                    match reader.read_event_into(buf)? {
                                        Event::End(e) if e.name().as_ref() == b"image" => break,
                                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                        _ => {}
                                    }
                                    buf.clear();
                                }
                            }
                            buf.clear();

                            image = Some(ImageContent {
                                size_to_parent: size_to_parent.unwrap_or(false),
                                width: width.ok_or_else(|| {
                                    DeserializeError::MissingField("width".to_string())
                                })?,
                                height: height.ok_or_else(|| {
                                    DeserializeError::MissingField("height".to_string())
                                })?,
                                resource,
                                data,
                            });
                        }
                        Event::Start(e) if e.name().as_ref() == b"popup" => {
                            // Popup can contain text_box, image, or video
                            loop {
                                match reader.read_event_into(buf)? {
                                    Event::Start(_) => {
                                        popup = Some(deserialize_popup_content(reader, buf)?);
                                        break;
                                    }
                                    Event::End(e) if e.name().as_ref() == b"popup" => break,
                                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                    _ => {}
                                }
                                buf.clear();
                            }
                        }
                        Event::Start(e) if e.name().as_ref() == b"link" => {
                            link = Some(deserialize_link(reader, buf)?);
                        }
                        Event::Start(e) if e.name().as_ref() == b"menu_action" => {
                            menu_action = Some(deserialize_menu_action(reader, buf)?);
                        }
                        Event::Start(e) | Event::Empty(e)
                            if e.name().as_ref() == b"switch_action" =>
                        {
                            let attrs = Attrs::from_start(&e, reader)?;
                            let entity_name = attrs.get_opt_string("entity_name");
                            let group_name = attrs.get_opt_string("group_name");
                            let module_name = attrs.get_opt_string("module_name");
                            let value = attrs.get_opt_f64("value")?;

                            // If it's a start tag, read until end
                            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                                loop {
                                    match reader.read_event_into(buf)? {
                                        Event::End(e) if e.name().as_ref() == b"switch_action" => {
                                            break;
                                        }
                                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                        _ => {}
                                    }
                                    buf.clear();
                                }
                            }
                            buf.clear();

                            switch_action = Some(SwitchAction {
                                entity_name,
                                group_name,
                                module_name,
                                value: value.ok_or_else(|| {
                                    DeserializeError::MissingField("value".to_string())
                                })?,
                            });
                        }
                        Event::End(e) if e.name().as_ref() == b"button" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(ButtonObject {
                uid: crate::Uid::new(
                    uid.ok_or_else(|| DeserializeError::MissingField("uid".to_string()))?,
                ),
                x: x.ok_or_else(|| DeserializeError::MissingField("x".to_string()))?,
                y: y.ok_or_else(|| DeserializeError::MissingField("y".to_string()))?,
                width: width.ok_or_else(|| DeserializeError::MissingField("width".to_string()))?,
                height: height
                    .ok_or_else(|| DeserializeError::MissingField("height".to_string()))?,
                appearance: appearance
                    .ok_or_else(|| DeserializeError::MissingField("appearance".to_string()))?,
                style: style.ok_or_else(|| DeserializeError::MissingField("style".to_string()))?,
                label,
                image,
                clicking_sound: clicking_sound.unwrap_or(false),
                sound,
                popup,
                link,
                menu_action,
                switch_action,
                color: display_attrs.color,
                background: display_attrs.background,
                z_index: display_attrs.z_index,
                font_family: display_attrs.font_family,
                font_size: display_attrs.font_size,
                font_weight: display_attrs.font_weight,
                font_style: display_attrs.font_style,
                text_decoration: display_attrs.text_decoration,
                text_align: display_attrs.text_align,
                text_background: display_attrs.text_background,
                vertical_text_align: display_attrs.vertical_text_align,
                text_padding: display_attrs.text_padding,
                font_color: display_attrs.font_color,
                text_border_color: display_attrs.text_border_color,
                text_border_width: display_attrs.text_border_width,
                text_border_style: display_attrs.text_border_style,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected button element".to_string(),
        )),
    }
}

/// Deserialize a Module from XML.
#[cfg(feature = "submodels")]
pub fn deserialize_module<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Module, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"module" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let name = attrs
                .get_opt("name")
                .map(|s| {
                    Identifier::parse_from_attribute(s).map_err(|err| {
                        DeserializeError::Custom(format!("Invalid module name: {}", err))
                    })
                })
                .transpose()?;
            let resource = attrs.get_opt_string("resource");

            let mut connections = Vec::new();
            let mut documentation: Option<Documentation> = None;

            loop {
                match reader.read_event_into(buf)? {
                    Event::Start(e) if e.name().as_ref() == b"connect" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let to = attrs.get_opt_string("to");
                        let from = attrs.get_opt_string("from");

                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(e) if e.name().as_ref() == b"connect" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        buf.clear();

                        if let (Some(to_val), Some(from_val)) = (to, from) {
                            connections.push(ModuleConnection {
                                to: to_val,
                                from: from_val,
                            });
                        }
                    }
                    Event::Empty(e) if e.name().as_ref() == b"connect" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let to = attrs.get_opt_string("to");
                        let from = attrs.get_opt_string("from");

                        if let (Some(to_val), Some(from_val)) = (to, from) {
                            connections.push(ModuleConnection {
                                to: to_val,
                                from: from_val,
                            });
                        }

                        buf.clear();
                    }
                    Event::Start(e) if e.name().as_ref() == b"doc" => {
                        let doc_text = read_text_content(reader, buf)?;
                        // Determine if it's HTML or plain text
                        documentation = Some(
                            if doc_text.trim().contains('<') && doc_text.trim().contains('>') {
                                Documentation::Html(doc_text)
                            } else {
                                Documentation::PlainText(doc_text)
                            },
                        );
                    }
                    Event::End(e) if e.name().as_ref() == b"module" => break,
                    Event::Eof => return Err(DeserializeError::UnexpectedEof),
                    _ => {}
                }
                buf.clear();
            }

            buf.clear();

            Ok(Module {
                name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
                resource,
                connections,
                documentation,
            })
        }
        Event::Empty(e) if e.name().as_ref() == b"module" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let name = attrs
                .get_opt("name")
                .map(|s| {
                    Identifier::parse_from_attribute(s).map_err(|err| {
                        DeserializeError::Custom(format!("Invalid module name: {}", err))
                    })
                })
                .transpose()?;
            let resource = attrs.get_opt_string("resource");

            buf.clear();

            Ok(Module {
                name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
                resource,
                connections: Vec::new(),
                documentation: None,
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected module element".to_string(),
        )),
    }
}

/// Deserialize a Module from XML when the start tag has already been consumed.
///
/// This is called when deserialize_variables_impl has already matched the <module> start tag.
#[cfg(feature = "submodels")]
#[allow(dead_code)]
pub(crate) fn deserialize_module_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<Identifier>,
    resource: Option<String>,
    is_empty_tag: bool,
) -> Result<Module, DeserializeError> {
    let mut connections = Vec::new();
    let mut documentation: Option<Documentation> = None;

    if !is_empty_tag {
        loop {
            match reader.read_event_into(buf)? {
                Event::Start(e) if e.name().as_ref() == b"connect" => {
                    let attrs = Attrs::from_start(&e, reader)?;
                    let to = attrs.get_opt_string("to");
                    let from = attrs.get_opt_string("from");

                    loop {
                        match reader.read_event_into(buf)? {
                            Event::End(e) if e.name().as_ref() == b"connect" => break,
                            Event::Eof => return Err(DeserializeError::UnexpectedEof),
                            _ => {}
                        }
                        buf.clear();
                    }
                    buf.clear();

                    if let (Some(to_val), Some(from_val)) = (to, from) {
                        connections.push(ModuleConnection {
                            to: to_val,
                            from: from_val,
                        });
                    }
                }
                Event::Empty(e) if e.name().as_ref() == b"connect" => {
                    let attrs = Attrs::from_start(&e, reader)?;
                    let to = attrs.get_opt_string("to");
                    let from = attrs.get_opt_string("from");

                    if let (Some(to_val), Some(from_val)) = (to, from) {
                        connections.push(ModuleConnection {
                            to: to_val,
                            from: from_val,
                        });
                    }

                    buf.clear();
                }
                Event::Start(e) if e.name().as_ref() == b"doc" => {
                    let doc_text = read_text_content(reader, buf)?;
                    // Determine if it's HTML or plain text
                    documentation = Some(
                        if doc_text.trim().contains('<') && doc_text.trim().contains('>') {
                            Documentation::Html(doc_text)
                        } else {
                            Documentation::PlainText(doc_text)
                        },
                    );
                }
                Event::End(e) if e.name().as_ref() == b"module" => break,
                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                _ => {}
            }
            buf.clear();
        }
    }

    buf.clear();

    Ok(Module {
        name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
        resource,
        connections,
        documentation,
    })
}

/// Deserialize a Group from XML.
pub fn deserialize_group<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Group, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"group" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let name = attrs
                .get_opt("name")
                .map(|s| {
                    Identifier::parse_from_attribute(s)
                        .map_err(|e| DeserializeError::Custom(format!("Invalid group name: {}", e)))
                })
                .transpose()?;

            let mut doc: Option<Documentation> = None;
            let mut entities = Vec::new();

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"doc" => {
                            let doc_text = read_text_content(reader, buf)?;
                            // Determine if it's HTML or plain text
                            doc = Some(
                                if doc_text.trim().contains('<') && doc_text.trim().contains('>') {
                                    Documentation::Html(doc_text)
                                } else {
                                    Documentation::PlainText(doc_text)
                                },
                            );
                        }
                        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"entity" => {
                            let attrs = Attrs::from_start(&e, reader)?;
                            let entity_name = attrs
                                .get_opt("name")
                                .map(|s| {
                                    Identifier::parse_from_attribute(s).map_err(|e| {
                                        DeserializeError::Custom(format!(
                                            "Invalid entity name: {}",
                                            e
                                        ))
                                    })
                                })
                                .transpose()?;
                            let run = attrs.get_opt_bool("run")?;

                            // If it's a start tag, read until end
                            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                                loop {
                                    match reader.read_event_into(buf)? {
                                        Event::End(e) if e.name().as_ref() == b"entity" => break,
                                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                        _ => {}
                                    }
                                    buf.clear();
                                }
                            }
                            buf.clear();

                            if let Some(name) = entity_name {
                                entities.push(GroupEntity {
                                    name,
                                    run: run.unwrap_or(false),
                                });
                            }
                        }
                        Event::End(e) if e.name().as_ref() == b"group" => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
            }
            buf.clear();

            Ok(Group {
                name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
                doc,
                entities,
                display: Vec::new(), // Display UIDs are handled separately in views
            })
        }
        _ => Err(DeserializeError::Custom(
            "Expected group element".to_string(),
        )),
    }
}

/// Parse ViewType from string.
fn parse_view_type(s: &str) -> Result<ViewType, DeserializeError> {
    if s == "stock_flow" {
        Ok(ViewType::StockFlow)
    } else if s == "interface" {
        Ok(ViewType::Interface)
    } else if s == "popup" {
        Ok(ViewType::Popup)
    } else if let Some(colon_pos) = s.find(':') {
        let (vendor_str, type_str) = s.split_at(colon_pos);
        let vendor = match vendor_str.to_lowercase().as_str() {
            "anylogic" => crate::Vendor::Anylogic,
            "forio" => crate::Vendor::Forio,
            "insightmaker" => crate::Vendor::Insightmaker,
            "isee" => crate::Vendor::Isee,
            "powersim" => crate::Vendor::Powersim,
            "simanticssd" => crate::Vendor::Simanticssd,
            "simile" => crate::Vendor::Simile,
            "sysdea" => crate::Vendor::Sysdea,
            "vensim" => crate::Vendor::Vensim,
            "simlab" => crate::Vendor::SimLab,
            _ => crate::Vendor::Other,
        };
        Ok(ViewType::VendorSpecific(vendor, type_str[1..].to_string()))
    } else {
        Err(DeserializeError::Custom(format!(
            "Invalid view type: {}",
            s
        )))
    }
}

/// Helper to read an optional expression from an element.
pub fn read_optional_expression<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    element_name: &str,
) -> Result<Option<Expression>, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == element_name.as_bytes() => {
            let expr = read_expression(reader, buf)?;
            Ok(Some(expr))
        }
        Event::Empty(e) if e.name().as_ref() == element_name.as_bytes() => Ok(None),
        _ => Ok(None),
    }
}

/// Deserialize an Auxiliary variable from XML.
///
/// This function expects the reader to be positioned at the start of an <aux> element.
pub fn deserialize_auxiliary<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Auxiliary, DeserializeError> {
    // Expect <aux> start tag
    let (name, access, autoexport) = match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"aux" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let name = attrs
                .get_opt("name")
                .map(|s| {
                    Identifier::parse_from_attribute(s)
                        .map_err(|e| DeserializeError::Custom(format!("Invalid identifier: {}", e)))
                })
                .transpose()?;
            let access = attrs
                .get_opt("access")
                .map(|s| match s {
                    "input" => Ok(crate::model::vars::AccessType::Input),
                    "output" => Ok(crate::model::vars::AccessType::Output),
                    _ => Err(DeserializeError::Custom(format!(
                        "Invalid access type: {}",
                        s
                    ))),
                })
                .transpose()?;
            let autoexport = attrs.get_opt_bool("autoexport")?;
            (name, access, autoexport)
        }
        Event::Start(e) => {
            return Err(DeserializeError::UnexpectedElement {
                expected: "aux".to_string(),
                found: String::from_utf8_lossy(e.name().as_ref()).to_string(),
            });
        }
        _ => {
            return Err(DeserializeError::Custom(
                "Expected aux start tag".to_string(),
            ));
        }
    };
    buf.clear();

    let mut equation: Option<Expression> = None;
    #[cfg(feature = "mathml")]
    let mut mathml_equation: Option<String> = None;
    let mut documentation: Option<Documentation> = None;
    let mut units: Option<UnitEquation> = None;
    let mut range: Option<DeviceRange> = None;
    let mut scale: Option<DeviceScale> = None;
    let mut format: Option<FormatOptions> = None;
    let mut _event_poster: Option<EventPoster> = None;
    #[cfg(feature = "arrays")]
    let mut _dimensions: Option<VariableDimensions> = None;
    #[cfg(feature = "arrays")]
    let mut elements: Vec<ArrayElement> = Vec::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"eqn" => {
                        equation = Some(read_expression(reader, buf)?);
                    }
                    #[cfg(feature = "mathml")]
                    b"mathml" => {
                        mathml_equation = Some(read_text_content(reader, buf)?);
                    }
                    b"doc" => {
                        let doc_text = read_text_content(reader, buf)?;
                        // Determine if it's HTML or plain text
                        documentation = Some(
                            if doc_text.trim().contains('<') && doc_text.trim().contains('>') {
                                Documentation::Html(doc_text)
                            } else {
                                Documentation::PlainText(doc_text)
                            },
                        );
                    }
                    b"units" => {
                        let units_str = read_text_content(reader, buf)?;
                        let (remaining, unit_eqn) = unit_equation(&units_str).map_err(|e| {
                            DeserializeError::Custom(format!(
                                "Failed to parse unit equation: {}",
                                e
                            ))
                        })?;
                        if !remaining.is_empty() {
                            return Err(DeserializeError::Custom(format!(
                                "Unexpected trailing characters after unit equation: '{}'",
                                remaining
                            )));
                        }
                        units = Some(unit_eqn);
                    }
                    b"range" => {
                        range = Some(deserialize_range(reader, buf)?);
                    }
                    b"scale" => {
                        scale = Some(deserialize_scale(reader, buf)?);
                    }
                    b"format" => {
                        format = Some(deserialize_format(reader, buf)?);
                    }
                    b"event_poster" => {
                        _event_poster = Some(deserialize_event_poster(reader, buf)?);
                    }
                    #[cfg(feature = "arrays")]
                    b"dimensions" => {
                        _dimensions = Some(deserialize_dimensions(reader, buf)?);
                    }
                    #[cfg(feature = "arrays")]
                    b"element" => {
                        let element = deserialize_array_element(reader, buf)?;
                        elements.push(element);
                    }
                    _ => {
                        // Skip unknown elements using the helper
                        let element_name = e.name().as_ref().to_vec();
                        skip_element(reader, buf, &element_name)?;
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"aux" => {
                break;
            }
            Event::Eof => {
                return Err(DeserializeError::UnexpectedEof);
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(Auxiliary {
        name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
        access,
        autoexport,
        documentation,
        equation: equation.ok_or_else(|| DeserializeError::MissingField("eqn".to_string()))?,
        #[cfg(feature = "mathml")]
        mathml_equation,
        units,
        range,
        scale,
        format,
        #[cfg(feature = "arrays")]
        dimensions: _dimensions,
        #[cfg(feature = "arrays")]
        elements,
        event_poster: _event_poster,
    })
}

/// Deserialize a BasicFlow variable from XML.
///
/// This function expects the reader to be positioned at the start of a <flow> element.
pub fn deserialize_basic_flow<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<BasicFlow, DeserializeError> {
    // Expect <flow> start tag
    let (name, access, autoexport) = match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"flow" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let name = attrs
                .get_opt("name")
                .map(|s| {
                    Identifier::parse_from_attribute(s)
                        .map_err(|e| DeserializeError::Custom(format!("Invalid identifier: {}", e)))
                })
                .transpose()?;
            let access = attrs
                .get_opt("access")
                .map(|s| match s {
                    "input" => Ok(crate::model::vars::AccessType::Input),
                    "output" => Ok(crate::model::vars::AccessType::Output),
                    _ => Err(DeserializeError::Custom(format!(
                        "Invalid access type: {}",
                        s
                    ))),
                })
                .transpose()?;
            let autoexport = attrs.get_opt_bool("autoexport")?;
            (name, access, autoexport)
        }
        Event::Start(e) => {
            return Err(DeserializeError::UnexpectedElement {
                expected: "flow".to_string(),
                found: String::from_utf8_lossy(e.name().as_ref()).to_string(),
            });
        }
        _ => {
            return Err(DeserializeError::Custom(
                "Expected flow start tag".to_string(),
            ));
        }
    };
    buf.clear();
    deserialize_basic_flow_impl(reader, buf, name, access, autoexport, false)
}

/// Internal implementation of basic flow deserialization.
pub(crate) fn deserialize_basic_flow_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<Identifier>,
    access: Option<crate::model::vars::AccessType>,
    autoexport: Option<bool>,
    _is_empty_tag: bool,
) -> Result<BasicFlow, DeserializeError> {
    let mut equation: Option<Expression> = None;
    let mut mathml_equation: Option<String> = None;
    let mut documentation: Option<Documentation> = None;
    let mut units: Option<UnitEquation> = None;
    let mut non_negative: Option<Option<bool>> = None;
    let mut multiplier: Option<f64> = None;
    let mut range: Option<DeviceRange> = None;
    let mut scale: Option<DeviceScale> = None;
    let mut format: Option<FormatOptions> = None;
    let mut _event_poster: Option<EventPoster> = None;
    #[cfg(feature = "arrays")]
    let mut _dimensions: Option<VariableDimensions> = None;
    #[cfg(feature = "arrays")]
    let mut elements: Vec<ArrayElement> = Vec::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"eqn" => {
                        equation = Some(read_expression(reader, buf)?);
                    }
                    b"mathml" => {
                        mathml_equation = Some(read_text_content(reader, buf)?);
                    }
                    b"multiplier" => {
                        multiplier = Some(read_number_content(reader, buf)?);
                    }
                    b"doc" => {
                        let doc_text = read_text_content(reader, buf)?;
                        documentation = Some(
                            if doc_text.trim().contains('<') && doc_text.trim().contains('>') {
                                Documentation::Html(doc_text)
                            } else {
                                Documentation::PlainText(doc_text)
                            },
                        );
                    }
                    b"units" => {
                        let units_str = read_text_content(reader, buf)?;
                        let (remaining, unit_eqn) = unit_equation(&units_str).map_err(|e| {
                            DeserializeError::Custom(format!(
                                "Failed to parse unit equation: {}",
                                e
                            ))
                        })?;
                        if !remaining.is_empty() {
                            return Err(DeserializeError::Custom(format!(
                                "Unexpected trailing characters after unit equation: '{}'",
                                remaining
                            )));
                        }
                        units = Some(unit_eqn);
                    }
                    b"non_negative" => {
                        // Read the text content of the non_negative element
                        let text = read_text_content(reader, buf)?;
                        non_negative = Some(match text.trim() {
                            "true" => Some(true),
                            "false" => Some(false),
                            "" => None, // Empty content means default (true)
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid non_negative value: {}",
                                    text
                                )));
                            }
                        });
                    }
                    b"range" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
                        buf.clear();
                        // Skip to end of range element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == b"range" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        if let (Some(min_val), Some(max_val)) = (min, max) {
                            range = Some(DeviceRange {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"scale" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
                        let auto = attrs.get_opt_bool("auto")?;
                        let group = attrs.get_opt_u32("group")?;
                        buf.clear();
                        // Skip to end of scale element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == b"scale" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        if let Some(auto_val) = auto {
                            scale = Some(DeviceScale::Auto(auto_val));
                        } else if let Some(group_val) = group {
                            scale = Some(DeviceScale::Group(group_val));
                        } else if let (Some(min_val), Some(max_val)) = (min, max) {
                            scale = Some(DeviceScale::MinMax {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"format" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let precision = attrs.get_opt_f64("precision")?;
                        let scale_by = attrs.get_opt_f64("scale_by")?;
                        let display_as = attrs
                            .get_opt("display_as")
                            .map(|s| match s {
                                "number" => Ok(DisplayAs::Number),
                                "currency" => Ok(DisplayAs::Currency),
                                "percent" => Ok(DisplayAs::Percent),
                                _ => Err(DeserializeError::Custom(format!(
                                    "Invalid display_as: {}",
                                    s
                                ))),
                            })
                            .transpose()?;
                        let delimit_000s = attrs.get_opt_bool("delimit_000s")?;
                        buf.clear();
                        // Skip to end of format element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == b"format" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        format = Some(FormatOptions {
                            precision,
                            scale_by,
                            display_as,
                            delimit_000s,
                        });
                    }
                    #[cfg(feature = "arrays")]
                    b"element" => {
                        buf.clear();
                        let element = deserialize_array_element(reader, buf)?;
                        elements.push(element);
                    }
                    _ => {
                        // Skip unknown elements
                        let element_name = e.name().as_ref().to_vec();
                        buf.clear();
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end)
                                    if end.name().as_ref() == element_name.as_slice() =>
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
            Event::Empty(e) => {
                // Handle empty tags like <non_negative/>
                match e.name().as_ref() {
                    b"non_negative" => {
                        // Empty tag <non_negative/> means default (true)
                        non_negative = Some(None);
                    }
                    b"range" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
                        if let (Some(min_val), Some(max_val)) = (min, max) {
                            range = Some(DeviceRange {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"scale" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
                        let auto = attrs.get_opt_bool("auto")?;
                        let group = attrs.get_opt_u32("group")?;
                        if let Some(auto_val) = auto {
                            scale = Some(DeviceScale::Auto(auto_val));
                        } else if let Some(group_val) = group {
                            scale = Some(DeviceScale::Group(group_val));
                        } else if let (Some(min_val), Some(max_val)) = (min, max) {
                            scale = Some(DeviceScale::MinMax {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"format" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let precision = attrs.get_opt_f64("precision")?;
                        let scale_by = attrs.get_opt_f64("scale_by")?;
                        let display_as = attrs.get_opt("display_as").and_then(|s| match s {
                            "number" => Some(DisplayAs::Number),
                            "currency" => Some(DisplayAs::Currency),
                            "percent" => Some(DisplayAs::Percent),
                            _ => None,
                        });
                        let delimit_000s = attrs.get_opt_bool("delimit_000s")?;
                        format = Some(FormatOptions {
                            precision,
                            scale_by,
                            display_as,
                            delimit_000s,
                        });
                    }
                    _ => {}
                }
            }
            Event::End(end) if end.name().as_ref() == b"flow" => {
                break;
            }
            Event::Eof => {
                return Err(DeserializeError::UnexpectedEof);
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(BasicFlow {
        name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
        access,
        autoexport,
        equation,
        mathml_equation,
        multiplier,
        non_negative,
        units,
        documentation,
        range,
        scale,
        format,
        #[cfg(feature = "arrays")]
        dimensions: _dimensions.map(|dims| dims.dims.into_iter().map(|d| d.name).collect()),
        #[cfg(feature = "arrays")]
        elements,
        event_poster: _event_poster,
    })
}

/// Deserialize a BasicStock variable from XML.
///
/// This function expects the reader to be positioned at the start of a <stock> element.
pub fn deserialize_basic_stock<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<BasicStock, DeserializeError> {
    // Expect <stock> start tag
    let (name, access, autoexport) = match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"stock" => {
            let attrs = Attrs::from_start(&e, reader)?;
            let name = attrs
                .get_opt("name")
                .map(|s| {
                    Identifier::parse_from_attribute(s)
                        .map_err(|e| DeserializeError::Custom(format!("Invalid identifier: {}", e)))
                })
                .transpose()?;
            let access = attrs
                .get_opt("access")
                .map(|s| match s {
                    "input" => Ok(crate::model::vars::AccessType::Input),
                    "output" => Ok(crate::model::vars::AccessType::Output),
                    _ => Err(DeserializeError::Custom(format!(
                        "Invalid access type: {}",
                        s
                    ))),
                })
                .transpose()?;
            let autoexport = attrs.get_opt_bool("autoexport")?;
            (name, access, autoexport)
        }
        Event::Start(e) => {
            return Err(DeserializeError::UnexpectedElement {
                expected: "stock".to_string(),
                found: String::from_utf8_lossy(e.name().as_ref()).to_string(),
            });
        }
        _ => {
            return Err(DeserializeError::Custom(
                "Expected stock start tag".to_string(),
            ));
        }
    };
    buf.clear();
    deserialize_basic_stock_impl(reader, buf, name, access, autoexport, false)
}

/// Parsed conveyor element data.
#[derive(Debug, Default)]
struct ConveyorData {
    length: Option<Expression>,
    capacity: Option<Expression>,
    inflow_limit: Option<Expression>,
    sample: Option<Expression>,
    arrest_value: Option<Expression>,
    discrete: Option<bool>,
    batch_integrity: Option<bool>,
    one_at_a_time: Option<bool>,
    exponential_leakage: Option<bool>,
}

/// Parse a conveyor element and its children.
#[allow(dead_code)]
fn parse_conveyor_element<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start_event: &quick_xml::events::BytesStart,
) -> Result<ConveyorData, DeserializeError> {
    let mut data = ConveyorData::default();

    let attrs = Attrs::from_start(start_event, reader)?;
    data.discrete = attrs.get_opt_bool("discrete")?;
    data.batch_integrity = attrs.get_opt_bool("batch_integrity")?;
    data.one_at_a_time = attrs.get_opt_bool("one_at_a_time")?;
    data.exponential_leakage = attrs.get_opt_bool("exponential_leak")?;

    // Read child elements
    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"len" => {
                        data.length = Some(read_expression(reader, buf)?);
                    }
                    b"capacity" => {
                        data.capacity = Some(read_expression(reader, buf)?);
                    }
                    b"in_limit" => {
                        data.inflow_limit = Some(read_expression(reader, buf)?);
                    }
                    b"sample" => {
                        data.sample = Some(read_expression(reader, buf)?);
                    }
                    b"arrest" => {
                        data.arrest_value = Some(read_expression(reader, buf)?);
                    }
                    _ => {
                        // Skip unknown elements - copy name before clearing buf
                        let name = e.name().as_ref().to_vec();
                        buf.clear();
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == name.as_slice() => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                    }
                }
            }
            Event::End(end) if end.name().as_ref() == b"conveyor" => {
                break;
            }
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(data)
}

/// Internal implementation of stock deserialization that returns Stock enum.
pub(crate) fn deserialize_stock_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<Identifier>,
    access: Option<crate::model::vars::AccessType>,
    autoexport: Option<bool>,
    _is_empty_tag: bool,
) -> Result<Stock, DeserializeError> {
    let mut initial_equation: Option<Expression> = None;
    #[cfg(feature = "mathml")]
    let mut mathml_equation: Option<String> = None;
    let mut inflows: Vec<Identifier> = Vec::new();
    let mut outflows: Vec<Identifier> = Vec::new();
    let mut documentation: Option<Documentation> = None;
    let mut units: Option<UnitEquation> = None;
    let mut non_negative: Option<Option<bool>> = None;
    let mut range: Option<DeviceRange> = None;
    let mut scale: Option<DeviceScale> = None;
    let mut format: Option<FormatOptions> = None;
    let mut _event_poster: Option<EventPoster> = None;
    #[cfg(feature = "arrays")]
    let mut _dimensions: Option<VariableDimensions> = None;
    #[cfg(feature = "arrays")]
    let mut elements: Vec<ArrayElement> = Vec::new();

    // Stock type indicators
    let mut conveyor_data: Option<ConveyorData> = None;
    let mut is_queue: bool = false;

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"eqn" => {
                        initial_equation = Some(read_expression(reader, buf)?);
                    }
                    #[cfg(feature = "mathml")]
                    b"mathml" => {
                        mathml_equation = Some(read_text_content(reader, buf)?);
                    }
                    b"inflow" => {
                        let inflow_text = read_text_content(reader, buf)?;
                        let inflow_id =
                            Identifier::parse_from_attribute(&inflow_text).map_err(|e| {
                                DeserializeError::Custom(format!(
                                    "Invalid inflow identifier: {}",
                                    e
                                ))
                            })?;
                        inflows.push(inflow_id);
                    }
                    b"outflow" => {
                        let outflow_text = read_text_content(reader, buf)?;
                        let outflow_id =
                            Identifier::parse_from_attribute(&outflow_text).map_err(|e| {
                                DeserializeError::Custom(format!(
                                    "Invalid outflow identifier: {}",
                                    e
                                ))
                            })?;
                        outflows.push(outflow_id);
                    }
                    b"doc" => {
                        let doc_text = read_text_content(reader, buf)?;
                        documentation = Some(
                            if doc_text.trim().contains('<') && doc_text.trim().contains('>') {
                                Documentation::Html(doc_text)
                            } else {
                                Documentation::PlainText(doc_text)
                            },
                        );
                    }
                    b"units" => {
                        let units_str = read_text_content(reader, buf)?;
                        let (remaining, unit_eqn) = unit_equation(&units_str).map_err(|e| {
                            DeserializeError::Custom(format!(
                                "Failed to parse unit equation: {}",
                                e
                            ))
                        })?;
                        if !remaining.is_empty() {
                            return Err(DeserializeError::Custom(format!(
                                "Unexpected trailing characters after unit equation: '{}'",
                                remaining
                            )));
                        }
                        units = Some(unit_eqn);
                    }
                    b"non_negative" => {
                        // Read the text content of the non_negative element
                        let text = read_text_content(reader, buf)?;
                        non_negative = Some(match text.trim() {
                            "true" => Some(true),
                            "false" => Some(false),
                            "" => None, // Empty content means default (true)
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid non_negative value: {}",
                                    text
                                )));
                            }
                        });
                    }
                    b"range" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
                        buf.clear();
                        // Skip to end of range element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == b"range" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        if let (Some(min_val), Some(max_val)) = (min, max) {
                            range = Some(DeviceRange {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"scale" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
                        let auto = attrs.get_opt_bool("auto")?;
                        let group = attrs.get_opt_u32("group")?;
                        buf.clear();
                        // Skip to end of scale element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == b"scale" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        if let Some(auto_val) = auto {
                            scale = Some(DeviceScale::Auto(auto_val));
                        } else if let Some(group_val) = group {
                            scale = Some(DeviceScale::Group(group_val));
                        } else if let (Some(min_val), Some(max_val)) = (min, max) {
                            scale = Some(DeviceScale::MinMax {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"format" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let precision = attrs.get_opt_f64("precision")?;
                        let scale_by = attrs.get_opt_f64("scale_by")?;
                        let display_as = attrs
                            .get_opt("display_as")
                            .map(|s| match s {
                                "number" => Ok(DisplayAs::Number),
                                "currency" => Ok(DisplayAs::Currency),
                                "percent" => Ok(DisplayAs::Percent),
                                _ => Err(DeserializeError::Custom(format!(
                                    "Invalid display_as: {}",
                                    s
                                ))),
                            })
                            .transpose()?;
                        let delimit_000s = attrs.get_opt_bool("delimit_000s")?;
                        buf.clear();
                        // Skip to end of format element
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == b"format" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        format = Some(FormatOptions {
                            precision,
                            scale_by,
                            display_as,
                            delimit_000s,
                        });
                    }
                    b"conveyor" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let mut cd = ConveyorData::default();
                        cd.discrete = attrs.get_opt_bool("discrete")?;
                        cd.batch_integrity = attrs.get_opt_bool("batch_integrity")?;
                        cd.one_at_a_time = attrs.get_opt_bool("one_at_a_time")?;
                        cd.exponential_leakage = attrs.get_opt_bool("exponential_leak")?;
                        buf.clear();
                        // Read conveyor child elements
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::Start(inner) => match inner.name().as_ref() {
                                    b"len" => cd.length = Some(read_expression(reader, buf)?),
                                    b"capacity" => {
                                        cd.capacity = Some(read_expression(reader, buf)?)
                                    }
                                    b"in_limit" => {
                                        cd.inflow_limit = Some(read_expression(reader, buf)?)
                                    }
                                    b"sample" => cd.sample = Some(read_expression(reader, buf)?),
                                    b"arrest" => {
                                        cd.arrest_value = Some(read_expression(reader, buf)?)
                                    }
                                    _ => {
                                        let name = inner.name().as_ref().to_vec();
                                        loop {
                                            match reader.read_event_into(buf)? {
                                                Event::End(e)
                                                    if e.name().as_ref() == name.as_slice() =>
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
                                },
                                Event::End(end) if end.name().as_ref() == b"conveyor" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                        conveyor_data = Some(cd);
                    }
                    b"queue" => {
                        is_queue = true;
                        buf.clear();
                        // Queue element may have content, skip it
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == b"queue" => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                    }
                    #[cfg(feature = "arrays")]
                    b"element" => {
                        buf.clear();
                        let element = deserialize_array_element(reader, buf)?;
                        elements.push(element);
                    }
                    _ => {
                        // Skip unknown elements
                        let name = e.name().as_ref().to_vec();
                        buf.clear();
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::End(end) if end.name().as_ref() == name.as_slice() => break,
                                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                _ => {}
                            }
                            buf.clear();
                        }
                    }
                }
            }
            Event::Empty(e) => {
                // Handle empty tags
                match e.name().as_ref() {
                    b"non_negative" => {
                        // Empty tag <non_negative/> means default (true)
                        non_negative = Some(None);
                    }
                    b"queue" => {
                        // Empty <queue/> tag
                        is_queue = true;
                    }
                    b"conveyor" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let mut cd = ConveyorData::default();
                        cd.discrete = attrs.get_opt_bool("discrete")?;
                        conveyor_data = Some(cd);
                    }
                    b"range" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
                        if let (Some(min_val), Some(max_val)) = (min, max) {
                            range = Some(DeviceRange {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"scale" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let min = attrs.get_opt_f64("min")?;
                        let max = attrs.get_opt_f64("max")?;
                        let auto = attrs.get_opt_bool("auto")?;
                        let group = attrs.get_opt_u32("group")?;
                        if let Some(auto_val) = auto {
                            scale = Some(DeviceScale::Auto(auto_val));
                        } else if let Some(group_val) = group {
                            scale = Some(DeviceScale::Group(group_val));
                        } else if let (Some(min_val), Some(max_val)) = (min, max) {
                            scale = Some(DeviceScale::MinMax {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"format" => {
                        let attrs = Attrs::from_start(&e, reader)?;
                        let precision = attrs.get_opt_f64("precision")?;
                        let scale_by = attrs.get_opt_f64("scale_by")?;
                        let display_as = attrs.get_opt("display_as").and_then(|s| match s {
                            "number" => Some(DisplayAs::Number),
                            "currency" => Some(DisplayAs::Currency),
                            "percent" => Some(DisplayAs::Percent),
                            _ => None,
                        });
                        let delimit_000s = attrs.get_opt_bool("delimit_000s")?;
                        format = Some(FormatOptions {
                            precision,
                            scale_by,
                            display_as,
                            delimit_000s,
                        });
                    }
                    _ => {}
                }
            }
            Event::End(e) if e.name().as_ref() == b"stock" => {
                break;
            }
            Event::Eof => {
                return Err(DeserializeError::UnexpectedEof);
            }
            _ => {}
        }
        buf.clear();
    }

    let name = name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?;
    let initial_equation =
        initial_equation.ok_or_else(|| DeserializeError::MissingField("eqn".to_string()))?;

    // Determine the stock type based on what we parsed
    if let Some(conv) = conveyor_data {
        // It's a conveyor stock
        let length = conv.length.ok_or_else(|| {
            DeserializeError::Custom("Conveyor stock requires a length element".to_string())
        })?;

        Ok(Stock::Conveyor(ConveyorStock {
            name,
            access,
            autoexport,
            inflows,
            outflows,
            initial_equation,
            length,
            capacity: conv.capacity,
            inflow_limit: conv.inflow_limit,
            sample: conv.sample,
            arrest_value: conv.arrest_value,
            discrete: conv.discrete,
            batch_integrity: conv.batch_integrity,
            one_at_a_time: conv.one_at_a_time,
            exponential_leakage: conv.exponential_leakage,
            units,
            documentation,
            range,
            scale,
            format,
            #[cfg(feature = "arrays")]
            dimensions: _dimensions.map(|dims| dims.dims.into_iter().map(|d| d.name).collect()),
            #[cfg(feature = "arrays")]
            elements,
            event_poster: _event_poster,
            #[cfg(feature = "mathml")]
            mathml_equation,
        }))
    } else if is_queue {
        // It's a queue stock
        Ok(Stock::Queue(QueueStock {
            name,
            access,
            autoexport,
            inflows,
            outflows,
            initial_equation,
            units,
            documentation,
            range,
            scale,
            format,
            #[cfg(feature = "arrays")]
            dimensions: _dimensions.map(|dims| dims.dims.into_iter().map(|d| d.name).collect()),
            #[cfg(feature = "arrays")]
            elements,
            event_poster: _event_poster,
            #[cfg(feature = "mathml")]
            mathml_equation,
        }))
    } else {
        // It's a basic stock
        Ok(Stock::Basic(BasicStock {
            name,
            access,
            autoexport,
            inflows,
            outflows,
            initial_equation,
            #[cfg(feature = "mathml")]
            mathml_equation,
            non_negative,
            units,
            documentation,
            range,
            scale,
            format,
            #[cfg(feature = "arrays")]
            dimensions: _dimensions.map(|dims| dims.dims.into_iter().map(|d| d.name).collect()),
            #[cfg(feature = "arrays")]
            elements,
            event_poster: _event_poster,
        }))
    }
}

/// Internal implementation of basic stock deserialization.
/// This is kept for backwards compatibility but delegates to deserialize_stock_impl.
pub(crate) fn deserialize_basic_stock_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<Identifier>,
    access: Option<crate::model::vars::AccessType>,
    autoexport: Option<bool>,
    is_empty_tag: bool,
) -> Result<BasicStock, DeserializeError> {
    match deserialize_stock_impl(reader, buf, name, access, autoexport, is_empty_tag)? {
        Stock::Basic(basic) => Ok(basic),
        Stock::Conveyor(_) => Err(DeserializeError::Custom(
            "Expected BasicStock but got ConveyorStock".to_string(),
        )),
        Stock::Queue(_) => Err(DeserializeError::Custom(
            "Expected BasicStock but got QueueStock".to_string(),
        )),
    }
}
