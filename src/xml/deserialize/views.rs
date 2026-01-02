//! Views deserialization module.
//!
//! This module handles deserialization of views and all view objects:
//! stocks, flows, auxes, modules, groups, connectors, aliases, and UI elements.

use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::BufRead;

// Re-export behavior deserialization functions
pub use crate::xml::deserialize::behavior::deserialize_behavior;

// Re-export data deserialization functions
pub use crate::xml::deserialize::data::deserialize_data;

use crate::Expression;
use crate::equation::Identifier;
use crate::equation::units::UnitEquation;
use crate::model::events::EventPoster;
use crate::model::groups::{Group, GroupEntity};
use crate::model::object::{DeviceRange, DeviceScale, DisplayAs, Documentation, FormatOptions};
#[cfg(feature = "arrays")]
use crate::model::vars::array::{ArrayElement, VariableDimensions};
#[cfg(feature = "submodels")]
use crate::model::vars::module::{Module, ModuleConnection};
use crate::model::vars::{
    aux::Auxiliary,
    flow::BasicFlow,
    stock::{BasicStock, ConveyorStock, QueueStock, Stock},
};
use crate::view::objects::{
    AliasObject, AuxObject, ButtonAppearance, ButtonObject, ButtonStyle, ConnectorObject,
    DataAction, FileAction, FlowObject, GaugeObject, GraphObject, GraphType, GraphicalInputObject,
    GraphicsFrameContent, GraphicsFrameObject, GroupObject, ImageContent, KnobObject, LampObject,
    LineStyle, Link, LinkEffect, LinkTarget, ListInputObject, MenuAction, MiscellaneousAction,
    ModuleObject, NumericDisplayObject, NumericInputObject, OptionEntity, OptionsLayout,
    OptionsObject, PenStyle, Plot, PlotScale, Point, Pointer, Polarity, PopupContent,
    PrintingAction, ReportBalances, ReportFlows, RestoreAction, Shape, SimulationAction,
    SliderObject, StackedContainerObject, StockObject, SwitchAction, SwitchObject, SwitchStyle,
    TableItem, TableItemType, TableObject, TableOrientation, TextBoxAppearance, TextBoxObject,
    VideoContent, Zone, ZoneType,
};
use crate::view::style::{
    BorderStyle, BorderWidth, Color, FontStyle, FontWeight, TextAlign, TextDecoration,
    VerticalTextAlign,
};
use crate::view::{PageOrientation, PageSequence, Style, View, ViewType};
use crate::xml::deserialize::DeserializeError;
use crate::xml::deserialize::helpers::{read_number_content, read_text_content};
#[cfg(feature = "arrays")]
use crate::xml::deserialize::variables::{deserialize_array_element, deserialize_dimensions};
use crate::xml::deserialize::variables::{
    deserialize_event_poster, deserialize_format, deserialize_range, deserialize_scale,
    read_expression,
};
use crate::xml::quick::de::skip_element;
pub fn deserialize_views<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<crate::xml::schema::Views, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"views" => {
            let mut visible_view: Option<u32> = None;

            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                if attr.key.as_ref() == b"visible_view" {
                    let visible_str = attr.decode_and_unescape_value(reader)?.to_string();
                    visible_view = Some(visible_str.parse().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid visible_view value: {}", e))
                    })?);
                }
            }

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
    let style: Option<Style> = None;
    let mut views = Vec::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) if e.name().as_ref() == b"view" => {
                // Extract attributes from start tag and call impl
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
                views.push(deserialize_view_impl(reader, buf, attrs)?);
            }
            Event::Start(e) if e.name().as_ref() == b"style" => {
                // TODO: Implement style deserialization
                let element_name = e.name().as_ref().to_vec();
                loop {
                    match reader.read_event_into(buf)? {
                        Event::End(e) if e.name().as_ref() == element_name.as_slice() => break,
                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                        _ => {}
                    }
                    buf.clear();
                }
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

            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"type" => {
                        let type_str = attr.decode_and_unescape_value(reader)?.to_string();
                        view_type = Some(parse_view_type(&type_str)?);
                    }
                    b"order" => {
                        let order_str = attr.decode_and_unescape_value(reader)?.to_string();
                        order = Some(order_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid order value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"zoom" => {
                        let zoom_str = attr.decode_and_unescape_value(reader)?.to_string();
                        zoom = Some(zoom_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid zoom value: {}", e))
                        })?);
                    }
                    b"scroll_x" => {
                        let scroll_x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        scroll_x = Some(scroll_x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid scroll_x value: {}", e))
                        })?);
                    }
                    b"scroll_y" => {
                        let scroll_y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        scroll_y = Some(scroll_y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid scroll_y value: {}", e))
                        })?);
                    }
                    b"background" => {
                        background = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"page_width" => {
                        let page_width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        page_width = Some(page_width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid page_width value: {}", e))
                        })?);
                    }
                    b"page_height" => {
                        let page_height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        page_height = Some(page_height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid page_height value: {}", e))
                        })?);
                    }
                    b"page_sequence" => {
                        let seq_str = attr.decode_and_unescape_value(reader)?.to_string();
                        page_sequence = Some(match seq_str.as_str() {
                            "row" => PageSequence::Row,
                            "column" => PageSequence::Column,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid page_sequence: {}",
                                    seq_str
                                )));
                            }
                        });
                    }
                    b"page_orientation" => {
                        let orient_str = attr.decode_and_unescape_value(reader)?.to_string();
                        page_orientation = Some(match orient_str.as_str() {
                            "landscape" => PageOrientation::Landscape,
                            "portrait" => PageOrientation::Portrait,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid page_orientation: {}",
                                    orient_str
                                )));
                            }
                        });
                    }
                    b"show_pages" => {
                        let show_pages_str = attr.decode_and_unescape_value(reader)?.to_string();
                        show_pages = Some(match show_pages_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid show_pages value: {}",
                                    show_pages_str
                                )));
                            }
                        });
                    }
                    b"home_page" => {
                        let home_page_str = attr.decode_and_unescape_value(reader)?.to_string();
                        home_page = Some(home_page_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid home_page value: {}", e))
                        })?);
                    }
                    b"home_view" => {
                        let home_view_str = attr.decode_and_unescape_value(reader)?.to_string();
                        home_view = Some(match home_view_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid home_view value: {}",
                                    home_view_str
                                )));
                            }
                        });
                    }
                    _ => {}
                }
            }

            // Read child elements
            let style: Option<Style> = None;
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
                                // TODO: Deserialize style
                                let element_name = e.name().as_ref().to_vec();
                                loop {
                                    match reader.read_event_into(buf)? {
                                        Event::End(e)
                                            if e.name().as_ref() == element_name.as_slice() =>
                                        {
                                            break;
                                        }
                                        Event::Eof => return Err(DeserializeError::UnexpectedEof),
                                        _ => {}
                                    }
                                    buf.clear();
                                }
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
                                let mut uid: Option<i32> = None;
                                let mut name: Option<String> = None;
                                let mut x: Option<f64> = None;
                                let mut y: Option<f64> = None;
                                let mut width: Option<f64> = None;
                                let mut height: Option<f64> = None;
                                let display_attrs = read_display_attributes(&e, reader)?;

                                for attr in e.attributes() {
                                    let attr = attr?;
                                    match attr.key.as_ref() {
                                        b"uid" => {
                                            let uid_str =
                                                attr.decode_and_unescape_value(reader)?.to_string();
                                            uid = Some(uid_str.parse().map_err(|err| {
                                                DeserializeError::Custom(format!(
                                                    "Invalid uid value: {}",
                                                    err
                                                ))
                                            })?);
                                        }
                                        b"name" => {
                                            name = Some(
                                                attr.decode_and_unescape_value(reader)?.to_string(),
                                            );
                                        }
                                        b"x" => {
                                            let x_str =
                                                attr.decode_and_unescape_value(reader)?.to_string();
                                            x = Some(x_str.parse().map_err(|err| {
                                                DeserializeError::Custom(format!(
                                                    "Invalid x value: {}",
                                                    err
                                                ))
                                            })?);
                                        }
                                        b"y" => {
                                            let y_str =
                                                attr.decode_and_unescape_value(reader)?.to_string();
                                            y = Some(y_str.parse().map_err(|err| {
                                                DeserializeError::Custom(format!(
                                                    "Invalid y value: {}",
                                                    err
                                                ))
                                            })?);
                                        }
                                        b"width" => {
                                            let width_str =
                                                attr.decode_and_unescape_value(reader)?.to_string();
                                            width = Some(width_str.parse().map_err(|err| {
                                                DeserializeError::Custom(format!(
                                                    "Invalid width value: {}",
                                                    err
                                                ))
                                            })?);
                                        }
                                        b"height" => {
                                            let height_str =
                                                attr.decode_and_unescape_value(reader)?.to_string();
                                            height = Some(height_str.parse().map_err(|err| {
                                                DeserializeError::Custom(format!(
                                                    "Invalid height value: {}",
                                                    err
                                                ))
                                            })?);
                                        }
                                        _ => {}
                                    }
                                }

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
    let style: Option<Style> = None;
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
                        let mut uid: Option<i32> = None;
                        let mut name: Option<String> = None;
                        let mut x: Option<f64> = None;
                        let mut y: Option<f64> = None;
                        let mut width: Option<f64> = None;
                        let mut height: Option<f64> = None;

                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"uid" => {
                                    uid = Some(
                                        attr.decode_and_unescape_value(reader)?
                                            .parse()
                                            .unwrap_or(0),
                                    )
                                }
                                b"name" => {
                                    name = Some(attr.decode_and_unescape_value(reader)?.to_string())
                                }
                                b"x" => {
                                    x = Some(
                                        attr.decode_and_unescape_value(reader)?
                                            .parse()
                                            .unwrap_or(0.0),
                                    )
                                }
                                b"y" => {
                                    y = Some(
                                        attr.decode_and_unescape_value(reader)?
                                            .parse()
                                            .unwrap_or(0.0),
                                    )
                                }
                                b"width" => {
                                    width = Some(
                                        attr.decode_and_unescape_value(reader)?
                                            .parse()
                                            .unwrap_or(0.0),
                                    )
                                }
                                b"height" => {
                                    height = Some(
                                        attr.decode_and_unescape_value(reader)?
                                            .parse()
                                            .unwrap_or(0.0),
                                    )
                                }
                                _ => {}
                            }
                        }
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
                        let mut uid: Option<i32> = None;
                        let mut name: Option<String> = None;
                        let mut x: Option<f64> = None;
                        let mut y: Option<f64> = None;
                        let mut width: Option<f64> = None;
                        let mut height: Option<f64> = None;
                        let display_attrs = read_display_attributes(&e, reader)?;

                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"uid" => {
                                    uid = Some(
                                        attr.decode_and_unescape_value(reader)?
                                            .parse()
                                            .unwrap_or(0),
                                    )
                                }
                                b"name" => {
                                    name = Some(attr.decode_and_unescape_value(reader)?.to_string())
                                }
                                b"x" => {
                                    x = Some(
                                        attr.decode_and_unescape_value(reader)?
                                            .parse()
                                            .unwrap_or(0.0),
                                    )
                                }
                                b"y" => {
                                    y = Some(
                                        attr.decode_and_unescape_value(reader)?
                                            .parse()
                                            .unwrap_or(0.0),
                                    )
                                }
                                b"width" => {
                                    width = Some(
                                        attr.decode_and_unescape_value(reader)?
                                            .parse()
                                            .unwrap_or(0.0),
                                    )
                                }
                                b"height" => {
                                    height = Some(
                                        attr.decode_and_unescape_value(reader)?
                                            .parse()
                                            .unwrap_or(0.0),
                                    )
                                }
                                _ => {}
                            }
                        }

                        // Read child elements (shape)
                        let mut shape: Option<Shape> = None;
                        buf.clear();
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::Start(inner_e) if inner_e.name().as_ref() == b"shape" => {
                                    // Extract shape attributes
                                    let mut shape_type: Option<String> = None;
                                    let mut shape_width: Option<f64> = None;
                                    let mut shape_height: Option<f64> = None;
                                    let mut corner_radius: Option<f64> = None;
                                    let mut radius: Option<f64> = None;

                                    for attr in inner_e.attributes() {
                                        let attr = attr?;
                                        match attr.key.as_ref() {
                                            b"type" => {
                                                shape_type = Some(
                                                    attr.decode_and_unescape_value(reader)?
                                                        .to_string(),
                                                )
                                            }
                                            b"width" => {
                                                shape_width = Some(
                                                    attr.decode_and_unescape_value(reader)?
                                                        .parse()
                                                        .unwrap_or(0.0),
                                                )
                                            }
                                            b"height" => {
                                                shape_height = Some(
                                                    attr.decode_and_unescape_value(reader)?
                                                        .parse()
                                                        .unwrap_or(0.0),
                                                )
                                            }
                                            b"corner_radius" => {
                                                corner_radius = Some(
                                                    attr.decode_and_unescape_value(reader)?
                                                        .parse()
                                                        .unwrap_or(0.0),
                                                )
                                            }
                                            b"radius" => {
                                                radius = Some(
                                                    attr.decode_and_unescape_value(reader)?
                                                        .parse()
                                                        .unwrap_or(0.0),
                                                )
                                            }
                                            _ => {}
                                        }
                                    }

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
                                    // Extract shape attributes from empty tag
                                    let mut shape_type: Option<String> = None;
                                    let mut shape_width: Option<f64> = None;
                                    let mut shape_height: Option<f64> = None;
                                    let mut corner_radius: Option<f64> = None;
                                    let mut radius: Option<f64> = None;

                                    for attr in inner_e.attributes() {
                                        let attr = attr?;
                                        match attr.key.as_ref() {
                                            b"type" => {
                                                shape_type = Some(
                                                    attr.decode_and_unescape_value(reader)?
                                                        .to_string(),
                                                )
                                            }
                                            b"width" => {
                                                shape_width = Some(
                                                    attr.decode_and_unescape_value(reader)?
                                                        .parse()
                                                        .unwrap_or(0.0),
                                                )
                                            }
                                            b"height" => {
                                                shape_height = Some(
                                                    attr.decode_and_unescape_value(reader)?
                                                        .parse()
                                                        .unwrap_or(0.0),
                                                )
                                            }
                                            b"corner_radius" => {
                                                corner_radius = Some(
                                                    attr.decode_and_unescape_value(reader)?
                                                        .parse()
                                                        .unwrap_or(0.0),
                                                )
                                            }
                                            b"radius" => {
                                                radius = Some(
                                                    attr.decode_and_unescape_value(reader)?
                                                        .parse()
                                                        .unwrap_or(0.0),
                                                )
                                            }
                                            _ => {}
                                        }
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
                        let mut uid: Option<i32> = None;
                        let mut name: Option<String> = None;
                        let mut x: Option<f64> = None;
                        let mut y: Option<f64> = None;
                        let mut width: Option<f64> = None;
                        let mut height: Option<f64> = None;

                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"uid" => {
                                    uid = Some(
                                        attr.decode_and_unescape_value(reader)?
                                            .parse()
                                            .unwrap_or(0),
                                    )
                                }
                                b"name" => {
                                    name = Some(attr.decode_and_unescape_value(reader)?.to_string())
                                }
                                b"x" => {
                                    x = Some(
                                        attr.decode_and_unescape_value(reader)?
                                            .parse()
                                            .unwrap_or(0.0),
                                    )
                                }
                                b"y" => {
                                    y = Some(
                                        attr.decode_and_unescape_value(reader)?
                                            .parse()
                                            .unwrap_or(0.0),
                                    )
                                }
                                b"width" => {
                                    width = Some(
                                        attr.decode_and_unescape_value(reader)?
                                            .parse()
                                            .unwrap_or(0.0),
                                    )
                                }
                                b"height" => {
                                    height = Some(
                                        attr.decode_and_unescape_value(reader)?
                                            .parse()
                                            .unwrap_or(0.0),
                                    )
                                }
                                _ => {}
                            }
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
            let mut uid: Option<i32> = None;
            let mut name: Option<String> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;

            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"name" => {
                        name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

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
            let mut uid: Option<i32> = None;
            let mut name: Option<String> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;

            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"name" => {
                        name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut pts = Vec::new();

            loop {
                match reader.read_event_into(buf)? {
                    Event::Start(e) if e.name().as_ref() == b"pts" => {
                        loop {
                            match reader.read_event_into(buf)? {
                                Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"pt" => {
                                    let mut pt_x: Option<f64> = None;
                                    let mut pt_y: Option<f64> = None;

                                    for attr in e.attributes() {
                                        let attr = attr?;
                                        match attr.key.as_ref() {
                                            b"x" => {
                                                let x_str = attr
                                                    .decode_and_unescape_value(reader)?
                                                    .to_string();
                                                pt_x = Some(x_str.parse().map_err(|e| {
                                                    DeserializeError::Custom(format!(
                                                        "Invalid pt x value: {}",
                                                        e
                                                    ))
                                                })?);
                                            }
                                            b"y" => {
                                                let y_str = attr
                                                    .decode_and_unescape_value(reader)?
                                                    .to_string();
                                                pt_y = Some(y_str.parse().map_err(|e| {
                                                    DeserializeError::Custom(format!(
                                                        "Invalid pt y value: {}",
                                                        e
                                                    ))
                                                })?);
                                            }
                                            _ => {}
                                        }
                                    }

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
            let mut uid: Option<i32> = None;
            let mut name: Option<String> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;

            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"name" => {
                        name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

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
    let mut color: Option<Color> = None;
    let mut background: Option<Color> = None;
    let mut z_index: Option<i32> = None;
    let mut font_family: Option<String> = None;
    let mut font_size: Option<f64> = None;
    let mut font_weight: Option<FontWeight> = None;
    let mut font_style: Option<FontStyle> = None;
    let mut text_decoration: Option<TextDecoration> = None;
    let mut text_align: Option<TextAlign> = None;
    let mut text_background: Option<Color> = None;
    let mut vertical_text_align: Option<VerticalTextAlign> = None;
    let mut text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)> = None;
    let mut font_color: Option<Color> = None;
    let mut text_border_color: Option<Color> = None;
    let mut text_border_width: Option<BorderWidth> = None;
    let mut text_border_style: Option<BorderStyle> = None;
    let mut label_side: Option<String> = None;
    let mut label_angle: Option<f64> = None;

    for attr in e.attributes() {
        let attr = attr?;
        match attr.key.as_ref() {
            b"color" => {
                let color_str = attr.decode_and_unescape_value(reader)?.to_string();
                color = Some(parse_color(&color_str)?);
            }
            b"background" => {
                let bg_str = attr.decode_and_unescape_value(reader)?.to_string();
                background = Some(parse_color(&bg_str)?);
            }
            b"z_index" => {
                let z_str = attr.decode_and_unescape_value(reader)?.to_string();
                z_index = Some(z_str.parse().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid z_index value: {}", e))
                })?);
            }
            b"font_family" => {
                font_family = Some(attr.decode_and_unescape_value(reader)?.to_string());
            }
            b"font_size" => {
                let font_size_str = attr.decode_and_unescape_value(reader)?.to_string();
                // Remove "pt" suffix if present
                let font_size_clean = font_size_str.trim_end_matches("pt").trim();
                font_size = Some(font_size_clean.parse().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid font_size value: {}", e))
                })?);
            }
            b"font_weight" => {
                let weight_str = attr.decode_and_unescape_value(reader)?.to_string();
                font_weight = Some(match weight_str.as_str() {
                    "normal" => FontWeight::Normal,
                    "bold" => FontWeight::Bold,
                    _ => {
                        return Err(DeserializeError::Custom(format!(
                            "Invalid font_weight: {}",
                            weight_str
                        )));
                    }
                });
            }
            b"font_style" => {
                let style_str = attr.decode_and_unescape_value(reader)?.to_string();
                font_style = Some(match style_str.as_str() {
                    "normal" => FontStyle::Normal,
                    "italic" => FontStyle::Italic,
                    _ => {
                        return Err(DeserializeError::Custom(format!(
                            "Invalid font_style: {}",
                            style_str
                        )));
                    }
                });
            }
            b"text_decoration" => {
                let dec_str = attr.decode_and_unescape_value(reader)?.to_string();
                text_decoration = Some(match dec_str.as_str() {
                    "normal" => TextDecoration::Normal,
                    "underline" => TextDecoration::Underline,
                    _ => {
                        return Err(DeserializeError::Custom(format!(
                            "Invalid text_decoration: {}",
                            dec_str
                        )));
                    }
                });
            }
            b"text_align" => {
                let align_str = attr.decode_and_unescape_value(reader)?.to_string();
                text_align = Some(match align_str.as_str() {
                    "left" => TextAlign::Left,
                    "right" => TextAlign::Right,
                    "center" => TextAlign::Center,
                    _ => {
                        return Err(DeserializeError::Custom(format!(
                            "Invalid text_align: {}",
                            align_str
                        )));
                    }
                });
            }
            b"text_background" => {
                let bg_str = attr.decode_and_unescape_value(reader)?.to_string();
                text_background = Some(parse_color(&bg_str)?);
            }
            b"vertical_text_align" => {
                let align_str = attr.decode_and_unescape_value(reader)?.to_string();
                vertical_text_align = Some(match align_str.as_str() {
                    "top" => VerticalTextAlign::Top,
                    "bottom" => VerticalTextAlign::Bottom,
                    "center" => VerticalTextAlign::Center,
                    _ => {
                        return Err(DeserializeError::Custom(format!(
                            "Invalid vertical_text_align: {}",
                            align_str
                        )));
                    }
                });
            }
            b"text_padding" => {
                let padding_str = attr.decode_and_unescape_value(reader)?.to_string();
                text_padding = Some(parse_text_padding(&padding_str)?);
            }
            b"font_color" => {
                let color_str = attr.decode_and_unescape_value(reader)?.to_string();
                font_color = Some(parse_color(&color_str)?);
            }
            b"text_border_color" => {
                let color_str = attr.decode_and_unescape_value(reader)?.to_string();
                text_border_color = Some(parse_color(&color_str)?);
            }
            b"text_border_width" => {
                let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                text_border_width = Some(parse_border_width(&width_str)?);
            }
            b"text_border_style" => {
                let style_str = attr.decode_and_unescape_value(reader)?.to_string();
                text_border_style = Some(match style_str.as_str() {
                    "none" => BorderStyle::None,
                    "solid" => BorderStyle::Solid,
                    _ => {
                        return Err(DeserializeError::Custom(format!(
                            "Invalid text_border_style: {}",
                            style_str
                        )));
                    }
                });
            }
            b"label_side" => {
                label_side = Some(attr.decode_and_unescape_value(reader)?.to_string());
            }
            b"label_angle" => {
                let angle_str = attr.decode_and_unescape_value(reader)?.to_string();
                label_angle = Some(angle_str.parse().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid label_angle value: {}", e))
                })?);
            }
            _ => {}
        }
    }

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

// Use parsing functions from helpers module
use crate::xml::deserialize::helpers::{
    parse_border_width, parse_color, parse_font_style, parse_font_weight, parse_text_align,
    parse_text_decoration, parse_text_padding, parse_vertical_text_align,
};

/// Deserialize a Shape from XML.
fn deserialize_shape<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Shape, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"shape" => {
            let mut shape_type: Option<String> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut corner_radius: Option<f64> = None;
            let mut radius: Option<f64> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"type" => {
                        shape_type = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"corner_radius" => {
                        let radius_str = attr.decode_and_unescape_value(reader)?.to_string();
                        corner_radius = Some(radius_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid corner_radius value: {}", e))
                        })?);
                    }
                    b"radius" => {
                        let radius_str = attr.decode_and_unescape_value(reader)?.to_string();
                        radius = Some(radius_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid radius value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

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
            let mut uid: Option<i32> = None;
            let mut name: Option<String> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;

            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"name" => {
                        name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

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
            let mut uid: Option<i32> = None;
            let mut name: Option<String> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;

            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|err| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", err))
                        })?);
                    }
                    b"name" => {
                        name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|err| {
                            DeserializeError::Custom(format!("Invalid x value: {}", err))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|err| {
                            DeserializeError::Custom(format!("Invalid y value: {}", err))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|err| {
                            DeserializeError::Custom(format!("Invalid width value: {}", err))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|err| {
                            DeserializeError::Custom(format!("Invalid height value: {}", err))
                        })?);
                    }
                    _ => {}
                }
            }

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
    let mut uid: Option<i32> = None;
    let mut name: Option<String> = None;
    let mut x: Option<f64> = None;
    let mut y: Option<f64> = None;
    let mut width: Option<f64> = None;
    let mut height: Option<f64> = None;

    // Read attributes
    for attr in start_event.attributes() {
        let attr = attr?;
        match attr.key.as_ref() {
            b"uid" => {
                let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                uid =
                    Some(uid_str.parse().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid uid value: {}", e))
                    })?);
            }
            b"name" => {
                name = Some(attr.decode_and_unescape_value(reader)?.to_string());
            }
            b"x" => {
                let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                x =
                    Some(x_str.parse().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid x value: {}", e))
                    })?);
            }
            b"y" => {
                let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                y =
                    Some(y_str.parse().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid y value: {}", e))
                    })?);
            }
            b"width" => {
                let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                width = Some(width_str.parse().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid width value: {}", e))
                })?);
            }
            b"height" => {
                let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                height = Some(height_str.parse().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid height value: {}", e))
                })?);
            }
            _ => {}
        }
    }

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
            let mut uid: Option<i32> = None;
            let mut name: Option<String> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut locked: Option<bool> = None;

            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"name" => {
                        name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"locked" => {
                        let locked_str = attr.decode_and_unescape_value(reader)?.to_string();
                        locked = Some(match locked_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid locked value: {}",
                                    locked_str
                                )));
                            }
                        });
                    }
                    _ => {}
                }
            }

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut items = Vec::new();

            // If it's a start tag (not empty), read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"item" => {
                            let mut item_uid: Option<i32> = None;
                            for attr in e.attributes() {
                                let attr = attr?;
                                if attr.key.as_ref() == b"uid" {
                                    let uid_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    item_uid = Some(uid_str.parse().map_err(|e| {
                                        DeserializeError::Custom(format!(
                                            "Invalid item uid value: {}",
                                            e
                                        ))
                                    })?);
                                }
                            }
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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut angle: Option<f64> = None;
            let mut line_style: Option<LineStyle> = None;
            let mut delay_mark: Option<bool> = None;
            let mut polarity: Option<Polarity> = None;

            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"angle" => {
                        let angle_str = attr.decode_and_unescape_value(reader)?.to_string();
                        angle = Some(angle_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid angle value: {}", e))
                        })?);
                    }
                    b"line_style" => {
                        let style_str = attr.decode_and_unescape_value(reader)?.to_string();
                        line_style = Some(parse_line_style(&style_str)?);
                    }
                    b"delay_mark" => {
                        let delay_str = attr.decode_and_unescape_value(reader)?.to_string();
                        delay_mark = Some(match delay_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid delay_mark value: {}",
                                    delay_str
                                )));
                            }
                        });
                    }
                    b"polarity" => {
                        let polarity_str = attr.decode_and_unescape_value(reader)?.to_string();
                        polarity = Some(parse_polarity(&polarity_str)?);
                    }
                    _ => {}
                }
            }

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
                                    let mut pt_x: Option<f64> = None;
                                    let mut pt_y: Option<f64> = None;

                                    for attr in e.attributes() {
                                        let attr = attr?;
                                        match attr.key.as_ref() {
                                            b"x" => {
                                                let x_str = attr
                                                    .decode_and_unescape_value(reader)?
                                                    .to_string();
                                                pt_x = Some(x_str.parse().map_err(|e| {
                                                    DeserializeError::Custom(format!(
                                                        "Invalid pt x value: {}",
                                                        e
                                                    ))
                                                })?);
                                            }
                                            b"y" => {
                                                let y_str = attr
                                                    .decode_and_unescape_value(reader)?
                                                    .to_string();
                                                pt_y = Some(y_str.parse().map_err(|e| {
                                                    DeserializeError::Custom(format!(
                                                        "Invalid pt y value: {}",
                                                        e
                                                    ))
                                                })?);
                                            }
                                            _ => {}
                                        }
                                    }

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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;

            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

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
            let mut uid: Option<i32> = None;
            for attr in e.attributes() {
                let attr = attr?;
                if attr.key.as_ref() == b"uid" {
                    let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                    uid = Some(uid_str.parse().map_err(|e| {
                        DeserializeError::Custom(format!("Invalid alias uid value: {}", e))
                    })?);
                }
            }

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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut visible_index: Option<usize> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"visible_index" => {
                        let index_str = attr.decode_and_unescape_value(reader)?.to_string();
                        visible_index = Some(index_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid visible_index value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut min: Option<f64> = None;
            let mut max: Option<f64> = None;
            let mut show_name: Option<bool> = None;
            let mut show_number: Option<bool> = None;
            let mut show_min_max: Option<bool> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"min" => {
                        let min_str = attr.decode_and_unescape_value(reader)?.to_string();
                        min = Some(min_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid min value: {}", e))
                        })?);
                    }
                    b"max" => {
                        let max_str = attr.decode_and_unescape_value(reader)?.to_string();
                        max = Some(max_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid max value: {}", e))
                        })?);
                    }
                    b"show_name" => {
                        let show_str = attr.decode_and_unescape_value(reader)?.to_string();
                        show_name = Some(match show_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid show_name value: {}",
                                    show_str
                                )));
                            }
                        });
                    }
                    b"show_number" => {
                        let show_str = attr.decode_and_unescape_value(reader)?.to_string();
                        show_number = Some(match show_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid show_number value: {}",
                                    show_str
                                )));
                            }
                        });
                    }
                    b"show_min_max" => {
                        let show_str = attr.decode_and_unescape_value(reader)?.to_string();
                        show_min_max = Some(match show_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid show_min_max value: {}",
                                    show_str
                                )));
                            }
                        });
                    }
                    _ => {}
                }
            }

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut entity_name: Option<String> = None;
            let mut reset_to: Option<(f64, String)> = None;

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"entity" => {
                            for attr in e.attributes() {
                                let attr = attr?;
                                if attr.key.as_ref() == b"name" {
                                    entity_name =
                                        Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                            }

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
                            let mut after: Option<String> = None;
                            for attr in e.attributes() {
                                let attr = attr?;
                                if attr.key.as_ref() == b"after" {
                                    after =
                                        Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                            }
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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut min: Option<f64> = None;
            let mut max: Option<f64> = None;
            let mut show_name: Option<bool> = None;
            let mut show_number: Option<bool> = None;
            let mut show_min_max: Option<bool> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"min" => {
                        let min_str = attr.decode_and_unescape_value(reader)?.to_string();
                        min = Some(min_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid min value: {}", e))
                        })?);
                    }
                    b"max" => {
                        let max_str = attr.decode_and_unescape_value(reader)?.to_string();
                        max = Some(max_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid max value: {}", e))
                        })?);
                    }
                    b"show_name" => {
                        let show_str = attr.decode_and_unescape_value(reader)?.to_string();
                        show_name = Some(match show_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid show_name value: {}",
                                    show_str
                                )));
                            }
                        });
                    }
                    b"show_number" => {
                        let show_str = attr.decode_and_unescape_value(reader)?.to_string();
                        show_number = Some(match show_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid show_number value: {}",
                                    show_str
                                )));
                            }
                        });
                    }
                    b"show_min_max" => {
                        let show_str = attr.decode_and_unescape_value(reader)?.to_string();
                        show_min_max = Some(match show_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid show_min_max value: {}",
                                    show_str
                                )));
                            }
                        });
                    }
                    _ => {}
                }
            }

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut entity_name: Option<String> = None;

            // If it's a start tag, read child elements (knobs don't have reset_to)
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"entity" => {
                            for attr in e.attributes() {
                                let attr = attr?;
                                if attr.key.as_ref() == b"name" {
                                    entity_name =
                                        Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                            }

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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut show_name: Option<bool> = None;
            let mut switch_style: Option<SwitchStyle> = None;
            let mut clicking_sound: Option<bool> = None;
            let mut entity_name: Option<String> = None;
            let mut entity_value: Option<f64> = None;
            let mut group_name: Option<String> = None;
            let mut module_name: Option<String> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"show_name" => {
                        let show_str = attr.decode_and_unescape_value(reader)?.to_string();
                        show_name = Some(match show_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid show_name value: {}",
                                    show_str
                                )));
                            }
                        });
                    }
                    b"switch_style" => {
                        let style_str = attr.decode_and_unescape_value(reader)?.to_string();
                        switch_style = Some(parse_switch_style(&style_str)?);
                    }
                    b"clicking_sound" => {
                        let sound_str = attr.decode_and_unescape_value(reader)?.to_string();
                        clicking_sound = Some(match sound_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid clicking_sound value: {}",
                                    sound_str
                                )));
                            }
                        });
                    }
                    b"entity_name" => {
                        entity_name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"entity_value" => {
                        let value_str = attr.decode_and_unescape_value(reader)?.to_string();
                        entity_value = Some(value_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid entity_value: {}", e))
                        })?);
                    }
                    b"group_name" => {
                        group_name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"module_name" => {
                        module_name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut reset_to: Option<(f64, String)> = None;

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) if e.name().as_ref() == b"reset_to" => {
                            let mut after: Option<String> = None;
                            for attr in e.attributes() {
                                let attr = attr?;
                                if attr.key.as_ref() == b"after" {
                                    after =
                                        Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                            }
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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut layout: Option<OptionsLayout> = None;
            let mut horizontal_spacing: Option<f64> = None;
            let mut vertical_spacing: Option<f64> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"layout" => {
                        let layout_str = attr.decode_and_unescape_value(reader)?.to_string();
                        layout = Some(parse_options_layout(&layout_str)?);
                    }
                    b"horizontal_spacing" => {
                        let spacing_str = attr.decode_and_unescape_value(reader)?.to_string();
                        horizontal_spacing = Some(spacing_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!(
                                "Invalid horizontal_spacing value: {}",
                                e
                            ))
                        })?);
                    }
                    b"vertical_spacing" => {
                        let spacing_str = attr.decode_and_unescape_value(reader)?.to_string();
                        vertical_spacing = Some(spacing_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!(
                                "Invalid vertical_spacing value: {}",
                                e
                            ))
                        })?);
                    }
                    _ => {}
                }
            }

            // Read common display attributes
            let display_attrs = read_display_attributes(&e, reader)?;

            let mut entities = Vec::new();

            // If it's a start tag, read child elements
            if !matches!(reader.read_event_into(buf)?, Event::Empty(_)) {
                loop {
                    match reader.read_event_into(buf)? {
                        Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"entity" => {
                            let mut entity_name: Option<String> = None;
                            let mut index: Option<String> = None;
                            let mut value: Option<f64> = None;

                            for attr in e.attributes() {
                                let attr = attr?;
                                match attr.key.as_ref() {
                                    b"name" => {
                                        entity_name = Some(
                                            attr.decode_and_unescape_value(reader)?.to_string(),
                                        );
                                    }
                                    b"index" => {
                                        index = Some(
                                            attr.decode_and_unescape_value(reader)?.to_string(),
                                        );
                                    }
                                    _ => {}
                                }
                            }

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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut entity_name: Option<String> = None;
            let mut entity_index: Option<String> = None;
            let mut min: Option<f64> = None;
            let mut max: Option<f64> = None;
            let mut precision: Option<f64> = None;
            let mut value: Option<f64> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"entity_name" => {
                        entity_name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"entity_index" => {
                        entity_index = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"min" => {
                        let min_str = attr.decode_and_unescape_value(reader)?.to_string();
                        min = Some(min_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid min value: {}", e))
                        })?);
                    }
                    b"max" => {
                        let max_str = attr.decode_and_unescape_value(reader)?.to_string();
                        max = Some(max_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid max value: {}", e))
                        })?);
                    }
                    b"precision" => {
                        let precision_str = attr.decode_and_unescape_value(reader)?.to_string();
                        precision = Some(precision_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid precision value: {}", e))
                        })?);
                    }
                    b"value" => {
                        let value_str = attr.decode_and_unescape_value(reader)?.to_string();
                        value = Some(value_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut name: Option<String> = None;
            let mut column_width: Option<f64> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"name" => {
                        name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"column_width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        column_width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid column_width value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

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
            let mut xscale_min: Option<f64> = None;
            let mut xscale_max: Option<f64> = None;
            let mut ypts: Vec<f64> = Vec::new();

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"xscale_min" => {
                        let min_str = attr.decode_and_unescape_value(reader)?.to_string();
                        xscale_min = Some(min_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid xscale_min value: {}", e))
                        })?);
                    }
                    b"xscale_max" => {
                        let max_str = attr.decode_and_unescape_value(reader)?.to_string();
                        xscale_max = Some(max_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid xscale_max value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut entity_name: Option<String> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"entity_name" => {
                        entity_name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }

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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut entity_name: Option<String> = None;
            let mut show_name: Option<bool> = None;
            let mut retain_ending_value: Option<bool> = None;
            let mut precision: Option<f64> = None;
            let mut delimit_000s: Option<bool> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"entity_name" => {
                        entity_name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"show_name" => {
                        let show_str = attr.decode_and_unescape_value(reader)?.to_string();
                        show_name = Some(match show_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid show_name value: {}",
                                    show_str
                                )));
                            }
                        });
                    }
                    b"retain_ending_value" => {
                        let retain_str = attr.decode_and_unescape_value(reader)?.to_string();
                        retain_ending_value = Some(match retain_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid retain_ending_value: {}",
                                    retain_str
                                )));
                            }
                        });
                    }
                    b"precision" => {
                        let precision_str = attr.decode_and_unescape_value(reader)?.to_string();
                        precision = Some(precision_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid precision value: {}", e))
                        })?);
                    }
                    b"delimit_000s" => {
                        let delimit_str = attr.decode_and_unescape_value(reader)?.to_string();
                        delimit_000s = Some(match delimit_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid delimit_000s value: {}",
                                    delimit_str
                                )));
                            }
                        });
                    }
                    _ => {}
                }
            }

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
            let mut zone_type: Option<ZoneType> = None;
            let mut color: Option<Color> = None;
            let mut min: Option<f64> = None;
            let mut max: Option<f64> = None;
            let mut sound: Option<String> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"type" => {
                        let type_str = attr.decode_and_unescape_value(reader)?.to_string();
                        zone_type = Some(parse_zone_type(&type_str)?);
                    }
                    b"color" => {
                        let color_str = attr.decode_and_unescape_value(reader)?.to_string();
                        color = Some(parse_color(&color_str)?);
                    }
                    b"min" => {
                        let min_str = attr.decode_and_unescape_value(reader)?.to_string();
                        min = Some(min_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid zone min value: {}", e))
                        })?);
                    }
                    b"max" => {
                        let max_str = attr.decode_and_unescape_value(reader)?.to_string();
                        max = Some(max_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid zone max value: {}", e))
                        })?);
                    }
                    b"sound" => {
                        sound = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }

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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut entity_name: Option<String> = None;
            let mut show_name: Option<bool> = None;
            let mut retain_ending_value: Option<bool> = None;
            let mut flash_on_panic: Option<bool> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"entity_name" => {
                        entity_name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"show_name" => {
                        let show_str = attr.decode_and_unescape_value(reader)?.to_string();
                        show_name = Some(match show_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid show_name value: {}",
                                    show_str
                                )));
                            }
                        });
                    }
                    b"retain_ending_value" => {
                        let retain_str = attr.decode_and_unescape_value(reader)?.to_string();
                        retain_ending_value = Some(match retain_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid retain_ending_value: {}",
                                    retain_str
                                )));
                            }
                        });
                    }
                    b"flash_on_panic" => {
                        let flash_str = attr.decode_and_unescape_value(reader)?.to_string();
                        flash_on_panic = Some(match flash_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid flash_on_panic: {}",
                                    flash_str
                                )));
                            }
                        });
                    }
                    _ => {}
                }
            }

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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut entity_name: Option<String> = None;
            let mut show_name: Option<bool> = None;
            let mut show_number: Option<bool> = None;
            let mut retain_ending_value: Option<bool> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"entity_name" => {
                        entity_name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"show_name" => {
                        let show_str = attr.decode_and_unescape_value(reader)?.to_string();
                        show_name = Some(match show_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid show_name value: {}",
                                    show_str
                                )));
                            }
                        });
                    }
                    b"show_number" => {
                        let show_str = attr.decode_and_unescape_value(reader)?.to_string();
                        show_number = Some(match show_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid show_number value: {}",
                                    show_str
                                )));
                            }
                        });
                    }
                    b"retain_ending_value" => {
                        let retain_str = attr.decode_and_unescape_value(reader)?.to_string();
                        retain_ending_value = Some(match retain_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid retain_ending_value: {}",
                                    retain_str
                                )));
                            }
                        });
                    }
                    _ => {}
                }
            }

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
            let mut min: Option<f64> = None;
            let mut max: Option<f64> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"min" => {
                        let min_str = attr.decode_and_unescape_value(reader)?.to_string();
                        min = Some(min_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid plot scale min value: {}", e))
                        })?);
                    }
                    b"max" => {
                        let max_str = attr.decode_and_unescape_value(reader)?.to_string();
                        max = Some(max_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid plot scale max value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

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
            let mut index: Option<u32> = None;
            let mut pen_width: Option<f64> = None;
            let mut pen_style: Option<PenStyle> = None;
            let mut show_y_axis: Option<bool> = None;
            let mut title: Option<String> = None;
            let mut right_axis: Option<bool> = None;
            let mut entity_name: Option<String> = None;
            let mut precision: Option<f64> = None;
            let mut color: Option<Color> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"index" => {
                        let index_str = attr.decode_and_unescape_value(reader)?.to_string();
                        index = Some(index_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid plot index value: {}", e))
                        })?);
                    }
                    b"pen_width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        pen_width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid pen_width value: {}", e))
                        })?);
                    }
                    b"pen_style" => {
                        let style_str = attr.decode_and_unescape_value(reader)?.to_string();
                        pen_style = Some(parse_pen_style(&style_str)?);
                    }
                    b"show_y_axis" => {
                        let show_str = attr.decode_and_unescape_value(reader)?.to_string();
                        show_y_axis = Some(match show_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid show_y_axis value: {}",
                                    show_str
                                )));
                            }
                        });
                    }
                    b"title" => {
                        title = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"right_axis" => {
                        let right_str = attr.decode_and_unescape_value(reader)?.to_string();
                        right_axis = Some(match right_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid right_axis value: {}",
                                    right_str
                                )));
                            }
                        });
                    }
                    b"entity_name" => {
                        entity_name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"precision" => {
                        let precision_str = attr.decode_and_unescape_value(reader)?.to_string();
                        precision = Some(precision_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid precision value: {}", e))
                        })?);
                    }
                    b"color" => {
                        let color_str = attr.decode_and_unescape_value(reader)?.to_string();
                        color = Some(parse_color(&color_str)?);
                    }
                    _ => {}
                }
            }

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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut graph_type: Option<GraphType> = None;
            let mut title: Option<String> = None;
            let mut doc: Option<String> = None;
            let mut show_grid: Option<bool> = None;
            let mut num_x_grid_lines: Option<u32> = None;
            let mut num_y_grid_lines: Option<u32> = None;
            let mut num_x_labels: Option<u32> = None;
            let mut num_y_labels: Option<u32> = None;
            let mut x_axis_title: Option<String> = None;
            let mut right_axis_title: Option<String> = None;
            let mut right_axis_auto_scale: Option<bool> = None;
            let mut right_axis_multi_scale: Option<bool> = None;
            let mut left_axis_title: Option<String> = None;
            let mut left_axis_auto_scale: Option<bool> = None;
            let mut left_axis_multi_scale: Option<bool> = None;
            let mut plot_numbers: Option<bool> = None;
            let mut comparative: Option<bool> = None;
            let mut from: Option<f64> = None;
            let mut to: Option<f64> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"graph_type" => {
                        let type_str = attr.decode_and_unescape_value(reader)?.to_string();
                        graph_type = Some(parse_graph_type(&type_str)?);
                    }
                    b"title" => {
                        title = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"doc" => {
                        doc = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"show_grid" => {
                        let show_str = attr.decode_and_unescape_value(reader)?.to_string();
                        show_grid = Some(match show_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid show_grid value: {}",
                                    show_str
                                )));
                            }
                        });
                    }
                    b"num_x_grid_lines" => {
                        let num_str = attr.decode_and_unescape_value(reader)?.to_string();
                        num_x_grid_lines = Some(num_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!(
                                "Invalid num_x_grid_lines value: {}",
                                e
                            ))
                        })?);
                    }
                    b"num_y_grid_lines" => {
                        let num_str = attr.decode_and_unescape_value(reader)?.to_string();
                        num_y_grid_lines = Some(num_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!(
                                "Invalid num_y_grid_lines value: {}",
                                e
                            ))
                        })?);
                    }
                    b"num_x_labels" => {
                        let num_str = attr.decode_and_unescape_value(reader)?.to_string();
                        num_x_labels = Some(num_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid num_x_labels value: {}", e))
                        })?);
                    }
                    b"num_y_labels" => {
                        let num_str = attr.decode_and_unescape_value(reader)?.to_string();
                        num_y_labels = Some(num_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid num_y_labels value: {}", e))
                        })?);
                    }
                    b"x_axis_title" => {
                        x_axis_title = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"right_axis_title" => {
                        right_axis_title =
                            Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"right_axis_auto_scale" => {
                        let auto_str = attr.decode_and_unescape_value(reader)?.to_string();
                        right_axis_auto_scale = Some(match auto_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid right_axis_auto_scale value: {}",
                                    auto_str
                                )));
                            }
                        });
                    }
                    b"right_axis_multi_scale" => {
                        let multi_str = attr.decode_and_unescape_value(reader)?.to_string();
                        right_axis_multi_scale = Some(match multi_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid right_axis_multi_scale value: {}",
                                    multi_str
                                )));
                            }
                        });
                    }
                    b"left_axis_title" => {
                        left_axis_title = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"left_axis_auto_scale" => {
                        let auto_str = attr.decode_and_unescape_value(reader)?.to_string();
                        left_axis_auto_scale = Some(match auto_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid left_axis_auto_scale value: {}",
                                    auto_str
                                )));
                            }
                        });
                    }
                    b"left_axis_multi_scale" => {
                        let multi_str = attr.decode_and_unescape_value(reader)?.to_string();
                        left_axis_multi_scale = Some(match multi_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid left_axis_multi_scale value: {}",
                                    multi_str
                                )));
                            }
                        });
                    }
                    b"plot_numbers" => {
                        let plot_str = attr.decode_and_unescape_value(reader)?.to_string();
                        plot_numbers = Some(match plot_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid plot_numbers value: {}",
                                    plot_str
                                )));
                            }
                        });
                    }
                    b"comparative" => {
                        let comp_str = attr.decode_and_unescape_value(reader)?.to_string();
                        comparative = Some(match comp_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid comparative value: {}",
                                    comp_str
                                )));
                            }
                        });
                    }
                    b"from" => {
                        let from_str = attr.decode_and_unescape_value(reader)?.to_string();
                        from = Some(from_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid from value: {}", e))
                        })?);
                    }
                    b"to" => {
                        let to_str = attr.decode_and_unescape_value(reader)?.to_string();
                        to = Some(to_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid to value: {}", e))
                        })?);
                    }
                    _ => {}
                }
            }

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
            let mut item_type: Option<TableItemType> = None;
            let mut entity_name: Option<String> = None;
            let mut precision: Option<f64> = None;
            let mut delimit_000s: Option<bool> = None;
            let mut column_width: Option<f64> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"type" => {
                        let type_str = attr.decode_and_unescape_value(reader)?.to_string();
                        item_type = Some(parse_table_item_type(&type_str)?);
                    }
                    b"entity_name" => {
                        entity_name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"precision" => {
                        let precision_str = attr.decode_and_unescape_value(reader)?.to_string();
                        precision = Some(precision_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid precision value: {}", e))
                        })?);
                    }
                    b"delimit_000s" => {
                        let delimit_str = attr.decode_and_unescape_value(reader)?.to_string();
                        delimit_000s = Some(match delimit_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid delimit_000s value: {}",
                                    delimit_str
                                )));
                            }
                        });
                    }
                    b"column_width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        column_width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid column_width value: {}", e))
                        })?);
                    }
                    _ => {}
                }
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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut title: Option<String> = None;
            let mut doc: Option<String> = None;
            let mut orientation: Option<TableOrientation> = None;
            let mut column_width: Option<f64> = None;
            let mut blank_column_width: Option<f64> = None;
            let mut interval: Option<String> = None;
            let mut report_balances: Option<ReportBalances> = None;
            let mut report_flows: Option<ReportFlows> = None;
            let mut comparative: Option<bool> = None;
            let mut wrap_text: Option<bool> = None;

            // Header style attributes
            let mut header_font_family: Option<String> = None;
            let mut header_font_size: Option<f64> = None;
            let mut header_font_weight: Option<FontWeight> = None;
            let mut header_font_style: Option<FontStyle> = None;
            let mut header_text_decoration: Option<TextDecoration> = None;
            let mut header_text_align: Option<TextAlign> = None;
            let mut header_vertical_text_align: Option<VerticalTextAlign> = None;
            let mut header_text_background: Option<Color> = None;
            let mut header_text_padding: Option<(
                Option<f64>,
                Option<f64>,
                Option<f64>,
                Option<f64>,
            )> = None;
            let mut header_font_color: Option<Color> = None;
            let mut header_text_border_color: Option<Color> = None;
            let mut header_text_border_width: Option<BorderWidth> = None;
            let mut header_text_border_style: Option<BorderStyle> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"title" => {
                        title = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"doc" => {
                        doc = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"orientation" => {
                        let orient_str = attr.decode_and_unescape_value(reader)?.to_string();
                        orientation = Some(parse_table_orientation(&orient_str)?);
                    }
                    b"column_width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        column_width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid column_width value: {}", e))
                        })?);
                    }
                    b"blank_column_width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        blank_column_width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!(
                                "Invalid blank_column_width value: {}",
                                e
                            ))
                        })?);
                    }
                    b"interval" => {
                        interval = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"report_balances" => {
                        let balances_str = attr.decode_and_unescape_value(reader)?.to_string();
                        report_balances = Some(parse_report_balances(&balances_str)?);
                    }
                    b"report_flows" => {
                        let flows_str = attr.decode_and_unescape_value(reader)?.to_string();
                        report_flows = Some(parse_report_flows(&flows_str)?);
                    }
                    b"comparative" => {
                        let comp_str = attr.decode_and_unescape_value(reader)?.to_string();
                        comparative = Some(match comp_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid comparative value: {}",
                                    comp_str
                                )));
                            }
                        });
                    }
                    b"wrap_text" => {
                        let wrap_str = attr.decode_and_unescape_value(reader)?.to_string();
                        wrap_text = Some(match wrap_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid wrap_text value: {}",
                                    wrap_str
                                )));
                            }
                        });
                    }
                    // Header style attributes
                    b"header_font_family" => {
                        header_font_family =
                            Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"header_font_size" => {
                        let size_str = attr.decode_and_unescape_value(reader)?.to_string();
                        // Remove "pt" suffix if present
                        let size_str = size_str.trim_end_matches("pt");
                        header_font_size = Some(size_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!(
                                "Invalid header_font_size value: {}",
                                e
                            ))
                        })?);
                    }
                    b"header_font_weight" => {
                        let weight_str = attr.decode_and_unescape_value(reader)?.to_string();
                        header_font_weight = Some(parse_font_weight(&weight_str)?);
                    }
                    b"header_font_style" => {
                        let style_str = attr.decode_and_unescape_value(reader)?.to_string();
                        header_font_style = Some(parse_font_style(&style_str)?);
                    }
                    b"header_text_decoration" => {
                        let dec_str = attr.decode_and_unescape_value(reader)?.to_string();
                        header_text_decoration = Some(parse_text_decoration(&dec_str)?);
                    }
                    b"header_text_align" => {
                        let align_str = attr.decode_and_unescape_value(reader)?.to_string();
                        header_text_align = Some(parse_text_align(&align_str)?);
                    }
                    b"header_vertical_text_align" => {
                        let align_str = attr.decode_and_unescape_value(reader)?.to_string();
                        header_vertical_text_align = Some(parse_vertical_text_align(&align_str)?);
                    }
                    b"header_text_background" => {
                        let bg_str = attr.decode_and_unescape_value(reader)?.to_string();
                        header_text_background = Some(parse_color(&bg_str)?);
                    }
                    b"header_text_padding" => {
                        let padding_str = attr.decode_and_unescape_value(reader)?.to_string();
                        header_text_padding = Some(parse_text_padding(&padding_str)?);
                    }
                    b"header_font_color" => {
                        let color_str = attr.decode_and_unescape_value(reader)?.to_string();
                        header_font_color = Some(parse_color(&color_str)?);
                    }
                    b"header_text_border_color" => {
                        let color_str = attr.decode_and_unescape_value(reader)?.to_string();
                        header_text_border_color = Some(parse_color(&color_str)?);
                    }
                    b"header_text_border_width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        header_text_border_width = Some(parse_border_width(&width_str)?);
                    }
                    b"header_text_border_style" => {
                        let style_str = attr.decode_and_unescape_value(reader)?.to_string();
                        header_text_border_style = Some(match style_str.as_str() {
                            "none" => BorderStyle::None,
                            "solid" => BorderStyle::Solid,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid header_text_border_style: {}",
                                    style_str
                                )));
                            }
                        });
                    }
                    _ => {}
                }
            }

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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut appearance: Option<TextBoxAppearance> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"appearance" => {
                        let app_str = attr.decode_and_unescape_value(reader)?.to_string();
                        appearance = Some(parse_text_box_appearance(&app_str)?);
                    }
                    _ => {}
                }
            }

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
            let mut size_to_parent: Option<bool> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut resource: Option<String> = None;
            let mut data: Option<String> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"size_to_parent" => {
                        let size_str = attr.decode_and_unescape_value(reader)?.to_string();
                        size_to_parent = Some(match size_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid size_to_parent value: {}",
                                    size_str
                                )));
                            }
                        });
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid image width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid image height value: {}", e))
                        })?);
                    }
                    b"resource" => {
                        resource = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"data" => {
                        data = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }

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
            let mut size_to_parent: Option<bool> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut resource: Option<String> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"size_to_parent" => {
                        let size_str = attr.decode_and_unescape_value(reader)?.to_string();
                        size_to_parent = Some(match size_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid size_to_parent value: {}",
                                    size_str
                                )));
                            }
                        });
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid video width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid video height value: {}", e))
                        })?);
                    }
                    b"resource" => {
                        resource = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }

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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut border_color: Option<Color> = None;
            let mut border_style: Option<BorderStyle> = None;
            let mut border_width: Option<BorderWidth> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"border_color" => {
                        let color_str = attr.decode_and_unescape_value(reader)?.to_string();
                        border_color = Some(parse_color(&color_str)?);
                    }
                    b"border_style" => {
                        let style_str = attr.decode_and_unescape_value(reader)?.to_string();
                        border_style = Some(match style_str.as_str() {
                            "none" => BorderStyle::None,
                            "solid" => BorderStyle::Solid,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid border_style: {}",
                                    style_str
                                )));
                            }
                        });
                    }
                    b"border_width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        border_width = Some(parse_border_width(&width_str)?);
                    }
                    _ => {}
                }
            }

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
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut zoom: Option<f64> = None;
            let mut effect: Option<LinkEffect> = None;
            let mut to_black: Option<bool> = None;
            let mut target_type: Option<String> = None;
            let mut view_type: Option<String> = None;
            let mut order: Option<String> = None;
            let mut page: Option<String> = None;
            let mut url: Option<String> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid link x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid link y value: {}", e))
                        })?);
                    }
                    b"zoom" => {
                        let zoom_str = attr.decode_and_unescape_value(reader)?.to_string();
                        zoom = Some(zoom_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid link zoom value: {}", e))
                        })?);
                    }
                    b"effect" => {
                        let effect_str = attr.decode_and_unescape_value(reader)?.to_string();
                        effect = Some(parse_link_effect(&effect_str)?);
                    }
                    b"to_black" => {
                        let black_str = attr.decode_and_unescape_value(reader)?.to_string();
                        to_black = Some(match black_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid to_black value: {}",
                                    black_str
                                )));
                            }
                        });
                    }
                    b"target" => {
                        target_type = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"view_type" => {
                        view_type = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"order" => {
                        order = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"page" => {
                        page = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"url" => {
                        url = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }

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
            let mut action: Option<String> = None;
            let mut run_name: Option<String> = None;
            let mut resource: Option<String> = None;
            let mut worksheet: Option<String> = None;
            let mut all: Option<bool> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"action" => {
                        action = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"run_name" => {
                        run_name = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"resource" => {
                        resource = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"worksheet" => {
                        worksheet = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"all" => {
                        let all_str = attr.decode_and_unescape_value(reader)?.to_string();
                        all = Some(match all_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid all value: {}",
                                    all_str
                                )));
                            }
                        });
                    }
                    _ => {}
                }
            }

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
            let mut action_type: Option<String> = None;
            let mut action: Option<String> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"type" => {
                        action_type = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"action" => {
                        action = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }

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
            let mut size_to_parent: Option<bool> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut resource: Option<String> = None;
            let mut data: Option<String> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"size_to_parent" => {
                        let size_str = attr.decode_and_unescape_value(reader)?.to_string();
                        size_to_parent = Some(match size_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid size_to_parent value: {}",
                                    size_str
                                )));
                            }
                        });
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid image width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid image height value: {}", e))
                        })?);
                    }
                    b"resource" => {
                        resource = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"data" => {
                        data = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }

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
            let mut size_to_parent: Option<bool> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut resource: Option<String> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"size_to_parent" => {
                        let size_str = attr.decode_and_unescape_value(reader)?.to_string();
                        size_to_parent = Some(match size_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid size_to_parent value: {}",
                                    size_str
                                )));
                            }
                        });
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid video width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid video height value: {}", e))
                        })?);
                    }
                    b"resource" => {
                        resource = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }

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
            let mut uid: Option<i32> = None;
            let mut x: Option<f64> = None;
            let mut y: Option<f64> = None;
            let mut width: Option<f64> = None;
            let mut height: Option<f64> = None;
            let mut appearance: Option<ButtonAppearance> = None;
            let mut style: Option<ButtonStyle> = None;
            let mut label: Option<String> = None;
            let mut clicking_sound: Option<bool> = None;
            let mut sound: Option<String> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"uid" => {
                        let uid_str = attr.decode_and_unescape_value(reader)?.to_string();
                        uid = Some(uid_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid uid value: {}", e))
                        })?);
                    }
                    b"x" => {
                        let x_str = attr.decode_and_unescape_value(reader)?.to_string();
                        x = Some(x_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid x value: {}", e))
                        })?);
                    }
                    b"y" => {
                        let y_str = attr.decode_and_unescape_value(reader)?.to_string();
                        y = Some(y_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid y value: {}", e))
                        })?);
                    }
                    b"width" => {
                        let width_str = attr.decode_and_unescape_value(reader)?.to_string();
                        width = Some(width_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid width value: {}", e))
                        })?);
                    }
                    b"height" => {
                        let height_str = attr.decode_and_unescape_value(reader)?.to_string();
                        height = Some(height_str.parse().map_err(|e| {
                            DeserializeError::Custom(format!("Invalid height value: {}", e))
                        })?);
                    }
                    b"appearance" => {
                        let app_str = attr.decode_and_unescape_value(reader)?.to_string();
                        appearance = Some(parse_button_appearance(&app_str)?);
                    }
                    b"style" => {
                        let style_str = attr.decode_and_unescape_value(reader)?.to_string();
                        style = Some(parse_button_style(&style_str)?);
                    }
                    b"label" => {
                        label = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    b"clicking_sound" => {
                        let sound_str = attr.decode_and_unescape_value(reader)?.to_string();
                        clicking_sound = Some(match sound_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid clicking_sound value: {}",
                                    sound_str
                                )));
                            }
                        });
                    }
                    b"sound" => {
                        sound = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }

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
                            let mut size_to_parent: Option<bool> = None;
                            let mut width: Option<f64> = None;
                            let mut height: Option<f64> = None;
                            let mut resource: Option<String> = None;
                            let mut data: Option<String> = None;

                            for attr in e.attributes() {
                                let attr = attr?;
                                match attr.key.as_ref() {
                                    b"size_to_parent" => {
                                        let size_str =
                                            attr.decode_and_unescape_value(reader)?.to_string();
                                        size_to_parent = Some(match size_str.as_str() {
                                            "true" => true,
                                            "false" => false,
                                            _ => {
                                                return Err(DeserializeError::Custom(format!(
                                                    "Invalid size_to_parent value: {}",
                                                    size_str
                                                )));
                                            }
                                        });
                                    }
                                    b"width" => {
                                        let width_str =
                                            attr.decode_and_unescape_value(reader)?.to_string();
                                        width = Some(width_str.parse().map_err(|e| {
                                            DeserializeError::Custom(format!(
                                                "Invalid image width value: {}",
                                                e
                                            ))
                                        })?);
                                    }
                                    b"height" => {
                                        let height_str =
                                            attr.decode_and_unescape_value(reader)?.to_string();
                                        height = Some(height_str.parse().map_err(|e| {
                                            DeserializeError::Custom(format!(
                                                "Invalid image height value: {}",
                                                e
                                            ))
                                        })?);
                                    }
                                    b"resource" => {
                                        resource = Some(
                                            attr.decode_and_unescape_value(reader)?.to_string(),
                                        );
                                    }
                                    b"data" => {
                                        data = Some(
                                            attr.decode_and_unescape_value(reader)?.to_string(),
                                        );
                                    }
                                    _ => {}
                                }
                            }

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
                            let mut entity_name: Option<String> = None;
                            let mut group_name: Option<String> = None;
                            let mut module_name: Option<String> = None;
                            let mut value: Option<f64> = None;

                            for attr in e.attributes() {
                                let attr = attr?;
                                match attr.key.as_ref() {
                                    b"entity_name" => {
                                        entity_name = Some(
                                            attr.decode_and_unescape_value(reader)?.to_string(),
                                        );
                                    }
                                    b"group_name" => {
                                        group_name = Some(
                                            attr.decode_and_unescape_value(reader)?.to_string(),
                                        );
                                    }
                                    b"module_name" => {
                                        module_name = Some(
                                            attr.decode_and_unescape_value(reader)?.to_string(),
                                        );
                                    }
                                    b"value" => {
                                        let value_str =
                                            attr.decode_and_unescape_value(reader)?.to_string();
                                        value = Some(value_str.parse().map_err(|e| {
                                            DeserializeError::Custom(format!(
                                                "Invalid switch_action value: {}",
                                                e
                                            ))
                                        })?);
                                    }
                                    _ => {}
                                }
                            }

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
            let mut name: Option<Identifier> = None;
            let mut resource: Option<String> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"name" => {
                        let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                        name =
                            Some(Identifier::parse_from_attribute(&name_str).map_err(|err| {
                                DeserializeError::Custom(format!("Invalid module name: {}", err))
                            })?);
                    }
                    b"resource" => {
                        resource = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }

            let mut connections = Vec::new();
            let mut documentation: Option<Documentation> = None;

            loop {
                match reader.read_event_into(buf)? {
                    Event::Start(e) if e.name().as_ref() == b"connect" => {
                        let mut to: Option<String> = None;
                        let mut from: Option<String> = None;

                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"to" => {
                                    to = Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                                b"from" => {
                                    from =
                                        Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                                _ => {}
                            }
                        }

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
                        let mut to: Option<String> = None;
                        let mut from: Option<String> = None;

                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"to" => {
                                    to = Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                                b"from" => {
                                    from =
                                        Some(attr.decode_and_unescape_value(reader)?.to_string());
                                }
                                _ => {}
                            }
                        }

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
            let mut name: Option<Identifier> = None;
            let mut resource: Option<String> = None;

            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"name" => {
                        let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                        name =
                            Some(Identifier::parse_from_attribute(&name_str).map_err(|err| {
                                DeserializeError::Custom(format!("Invalid module name: {}", err))
                            })?);
                    }
                    b"resource" => {
                        resource = Some(attr.decode_and_unescape_value(reader)?.to_string());
                    }
                    _ => {}
                }
            }

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
                    let mut to: Option<String> = None;
                    let mut from: Option<String> = None;

                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            b"to" => {
                                to = Some(attr.decode_and_unescape_value(reader)?.to_string());
                            }
                            b"from" => {
                                from = Some(attr.decode_and_unescape_value(reader)?.to_string());
                            }
                            _ => {}
                        }
                    }

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
                    let mut to: Option<String> = None;
                    let mut from: Option<String> = None;

                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            b"to" => {
                                to = Some(attr.decode_and_unescape_value(reader)?.to_string());
                            }
                            b"from" => {
                                from = Some(attr.decode_and_unescape_value(reader)?.to_string());
                            }
                            _ => {}
                        }
                    }

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
            let mut name: Option<Identifier> = None;

            for attr in e.attributes() {
                let attr = attr?;
                if attr.key.as_ref() == b"name" {
                    let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                    name = Some(Identifier::parse_from_attribute(&name_str).map_err(|e| {
                        DeserializeError::Custom(format!("Invalid group name: {}", e))
                    })?);
                }
            }

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
                            let mut entity_name: Option<Identifier> = None;
                            let mut run: Option<bool> = None;

                            for attr in e.attributes() {
                                let attr = attr?;
                                match attr.key.as_ref() {
                                    b"name" => {
                                        let name_str =
                                            attr.decode_and_unescape_value(reader)?.to_string();
                                        entity_name = Some(
                                            Identifier::parse_from_attribute(&name_str).map_err(
                                                |e| {
                                                    DeserializeError::Custom(format!(
                                                        "Invalid entity name: {}",
                                                        e
                                                    ))
                                                },
                                            )?,
                                        );
                                    }
                                    b"run" => {
                                        let run_str =
                                            attr.decode_and_unescape_value(reader)?.to_string();
                                        run = Some(match run_str.as_str() {
                                            "true" => true,
                                            "false" => false,
                                            _ => {
                                                return Err(DeserializeError::Custom(format!(
                                                    "Invalid run value: {}",
                                                    run_str
                                                )));
                                            }
                                        });
                                    }
                                    _ => {}
                                }
                            }

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

/// Internal implementation of Group deserialization when start tag is already read.
pub(crate) fn deserialize_group_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<Identifier>,
    is_empty_tag: bool,
) -> Result<Group, DeserializeError> {
    let mut doc: Option<Documentation> = None;
    let mut entities = Vec::new();

    if !is_empty_tag {
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
                Event::Start(e) if e.name().as_ref() == b"entity" => {
                    let mut entity_name: Option<Identifier> = None;
                    let mut run: Option<bool> = None;

                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            b"name" => {
                                let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                                entity_name = Some(
                                    Identifier::parse_from_attribute(&name_str).map_err(|err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid entity name: {}",
                                            err
                                        ))
                                    })?,
                                );
                            }
                            b"run" => {
                                let run_str = attr.decode_and_unescape_value(reader)?.to_string();
                                run = Some(run_str == "true");
                            }
                            _ => {}
                        }
                    }
                    buf.clear();

                    // Read until end of entity
                    loop {
                        match reader.read_event_into(buf)? {
                            Event::End(end) if end.name().as_ref() == b"entity" => break,
                            Event::Eof => return Err(DeserializeError::UnexpectedEof),
                            _ => {}
                        }
                        buf.clear();
                    }

                    if let Some(name) = entity_name {
                        entities.push(GroupEntity {
                            name,
                            run: run.unwrap_or(false),
                        });
                    }
                }
                Event::Empty(e) if e.name().as_ref() == b"entity" => {
                    let mut entity_name: Option<Identifier> = None;
                    let mut run: Option<bool> = None;

                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            b"name" => {
                                let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                                entity_name = Some(
                                    Identifier::parse_from_attribute(&name_str).map_err(|err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid entity name: {}",
                                            err
                                        ))
                                    })?,
                                );
                            }
                            b"run" => {
                                let run_str = attr.decode_and_unescape_value(reader)?.to_string();
                                run = Some(run_str == "true");
                            }
                            _ => {}
                        }
                    }

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

    Ok(Group {
        name: name.ok_or_else(|| DeserializeError::MissingField("name".to_string()))?,
        doc,
        entities,
        display: Vec::new(), // Display UIDs are handled separately in views
    })
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
    let mut name: Option<Identifier> = None;
    let mut access: Option<crate::model::vars::AccessType> = None;
    let mut autoexport: Option<bool> = None;

    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"aux" => {
            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"name" => {
                        let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                        name = Some(Identifier::parse_from_attribute(&name_str).map_err(|e| {
                            DeserializeError::Custom(format!("Invalid identifier: {}", e))
                        })?);
                    }
                    b"access" => {
                        let access_str = attr.decode_and_unescape_value(reader)?.to_string();
                        access = Some(match access_str.as_str() {
                            "input" => crate::model::vars::AccessType::Input,
                            "output" => crate::model::vars::AccessType::Output,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid access type: {}",
                                    access_str
                                )));
                            }
                        });
                    }
                    b"autoexport" => {
                        let autoexport_str = attr.decode_and_unescape_value(reader)?.to_string();
                        autoexport = Some(match autoexport_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid autoexport value: {}",
                                    autoexport_str
                                )));
                            }
                        });
                    }
                    _ => {}
                }
            }
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
    }
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
                        use crate::equation::parse::unit_equation;
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
        dimensions: None, // TODO: Phase 3
        #[cfg(feature = "arrays")]
        elements,
        event_poster: None, // TODO: Phase 3
    })
}

/// Internal implementation of auxiliary deserialization.
pub(crate) fn deserialize_auxiliary_impl<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: Option<Identifier>,
    access: Option<crate::model::vars::AccessType>,
    autoexport: Option<bool>,
    _is_empty_tag: bool,
) -> Result<Auxiliary, DeserializeError> {
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
                        use crate::equation::parse::unit_equation;
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
        dimensions: None, // TODO: Phase 3
        #[cfg(feature = "arrays")]
        elements,
        event_poster: None, // TODO: Phase 3
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
    let mut name: Option<Identifier> = None;
    let mut access: Option<crate::model::vars::AccessType> = None;
    let mut autoexport: Option<bool> = None;

    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"flow" => {
            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"name" => {
                        let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                        name = Some(Identifier::parse_from_attribute(&name_str).map_err(|e| {
                            DeserializeError::Custom(format!("Invalid identifier: {}", e))
                        })?);
                    }
                    b"access" => {
                        let access_str = attr.decode_and_unescape_value(reader)?.to_string();
                        access = Some(match access_str.as_str() {
                            "input" => crate::model::vars::AccessType::Input,
                            "output" => crate::model::vars::AccessType::Output,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid access type: {}",
                                    access_str
                                )));
                            }
                        });
                    }
                    b"autoexport" => {
                        let autoexport_str = attr.decode_and_unescape_value(reader)?.to_string();
                        autoexport = Some(match autoexport_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid autoexport value: {}",
                                    autoexport_str
                                )));
                            }
                        });
                    }
                    _ => {}
                }
            }
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
    }
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
                        use crate::equation::parse::unit_equation;
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
                        // Extract range attributes
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid max: {}", err))
                                    })?);
                                }
                                _ => {}
                            }
                        }
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
                        // Extract scale attributes
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        let mut auto: Option<bool> = None;
                        let mut group: Option<u32> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid max: {}", err))
                                    })?);
                                }
                                b"auto" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    auto = Some(s == "true");
                                }
                                b"group" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    group = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid group: {}", err))
                                    })?);
                                }
                                _ => {}
                            }
                        }
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
                        // Extract format attributes
                        let mut precision: Option<f64> = None;
                        let mut scale_by: Option<f64> = None;
                        let mut display_as: Option<DisplayAs> = None;
                        let mut delimit_000s: Option<bool> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"precision" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    precision = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid precision: {}",
                                            err
                                        ))
                                    })?);
                                }
                                b"scale_by" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    scale_by = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid scale_by: {}",
                                            err
                                        ))
                                    })?);
                                }
                                b"display_as" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    display_as = Some(match s.as_str() {
                                        "number" => DisplayAs::Number,
                                        "currency" => DisplayAs::Currency,
                                        "percent" => DisplayAs::Percent,
                                        _ => {
                                            return Err(DeserializeError::Custom(format!(
                                                "Invalid display_as: {}",
                                                s
                                            )));
                                        }
                                    });
                                }
                                b"delimit_000s" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    delimit_000s = Some(s == "true");
                                }
                                _ => {}
                            }
                        }
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
                        // Parse range from empty tag
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid max: {}", err))
                                    })?);
                                }
                                _ => {}
                            }
                        }
                        if let (Some(min_val), Some(max_val)) = (min, max) {
                            range = Some(DeviceRange {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"scale" => {
                        // Parse scale from empty tag
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        let mut auto: Option<bool> = None;
                        let mut group: Option<u32> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(s.parse().unwrap_or(0.0));
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(s.parse().unwrap_or(0.0));
                                }
                                b"auto" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    auto = Some(s == "true");
                                }
                                b"group" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    group = Some(s.parse().unwrap_or(0));
                                }
                                _ => {}
                            }
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
                        // Parse format from empty tag
                        let mut precision: Option<f64> = None;
                        let mut scale_by: Option<f64> = None;
                        let mut display_as: Option<DisplayAs> = None;
                        let mut delimit_000s: Option<bool> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"precision" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    precision = Some(s.parse().unwrap_or(0.0));
                                }
                                b"scale_by" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    scale_by = Some(s.parse().unwrap_or(0.0));
                                }
                                b"display_as" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    display_as = match s.as_str() {
                                        "number" => Some(DisplayAs::Number),
                                        "currency" => Some(DisplayAs::Currency),
                                        "percent" => Some(DisplayAs::Percent),
                                        _ => None,
                                    };
                                }
                                b"delimit_000s" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    delimit_000s = Some(s == "true");
                                }
                                _ => {}
                            }
                        }
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
        dimensions: None, // TODO: Phase 3
        #[cfg(feature = "arrays")]
        elements,
        event_poster: None, // TODO: Phase 3
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
    let mut name: Option<Identifier> = None;
    let mut access: Option<crate::model::vars::AccessType> = None;
    let mut autoexport: Option<bool> = None;

    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == b"stock" => {
            // Read attributes
            for attr in e.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"name" => {
                        let name_str = attr.decode_and_unescape_value(reader)?.to_string();
                        name = Some(Identifier::parse_from_attribute(&name_str).map_err(|e| {
                            DeserializeError::Custom(format!("Invalid identifier: {}", e))
                        })?);
                    }
                    b"access" => {
                        let access_str = attr.decode_and_unescape_value(reader)?.to_string();
                        access = Some(match access_str.as_str() {
                            "input" => crate::model::vars::AccessType::Input,
                            "output" => crate::model::vars::AccessType::Output,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid access type: {}",
                                    access_str
                                )));
                            }
                        });
                    }
                    b"autoexport" => {
                        let autoexport_str = attr.decode_and_unescape_value(reader)?.to_string();
                        autoexport = Some(match autoexport_str.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(DeserializeError::Custom(format!(
                                    "Invalid autoexport value: {}",
                                    autoexport_str
                                )));
                            }
                        });
                    }
                    _ => {}
                }
            }
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
    }
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

    // Read attributes
    for attr in start_event.attributes() {
        let attr = attr?;
        match attr.key.as_ref() {
            b"discrete" => {
                let value = attr.decode_and_unescape_value(reader)?.to_string();
                data.discrete = Some(value.parse::<bool>().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid discrete value: {}", e))
                })?);
            }
            b"batch_integrity" => {
                let value = attr.decode_and_unescape_value(reader)?.to_string();
                data.batch_integrity = Some(value.parse::<bool>().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid batch_integrity value: {}", e))
                })?);
            }
            b"one_at_a_time" => {
                let value = attr.decode_and_unescape_value(reader)?.to_string();
                data.one_at_a_time = Some(value.parse::<bool>().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid one_at_a_time value: {}", e))
                })?);
            }
            b"exponential_leak" => {
                let value = attr.decode_and_unescape_value(reader)?.to_string();
                data.exponential_leakage = Some(value.parse::<bool>().map_err(|e| {
                    DeserializeError::Custom(format!("Invalid exponential_leak value: {}", e))
                })?);
            }
            _ => {}
        }
    }

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
                        use crate::equation::parse::unit_equation;
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
                        // Extract attributes before clearing buf
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let min_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(min_str.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let max_str =
                                        attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(max_str.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid max: {}", err))
                                    })?);
                                }
                                _ => {}
                            }
                        }
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
                        // Extract attributes before clearing buf
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        let mut auto: Option<bool> = None;
                        let mut group: Option<u32> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid min: {}", err))
                                    })?);
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid max: {}", err))
                                    })?);
                                }
                                b"auto" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    auto = Some(s == "true");
                                }
                                b"group" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    group = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!("Invalid group: {}", err))
                                    })?);
                                }
                                _ => {}
                            }
                        }
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
                        // Extract attributes before clearing buf
                        let mut precision: Option<f64> = None;
                        let mut scale_by: Option<f64> = None;
                        let mut display_as: Option<DisplayAs> = None;
                        let mut delimit_000s: Option<bool> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"precision" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    precision = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid precision: {}",
                                            err
                                        ))
                                    })?);
                                }
                                b"scale_by" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    scale_by = Some(s.parse().map_err(|err| {
                                        DeserializeError::Custom(format!(
                                            "Invalid scale_by: {}",
                                            err
                                        ))
                                    })?);
                                }
                                b"display_as" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    display_as = Some(match s.as_str() {
                                        "number" => DisplayAs::Number,
                                        "currency" => DisplayAs::Currency,
                                        "percent" => DisplayAs::Percent,
                                        _ => {
                                            return Err(DeserializeError::Custom(format!(
                                                "Invalid display_as: {}",
                                                s
                                            )));
                                        }
                                    });
                                }
                                b"delimit_000s" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    delimit_000s = Some(s == "true");
                                }
                                _ => {}
                            }
                        }
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
                        // Extract attributes first
                        let mut cd = ConveyorData::default();
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"discrete" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    cd.discrete = Some(s == "true");
                                }
                                b"batch_integrity" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    cd.batch_integrity = Some(s == "true");
                                }
                                b"one_at_a_time" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    cd.one_at_a_time = Some(s == "true");
                                }
                                b"exponential_leak" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    cd.exponential_leakage = Some(s == "true");
                                }
                                _ => {}
                            }
                        }
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
                        // Empty <conveyor/> tag - parse attributes
                        conveyor_data = Some(ConveyorData::default());
                        // Read attributes from empty conveyor tag
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"discrete" => {
                                    let value = attr.decode_and_unescape_value(reader)?.to_string();
                                    if let Some(ref mut cd) = conveyor_data {
                                        cd.discrete = Some(value.parse::<bool>().unwrap_or(false));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    b"range" => {
                        // Parse range from empty tag
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(s.parse().unwrap_or(0.0));
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(s.parse().unwrap_or(0.0));
                                }
                                _ => {}
                            }
                        }
                        if let (Some(min_val), Some(max_val)) = (min, max) {
                            range = Some(DeviceRange {
                                min: min_val,
                                max: max_val,
                            });
                        }
                    }
                    b"scale" => {
                        // Parse scale from empty tag
                        let mut min: Option<f64> = None;
                        let mut max: Option<f64> = None;
                        let mut auto: Option<bool> = None;
                        let mut group: Option<u32> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"min" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    min = Some(s.parse().unwrap_or(0.0));
                                }
                                b"max" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    max = Some(s.parse().unwrap_or(0.0));
                                }
                                b"auto" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    auto = Some(s == "true");
                                }
                                b"group" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    group = Some(s.parse().unwrap_or(0));
                                }
                                _ => {}
                            }
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
                        // Parse format from empty tag
                        let mut precision: Option<f64> = None;
                        let mut scale_by: Option<f64> = None;
                        let mut display_as: Option<DisplayAs> = None;
                        let mut delimit_000s: Option<bool> = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"precision" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    precision = Some(s.parse().unwrap_or(0.0));
                                }
                                b"scale_by" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    scale_by = Some(s.parse().unwrap_or(0.0));
                                }
                                b"display_as" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    display_as = match s.as_str() {
                                        "number" => Some(DisplayAs::Number),
                                        "currency" => Some(DisplayAs::Currency),
                                        "percent" => Some(DisplayAs::Percent),
                                        _ => None,
                                    };
                                }
                                b"delimit_000s" => {
                                    let s = attr.decode_and_unescape_value(reader)?.to_string();
                                    delimit_000s = Some(s == "true");
                                }
                                _ => {}
                            }
                        }
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

// deserialize_macro moved to deserialize_macros module
