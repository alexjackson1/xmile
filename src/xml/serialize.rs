//! XML serialization using quick-xml.
//!
//! This module provides manual XML serialization for XMILE structures using quick-xml.
//! It handles edge cases (empty tags, optional fields, CDATA sections, etc.) naturally
//! and enables reliable round-trip testing with the deserialization module.

use quick_xml::Writer;
use quick_xml::events::{BytesDecl, Event};
use std::io::Write;
use thiserror::Error;

use crate::Expression;
use crate::behavior::Behavior;
use crate::data::{Data, DataExport, DataImport};
use crate::dimensions::{Dimension, Dimensions};
use crate::header::Header;
#[cfg(feature = "macros")]
use crate::r#macro::Macro;
use crate::model::events::{Event as ModelEvent, EventPoster, Threshold};
use crate::model::groups::Group;
use crate::model::object::{DeviceRange, DeviceScale, DisplayAs, FormatOptions};
use crate::model::vars::Variable;
#[cfg(feature = "arrays")]
use crate::model::vars::array::ArrayElement;
#[cfg(feature = "arrays")]
use crate::model::vars::array::VariableDimensions;
use crate::model::vars::gf::{
    GraphicalFunctionData, GraphicalFunctionPoints, GraphicalFunctionScale,
};
#[cfg(feature = "submodels")]
use crate::model::vars::module::Module;
use crate::model::vars::{
    aux::Auxiliary, flow::BasicFlow, gf::GraphicalFunction, stock::BasicStock,
};
#[allow(unused_imports)]
use crate::namespace::Namespace;
use crate::specs::SimulationSpecs;
use crate::units::{ModelUnits, UnitDefinition};
use crate::view::objects::{
    AliasObject, AuxObject, ButtonAppearance, ButtonObject, ButtonStyle, ConnectorObject,
    DataAction, FileAction, FlowObject, GaugeObject, GraphObject, GraphType, GraphicalInputObject,
    GraphicsFrameContent, GraphicsFrameObject, GroupObject, KnobObject, LampObject, LineStyle,
    Link, LinkEffect, LinkTarget, ListInputObject, MenuAction, MiscellaneousAction, ModuleObject,
    NumericDisplayObject, NumericInputObject, OptionsLayout, OptionsObject, PenStyle, Plot,
    PlotScale, Pointer, Polarity, PopupContent, PrintingAction, ReportBalances, ReportFlows,
    RestoreAction, Shape, SimulationAction, SliderObject, StackedContainerObject, StockObject,
    SwitchAction, SwitchObject, SwitchStyle, TableItem, TableItemType, TableObject,
    TableOrientation, TextBoxAppearance, TextBoxObject, Zone, ZoneType,
};
use crate::view::style::{
    BorderStyle, BorderWidth, Color, FontStyle, FontWeight, ObjectStyle, Padding, Style, TextAlign,
    TextDecoration, VerticalTextAlign,
};
use crate::view::{PageOrientation, PageSequence, View, ViewType};
use crate::xml::quick::ser::{AttrList, XmlEmitter};
use crate::xml::schema::{Model, Variables};

/// Errors that can occur during XML serialization.
#[derive(Debug, Error)]
pub enum SerializeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 encoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("XML serialization error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("Serialization error: {0}")]
    Custom(String),
}

/// Helper type for XML writer operations.
pub type XmlWriter = Writer<Vec<u8>>;

/// Write an XML declaration.
pub fn write_xml_declaration(writer: &mut XmlWriter) -> Result<(), SerializeError> {
    let decl = BytesDecl::new("1.0", Some("UTF-8"), None);
    writer.write_event(Event::Decl(decl))?;
    Ok(())
}

/// Write an element with text content.
pub fn write_element_with_text<W: Write>(
    writer: &mut Writer<W>,
    name: &str,
    text: &str,
) -> Result<(), SerializeError> {
    writer
        .create_element(name)
        .write_text_content(quick_xml::events::BytesText::new(text))?;
    Ok(())
}

/// Write an optional element with text content (only if Some).
pub fn write_optional_element_with_text<W: Write>(
    writer: &mut Writer<W>,
    name: &str,
    text: Option<&str>,
) -> Result<(), SerializeError> {
    if let Some(text) = text {
        write_element_with_text(writer, name, text)?;
    }
    Ok(())
}

/// Write an element with a numeric value.
pub fn write_element_with_number<W: Write>(
    writer: &mut Writer<W>,
    name: &str,
    value: f64,
) -> Result<(), SerializeError> {
    // Format number without unnecessary decimal places
    let text = if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{}", value)
    };
    write_element_with_text(writer, name, &text)
}

/// Write an optional element with a numeric value (only if Some).
pub fn write_optional_element_with_number<W: Write>(
    writer: &mut Writer<W>,
    name: &str,
    value: Option<f64>,
) -> Result<(), SerializeError> {
    if let Some(value) = value {
        write_element_with_number(writer, name, value)?;
    }
    Ok(())
}

/// Write an empty element (self-closing tag).
pub fn write_empty_element<W: Write>(
    writer: &mut Writer<W>,
    name: &str,
) -> Result<(), SerializeError> {
    writer.create_element(name).write_empty()?;
    Ok(())
}

/// Write an optional empty element (only if Some(true) or Some(value)).
pub fn write_optional_empty_element<W: Write>(
    writer: &mut Writer<W>,
    name: &str,
    present: Option<bool>,
) -> Result<(), SerializeError> {
    if present == Some(true) {
        write_empty_element(writer, name)?;
    }
    Ok(())
}

/// Write a non_negative element.
///
/// non_negative is Option<Option<bool>>:
/// - None = don't write anything
/// - Some(None) = write empty tag <non_negative/>
/// - Some(Some(false)) = write <non_negative>false</non_negative>
/// - Some(Some(true)) = write <non_negative>true</non_negative>
pub fn write_non_negative<W: Write>(
    writer: &mut Writer<W>,
    non_negative: Option<Option<bool>>,
) -> Result<(), SerializeError> {
    if let Some(inner) = non_negative {
        match inner {
            None => {
                // Empty tag means true (default)
                write_empty_element(writer, "non_negative")?;
            }
            Some(false) => {
                write_element_with_text(writer, "non_negative", "false")?;
            }
            Some(true) => {
                write_element_with_text(writer, "non_negative", "true")?;
            }
        }
    }
    Ok(())
}

/// Serialize a Header structure to XML.
pub fn serialize_header<W: Write>(
    writer: &mut Writer<W>,
    header: &Header,
) -> Result<(), SerializeError> {
    writer.create_element("header").write_inner_content(
        |writer| -> Result<(), SerializeError> {
            let mut emit = XmlEmitter::new(writer);

            // Required: vendor
            emit.text_elem("vendor", &header.vendor)?;

            // Required: product (with attributes and text content)
            let mut product_attrs = AttrList::new();
            product_attrs
                .add("version", &header.product.version)
                .add_opt("lang", header.product.lang.as_deref());
            emit.text_elem_with_attrs("product", &product_attrs, &header.product.name)?;

            // Optional fields
            emit.opt_text_elem("name", header.name.as_deref())?;
            emit.opt_text_elem("version", header.version_info.as_deref())?;
            emit.opt_text_elem("caption", header.caption.as_deref())?;
            emit.opt_text_elem("image", header.image.as_deref())?;
            emit.opt_text_elem("author", header.author.as_deref())?;
            emit.opt_text_elem("affiliation", header.affiliation.as_deref())?;
            emit.opt_text_elem("client", header.client.as_deref())?;
            emit.opt_text_elem("copyright", header.copyright.as_deref())?;

            // Contact (optional, complex structure)
            if let Some(contact) = &header.contact {
                emit.writer()
                    .create_element("contact")
                    .write_inner_content(|writer| -> Result<(), SerializeError> {
                        let mut emit = XmlEmitter::new(writer);
                        emit.opt_text_elem("address", contact.address.as_deref())?;
                        emit.opt_text_elem("phone", contact.phone.as_deref())?;
                        emit.opt_text_elem("fax", contact.fax.as_deref())?;
                        emit.opt_text_elem("email", contact.email.as_deref())?;
                        emit.opt_text_elem("website", contact.website.as_deref())?;
                        Ok(())
                    })?;
            }

            emit.opt_text_elem("created", header.created.as_deref())?;
            emit.opt_text_elem("modified", header.modified.as_deref())?;
            emit.opt_text_elem("uuid", header.uuid.as_deref())?;

            // Options (optional, complex structure)
            // TODO: Implement full Options serialization in Phase 2

            // Includes (optional)
            if let Some(includes) = &header.includes {
                emit.writer()
                    .create_element("includes")
                    .write_inner_content(|writer| -> Result<(), SerializeError> {
                        let mut emit = XmlEmitter::new(writer);
                        for include in &includes.includes {
                            let mut attrs = AttrList::new();
                            attrs.add("resource", &include.resource);
                            emit.empty_elem_with_attrs("include", &attrs)?;
                        }
                        Ok(())
                    })?;
            }

            Ok(())
        },
    )?;

    Ok(())
}

/// Serialize a SimulationSpecs structure to XML.
pub fn serialize_sim_specs<W: Write>(
    writer: &mut Writer<W>,
    specs: &SimulationSpecs,
) -> Result<(), SerializeError> {
    writer.create_element("sim_specs").write_inner_content(
        |writer| -> Result<(), SerializeError> {
            // Required: start and stop
            write_element_with_number(writer, "start", specs.start)?;
            write_element_with_number(writer, "stop", specs.stop)?;

            // Optional fields
            write_optional_element_with_number(writer, "dt", specs.dt)?;
            write_optional_element_with_text(writer, "method", specs.method.as_deref())?;
            write_optional_element_with_text(writer, "time_units", specs.time_units.as_deref())?;
            write_optional_element_with_number(writer, "pause", specs.pause)?;
            write_optional_element_with_text(writer, "run", specs.run_by.as_deref())?;

            Ok(())
        },
    )?;

    Ok(())
}

/// Write an element with CDATA content (for equations that may contain special characters).
pub fn write_element_with_cdata<W: Write>(
    writer: &mut Writer<W>,
    name: &str,
    text: &str,
) -> Result<(), SerializeError> {
    writer
        .create_element(name)
        .write_inner_content(|writer| -> Result<(), SerializeError> {
            writer.write_event(quick_xml::events::Event::CData(
                quick_xml::events::BytesCData::new(text),
            ))?;
            Ok(())
        })?;
    Ok(())
}

/// Write an optional element with CDATA content.
pub fn write_optional_element_with_cdata<W: Write>(
    writer: &mut Writer<W>,
    name: &str,
    text: Option<&str>,
) -> Result<(), SerializeError> {
    if let Some(text) = text {
        write_element_with_cdata(writer, name, text)?;
    }
    Ok(())
}

/// Write an expression element (using CDATA for safety).
pub fn write_expression<W: Write>(
    writer: &mut Writer<W>,
    expr: &Expression,
) -> Result<(), SerializeError> {
    let expr_str = expr.to_string();
    write_element_with_cdata(writer, "eqn", &expr_str)
}

/// Write an optional expression element.
pub fn write_optional_expression<W: Write>(
    writer: &mut Writer<W>,
    expr: Option<&Expression>,
) -> Result<(), SerializeError> {
    if let Some(expr) = expr {
        write_expression(writer, expr)?;
    }
    Ok(())
}

/// Serialize an Auxiliary variable to XML.
pub fn serialize_auxiliary<W: Write>(
    writer: &mut Writer<W>,
    aux: &Auxiliary,
) -> Result<(), SerializeError> {
    let mut aux_elem = writer.create_element("aux");

    // Required: name attribute
    aux_elem = aux_elem.with_attribute(("name", aux.name.to_string().as_str()));

    // Optional attributes
    if let Some(access) = &aux.access {
        aux_elem = aux_elem.with_attribute((
            "access",
            match access {
                crate::model::vars::AccessType::Input => "input",
                crate::model::vars::AccessType::Output => "output",
            },
        ));
    }
    if let Some(autoexport) = aux.autoexport {
        aux_elem =
            aux_elem.with_attribute(("autoexport", if autoexport { "true" } else { "false" }));
    }

    aux_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        // Required: equation
        write_expression(writer, &aux.equation)?;

        // Optional: MathML equation
        #[cfg(feature = "mathml")]
        if let Some(ref mathml) = aux.mathml_equation {
            write_element_with_text(writer, "mathml", mathml)?;
        }

        // Optional fields
        if let Some(ref doc) = aux.documentation {
            let doc_str = match doc {
                crate::model::object::Documentation::PlainText(text) => text,
                crate::model::object::Documentation::Html(html) => html,
            };
            write_element_with_text(writer, "doc", doc_str)?;
        }
        if let Some(ref units) = aux.units {
            write_element_with_text(writer, "units", &units.to_string())?;
        }
        if let Some(ref range) = aux.range {
            serialize_range(writer, range)?;
        }
        if let Some(ref scale) = aux.scale {
            serialize_scale(writer, scale)?;
        }
        if let Some(ref format) = aux.format {
            serialize_format(writer, format)?;
        }
        if let Some(ref event_poster) = aux.event_poster {
            serialize_event_poster(writer, event_poster)?;
        }
        // Array dimensions
        #[cfg(feature = "arrays")]
        {
            if let Some(ref dimensions) = aux.dimensions {
                serialize_dimensions(writer, dimensions)?;
            }
        }
        // Array elements
        #[cfg(feature = "arrays")]
        {
            for element in &aux.elements {
                serialize_array_element(writer, element)?;
            }
        }

        Ok(())
    })?;

    Ok(())
}

/// Serialize a BasicFlow variable to XML.
pub fn serialize_basic_flow<W: Write>(
    writer: &mut Writer<W>,
    flow: &BasicFlow,
) -> Result<(), SerializeError> {
    let mut flow_elem = writer.create_element("flow");

    // Required: name attribute
    flow_elem = flow_elem.with_attribute(("name", flow.name.to_string().as_str()));

    // Optional attributes
    if let Some(access) = &flow.access {
        flow_elem = flow_elem.with_attribute((
            "access",
            match access {
                crate::model::vars::AccessType::Input => "input",
                crate::model::vars::AccessType::Output => "output",
            },
        ));
    }
    if let Some(autoexport) = flow.autoexport {
        flow_elem =
            flow_elem.with_attribute(("autoexport", if autoexport { "true" } else { "false" }));
    }

    flow_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        // Optional: equation
        write_optional_expression(writer, flow.equation.as_ref())?;

        // Optional: MathML equation
        #[cfg(feature = "mathml")]
        if let Some(ref mathml) = flow.mathml_equation {
            write_element_with_text(writer, "mathml", mathml)?;
        }

        // Optional: multiplier
        if let Some(multiplier) = flow.multiplier {
            write_element_with_number(writer, "multiplier", multiplier)?;
        }

        // Optional fields
        if let Some(ref doc) = flow.documentation {
            let doc_str = match doc {
                crate::model::object::Documentation::PlainText(text) => text,
                crate::model::object::Documentation::Html(html) => html,
            };
            write_element_with_text(writer, "doc", doc_str)?;
        }
        if let Some(ref units) = flow.units {
            write_element_with_text(writer, "units", &units.to_string())?;
        }
        // non_negative
        write_non_negative(writer, flow.non_negative)?;
        if let Some(ref range) = flow.range {
            serialize_range(writer, range)?;
        }
        if let Some(ref scale) = flow.scale {
            serialize_scale(writer, scale)?;
        }
        if let Some(ref format) = flow.format {
            serialize_format(writer, format)?;
        }
        if let Some(ref event_poster) = flow.event_poster {
            serialize_event_poster(writer, event_poster)?;
        }
        // Array dimensions
        #[cfg(feature = "arrays")]
        {
            if let Some(ref dimensions) = flow.dimensions {
                // flow.dimensions is Option<Vec<String>>, need to convert to VariableDimensions
                use crate::model::vars::array::Dimension;
                let dims = dimensions
                    .iter()
                    .map(|name| Dimension { name: name.clone() })
                    .collect();
                let var_dims = VariableDimensions { dims };
                serialize_dimensions(writer, &var_dims)?;
            }
        }
        // Array elements
        #[cfg(feature = "arrays")]
        {
            for element in &flow.elements {
                serialize_array_element(writer, element)?;
            }
        }

        Ok(())
    })?;

    Ok(())
}

/// Serialize a BasicStock variable to XML.
pub fn serialize_basic_stock<W: Write>(
    writer: &mut Writer<W>,
    stock: &BasicStock,
) -> Result<(), SerializeError> {
    let mut stock_elem = writer.create_element("stock");

    // Required: name attribute
    stock_elem = stock_elem.with_attribute(("name", stock.name.to_string().as_str()));

    // Optional attributes
    if let Some(access) = &stock.access {
        stock_elem = stock_elem.with_attribute((
            "access",
            match access {
                crate::model::vars::AccessType::Input => "input",
                crate::model::vars::AccessType::Output => "output",
            },
        ));
    }
    if let Some(autoexport) = stock.autoexport {
        stock_elem =
            stock_elem.with_attribute(("autoexport", if autoexport { "true" } else { "false" }));
    }

    stock_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        // Required: initial equation
        write_expression(writer, &stock.initial_equation)?;

        // Optional: MathML equation
        #[cfg(feature = "mathml")]
        if let Some(ref mathml) = stock.mathml_equation {
            write_element_with_text(writer, "mathml", mathml)?;
        }

        // Inflows (multiple)
        for inflow in &stock.inflows {
            write_element_with_text(writer, "inflow", &inflow.to_string())?;
        }

        // Outflows (multiple)
        for outflow in &stock.outflows {
            write_element_with_text(writer, "outflow", &outflow.to_string())?;
        }

        // Optional fields
        if let Some(ref doc) = stock.documentation {
            let doc_str = match doc {
                crate::model::object::Documentation::PlainText(text) => text,
                crate::model::object::Documentation::Html(html) => html,
            };
            write_element_with_text(writer, "doc", doc_str)?;
        }
        if let Some(ref units) = stock.units {
            write_element_with_text(writer, "units", &units.to_string())?;
        }
        // non_negative
        write_non_negative(writer, stock.non_negative)?;
        if let Some(ref range) = stock.range {
            serialize_range(writer, range)?;
        }
        if let Some(ref scale) = stock.scale {
            serialize_scale(writer, scale)?;
        }
        if let Some(ref format) = stock.format {
            serialize_format(writer, format)?;
        }
        if let Some(ref event_poster) = stock.event_poster {
            serialize_event_poster(writer, event_poster)?;
        }
        // Array dimensions
        #[cfg(feature = "arrays")]
        {
            if let Some(ref dimensions) = stock.dimensions {
                // stock.dimensions is Option<Vec<String>>, need to convert to VariableDimensions
                use crate::model::vars::array::Dimension;
                let dims = dimensions
                    .iter()
                    .map(|name| Dimension { name: name.clone() })
                    .collect();
                let var_dims = VariableDimensions { dims };
                serialize_dimensions(writer, &var_dims)?;
            }
        }
        // Array elements
        #[cfg(feature = "arrays")]
        {
            for element in &stock.elements {
                serialize_array_element(writer, element)?;
            }
        }

        Ok(())
    })?;

    Ok(())
}

/// Serialize a GraphicalFunction variable to XML.
///
/// Note: This is a simplified version. Full implementation will handle
/// different data formats (uniform scale, x-y pairs) in later phases.
pub fn serialize_graphical_function<W: Write>(
    writer: &mut Writer<W>,
    gf: &GraphicalFunction,
) -> Result<(), SerializeError> {
    let mut gf_elem = writer.create_element("gf");

    // Optional: name attribute
    if let Some(ref name) = gf.name {
        gf_elem = gf_elem.with_attribute(("name", name.to_string().as_str()));
    }

    // Optional: type attribute
    if let Some(ref gf_type) = gf.r#type {
        let type_str = match gf_type {
            crate::model::vars::gf::GraphicalFunctionType::Continuous => "continuous",
            crate::model::vars::gf::GraphicalFunctionType::Extrapolate => "extrapolate",
            crate::model::vars::gf::GraphicalFunctionType::Discrete => "discrete",
        };
        gf_elem = gf_elem.with_attribute(("type", type_str));
    }

    gf_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        // Optional: equation
        write_optional_expression(writer, gf.equation.as_ref())?;

        // Optional: MathML equation
        #[cfg(feature = "mathml")]
        if let Some(ref mathml) = gf.mathml_equation {
            write_element_with_text(writer, "mathml", mathml)?;
        }

        // Required: data
        serialize_graphical_function_data(writer, &gf.data)?;

        // Optional fields
        if let Some(ref doc) = gf.documentation {
            let doc_str = match doc {
                crate::model::object::Documentation::PlainText(text) => text,
                crate::model::object::Documentation::Html(html) => html,
            };
            write_element_with_text(writer, "doc", doc_str)?;
        }
        if let Some(ref units) = gf.units {
            write_element_with_text(writer, "units", &units.to_string())?;
        }
        if let Some(ref range) = gf.range {
            serialize_range(writer, range)?;
        }
        if let Some(ref scale) = gf.scale {
            serialize_scale(writer, scale)?;
        }
        if let Some(ref format) = gf.format {
            serialize_format(writer, format)?;
        }
        // TODO: dimensions, elements

        Ok(())
    })?;

    Ok(())
}

/// Serialize GraphicalFunctionScale to XML.
fn serialize_gf_scale<W: Write>(
    writer: &mut Writer<W>,
    name: &str,
    scale: &GraphicalFunctionScale,
) -> Result<(), SerializeError> {
    let min_str = format!("{}", scale.min);
    let max_str = format!("{}", scale.max);
    writer
        .create_element(name)
        .with_attribute(("min", min_str.as_str()))
        .with_attribute(("max", max_str.as_str()))
        .write_empty()?;
    Ok(())
}

/// Serialize GraphicalFunctionPoints to XML.
fn serialize_gf_points<W: Write>(
    writer: &mut Writer<W>,
    name: &str,
    points: &GraphicalFunctionPoints,
) -> Result<(), SerializeError> {
    let sep = points.separator.as_deref().unwrap_or(",");
    let data: String = points
        .values
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(sep);

    let mut points_elem = writer.create_element(name);
    if let Some(ref separator) = points.separator {
        points_elem = points_elem.with_attribute(("sep", separator.as_str()));
    }
    points_elem.write_text_content(quick_xml::events::BytesText::new(&data))?;
    Ok(())
}

/// Serialize GraphicalFunctionData to XML.
fn serialize_graphical_function_data<W: Write>(
    writer: &mut Writer<W>,
    data: &GraphicalFunctionData,
) -> Result<(), SerializeError> {
    match data {
        GraphicalFunctionData::UniformScale {
            x_scale,
            y_scale,
            y_values,
        } => {
            // xscale element
            serialize_gf_scale(writer, "xscale", x_scale)?;

            // Optional yscale
            if let Some(y_scale) = y_scale {
                serialize_gf_scale(writer, "yscale", y_scale)?;
            }

            // ypts element (required)
            serialize_gf_points(writer, "ypts", y_values)?;
        }
        GraphicalFunctionData::XYPairs {
            x_values,
            y_values,
            y_scale,
        } => {
            // xpts element
            serialize_gf_points(writer, "xpts", x_values)?;

            // Optional yscale
            if let Some(y_scale) = y_scale {
                serialize_gf_scale(writer, "yscale", y_scale)?;
            }

            // ypts element (required)
            serialize_gf_points(writer, "ypts", y_values)?;
        }
    }
    Ok(())
}

/// Serialize VariableDimensions to XML.
#[cfg(feature = "arrays")]
pub fn serialize_dimensions<W: Write>(
    writer: &mut Writer<W>,
    dimensions: &VariableDimensions,
) -> Result<(), SerializeError> {
    writer.create_element("dimensions").write_inner_content(
        |writer| -> Result<(), SerializeError> {
            for dim in &dimensions.dims {
                writer
                    .create_element("dim")
                    .with_attribute(("name", dim.name.as_str()))
                    .write_empty()?;
            }
            Ok(())
        },
    )?;
    Ok(())
}

/// Serialize EventPoster to XML.
pub fn serialize_event_poster<W: Write>(
    writer: &mut Writer<W>,
    poster: &EventPoster,
) -> Result<(), SerializeError> {
    let min_str = format!("{}", poster.min);
    let max_str = format!("{}", poster.max);
    writer
        .create_element("event_poster")
        .with_attribute(("min", min_str.as_str()))
        .with_attribute(("max", max_str.as_str()))
        .write_inner_content(|writer| -> Result<(), SerializeError> {
            for threshold in &poster.thresholds {
                serialize_threshold(writer, threshold)?;
            }
            Ok(())
        })?;
    Ok(())
}

/// Serialize Threshold to XML.
fn serialize_threshold<W: Write>(
    writer: &mut Writer<W>,
    threshold: &Threshold,
) -> Result<(), SerializeError> {
    let value_str = format!("{}", threshold.value);
    let mut threshold_elem = writer
        .create_element("threshold")
        .with_attribute(("value", value_str.as_str()));

    if let Some(ref direction) = threshold.direction {
        threshold_elem = threshold_elem.with_attribute(("direction", direction.as_str()));
    }
    if let Some(ref repeat) = threshold.repeat {
        threshold_elem = threshold_elem.with_attribute(("repeat", repeat.as_str()));
    }
    if let Some(interval) = threshold.interval {
        let interval_str = format!("{}", interval);
        threshold_elem = threshold_elem.with_attribute(("interval", interval_str.as_str()));
    }

    threshold_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        for event in &threshold.events {
            serialize_event(writer, event)?;
        }
        Ok(())
    })?;
    Ok(())
}

/// Serialize Event to XML.
fn serialize_event<W: Write>(
    writer: &mut Writer<W>,
    event: &ModelEvent,
) -> Result<(), SerializeError> {
    let mut event_elem = writer.create_element("event");

    if let Some(ref sim_action) = event.sim_action {
        event_elem = event_elem.with_attribute(("sim_action", sim_action.as_str()));
    }

    // Events can have text content (actions)
    if !event.actions.is_empty() {
        let actions_text = event.actions.join(" ");
        event_elem.write_text_content(quick_xml::events::BytesText::new(&actions_text))?;
    } else {
        event_elem.write_empty()?;
    }
    Ok(())
}

/// Serialize Views structure to XML.
pub fn serialize_views<W: Write>(
    writer: &mut Writer<W>,
    views: &crate::xml::schema::Views,
) -> Result<(), SerializeError> {
    let mut views_elem = writer.create_element("views");

    if let Some(visible_view) = views.visible_view {
        let visible_str = format!("{}", visible_view);
        views_elem = views_elem.with_attribute(("visible_view", visible_str.as_str()));
    }

    views_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        // Optional style
        if let Some(ref style) = views.style {
            serialize_view_style(writer, style)?;
        }

        // Serialize all views
        for view in &views.views {
            serialize_view(writer, view)?;
        }

        Ok(())
    })?;
    Ok(())
}

/// Serialize a View to XML.
pub fn serialize_view<W: Write>(writer: &mut Writer<W>, view: &View) -> Result<(), SerializeError> {
    let mut view_elem = writer.create_element("view");

    // Required attributes
    let uid_str = format!("{}", view.uid.value);
    let width_str = format!("{}", view.width);
    let height_str = format!("{}", view.height);
    let page_width_str = format!("{}", view.page_width);
    let page_height_str = format!("{}", view.page_height);
    let home_page_str = format!("{}", view.home_page);

    view_elem = view_elem.with_attribute(("uid", uid_str.as_str()));
    view_elem = view_elem.with_attribute(("width", width_str.as_str()));
    view_elem = view_elem.with_attribute(("height", height_str.as_str()));
    view_elem = view_elem.with_attribute(("page_width", page_width_str.as_str()));
    view_elem = view_elem.with_attribute(("page_height", page_height_str.as_str()));
    view_elem = view_elem.with_attribute((
        "page_sequence",
        match view.page_sequence {
            PageSequence::Row => "row",
            PageSequence::Column => "column",
        },
    ));
    view_elem = view_elem.with_attribute((
        "page_orientation",
        match view.page_orientation {
            PageOrientation::Landscape => "landscape",
            PageOrientation::Portrait => "portrait",
        },
    ));
    view_elem =
        view_elem.with_attribute(("show_pages", if view.show_pages { "true" } else { "false" }));
    view_elem = view_elem.with_attribute(("home_page", home_page_str.as_str()));
    view_elem =
        view_elem.with_attribute(("home_view", if view.home_view { "true" } else { "false" }));

    // Optional attributes
    if let Some(order) = view.order {
        let order_str = format!("{}", order);
        view_elem = view_elem.with_attribute(("order", order_str.as_str()));
    }
    if let Some(zoom) = view.zoom {
        let zoom_str = format!("{}", zoom);
        view_elem = view_elem.with_attribute(("zoom", zoom_str.as_str()));
    }
    if let Some(scroll_x) = view.scroll_x {
        let scroll_x_str = format!("{}", scroll_x);
        view_elem = view_elem.with_attribute(("scroll_x", scroll_x_str.as_str()));
    }
    if let Some(scroll_y) = view.scroll_y {
        let scroll_y_str = format!("{}", scroll_y);
        view_elem = view_elem.with_attribute(("scroll_y", scroll_y_str.as_str()));
    }
    if let Some(ref background) = view.background {
        view_elem = view_elem.with_attribute(("background", background.as_str()));
    }

    // View type
    match &view.view_type {
        ViewType::StockFlow => {
            view_elem = view_elem.with_attribute(("type", "stock_flow"));
        }
        ViewType::Interface => {
            view_elem = view_elem.with_attribute(("type", "interface"));
        }
        ViewType::Popup => {
            view_elem = view_elem.with_attribute(("type", "popup"));
        }
        ViewType::VendorSpecific(vendor, type_str) => {
            let vendor_str = match vendor {
                crate::Vendor::Anylogic => "anylogic",
                crate::Vendor::Forio => "forio",
                crate::Vendor::Insightmaker => "insightmaker",
                crate::Vendor::Isee => "isee",
                crate::Vendor::Powersim => "powersim",
                crate::Vendor::Simanticssd => "simanticssd",
                crate::Vendor::Simile => "simile",
                crate::Vendor::Sysdea => "sysdea",
                crate::Vendor::Vensim => "vensim",
                crate::Vendor::SimLab => "simlab",
                crate::Vendor::Other => "other",
            };
            let type_attr = format!("{}:{}", vendor_str, type_str);
            view_elem = view_elem.with_attribute(("type", type_attr.as_str()));
        }
    }

    view_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        // Optional style
        if let Some(ref style) = view.style {
            serialize_view_style(writer, style)?;
        }

        // Serialize view objects
        for stock in &view.stocks {
            serialize_stock_object(writer, stock)?;
        }
        for flow in &view.flows {
            serialize_flow_object(writer, flow)?;
        }
        for aux in &view.auxes {
            serialize_aux_object(writer, aux)?;
        }
        for module in &view.modules {
            serialize_module_object(writer, module)?;
        }
        for group in &view.groups {
            serialize_group_object(writer, group)?;
        }
        for connector in &view.connectors {
            serialize_connector_object(writer, connector)?;
        }
        for alias in &view.aliases {
            serialize_alias_object(writer, alias)?;
        }
        for stacked_container in &view.stacked_containers {
            serialize_stacked_container_object(writer, stacked_container)?;
        }
        for slider in &view.sliders {
            serialize_slider_object(writer, slider)?;
        }
        for knob in &view.knobs {
            serialize_knob_object(writer, knob)?;
        }
        for switch in &view.switches {
            serialize_switch_object(writer, switch)?;
        }
        for options in &view.options {
            serialize_options_object(writer, options)?;
        }
        for numeric_input in &view.numeric_inputs {
            serialize_numeric_input_object(writer, numeric_input)?;
        }
        for list_input in &view.list_inputs {
            serialize_list_input_object(writer, list_input)?;
        }
        for graphical_input in &view.graphical_inputs {
            serialize_graphical_input_object(writer, graphical_input)?;
        }
        for numeric_display in &view.numeric_displays {
            serialize_numeric_display_object(writer, numeric_display)?;
        }
        for lamp in &view.lamps {
            serialize_lamp_object(writer, lamp)?;
        }
        for gauge in &view.gauges {
            serialize_gauge_object(writer, gauge)?;
        }
        for graph in &view.graphs {
            serialize_graph_object(writer, graph)?;
        }
        for table in &view.tables {
            serialize_table_object(writer, table)?;
        }
        for text_box in &view.text_boxes {
            serialize_text_box_object(writer, text_box)?;
        }
        for graphics_frame in &view.graphics_frames {
            serialize_graphics_frame_object(writer, graphics_frame)?;
        }
        for button in &view.buttons {
            serialize_button_object(writer, button)?;
        }

        Ok(())
    })?;
    Ok(())
}

/// Serialize Style to XML (for views).
fn serialize_view_style<W: Write>(
    writer: &mut Writer<W>,
    _style: &Style,
) -> Result<(), SerializeError> {
    // TODO: Implement full Style serialization
    // For now, just write an empty style tag
    writer.create_element("style").write_empty()?;
    Ok(())
}

/// Serialize a StockObject to XML.
fn serialize_stock_object<W: Write>(
    writer: &mut Writer<W>,
    stock: &StockObject,
) -> Result<(), SerializeError> {
    let mut stock_elem = writer.create_element("stock");

    // Required attributes
    let uid_str = format!("{}", stock.uid.value);
    stock_elem = stock_elem.with_attribute(("uid", uid_str.as_str()));
    stock_elem = stock_elem.with_attribute(("name", stock.name.as_str()));
    let width_str = format!("{}", stock.width);
    stock_elem = stock_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", stock.height);
    stock_elem = stock_elem.with_attribute(("height", height_str.as_str()));

    // Optional attributes
    if let Some(x) = stock.x {
        let x_str = format!("{}", x);
        stock_elem = stock_elem.with_attribute(("x", x_str.as_str()));
    }
    if let Some(y) = stock.y {
        let y_str = format!("{}", y);
        stock_elem = stock_elem.with_attribute(("y", y_str.as_str()));
    }

    // Add common display attributes
    if let Some(ref color) = stock.color {
        stock_elem = stock_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = stock.background {
        stock_elem =
            stock_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = stock.z_index {
        let z_str = format!("{}", z_index);
        stock_elem = stock_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = stock.font_family {
        stock_elem = stock_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = stock.font_size {
        let font_size_str = format!("{}pt", font_size);
        stock_elem = stock_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = stock.font_weight {
        stock_elem = stock_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = stock.font_style {
        stock_elem = stock_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = stock.text_decoration {
        stock_elem = stock_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = stock.text_align {
        stock_elem = stock_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = stock.text_background {
        stock_elem = stock_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = stock.vertical_text_align {
        stock_elem = stock_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = stock.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        stock_elem = stock_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = stock.font_color {
        stock_elem =
            stock_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = stock.text_border_color {
        stock_elem = stock_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = stock.text_border_width {
        stock_elem = stock_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = stock.text_border_style {
        stock_elem = stock_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }
    if let Some(ref label_side) = stock.label_side {
        stock_elem = stock_elem.with_attribute(("label_side", label_side.as_str()));
    }
    if let Some(label_angle) = stock.label_angle {
        let label_angle_str = format!("{}", label_angle);
        stock_elem = stock_elem.with_attribute(("label_angle", label_angle_str.as_str()));
    }

    // Optional shape
    if let Some(ref shape) = stock.shape {
        stock_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            serialize_shape(writer, shape)?;
            Ok(())
        })?;
    } else {
        stock_elem.write_empty()?;
    }

    Ok(())
}

/// Serialize a FlowObject to XML.
fn serialize_flow_object<W: Write>(
    writer: &mut Writer<W>,
    flow: &FlowObject,
) -> Result<(), SerializeError> {
    let mut flow_elem = writer.create_element("flow");

    // Required attributes
    let uid_str = format!("{}", flow.uid.value);
    flow_elem = flow_elem.with_attribute(("uid", uid_str.as_str()));
    flow_elem = flow_elem.with_attribute(("name", flow.name.as_str()));
    let width_str = format!("{}", flow.width);
    flow_elem = flow_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", flow.height);
    flow_elem = flow_elem.with_attribute(("height", height_str.as_str()));

    // Optional attributes
    if let Some(x) = flow.x {
        let x_str = format!("{}", x);
        flow_elem = flow_elem.with_attribute(("x", x_str.as_str()));
    }
    if let Some(y) = flow.y {
        let y_str = format!("{}", y);
        flow_elem = flow_elem.with_attribute(("y", y_str.as_str()));
    }

    // Add common display attributes (same as stock)
    if let Some(ref color) = flow.color {
        flow_elem = flow_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = flow.background {
        flow_elem = flow_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = flow.z_index {
        let z_str = format!("{}", z_index);
        flow_elem = flow_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = flow.font_family {
        flow_elem = flow_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = flow.font_size {
        let font_size_str = format!("{}pt", font_size);
        flow_elem = flow_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = flow.font_weight {
        flow_elem = flow_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = flow.font_style {
        flow_elem = flow_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = flow.text_decoration {
        flow_elem = flow_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = flow.text_align {
        flow_elem = flow_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = flow.text_background {
        flow_elem = flow_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = flow.vertical_text_align {
        flow_elem = flow_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = flow.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        flow_elem = flow_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = flow.font_color {
        flow_elem = flow_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = flow.text_border_color {
        flow_elem = flow_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = flow.text_border_width {
        flow_elem = flow_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = flow.text_border_style {
        flow_elem = flow_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }
    if let Some(ref label_side) = flow.label_side {
        flow_elem = flow_elem.with_attribute(("label_side", label_side.as_str()));
    }
    if let Some(label_angle) = flow.label_angle {
        let label_angle_str = format!("{}", label_angle);
        flow_elem = flow_elem.with_attribute(("label_angle", label_angle_str.as_str()));
    }

    // Required pts element
    flow_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        writer.create_element("pts").write_inner_content(
            |writer| -> Result<(), SerializeError> {
                for pt in &flow.pts {
                    let mut pt_elem = writer.create_element("pt");
                    let x_str = format!("{}", pt.x);
                    pt_elem = pt_elem.with_attribute(("x", x_str.as_str()));
                    let y_str = format!("{}", pt.y);
                    pt_elem = pt_elem.with_attribute(("y", y_str.as_str()));
                    pt_elem.write_empty()?;
                }
                Ok(())
            },
        )?;
        Ok(())
    })?;

    Ok(())
}

/// Serialize an AuxObject to XML.
fn serialize_aux_object<W: Write>(
    writer: &mut Writer<W>,
    aux: &AuxObject,
) -> Result<(), SerializeError> {
    let mut aux_elem = writer.create_element("aux");

    // Required attributes
    let uid_str = format!("{}", aux.uid.value);
    aux_elem = aux_elem.with_attribute(("uid", uid_str.as_str()));
    aux_elem = aux_elem.with_attribute(("name", aux.name.as_str()));

    // Optional attributes
    if let Some(x) = aux.x {
        let x_str = format!("{}", x);
        aux_elem = aux_elem.with_attribute(("x", x_str.as_str()));
    }
    if let Some(y) = aux.y {
        let y_str = format!("{}", y);
        aux_elem = aux_elem.with_attribute(("y", y_str.as_str()));
    }
    if let Some(width) = aux.width {
        let width_str = format!("{}", width);
        aux_elem = aux_elem.with_attribute(("width", width_str.as_str()));
    }
    if let Some(height) = aux.height {
        let height_str = format!("{}", height);
        aux_elem = aux_elem.with_attribute(("height", height_str.as_str()));
    }

    // Add common display attributes (same as stock)
    if let Some(ref color) = aux.color {
        aux_elem = aux_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = aux.background {
        aux_elem = aux_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = aux.z_index {
        let z_str = format!("{}", z_index);
        aux_elem = aux_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = aux.font_family {
        aux_elem = aux_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = aux.font_size {
        let font_size_str = format!("{}pt", font_size);
        aux_elem = aux_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = aux.font_weight {
        aux_elem = aux_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = aux.font_style {
        aux_elem = aux_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = aux.text_decoration {
        aux_elem = aux_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = aux.text_align {
        aux_elem = aux_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = aux.text_background {
        aux_elem =
            aux_elem.with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = aux.vertical_text_align {
        aux_elem = aux_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = aux.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        aux_elem = aux_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = aux.font_color {
        aux_elem = aux_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = aux.text_border_color {
        aux_elem = aux_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = aux.text_border_width {
        aux_elem = aux_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = aux.text_border_style {
        aux_elem = aux_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }
    if let Some(ref label_side) = aux.label_side {
        aux_elem = aux_elem.with_attribute(("label_side", label_side.as_str()));
    }
    if let Some(label_angle) = aux.label_angle {
        let label_angle_str = format!("{}", label_angle);
        aux_elem = aux_elem.with_attribute(("label_angle", label_angle_str.as_str()));
    }

    // Optional shape
    if let Some(ref shape) = aux.shape {
        aux_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            serialize_shape(writer, shape)?;
            Ok(())
        })?;
    } else {
        aux_elem.write_empty()?;
    }

    Ok(())
}

// Removed DisplayAttributes trait and add_display_attributes function - inlined attribute setting instead

/// Serialize a Shape to XML.
fn serialize_shape<W: Write>(writer: &mut Writer<W>, shape: &Shape) -> Result<(), SerializeError> {
    match shape {
        Shape::Rectangle {
            width,
            height,
            corner_radius,
        } => {
            let mut shape_elem = writer.create_element("shape");
            shape_elem = shape_elem.with_attribute(("type", "rectangle"));
            let width_str = format!("{}", width);
            shape_elem = shape_elem.with_attribute(("width", width_str.as_str()));
            let height_str = format!("{}", height);
            shape_elem = shape_elem.with_attribute(("height", height_str.as_str()));
            if let Some(corner_radius) = corner_radius {
                let corner_radius_str = format!("{}", corner_radius);
                shape_elem =
                    shape_elem.with_attribute(("corner_radius", corner_radius_str.as_str()));
            }
            shape_elem.write_empty()?;
        }
        Shape::Circle { radius } => {
            let mut shape_elem = writer.create_element("shape");
            shape_elem = shape_elem.with_attribute(("type", "circle"));
            let radius_str = format!("{}", radius);
            shape_elem = shape_elem.with_attribute(("radius", radius_str.as_str()));
            shape_elem.write_empty()?;
        }
        Shape::NameOnly { width, height } => {
            let mut shape_elem = writer.create_element("shape");
            shape_elem = shape_elem.with_attribute(("type", "name_only"));
            if let Some(width) = width {
                let width_str = format!("{}", width);
                shape_elem = shape_elem.with_attribute(("width", width_str.as_str()));
            }
            if let Some(height) = height {
                let height_str = format!("{}", height);
                shape_elem = shape_elem.with_attribute(("height", height_str.as_str()));
            }
            shape_elem.write_empty()?;
        }
    }
    Ok(())
}

/// Serialize Color to string.
fn serialize_color(color: &Color) -> String {
    match color {
        Color::Hex(hex) => hex.clone(),
        Color::Predefined(predef) => predef.to_hex().to_string(),
    }
}

/// Serialize FontWeight to string.
fn serialize_font_weight(weight: &FontWeight) -> &'static str {
    match weight {
        FontWeight::Normal => "normal",
        FontWeight::Bold => "bold",
    }
}

/// Serialize FontStyle to string.
fn serialize_font_style(style: &FontStyle) -> &'static str {
    match style {
        FontStyle::Normal => "normal",
        FontStyle::Italic => "italic",
    }
}

/// Serialize TextDecoration to string.
fn serialize_text_decoration(decoration: &TextDecoration) -> &'static str {
    match decoration {
        TextDecoration::Normal => "normal",
        TextDecoration::Underline => "underline",
    }
}

/// Serialize TextAlign to string.
fn serialize_text_align(align: &TextAlign) -> &'static str {
    match align {
        TextAlign::Left => "left",
        TextAlign::Right => "right",
        TextAlign::Center => "center",
    }
}

/// Serialize VerticalTextAlign to string.
fn serialize_vertical_text_align(align: &VerticalTextAlign) -> &'static str {
    match align {
        VerticalTextAlign::Top => "top",
        VerticalTextAlign::Bottom => "bottom",
        VerticalTextAlign::Center => "center",
    }
}

/// Serialize text padding tuple to string.
fn serialize_text_padding(
    padding: &(Option<f64>, Option<f64>, Option<f64>, Option<f64>),
) -> String {
    let mut parts = Vec::new();
    if let Some(top) = padding.0 {
        parts.push(format!("{}", top));
    }
    if let Some(right) = padding.1 {
        parts.push(format!("{}", right));
    }
    if let Some(bottom) = padding.2 {
        parts.push(format!("{}", bottom));
    }
    if let Some(left) = padding.3 {
        parts.push(format!("{}", left));
    }
    parts.join(",")
}

/// Serialize BorderWidth to string.
fn serialize_border_width(width: &BorderWidth) -> String {
    match width {
        BorderWidth::Thick => "thick".to_string(),
        BorderWidth::Thin => "thin".to_string(),
        BorderWidth::Px(v) => format!("{}", v),
    }
}

/// Serialize BorderStyle to string.
fn serialize_border_style(style: &BorderStyle) -> &'static str {
    match style {
        BorderStyle::None => "none",
        BorderStyle::Solid => "solid",
    }
}

/// Serialize a ModuleObject to XML.
fn serialize_module_object<W: Write>(
    writer: &mut Writer<W>,
    module: &ModuleObject,
) -> Result<(), SerializeError> {
    let mut module_elem = writer.create_element("module");

    // Required attributes
    let uid_str = format!("{}", module.uid.value);
    module_elem = module_elem.with_attribute(("uid", uid_str.as_str()));
    module_elem = module_elem.with_attribute(("name", module.name.as_str()));
    let x_str = format!("{}", module.x);
    module_elem = module_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", module.y);
    module_elem = module_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", module.width);
    module_elem = module_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", module.height);
    module_elem = module_elem.with_attribute(("height", height_str.as_str()));

    // Add common display attributes (same as stock)
    if let Some(ref color) = module.color {
        module_elem = module_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = module.background {
        module_elem =
            module_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = module.z_index {
        let z_str = format!("{}", z_index);
        module_elem = module_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = module.font_family {
        module_elem = module_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = module.font_size {
        let font_size_str = format!("{}pt", font_size);
        module_elem = module_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = module.font_weight {
        module_elem =
            module_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = module.font_style {
        module_elem = module_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = module.text_decoration {
        module_elem = module_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = module.text_align {
        module_elem = module_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = module.text_background {
        module_elem = module_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = module.vertical_text_align {
        module_elem = module_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = module.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        module_elem = module_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = module.font_color {
        module_elem =
            module_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = module.text_border_color {
        module_elem = module_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = module.text_border_width {
        module_elem = module_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = module.text_border_style {
        module_elem = module_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }
    if let Some(ref label_side) = module.label_side {
        module_elem = module_elem.with_attribute(("label_side", label_side.as_str()));
    }
    if let Some(label_angle) = module.label_angle {
        let label_angle_str = format!("{}", label_angle);
        module_elem = module_elem.with_attribute(("label_angle", label_angle_str.as_str()));
    }

    // Optional shape
    if let Some(ref shape) = module.shape {
        module_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            serialize_shape(writer, shape)?;
            Ok(())
        })?;
    } else {
        module_elem.write_empty()?;
    }

    Ok(())
}

/// Serialize a GroupObject to XML.
fn serialize_group_object<W: Write>(
    writer: &mut Writer<W>,
    group: &GroupObject,
) -> Result<(), SerializeError> {
    let mut group_elem = writer.create_element("group");

    // Required attributes
    let uid_str = format!("{}", group.uid.value);
    group_elem = group_elem.with_attribute(("uid", uid_str.as_str()));
    group_elem = group_elem.with_attribute(("name", group.name.as_str()));
    let x_str = format!("{}", group.x);
    group_elem = group_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", group.y);
    group_elem = group_elem.with_attribute(("y", y_str.as_str()));
    group_elem = group_elem.with_attribute(("locked", if group.locked { "true" } else { "false" }));

    // Add common display attributes
    if let Some(ref color) = group.color {
        group_elem = group_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = group.background {
        group_elem =
            group_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = group.z_index {
        let z_str = format!("{}", z_index);
        group_elem = group_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = group.font_family {
        group_elem = group_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = group.font_size {
        let font_size_str = format!("{}pt", font_size);
        group_elem = group_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = group.font_weight {
        group_elem = group_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = group.font_style {
        group_elem = group_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = group.text_decoration {
        group_elem = group_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = group.text_align {
        group_elem = group_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = group.text_background {
        group_elem = group_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = group.vertical_text_align {
        group_elem = group_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = group.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        group_elem = group_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = group.font_color {
        group_elem =
            group_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = group.text_border_color {
        group_elem = group_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = group.text_border_width {
        group_elem = group_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = group.text_border_style {
        group_elem = group_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    // Optional items
    if !group.items.is_empty() {
        group_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            for item_uid in &group.items {
                let mut item_elem = writer.create_element("item");
                let uid_str = format!("{}", item_uid.value);
                item_elem = item_elem.with_attribute(("uid", uid_str.as_str()));
                item_elem.write_empty()?;
            }
            Ok(())
        })?;
    } else {
        group_elem.write_empty()?;
    }

    Ok(())
}

/// Serialize a ConnectorObject to XML.
fn serialize_connector_object<W: Write>(
    writer: &mut Writer<W>,
    connector: &ConnectorObject,
) -> Result<(), SerializeError> {
    let mut connector_elem = writer.create_element("connector");

    // Required attributes
    let uid_str = format!("{}", connector.uid.value);
    connector_elem = connector_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", connector.x);
    connector_elem = connector_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", connector.y);
    connector_elem = connector_elem.with_attribute(("y", y_str.as_str()));
    let angle_str = format!("{}", connector.angle);
    connector_elem = connector_elem.with_attribute(("angle", angle_str.as_str()));
    connector_elem = connector_elem.with_attribute((
        "delay_mark",
        if connector.delay_mark {
            "true"
        } else {
            "false"
        },
    ));

    // Optional attributes
    if let Some(ref line_style) = connector.line_style {
        let line_style_str = serialize_line_style(line_style);
        connector_elem = connector_elem.with_attribute(("line_style", line_style_str.as_str()));
    }
    if let Some(ref polarity) = connector.polarity {
        connector_elem = connector_elem.with_attribute(("polarity", serialize_polarity(polarity)));
    }

    // Add common display attributes
    if let Some(ref color) = connector.color {
        connector_elem = connector_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = connector.background {
        connector_elem =
            connector_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = connector.z_index {
        let z_str = format!("{}", z_index);
        connector_elem = connector_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = connector.font_family {
        connector_elem = connector_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = connector.font_size {
        let font_size_str = format!("{}pt", font_size);
        connector_elem = connector_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = connector.font_weight {
        connector_elem =
            connector_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = connector.font_style {
        connector_elem =
            connector_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = connector.text_decoration {
        connector_elem = connector_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = connector.text_align {
        connector_elem =
            connector_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = connector.text_background {
        connector_elem = connector_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = connector.vertical_text_align {
        connector_elem = connector_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = connector.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        connector_elem = connector_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = connector.font_color {
        connector_elem =
            connector_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = connector.text_border_color {
        connector_elem = connector_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = connector.text_border_width {
        connector_elem = connector_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = connector.text_border_style {
        connector_elem = connector_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    // Required from, to, and pts
    connector_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        // Serialize from
        writer
            .create_element("from")
            .write_inner_content(|w| -> Result<(), SerializeError> {
                serialize_pointer(w, &connector.from)?;
                Ok(())
            })?;

        // Serialize to
        writer
            .create_element("to")
            .write_inner_content(|w| -> Result<(), SerializeError> {
                serialize_pointer(w, &connector.to)?;
                Ok(())
            })?;

        // Serialize pts
        writer.create_element("pts").write_inner_content(
            |writer| -> Result<(), SerializeError> {
                for pt in &connector.pts {
                    let mut pt_elem = writer.create_element("pt");
                    let x_str = format!("{}", pt.x);
                    pt_elem = pt_elem.with_attribute(("x", x_str.as_str()));
                    let y_str = format!("{}", pt.y);
                    pt_elem = pt_elem.with_attribute(("y", y_str.as_str()));
                    pt_elem.write_empty()?;
                }
                Ok(())
            },
        )?;

        Ok(())
    })?;

    Ok(())
}

/// Serialize an AliasObject to XML.
fn serialize_alias_object<W: Write>(
    writer: &mut Writer<W>,
    alias: &AliasObject,
) -> Result<(), SerializeError> {
    let mut alias_elem = writer.create_element("alias");

    // Required attributes
    let uid_str = format!("{}", alias.uid.value);
    alias_elem = alias_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", alias.x);
    alias_elem = alias_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", alias.y);
    alias_elem = alias_elem.with_attribute(("y", y_str.as_str()));

    // Add common display attributes
    if let Some(ref color) = alias.color {
        alias_elem = alias_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = alias.background {
        alias_elem =
            alias_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = alias.z_index {
        let z_str = format!("{}", z_index);
        alias_elem = alias_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = alias.font_family {
        alias_elem = alias_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = alias.font_size {
        let font_size_str = format!("{}pt", font_size);
        alias_elem = alias_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = alias.font_weight {
        alias_elem = alias_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = alias.font_style {
        alias_elem = alias_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = alias.text_decoration {
        alias_elem = alias_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = alias.text_align {
        alias_elem = alias_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = alias.text_background {
        alias_elem = alias_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = alias.vertical_text_align {
        alias_elem = alias_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = alias.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        alias_elem = alias_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = alias.font_color {
        alias_elem =
            alias_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = alias.text_border_color {
        alias_elem = alias_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = alias.text_border_width {
        alias_elem = alias_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = alias.text_border_style {
        alias_elem = alias_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    // Required of element and optional shape
    alias_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        write_element_with_text(writer, "of", &alias.of)?;
        if let Some(ref shape) = alias.shape {
            serialize_shape(writer, shape)?;
        }
        Ok(())
    })?;

    Ok(())
}

/// Serialize a Pointer to XML.
fn serialize_pointer<W: Write>(
    writer: &mut Writer<W>,
    pointer: &Pointer,
) -> Result<(), SerializeError> {
    match pointer {
        Pointer::Alias(uid) => {
            let mut alias_elem = writer.create_element("alias");
            let uid_str = format!("{}", uid.value);
            alias_elem = alias_elem.with_attribute(("uid", uid_str.as_str()));
            alias_elem.write_empty()?;
        }
        Pointer::Name(name) => {
            // For text content in to/from, write as text
            writer.write_event(quick_xml::events::Event::Text(
                quick_xml::events::BytesText::new(name.as_str()),
            ))?;
        }
    }
    Ok(())
}

/// Serialize Polarity to string.
fn serialize_polarity(polarity: &Polarity) -> &'static str {
    match polarity {
        Polarity::Positive => "+",
        Polarity::Negative => "-",
        Polarity::None => "none",
    }
}

/// Serialize LineStyle to string.
fn serialize_line_style(style: &LineStyle) -> String {
    match style {
        LineStyle::Solid => "solid".to_string(),
        LineStyle::Dashed => "dashed".to_string(),
        LineStyle::VendorSpecific(s) => s.clone(),
    }
}

/// Serialize a StackedContainerObject to XML.
fn serialize_stacked_container_object<W: Write>(
    writer: &mut Writer<W>,
    container: &StackedContainerObject,
) -> Result<(), SerializeError> {
    let mut container_elem = writer.create_element("stacked_container");

    let uid_str = format!("{}", container.uid.value);
    container_elem = container_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", container.x);
    container_elem = container_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", container.y);
    container_elem = container_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", container.width);
    container_elem = container_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", container.height);
    container_elem = container_elem.with_attribute(("height", height_str.as_str()));
    let visible_index_str = format!("{}", container.visible_index);
    container_elem = container_elem.with_attribute(("visible_index", visible_index_str.as_str()));

    container_elem.write_empty()?;
    Ok(())
}

/// Serialize a SliderObject to XML.
fn serialize_slider_object<W: Write>(
    writer: &mut Writer<W>,
    slider: &SliderObject,
) -> Result<(), SerializeError> {
    let mut slider_elem = writer.create_element("slider");

    // Required attributes
    let uid_str = format!("{}", slider.uid.value);
    slider_elem = slider_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", slider.x);
    slider_elem = slider_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", slider.y);
    slider_elem = slider_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", slider.width);
    slider_elem = slider_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", slider.height);
    slider_elem = slider_elem.with_attribute(("height", height_str.as_str()));
    let min_str = format!("{}", slider.min);
    slider_elem = slider_elem.with_attribute(("min", min_str.as_str()));
    let max_str = format!("{}", slider.max);
    slider_elem = slider_elem.with_attribute(("max", max_str.as_str()));

    // Optional boolean attributes (only serialize if false)
    if !slider.show_name {
        slider_elem = slider_elem.with_attribute(("show_name", "false"));
    }
    if !slider.show_number {
        slider_elem = slider_elem.with_attribute(("show_number", "false"));
    }
    if !slider.show_min_max {
        slider_elem = slider_elem.with_attribute(("show_min_max", "false"));
    }

    // Add common display attributes
    if let Some(ref color) = slider.color {
        slider_elem = slider_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = slider.background {
        slider_elem =
            slider_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = slider.z_index {
        let z_str = format!("{}", z_index);
        slider_elem = slider_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = slider.font_family {
        slider_elem = slider_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = slider.font_size {
        let font_size_str = format!("{}pt", font_size);
        slider_elem = slider_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = slider.font_weight {
        slider_elem =
            slider_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = slider.font_style {
        slider_elem = slider_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = slider.text_decoration {
        slider_elem = slider_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = slider.text_align {
        slider_elem = slider_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = slider.text_background {
        slider_elem = slider_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = slider.vertical_text_align {
        slider_elem = slider_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = slider.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        slider_elem = slider_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = slider.font_color {
        slider_elem =
            slider_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = slider.text_border_color {
        slider_elem = slider_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = slider.text_border_width {
        slider_elem = slider_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = slider.text_border_style {
        slider_elem = slider_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    // Required entity and optional reset_to
    slider_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        writer
            .create_element("entity")
            .with_attribute(("name", slider.entity_name.as_str()))
            .write_empty()?;

        if let Some((value, after)) = &slider.reset_to {
            writer
                .create_element("reset_to")
                .with_attribute(("after", after.as_str()))
                .write_text_content(quick_xml::events::BytesText::new(&format!("{}", value)))?;
        }

        Ok(())
    })?;

    Ok(())
}

/// Serialize a KnobObject to XML (same as SliderObject).
fn serialize_knob_object<W: Write>(
    writer: &mut Writer<W>,
    knob: &KnobObject,
) -> Result<(), SerializeError> {
    // KnobObject is a type alias for SliderObject, but uses <knob> tag
    let mut knob_elem = writer.create_element("knob");

    // Same as slider but with <knob> tag
    let uid_str = format!("{}", knob.uid.value);
    knob_elem = knob_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", knob.x);
    knob_elem = knob_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", knob.y);
    knob_elem = knob_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", knob.width);
    knob_elem = knob_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", knob.height);
    knob_elem = knob_elem.with_attribute(("height", height_str.as_str()));
    let min_str = format!("{}", knob.min);
    knob_elem = knob_elem.with_attribute(("min", min_str.as_str()));
    let max_str = format!("{}", knob.max);
    knob_elem = knob_elem.with_attribute(("max", max_str.as_str()));

    if !knob.show_name {
        knob_elem = knob_elem.with_attribute(("show_name", "false"));
    }
    if !knob.show_number {
        knob_elem = knob_elem.with_attribute(("show_number", "false"));
    }
    if !knob.show_min_max {
        knob_elem = knob_elem.with_attribute(("show_min_max", "false"));
    }

    // Add common display attributes (same as slider)
    if let Some(ref color) = knob.color {
        knob_elem = knob_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = knob.background {
        knob_elem = knob_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = knob.z_index {
        let z_str = format!("{}", z_index);
        knob_elem = knob_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = knob.font_family {
        knob_elem = knob_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = knob.font_size {
        let font_size_str = format!("{}pt", font_size);
        knob_elem = knob_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = knob.font_weight {
        knob_elem = knob_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = knob.font_style {
        knob_elem = knob_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = knob.text_decoration {
        knob_elem = knob_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = knob.text_align {
        knob_elem = knob_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = knob.text_background {
        knob_elem = knob_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = knob.vertical_text_align {
        knob_elem = knob_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = knob.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        knob_elem = knob_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = knob.font_color {
        knob_elem = knob_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = knob.text_border_color {
        knob_elem = knob_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = knob.text_border_width {
        knob_elem = knob_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = knob.text_border_style {
        knob_elem = knob_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    // Required entity (knobs don't have reset_to)
    knob_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        writer
            .create_element("entity")
            .with_attribute(("name", knob.entity_name.as_str()))
            .write_empty()?;
        Ok(())
    })?;

    Ok(())
}

/// Serialize SwitchStyle to string.
fn serialize_switch_style(style: &SwitchStyle) -> &'static str {
    match style {
        SwitchStyle::Toggle => "toggle",
        SwitchStyle::PushButton => "push_button",
    }
}

/// Serialize a SwitchObject to XML.
fn serialize_switch_object<W: Write>(
    writer: &mut Writer<W>,
    switch: &SwitchObject,
) -> Result<(), SerializeError> {
    let mut switch_elem = writer.create_element("switch");

    // Required attributes
    let uid_str = format!("{}", switch.uid.value);
    switch_elem = switch_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", switch.x);
    switch_elem = switch_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", switch.y);
    switch_elem = switch_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", switch.width);
    switch_elem = switch_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", switch.height);
    switch_elem = switch_elem.with_attribute(("height", height_str.as_str()));

    switch_elem =
        switch_elem.with_attribute(("show_name", if switch.show_name { "true" } else { "false" }));
    switch_elem =
        switch_elem.with_attribute(("switch_style", serialize_switch_style(&switch.switch_style)));
    switch_elem = switch_elem.with_attribute((
        "clicking_sound",
        if switch.clicking_sound {
            "true"
        } else {
            "false"
        },
    ));

    // Optional attributes
    if let Some(ref entity_name) = switch.entity_name {
        switch_elem = switch_elem.with_attribute(("entity_name", entity_name.as_str()));
    }
    if let Some(entity_value) = switch.entity_value {
        let value_str = format!("{}", entity_value);
        switch_elem = switch_elem.with_attribute(("entity_value", value_str.as_str()));
    }
    if let Some(ref group_name) = switch.group_name {
        switch_elem = switch_elem.with_attribute(("group_name", group_name.as_str()));
    }
    if let Some(ref module_name) = switch.module_name {
        switch_elem = switch_elem.with_attribute(("module_name", module_name.as_str()));
    }

    // Add common display attributes
    if let Some(ref color) = switch.color {
        switch_elem = switch_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = switch.background {
        switch_elem =
            switch_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = switch.z_index {
        let z_str = format!("{}", z_index);
        switch_elem = switch_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = switch.font_family {
        switch_elem = switch_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = switch.font_size {
        let font_size_str = format!("{}pt", font_size);
        switch_elem = switch_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = switch.font_weight {
        switch_elem =
            switch_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = switch.font_style {
        switch_elem = switch_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = switch.text_decoration {
        switch_elem = switch_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = switch.text_align {
        switch_elem = switch_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = switch.text_background {
        switch_elem = switch_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = switch.vertical_text_align {
        switch_elem = switch_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = switch.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        switch_elem = switch_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = switch.font_color {
        switch_elem =
            switch_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = switch.text_border_color {
        switch_elem = switch_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = switch.text_border_width {
        switch_elem = switch_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = switch.text_border_style {
        switch_elem = switch_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }
    if let Some(ref label_side) = switch.label_side {
        switch_elem = switch_elem.with_attribute(("label_side", label_side.as_str()));
    }
    if let Some(label_angle) = switch.label_angle {
        let label_angle_str = format!("{}", label_angle);
        switch_elem = switch_elem.with_attribute(("label_angle", label_angle_str.as_str()));
    }

    // Optional reset_to
    if let Some((value, after)) = &switch.reset_to {
        switch_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            writer
                .create_element("reset_to")
                .with_attribute(("after", after.as_str()))
                .write_text_content(quick_xml::events::BytesText::new(&format!("{}", value)))?;
            Ok(())
        })?;
    } else {
        switch_elem.write_empty()?;
    }

    Ok(())
}

/// Serialize OptionsLayout to string.
fn serialize_options_layout(layout: &OptionsLayout) -> &'static str {
    match layout {
        OptionsLayout::Vertical => "vertical",
        OptionsLayout::Horizontal => "horizontal",
        OptionsLayout::Grid => "grid",
    }
}

/// Serialize an OptionsObject to XML.
fn serialize_options_object<W: Write>(
    writer: &mut Writer<W>,
    options: &OptionsObject,
) -> Result<(), SerializeError> {
    let mut options_elem = writer.create_element("options");

    // Required attributes
    let uid_str = format!("{}", options.uid.value);
    options_elem = options_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", options.x);
    options_elem = options_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", options.y);
    options_elem = options_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", options.width);
    options_elem = options_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", options.height);
    options_elem = options_elem.with_attribute(("height", height_str.as_str()));
    options_elem =
        options_elem.with_attribute(("layout", serialize_options_layout(&options.layout)));
    let h_spacing_str = format!("{}", options.horizontal_spacing);
    options_elem = options_elem.with_attribute(("horizontal_spacing", h_spacing_str.as_str()));
    let v_spacing_str = format!("{}", options.vertical_spacing);
    options_elem = options_elem.with_attribute(("vertical_spacing", v_spacing_str.as_str()));

    // Add common display attributes (same pattern as other objects)
    if let Some(ref color) = options.color {
        options_elem = options_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = options.background {
        options_elem =
            options_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = options.z_index {
        let z_str = format!("{}", z_index);
        options_elem = options_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = options.font_family {
        options_elem = options_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = options.font_size {
        let font_size_str = format!("{}pt", font_size);
        options_elem = options_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = options.font_weight {
        options_elem =
            options_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = options.font_style {
        options_elem =
            options_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = options.text_decoration {
        options_elem = options_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = options.text_align {
        options_elem =
            options_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = options.text_background {
        options_elem = options_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = options.vertical_text_align {
        options_elem = options_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = options.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        options_elem = options_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = options.font_color {
        options_elem =
            options_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = options.text_border_color {
        options_elem = options_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = options.text_border_width {
        options_elem = options_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = options.text_border_style {
        options_elem = options_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    // Serialize entities
    if !options.entities.is_empty() {
        options_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            for entity in &options.entities {
                let mut entity_elem = writer.create_element("entity");
                entity_elem = entity_elem.with_attribute(("name", entity.entity_name.as_str()));
                if let Some(ref index) = entity.index {
                    entity_elem = entity_elem.with_attribute(("index", index.as_str()));
                }
                let value_str = format!("{}", entity.value);
                entity_elem.write_text_content(quick_xml::events::BytesText::new(&value_str))?;
            }
            Ok(())
        })?;
    } else {
        options_elem.write_empty()?;
    }

    Ok(())
}

/// Serialize a NumericInputObject to XML.
fn serialize_numeric_input_object<W: Write>(
    writer: &mut Writer<W>,
    input: &NumericInputObject,
) -> Result<(), SerializeError> {
    let mut input_elem = writer.create_element("numeric_input");

    // Required attributes
    let uid_str = format!("{}", input.uid.value);
    input_elem = input_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", input.x);
    input_elem = input_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", input.y);
    input_elem = input_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", input.width);
    input_elem = input_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", input.height);
    input_elem = input_elem.with_attribute(("height", height_str.as_str()));
    input_elem = input_elem.with_attribute(("entity_name", input.entity_name.as_str()));
    let min_str = format!("{}", input.min);
    input_elem = input_elem.with_attribute(("min", min_str.as_str()));
    let max_str = format!("{}", input.max);
    input_elem = input_elem.with_attribute(("max", max_str.as_str()));
    let value_str = format!("{}", input.value);
    input_elem = input_elem.with_attribute(("value", value_str.as_str()));

    // Optional attributes
    if let Some(ref entity_index) = input.entity_index {
        input_elem = input_elem.with_attribute(("entity_index", entity_index.as_str()));
    }
    if let Some(precision) = input.precision {
        let precision_str = format!("{}", precision);
        input_elem = input_elem.with_attribute(("precision", precision_str.as_str()));
    }

    // Add common display attributes (same pattern)
    if let Some(ref color) = input.color {
        input_elem = input_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = input.background {
        input_elem =
            input_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = input.z_index {
        let z_str = format!("{}", z_index);
        input_elem = input_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = input.font_family {
        input_elem = input_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = input.font_size {
        let font_size_str = format!("{}pt", font_size);
        input_elem = input_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = input.font_weight {
        input_elem = input_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = input.font_style {
        input_elem = input_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = input.text_decoration {
        input_elem = input_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = input.text_align {
        input_elem = input_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = input.text_background {
        input_elem = input_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = input.vertical_text_align {
        input_elem = input_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = input.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        input_elem = input_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = input.font_color {
        input_elem =
            input_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = input.text_border_color {
        input_elem = input_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = input.text_border_width {
        input_elem = input_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = input.text_border_style {
        input_elem = input_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    input_elem.write_empty()?;
    Ok(())
}

/// Serialize a ListInputObject to XML.
fn serialize_list_input_object<W: Write>(
    writer: &mut Writer<W>,
    input: &ListInputObject,
) -> Result<(), SerializeError> {
    let mut input_elem = writer.create_element("list_input");

    // Required attributes
    let uid_str = format!("{}", input.uid.value);
    input_elem = input_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", input.x);
    input_elem = input_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", input.y);
    input_elem = input_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", input.width);
    input_elem = input_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", input.height);
    input_elem = input_elem.with_attribute(("height", height_str.as_str()));
    input_elem = input_elem.with_attribute(("name", input.name.as_str()));
    let column_width_str = format!("{}", input.column_width);
    input_elem = input_elem.with_attribute(("column_width", column_width_str.as_str()));

    // Add common display attributes
    if let Some(ref color) = input.color {
        input_elem = input_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = input.background {
        input_elem =
            input_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = input.z_index {
        let z_str = format!("{}", z_index);
        input_elem = input_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = input.font_family {
        input_elem = input_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = input.font_size {
        let font_size_str = format!("{}pt", font_size);
        input_elem = input_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = input.font_weight {
        input_elem = input_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = input.font_style {
        input_elem = input_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = input.text_decoration {
        input_elem = input_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = input.text_align {
        input_elem = input_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = input.text_background {
        input_elem = input_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = input.vertical_text_align {
        input_elem = input_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = input.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        input_elem = input_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = input.font_color {
        input_elem =
            input_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = input.text_border_color {
        input_elem = input_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = input.text_border_width {
        input_elem = input_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = input.text_border_style {
        input_elem = input_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    // Serialize nested numeric_inputs
    if !input.numeric_inputs.is_empty() {
        input_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            for numeric_input in &input.numeric_inputs {
                serialize_numeric_input_object(writer, numeric_input)?;
            }
            Ok(())
        })?;
    } else {
        input_elem.write_empty()?;
    }

    Ok(())
}

/// Serialize a GraphicalInputObject to XML.
fn serialize_graphical_input_object<W: Write>(
    writer: &mut Writer<W>,
    input: &GraphicalInputObject,
) -> Result<(), SerializeError> {
    let mut input_elem = writer.create_element("graphical_input");

    // Required attributes
    let uid_str = format!("{}", input.uid.value);
    input_elem = input_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", input.x);
    input_elem = input_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", input.y);
    input_elem = input_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", input.width);
    input_elem = input_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", input.height);
    input_elem = input_elem.with_attribute(("height", height_str.as_str()));
    input_elem = input_elem.with_attribute(("entity_name", input.entity_name.as_str()));

    // Add common display attributes
    if let Some(ref color) = input.color {
        input_elem = input_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = input.background {
        input_elem =
            input_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = input.z_index {
        let z_str = format!("{}", z_index);
        input_elem = input_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = input.font_family {
        input_elem = input_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = input.font_size {
        let font_size_str = format!("{}pt", font_size);
        input_elem = input_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = input.font_weight {
        input_elem = input_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = input.font_style {
        input_elem = input_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = input.text_decoration {
        input_elem = input_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = input.text_align {
        input_elem = input_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = input.text_background {
        input_elem = input_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = input.vertical_text_align {
        input_elem = input_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = input.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        input_elem = input_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = input.font_color {
        input_elem =
            input_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = input.text_border_color {
        input_elem = input_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = input.text_border_width {
        input_elem = input_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = input.text_border_style {
        input_elem = input_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    // Optional graphical_function
    if let Some(ref gf) = input.graphical_function {
        input_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            let mut gf_elem = writer.create_element("gf");
            let xscale_min_str = format!("{}", gf.xscale_min);
            gf_elem = gf_elem.with_attribute(("xscale_min", xscale_min_str.as_str()));
            let xscale_max_str = format!("{}", gf.xscale_max);
            gf_elem = gf_elem.with_attribute(("xscale_max", xscale_max_str.as_str()));

            // Serialize ypts
            let ypts_str = gf
                .ypts
                .iter()
                .map(|y| y.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            gf_elem.write_text_content(quick_xml::events::BytesText::new(&ypts_str))?;
            Ok(())
        })?;
    } else {
        input_elem.write_empty()?;
    }

    Ok(())
}

/// Serialize a NumericDisplayObject to XML.
fn serialize_numeric_display_object<W: Write>(
    writer: &mut Writer<W>,
    display: &NumericDisplayObject,
) -> Result<(), SerializeError> {
    let mut display_elem = writer.create_element("numeric_display");

    // Required attributes
    let uid_str = format!("{}", display.uid.value);
    display_elem = display_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", display.x);
    display_elem = display_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", display.y);
    display_elem = display_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", display.width);
    display_elem = display_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", display.height);
    display_elem = display_elem.with_attribute(("height", height_str.as_str()));
    display_elem = display_elem.with_attribute(("entity_name", display.entity_name.as_str()));
    display_elem = display_elem.with_attribute((
        "show_name",
        if display.show_name { "true" } else { "false" },
    ));
    display_elem = display_elem.with_attribute((
        "retain_ending_value",
        if display.retain_ending_value {
            "true"
        } else {
            "false"
        },
    ));
    display_elem = display_elem.with_attribute((
        "delimit_000s",
        if display.delimit_000s {
            "true"
        } else {
            "false"
        },
    ));

    // Optional attributes
    if let Some(precision) = display.precision {
        let precision_str = format!("{}", precision);
        display_elem = display_elem.with_attribute(("precision", precision_str.as_str()));
    }

    // Add common display attributes
    if let Some(ref color) = display.color {
        display_elem = display_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = display.background {
        display_elem =
            display_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = display.z_index {
        let z_str = format!("{}", z_index);
        display_elem = display_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = display.font_family {
        display_elem = display_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = display.font_size {
        let font_size_str = format!("{}pt", font_size);
        display_elem = display_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = display.font_weight {
        display_elem =
            display_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = display.font_style {
        display_elem =
            display_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = display.text_decoration {
        display_elem = display_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = display.text_align {
        display_elem =
            display_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = display.text_background {
        display_elem = display_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = display.vertical_text_align {
        display_elem = display_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = display.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        display_elem = display_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = display.font_color {
        display_elem =
            display_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = display.text_border_color {
        display_elem = display_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = display.text_border_width {
        display_elem = display_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = display.text_border_style {
        display_elem = display_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    display_elem.write_empty()?;
    Ok(())
}

/// Serialize ZoneType to string.
fn serialize_zone_type(zone_type: &ZoneType) -> &'static str {
    match zone_type {
        ZoneType::Normal => "normal",
        ZoneType::Caution => "caution",
        ZoneType::Panic => "panic",
    }
}

/// Serialize a Zone to XML.
fn serialize_zone<W: Write>(writer: &mut Writer<W>, zone: &Zone) -> Result<(), SerializeError> {
    let mut zone_elem = writer.create_element("zone");
    zone_elem = zone_elem.with_attribute(("type", serialize_zone_type(&zone.zone_type)));
    zone_elem = zone_elem.with_attribute(("color", serialize_color(&zone.color).as_str()));
    let min_str = format!("{}", zone.min);
    zone_elem = zone_elem.with_attribute(("min", min_str.as_str()));
    let max_str = format!("{}", zone.max);
    zone_elem = zone_elem.with_attribute(("max", max_str.as_str()));

    if let Some(ref sound) = zone.sound {
        zone_elem = zone_elem.with_attribute(("sound", sound.as_str()));
    }

    zone_elem.write_empty()?;
    Ok(())
}

/// Serialize a LampObject to XML.
fn serialize_lamp_object<W: Write>(
    writer: &mut Writer<W>,
    lamp: &LampObject,
) -> Result<(), SerializeError> {
    let mut lamp_elem = writer.create_element("lamp");

    // Required attributes
    let uid_str = format!("{}", lamp.uid.value);
    lamp_elem = lamp_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", lamp.x);
    lamp_elem = lamp_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", lamp.y);
    lamp_elem = lamp_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", lamp.width);
    lamp_elem = lamp_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", lamp.height);
    lamp_elem = lamp_elem.with_attribute(("height", height_str.as_str()));
    lamp_elem = lamp_elem.with_attribute(("entity_name", lamp.entity_name.as_str()));
    lamp_elem =
        lamp_elem.with_attribute(("show_name", if lamp.show_name { "true" } else { "false" }));
    lamp_elem = lamp_elem.with_attribute((
        "retain_ending_value",
        if lamp.retain_ending_value {
            "true"
        } else {
            "false"
        },
    ));
    lamp_elem = lamp_elem.with_attribute((
        "flash_on_panic",
        if lamp.flash_on_panic { "true" } else { "false" },
    ));

    // Add common display attributes
    if let Some(ref color) = lamp.color {
        lamp_elem = lamp_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = lamp.background {
        lamp_elem = lamp_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = lamp.z_index {
        let z_str = format!("{}", z_index);
        lamp_elem = lamp_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = lamp.font_family {
        lamp_elem = lamp_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = lamp.font_size {
        let font_size_str = format!("{}pt", font_size);
        lamp_elem = lamp_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = lamp.font_weight {
        lamp_elem = lamp_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = lamp.font_style {
        lamp_elem = lamp_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = lamp.text_decoration {
        lamp_elem = lamp_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = lamp.text_align {
        lamp_elem = lamp_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = lamp.text_background {
        lamp_elem = lamp_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = lamp.vertical_text_align {
        lamp_elem = lamp_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = lamp.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        lamp_elem = lamp_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = lamp.font_color {
        lamp_elem = lamp_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = lamp.text_border_color {
        lamp_elem = lamp_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = lamp.text_border_width {
        lamp_elem = lamp_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = lamp.text_border_style {
        lamp_elem = lamp_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    // Serialize zones
    if !lamp.zones.is_empty() {
        lamp_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            for zone in &lamp.zones {
                serialize_zone(writer, zone)?;
            }
            Ok(())
        })?;
    } else {
        lamp_elem.write_empty()?;
    }

    Ok(())
}

/// Serialize a GaugeObject to XML.
fn serialize_gauge_object<W: Write>(
    writer: &mut Writer<W>,
    gauge: &GaugeObject,
) -> Result<(), SerializeError> {
    let mut gauge_elem = writer.create_element("gauge");

    // Required attributes
    let uid_str = format!("{}", gauge.uid.value);
    gauge_elem = gauge_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", gauge.x);
    gauge_elem = gauge_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", gauge.y);
    gauge_elem = gauge_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", gauge.width);
    gauge_elem = gauge_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", gauge.height);
    gauge_elem = gauge_elem.with_attribute(("height", height_str.as_str()));
    gauge_elem = gauge_elem.with_attribute(("entity_name", gauge.entity_name.as_str()));
    gauge_elem =
        gauge_elem.with_attribute(("show_name", if gauge.show_name { "true" } else { "false" }));
    gauge_elem = gauge_elem.with_attribute((
        "show_number",
        if gauge.show_number { "true" } else { "false" },
    ));
    gauge_elem = gauge_elem.with_attribute((
        "retain_ending_value",
        if gauge.retain_ending_value {
            "true"
        } else {
            "false"
        },
    ));

    // Add common display attributes
    if let Some(ref color) = gauge.color {
        gauge_elem = gauge_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = gauge.background {
        gauge_elem =
            gauge_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = gauge.z_index {
        let z_str = format!("{}", z_index);
        gauge_elem = gauge_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = gauge.font_family {
        gauge_elem = gauge_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = gauge.font_size {
        let font_size_str = format!("{}pt", font_size);
        gauge_elem = gauge_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = gauge.font_weight {
        gauge_elem = gauge_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = gauge.font_style {
        gauge_elem = gauge_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = gauge.text_decoration {
        gauge_elem = gauge_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = gauge.text_align {
        gauge_elem = gauge_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = gauge.text_background {
        gauge_elem = gauge_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = gauge.vertical_text_align {
        gauge_elem = gauge_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = gauge.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        gauge_elem = gauge_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = gauge.font_color {
        gauge_elem =
            gauge_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = gauge.text_border_color {
        gauge_elem = gauge_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = gauge.text_border_width {
        gauge_elem = gauge_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = gauge.text_border_style {
        gauge_elem = gauge_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    // Serialize zones
    if !gauge.zones.is_empty() {
        gauge_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            for zone in &gauge.zones {
                serialize_zone(writer, zone)?;
            }
            Ok(())
        })?;
    } else {
        gauge_elem.write_empty()?;
    }

    Ok(())
}

/// Serialize GraphType to string.
fn serialize_graph_type(graph_type: &GraphType) -> &'static str {
    match graph_type {
        GraphType::TimeSeries => "time_series",
        GraphType::Scatter => "scatter",
        GraphType::Bar => "bar",
    }
}

/// Serialize PenStyle to string.
fn serialize_pen_style(pen_style: &PenStyle) -> &'static str {
    match pen_style {
        PenStyle::Solid => "solid",
        PenStyle::Dotted => "dotted",
        PenStyle::Dashed => "dashed",
        PenStyle::DotDashed => "dot_dashed",
    }
}

/// Serialize a PlotScale to XML.
fn serialize_plot_scale<W: Write>(
    writer: &mut Writer<W>,
    scale: &PlotScale,
) -> Result<(), SerializeError> {
    let mut scale_elem = writer.create_element("scale");
    let min_str = format!("{}", scale.min);
    scale_elem = scale_elem.with_attribute(("min", min_str.as_str()));
    let max_str = format!("{}", scale.max);
    scale_elem = scale_elem.with_attribute(("max", max_str.as_str()));
    scale_elem.write_empty()?;
    Ok(())
}

/// Serialize a Plot to XML.
fn serialize_plot<W: Write>(writer: &mut Writer<W>, plot: &Plot) -> Result<(), SerializeError> {
    let mut plot_elem = writer.create_element("plot");
    let index_str = format!("{}", plot.index);
    plot_elem = plot_elem.with_attribute(("index", index_str.as_str()));
    let pen_width_str = format!("{}", plot.pen_width);
    plot_elem = plot_elem.with_attribute(("pen_width", pen_width_str.as_str()));
    plot_elem = plot_elem.with_attribute(("pen_style", serialize_pen_style(&plot.pen_style)));
    plot_elem = plot_elem.with_attribute((
        "show_y_axis",
        if plot.show_y_axis { "true" } else { "false" },
    ));
    plot_elem = plot_elem.with_attribute(("title", plot.title.as_str()));
    plot_elem =
        plot_elem.with_attribute(("right_axis", if plot.right_axis { "true" } else { "false" }));
    plot_elem = plot_elem.with_attribute(("entity_name", plot.entity_name.as_str()));

    if let Some(precision) = plot.precision {
        let precision_str = format!("{}", precision);
        plot_elem = plot_elem.with_attribute(("precision", precision_str.as_str()));
    }
    if let Some(ref color) = plot.color {
        plot_elem = plot_elem.with_attribute(("color", serialize_color(color).as_str()));
    }

    // Optional scale
    if let Some(ref scale) = plot.scale {
        plot_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            serialize_plot_scale(writer, scale)?;
            Ok(())
        })?;
    } else {
        plot_elem.write_empty()?;
    }

    Ok(())
}

/// Serialize a GraphObject to XML.
fn serialize_graph_object<W: Write>(
    writer: &mut Writer<W>,
    graph: &GraphObject,
) -> Result<(), SerializeError> {
    let mut graph_elem = writer.create_element("graph");

    // Required attributes
    let uid_str = format!("{}", graph.uid.value);
    graph_elem = graph_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", graph.x);
    graph_elem = graph_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", graph.y);
    graph_elem = graph_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", graph.width);
    graph_elem = graph_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", graph.height);
    graph_elem = graph_elem.with_attribute(("height", height_str.as_str()));
    graph_elem = graph_elem.with_attribute(("graph_type", serialize_graph_type(&graph.graph_type)));
    graph_elem =
        graph_elem.with_attribute(("show_grid", if graph.show_grid { "true" } else { "false" }));
    let num_x_grid_str = format!("{}", graph.num_x_grid_lines);
    graph_elem = graph_elem.with_attribute(("num_x_grid_lines", num_x_grid_str.as_str()));
    let num_y_grid_str = format!("{}", graph.num_y_grid_lines);
    graph_elem = graph_elem.with_attribute(("num_y_grid_lines", num_y_grid_str.as_str()));
    let num_x_labels_str = format!("{}", graph.num_x_labels);
    graph_elem = graph_elem.with_attribute(("num_x_labels", num_x_labels_str.as_str()));
    let num_y_labels_str = format!("{}", graph.num_y_labels);
    graph_elem = graph_elem.with_attribute(("num_y_labels", num_y_labels_str.as_str()));
    graph_elem = graph_elem.with_attribute((
        "right_axis_auto_scale",
        if graph.right_axis_auto_scale {
            "true"
        } else {
            "false"
        },
    ));
    graph_elem = graph_elem.with_attribute((
        "right_axis_multi_scale",
        if graph.right_axis_multi_scale {
            "true"
        } else {
            "false"
        },
    ));
    graph_elem = graph_elem.with_attribute((
        "left_axis_auto_scale",
        if graph.left_axis_auto_scale {
            "true"
        } else {
            "false"
        },
    ));
    graph_elem = graph_elem.with_attribute((
        "left_axis_multi_scale",
        if graph.left_axis_multi_scale {
            "true"
        } else {
            "false"
        },
    ));
    graph_elem = graph_elem.with_attribute((
        "plot_numbers",
        if graph.plot_numbers { "true" } else { "false" },
    ));
    graph_elem = graph_elem.with_attribute((
        "comparative",
        if graph.comparative { "true" } else { "false" },
    ));

    // Optional attributes
    if let Some(ref title) = graph.title {
        graph_elem = graph_elem.with_attribute(("title", title.as_str()));
    }
    if let Some(ref doc) = graph.doc {
        graph_elem = graph_elem.with_attribute(("doc", doc.as_str()));
    }
    if let Some(ref x_axis_title) = graph.x_axis_title {
        graph_elem = graph_elem.with_attribute(("x_axis_title", x_axis_title.as_str()));
    }
    if let Some(ref right_axis_title) = graph.right_axis_title {
        graph_elem = graph_elem.with_attribute(("right_axis_title", right_axis_title.as_str()));
    }
    if let Some(ref left_axis_title) = graph.left_axis_title {
        graph_elem = graph_elem.with_attribute(("left_axis_title", left_axis_title.as_str()));
    }
    if let Some(from) = graph.from {
        let from_str = format!("{}", from);
        graph_elem = graph_elem.with_attribute(("from", from_str.as_str()));
    }
    if let Some(to) = graph.to {
        let to_str = format!("{}", to);
        graph_elem = graph_elem.with_attribute(("to", to_str.as_str()));
    }

    // Add common display attributes
    if let Some(ref color) = graph.color {
        graph_elem = graph_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = graph.background {
        graph_elem =
            graph_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = graph.z_index {
        let z_str = format!("{}", z_index);
        graph_elem = graph_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = graph.font_family {
        graph_elem = graph_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = graph.font_size {
        let font_size_str = format!("{}pt", font_size);
        graph_elem = graph_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = graph.font_weight {
        graph_elem = graph_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = graph.font_style {
        graph_elem = graph_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = graph.text_decoration {
        graph_elem = graph_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = graph.text_align {
        graph_elem = graph_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = graph.text_background {
        graph_elem = graph_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = graph.vertical_text_align {
        graph_elem = graph_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = graph.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        graph_elem = graph_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = graph.font_color {
        graph_elem =
            graph_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = graph.text_border_color {
        graph_elem = graph_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = graph.text_border_width {
        graph_elem = graph_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = graph.text_border_style {
        graph_elem = graph_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    // Serialize plots
    if !graph.plots.is_empty() {
        graph_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            for plot in &graph.plots {
                serialize_plot(writer, plot)?;
            }
            Ok(())
        })?;
    } else {
        graph_elem.write_empty()?;
    }

    Ok(())
}

/// Serialize TableItemType to string.
fn serialize_table_item_type(item_type: &TableItemType) -> &'static str {
    match item_type {
        TableItemType::Time => "time",
        TableItemType::Variable => "variable",
        TableItemType::Blank => "blank",
    }
}

/// Serialize TableOrientation to string.
fn serialize_table_orientation(orientation: &TableOrientation) -> &'static str {
    match orientation {
        TableOrientation::Horizontal => "horizontal",
        TableOrientation::Vertical => "vertical",
    }
}

/// Serialize ReportBalances to string.
fn serialize_report_balances(balances: &ReportBalances) -> &'static str {
    match balances {
        ReportBalances::Beginning => "beginning",
        ReportBalances::Ending => "ending",
    }
}

/// Serialize ReportFlows to string.
fn serialize_report_flows(flows: &ReportFlows) -> &'static str {
    match flows {
        ReportFlows::Instantaneous => "instantaneous",
        ReportFlows::Summed => "summed",
    }
}

/// Serialize a TableItem to XML.
fn serialize_table_item<W: Write>(
    writer: &mut Writer<W>,
    item: &TableItem,
) -> Result<(), SerializeError> {
    let mut item_elem = writer.create_element("item");
    item_elem = item_elem.with_attribute(("type", serialize_table_item_type(&item.item_type)));

    if let Some(ref entity_name) = item.entity_name {
        item_elem = item_elem.with_attribute(("entity_name", entity_name.as_str()));
    }
    if let Some(precision) = item.precision {
        let precision_str = format!("{}", precision);
        item_elem = item_elem.with_attribute(("precision", precision_str.as_str()));
    }
    item_elem = item_elem.with_attribute((
        "delimit_000s",
        if item.delimit_000s { "true" } else { "false" },
    ));

    if let Some(column_width) = item.column_width {
        let width_str = format!("{}", column_width);
        item_elem = item_elem.with_attribute(("column_width", width_str.as_str()));
    }

    item_elem.write_empty()?;
    Ok(())
}

/// Serialize a TableObject to XML.
fn serialize_table_object<W: Write>(
    writer: &mut Writer<W>,
    table: &TableObject,
) -> Result<(), SerializeError> {
    let mut table_elem = writer.create_element("table");

    // Required attributes
    let uid_str = format!("{}", table.uid.value);
    table_elem = table_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", table.x);
    table_elem = table_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", table.y);
    table_elem = table_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", table.width);
    table_elem = table_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", table.height);
    table_elem = table_elem.with_attribute(("height", height_str.as_str()));
    table_elem = table_elem.with_attribute((
        "orientation",
        serialize_table_orientation(&table.orientation),
    ));
    let column_width_str = format!("{}", table.column_width);
    table_elem = table_elem.with_attribute(("column_width", column_width_str.as_str()));
    table_elem = table_elem.with_attribute(("interval", table.interval.as_str()));
    table_elem = table_elem.with_attribute((
        "report_balances",
        serialize_report_balances(&table.report_balances),
    ));
    table_elem =
        table_elem.with_attribute(("report_flows", serialize_report_flows(&table.report_flows)));
    table_elem = table_elem.with_attribute((
        "comparative",
        if table.comparative { "true" } else { "false" },
    ));
    table_elem =
        table_elem.with_attribute(("wrap_text", if table.wrap_text { "true" } else { "false" }));

    // Optional attributes
    if let Some(ref title) = table.title {
        table_elem = table_elem.with_attribute(("title", title.as_str()));
    }
    if let Some(ref doc) = table.doc {
        table_elem = table_elem.with_attribute(("doc", doc.as_str()));
    }
    if let Some(blank_column_width) = table.blank_column_width {
        let blank_width_str = format!("{}", blank_column_width);
        table_elem = table_elem.with_attribute(("blank_column_width", blank_width_str.as_str()));
    }

    // Add common display attributes
    if let Some(ref color) = table.color {
        table_elem = table_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = table.background {
        table_elem =
            table_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = table.z_index {
        let z_str = format!("{}", z_index);
        table_elem = table_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = table.font_family {
        table_elem = table_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = table.font_size {
        let font_size_str = format!("{}pt", font_size);
        table_elem = table_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = table.font_weight {
        table_elem = table_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = table.font_style {
        table_elem = table_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = table.text_decoration {
        table_elem = table_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = table.text_align {
        table_elem = table_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = table.text_background {
        table_elem = table_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = table.vertical_text_align {
        table_elem = table_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = table.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        table_elem = table_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = table.font_color {
        table_elem =
            table_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = table.text_border_color {
        table_elem = table_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = table.text_border_width {
        table_elem = table_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = table.text_border_style {
        table_elem = table_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    // Header style attributes
    if let Some(ref header_font_family) = table.header_font_family {
        table_elem = table_elem.with_attribute(("header_font_family", header_font_family.as_str()));
    }
    if let Some(header_font_size) = table.header_font_size {
        let header_font_size_str = format!("{}pt", header_font_size);
        table_elem = table_elem.with_attribute(("header_font_size", header_font_size_str.as_str()));
    }
    if let Some(ref header_font_weight) = table.header_font_weight {
        table_elem = table_elem.with_attribute((
            "header_font_weight",
            serialize_font_weight(header_font_weight),
        ));
    }
    if let Some(ref header_font_style) = table.header_font_style {
        table_elem = table_elem
            .with_attribute(("header_font_style", serialize_font_style(header_font_style)));
    }
    if let Some(ref header_text_decoration) = table.header_text_decoration {
        table_elem = table_elem.with_attribute((
            "header_text_decoration",
            serialize_text_decoration(header_text_decoration),
        ));
    }
    if let Some(ref header_text_align) = table.header_text_align {
        table_elem = table_elem
            .with_attribute(("header_text_align", serialize_text_align(header_text_align)));
    }
    if let Some(ref header_vertical_text_align) = table.header_vertical_text_align {
        table_elem = table_elem.with_attribute((
            "header_vertical_text_align",
            serialize_vertical_text_align(header_vertical_text_align),
        ));
    }
    if let Some(ref header_text_background) = table.header_text_background {
        table_elem = table_elem.with_attribute((
            "header_text_background",
            serialize_color(header_text_background).as_str(),
        ));
    }
    if let Some(ref header_text_padding) = table.header_text_padding {
        let padding_str = serialize_text_padding(header_text_padding);
        table_elem = table_elem.with_attribute(("header_text_padding", padding_str.as_str()));
    }
    if let Some(ref header_font_color) = table.header_font_color {
        table_elem = table_elem.with_attribute((
            "header_font_color",
            serialize_color(header_font_color).as_str(),
        ));
    }
    if let Some(ref header_text_border_color) = table.header_text_border_color {
        table_elem = table_elem.with_attribute((
            "header_text_border_color",
            serialize_color(header_text_border_color).as_str(),
        ));
    }
    if let Some(ref header_text_border_width) = table.header_text_border_width {
        table_elem = table_elem.with_attribute((
            "header_text_border_width",
            serialize_border_width(header_text_border_width).as_str(),
        ));
    }
    if let Some(ref header_text_border_style) = table.header_text_border_style {
        table_elem = table_elem.with_attribute((
            "header_text_border_style",
            serialize_border_style(header_text_border_style),
        ));
    }

    // Serialize items
    if !table.items.is_empty() {
        table_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            for item in &table.items {
                serialize_table_item(writer, item)?;
            }
            Ok(())
        })?;
    } else {
        table_elem.write_empty()?;
    }

    Ok(())
}

/// Serialize TextBoxAppearance to string.
fn serialize_text_box_appearance(appearance: &TextBoxAppearance) -> &'static str {
    match appearance {
        TextBoxAppearance::Transparent => "transparent",
        TextBoxAppearance::Normal => "normal",
    }
}

/// Serialize a TextBoxObject to XML.
fn serialize_text_box_object<W: Write>(
    writer: &mut Writer<W>,
    text_box: &TextBoxObject,
) -> Result<(), SerializeError> {
    let mut text_box_elem = writer.create_element("text_box");

    // Required attributes
    let uid_str = format!("{}", text_box.uid.value);
    text_box_elem = text_box_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", text_box.x);
    text_box_elem = text_box_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", text_box.y);
    text_box_elem = text_box_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", text_box.width);
    text_box_elem = text_box_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", text_box.height);
    text_box_elem = text_box_elem.with_attribute(("height", height_str.as_str()));
    text_box_elem = text_box_elem.with_attribute((
        "appearance",
        serialize_text_box_appearance(&text_box.appearance),
    ));

    // Add common display attributes
    if let Some(ref color) = text_box.color {
        text_box_elem = text_box_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = text_box.background {
        text_box_elem =
            text_box_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = text_box.z_index {
        let z_str = format!("{}", z_index);
        text_box_elem = text_box_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = text_box.font_family {
        text_box_elem = text_box_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = text_box.font_size {
        let font_size_str = format!("{}pt", font_size);
        text_box_elem = text_box_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = text_box.font_weight {
        text_box_elem =
            text_box_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = text_box.font_style {
        text_box_elem =
            text_box_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = text_box.text_decoration {
        text_box_elem = text_box_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = text_box.text_align {
        text_box_elem =
            text_box_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = text_box.text_background {
        text_box_elem = text_box_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = text_box.vertical_text_align {
        text_box_elem = text_box_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = text_box.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        text_box_elem = text_box_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = text_box.font_color {
        text_box_elem =
            text_box_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = text_box.text_border_color {
        text_box_elem = text_box_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = text_box.text_border_width {
        text_box_elem = text_box_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = text_box.text_border_style {
        text_box_elem = text_box_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    // Write content as text
    text_box_elem.write_text_content(quick_xml::events::BytesText::new(&text_box.content))?;

    Ok(())
}

/// Serialize GraphicsFrameContent to XML.
fn serialize_graphics_frame_content<W: Write>(
    writer: &mut Writer<W>,
    content: &GraphicsFrameContent,
) -> Result<(), SerializeError> {
    match content {
        GraphicsFrameContent::Image(img) => {
            let mut img_elem = writer.create_element("image");
            img_elem = img_elem.with_attribute((
                "size_to_parent",
                if img.size_to_parent { "true" } else { "false" },
            ));
            let width_str = format!("{}", img.width);
            img_elem = img_elem.with_attribute(("width", width_str.as_str()));
            let height_str = format!("{}", img.height);
            img_elem = img_elem.with_attribute(("height", height_str.as_str()));

            if let Some(ref resource) = img.resource {
                img_elem = img_elem.with_attribute(("resource", resource.as_str()));
            }
            if let Some(ref data) = img.data {
                img_elem = img_elem.with_attribute(("data", data.as_str()));
            }

            img_elem.write_empty()?;
        }
        GraphicsFrameContent::Video(video) => {
            let mut video_elem = writer.create_element("video");
            video_elem = video_elem.with_attribute((
                "size_to_parent",
                if video.size_to_parent {
                    "true"
                } else {
                    "false"
                },
            ));
            let width_str = format!("{}", video.width);
            video_elem = video_elem.with_attribute(("width", width_str.as_str()));
            let height_str = format!("{}", video.height);
            video_elem = video_elem.with_attribute(("height", height_str.as_str()));
            video_elem = video_elem.with_attribute(("resource", video.resource.as_str()));
            video_elem.write_empty()?;
        }
    }
    Ok(())
}

/// Serialize a GraphicsFrameObject to XML.
fn serialize_graphics_frame_object<W: Write>(
    writer: &mut Writer<W>,
    frame: &GraphicsFrameObject,
) -> Result<(), SerializeError> {
    let mut frame_elem = writer.create_element("graphics_frame");

    // Required attributes
    let uid_str = format!("{}", frame.uid.value);
    frame_elem = frame_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", frame.x);
    frame_elem = frame_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", frame.y);
    frame_elem = frame_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", frame.width);
    frame_elem = frame_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", frame.height);
    frame_elem = frame_elem.with_attribute(("height", height_str.as_str()));

    // Optional border attributes
    if let Some(ref border_color) = frame.border_color {
        frame_elem =
            frame_elem.with_attribute(("border_color", serialize_color(border_color).as_str()));
    }
    if let Some(ref border_style) = frame.border_style {
        frame_elem =
            frame_elem.with_attribute(("border_style", serialize_border_style(border_style)));
    }
    if let Some(ref border_width) = frame.border_width {
        frame_elem = frame_elem.with_attribute((
            "border_width",
            serialize_border_width(border_width).as_str(),
        ));
    }

    // Add common display attributes
    if let Some(ref color) = frame.color {
        frame_elem = frame_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = frame.background {
        frame_elem =
            frame_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = frame.z_index {
        let z_str = format!("{}", z_index);
        frame_elem = frame_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = frame.font_family {
        frame_elem = frame_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = frame.font_size {
        let font_size_str = format!("{}pt", font_size);
        frame_elem = frame_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = frame.font_weight {
        frame_elem = frame_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = frame.font_style {
        frame_elem = frame_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = frame.text_decoration {
        frame_elem = frame_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = frame.text_align {
        frame_elem = frame_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = frame.text_background {
        frame_elem = frame_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = frame.vertical_text_align {
        frame_elem = frame_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = frame.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        frame_elem = frame_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = frame.font_color {
        frame_elem =
            frame_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = frame.text_border_color {
        frame_elem = frame_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = frame.text_border_width {
        frame_elem = frame_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = frame.text_border_style {
        frame_elem = frame_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    // Serialize content
    frame_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        serialize_graphics_frame_content(writer, &frame.content)?;
        Ok(())
    })?;

    Ok(())
}

/// Serialize ButtonAppearance to string.
fn serialize_button_appearance(appearance: &ButtonAppearance) -> &'static str {
    match appearance {
        ButtonAppearance::Opaque => "opaque",
        ButtonAppearance::Transparent => "transparent",
    }
}

/// Serialize ButtonStyle to string.
fn serialize_button_style(style: &ButtonStyle) -> &'static str {
    match style {
        ButtonStyle::Square => "square",
        ButtonStyle::Rounded => "rounded",
        ButtonStyle::Capsule => "capsule",
    }
}

/// Serialize LinkEffect to string.
fn serialize_link_effect(effect: &LinkEffect) -> &'static str {
    match effect {
        LinkEffect::Dissolve => "dissolve",
        LinkEffect::Checkerboard => "checkerboard",
        LinkEffect::Bars => "bars",
        LinkEffect::WipeLeft => "wipe_left",
        LinkEffect::WipeRight => "wipe_right",
        LinkEffect::WipeTop => "wipe_top",
        LinkEffect::WipeBottom => "wipe_bottom",
        LinkEffect::WipeClockwise => "wipe_clockwise",
        LinkEffect::WipeCounterclockwise => "wipe_counterclockwise",
        LinkEffect::IrisIn => "iris_in",
        LinkEffect::IrisOut => "iris_out",
        LinkEffect::DoorsClose => "doors_close",
        LinkEffect::DoorsOpen => "doors_open",
        LinkEffect::VenetianLeft => "venetian_left",
        LinkEffect::VenetianRight => "venetian_right",
        LinkEffect::VenetianTop => "venetian_top",
        LinkEffect::VenetianBottom => "venetian_bottom",
        LinkEffect::PushBottom => "push_bottom",
        LinkEffect::PushTop => "push_top",
        LinkEffect::PushLeft => "push_left",
        LinkEffect::PushRight => "push_right",
    }
}

/// Serialize a LinkTarget to XML attributes.
fn serialize_link_target_attributes(target: &LinkTarget) -> Vec<(&'static str, String)> {
    match target {
        LinkTarget::View { view_type, order } => {
            vec![
                ("target", "view".to_string()),
                ("view_type", view_type.clone()),
                ("order", order.clone()),
            ]
        }
        LinkTarget::Page {
            view_type,
            order,
            page,
        } => {
            vec![
                ("target", "page".to_string()),
                ("view_type", view_type.clone()),
                ("order", order.clone()),
                ("page", page.clone()),
            ]
        }
        LinkTarget::NextPage => vec![("target", "next_page".to_string())],
        LinkTarget::PreviousPage => vec![("target", "previous_page".to_string())],
        LinkTarget::HomePage => vec![("target", "home_page".to_string())],
        LinkTarget::NextView => vec![("target", "next_view".to_string())],
        LinkTarget::PreviousView => vec![("target", "previous_view".to_string())],
        LinkTarget::HomeView => vec![("target", "home_view".to_string())],
        LinkTarget::BackPage => vec![("target", "back_page".to_string())],
        LinkTarget::BackView => vec![("target", "back_view".to_string())],
        LinkTarget::Url(url) => vec![("target", "url".to_string()), ("url", url.clone())],
    }
}

/// Serialize PopupContent to XML.
fn serialize_popup_content<W: Write>(
    writer: &mut Writer<W>,
    popup: &PopupContent,
) -> Result<(), SerializeError> {
    match popup {
        PopupContent::TextBox(text_box) => {
            serialize_text_box_object(writer, text_box)?;
        }
        PopupContent::Image(img) => {
            let mut img_elem = writer.create_element("image");
            img_elem = img_elem.with_attribute((
                "size_to_parent",
                if img.size_to_parent { "true" } else { "false" },
            ));
            let width_str = format!("{}", img.width);
            img_elem = img_elem.with_attribute(("width", width_str.as_str()));
            let height_str = format!("{}", img.height);
            img_elem = img_elem.with_attribute(("height", height_str.as_str()));
            if let Some(ref resource) = img.resource {
                img_elem = img_elem.with_attribute(("resource", resource.as_str()));
            }
            if let Some(ref data) = img.data {
                img_elem = img_elem.with_attribute(("data", data.as_str()));
            }
            img_elem.write_empty()?;
        }
        PopupContent::Video(video) => {
            let mut video_elem = writer.create_element("video");
            video_elem = video_elem.with_attribute((
                "size_to_parent",
                if video.size_to_parent {
                    "true"
                } else {
                    "false"
                },
            ));
            let width_str = format!("{}", video.width);
            video_elem = video_elem.with_attribute(("width", width_str.as_str()));
            let height_str = format!("{}", video.height);
            video_elem = video_elem.with_attribute(("height", height_str.as_str()));
            video_elem = video_elem.with_attribute(("resource", video.resource.as_str()));
            video_elem.write_empty()?;
        }
    }
    Ok(())
}

/// Serialize a Link to XML.
fn serialize_link<W: Write>(writer: &mut Writer<W>, link: &Link) -> Result<(), SerializeError> {
    let mut link_elem = writer.create_element("link");
    let x_str = format!("{}", link.x);
    link_elem = link_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", link.y);
    link_elem = link_elem.with_attribute(("y", y_str.as_str()));
    let zoom_str = format!("{}", link.zoom);
    link_elem = link_elem.with_attribute(("zoom", zoom_str.as_str()));
    link_elem =
        link_elem.with_attribute(("to_black", if link.to_black { "true" } else { "false" }));

    if let Some(ref effect) = link.effect {
        link_elem = link_elem.with_attribute(("effect", serialize_link_effect(effect)));
    }

    // Add LinkTarget attributes
    for (key, value) in serialize_link_target_attributes(&link.target) {
        link_elem = link_elem.with_attribute((key, value.as_str()));
    }

    link_elem.write_empty()?;
    Ok(())
}

/// Serialize FileAction to string.
fn serialize_file_action(action: &FileAction) -> &'static str {
    match action {
        FileAction::Open => "open",
        FileAction::Close => "close",
        FileAction::Save => "save",
        FileAction::SaveAs => "save_as",
        FileAction::SaveAsImage => "save_as_image",
        FileAction::Revert => "revert",
    }
}

/// Serialize PrintingAction to string.
fn serialize_printing_action(action: &PrintingAction) -> &'static str {
    match action {
        PrintingAction::PrintSetup => "print_setup",
        PrintingAction::Print => "print",
        PrintingAction::PrintScreen => "print_screen",
    }
}

/// Serialize SimulationAction to string.
fn serialize_simulation_action(action: &SimulationAction) -> &'static str {
    match action {
        SimulationAction::Run => "run",
        SimulationAction::Pause => "pause",
        SimulationAction::Resume => "resume",
        SimulationAction::Stop => "stop",
        SimulationAction::RunRestore => "run_restore",
    }
}

/// Serialize RestoreAction to string.
fn serialize_restore_action(action: &RestoreAction) -> &'static str {
    match action {
        RestoreAction::RestoreAll => "restore_all",
        RestoreAction::RestoreSliders => "restore_sliders",
        RestoreAction::RestoreKnobs => "restore_knobs",
        RestoreAction::RestoreListInputs => "restore_list_inputs",
        RestoreAction::RestoreGraphicalInputs => "restore_graphical_inputs",
        RestoreAction::RestoreSwitches => "restore_switches",
        RestoreAction::RestoreNumericDisplays => "restore_numeric_displays",
        RestoreAction::RestoreGraphsTables => "restore_graphs_tables",
        RestoreAction::RestoreLampsGauges => "restore_lamps_gauges",
    }
}

/// Serialize DataAction to XML.
fn serialize_data_action<W: Write>(
    writer: &mut Writer<W>,
    action: &DataAction,
) -> Result<(), SerializeError> {
    match action {
        DataAction::DataManager => {
            writer
                .create_element("data_action")
                .with_attribute(("action", "data_manager"))
                .write_empty()?;
        }
        DataAction::SaveDataNow { run_name } => {
            let mut action_elem = writer.create_element("data_action");
            action_elem = action_elem.with_attribute(("action", "save_data_now"));
            action_elem = action_elem.with_attribute(("run_name", run_name.as_str()));
            action_elem.write_empty()?;
        }
        DataAction::ImportNow {
            resource,
            worksheet,
            all,
        } => {
            let mut action_elem = writer.create_element("data_action");
            action_elem = action_elem.with_attribute(("action", "import_now"));
            action_elem = action_elem.with_attribute(("resource", resource.as_str()));
            if let Some(ws) = worksheet {
                action_elem = action_elem.with_attribute(("worksheet", ws.as_str()));
            }
            action_elem = action_elem.with_attribute(("all", if *all { "true" } else { "false" }));
            action_elem.write_empty()?;
        }
        DataAction::ExportNow {
            resource,
            worksheet,
            all,
        } => {
            let mut action_elem = writer.create_element("data_action");
            action_elem = action_elem.with_attribute(("action", "export_now"));
            action_elem = action_elem.with_attribute(("resource", resource.as_str()));
            if let Some(ws) = worksheet {
                action_elem = action_elem.with_attribute(("worksheet", ws.as_str()));
            }
            action_elem = action_elem.with_attribute(("all", if *all { "true" } else { "false" }));
            action_elem.write_empty()?;
        }
    }
    Ok(())
}

/// Serialize MiscellaneousAction to string.
fn serialize_miscellaneous_action(action: &MiscellaneousAction) -> &'static str {
    match action {
        MiscellaneousAction::Exit => "exit",
        MiscellaneousAction::Find => "find",
        MiscellaneousAction::RunSpecs => "run_specs",
    }
}

/// Serialize a MenuAction to XML.
fn serialize_menu_action<W: Write>(
    writer: &mut Writer<W>,
    action: &MenuAction,
) -> Result<(), SerializeError> {
    let mut action_elem = writer.create_element("menu_action");

    match action {
        MenuAction::File(file_action) => {
            action_elem = action_elem.with_attribute(("type", "file"));
            action_elem =
                action_elem.with_attribute(("action", serialize_file_action(file_action)));
        }
        MenuAction::Printing(printing_action) => {
            action_elem = action_elem.with_attribute(("type", "printing"));
            action_elem =
                action_elem.with_attribute(("action", serialize_printing_action(printing_action)));
        }
        MenuAction::Simulation(sim_action) => {
            action_elem = action_elem.with_attribute(("type", "simulation"));
            action_elem =
                action_elem.with_attribute(("action", serialize_simulation_action(sim_action)));
        }
        MenuAction::Restore(restore_action) => {
            action_elem = action_elem.with_attribute(("type", "restore"));
            action_elem =
                action_elem.with_attribute(("action", serialize_restore_action(restore_action)));
        }
        MenuAction::Data(data_action) => {
            action_elem = action_elem.with_attribute(("type", "data"));
            // DataAction needs special handling with nested content
            action_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
                serialize_data_action(writer, data_action)?;
                Ok(())
            })?;
            return Ok(());
        }
        MenuAction::Miscellaneous(misc_action) => {
            action_elem = action_elem.with_attribute(("type", "miscellaneous"));
            action_elem =
                action_elem.with_attribute(("action", serialize_miscellaneous_action(misc_action)));
        }
    }

    action_elem.write_empty()?;
    Ok(())
}

/// Serialize a SwitchAction to XML.
fn serialize_switch_action<W: Write>(
    writer: &mut Writer<W>,
    action: &SwitchAction,
) -> Result<(), SerializeError> {
    let mut action_elem = writer.create_element("switch_action");
    let value_str = format!("{}", action.value);
    action_elem = action_elem.with_attribute(("value", value_str.as_str()));

    if let Some(ref entity_name) = action.entity_name {
        action_elem = action_elem.with_attribute(("entity_name", entity_name.as_str()));
    }
    if let Some(ref group_name) = action.group_name {
        action_elem = action_elem.with_attribute(("group_name", group_name.as_str()));
    }
    if let Some(ref module_name) = action.module_name {
        action_elem = action_elem.with_attribute(("module_name", module_name.as_str()));
    }

    action_elem.write_empty()?;
    Ok(())
}

/// Serialize a ButtonObject to XML.
fn serialize_button_object<W: Write>(
    writer: &mut Writer<W>,
    button: &ButtonObject,
) -> Result<(), SerializeError> {
    let mut button_elem = writer.create_element("button");

    // Required attributes
    let uid_str = format!("{}", button.uid.value);
    button_elem = button_elem.with_attribute(("uid", uid_str.as_str()));
    let x_str = format!("{}", button.x);
    button_elem = button_elem.with_attribute(("x", x_str.as_str()));
    let y_str = format!("{}", button.y);
    button_elem = button_elem.with_attribute(("y", y_str.as_str()));
    let width_str = format!("{}", button.width);
    button_elem = button_elem.with_attribute(("width", width_str.as_str()));
    let height_str = format!("{}", button.height);
    button_elem = button_elem.with_attribute(("height", height_str.as_str()));
    button_elem = button_elem.with_attribute((
        "appearance",
        serialize_button_appearance(&button.appearance),
    ));
    button_elem = button_elem.with_attribute(("style", serialize_button_style(&button.style)));
    button_elem = button_elem.with_attribute((
        "clicking_sound",
        if button.clicking_sound {
            "true"
        } else {
            "false"
        },
    ));

    // Optional attributes
    if let Some(ref label) = button.label {
        button_elem = button_elem.with_attribute(("label", label.as_str()));
    }
    if let Some(ref sound) = button.sound {
        button_elem = button_elem.with_attribute(("sound", sound.as_str()));
    }

    // Add common display attributes
    if let Some(ref color) = button.color {
        button_elem = button_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = button.background {
        button_elem =
            button_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = button.z_index {
        let z_str = format!("{}", z_index);
        button_elem = button_elem.with_attribute(("z_index", z_str.as_str()));
    }
    if let Some(ref font_family) = button.font_family {
        button_elem = button_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(font_size) = button.font_size {
        let font_size_str = format!("{}pt", font_size);
        button_elem = button_elem.with_attribute(("font_size", font_size_str.as_str()));
    }
    if let Some(ref font_weight) = button.font_weight {
        button_elem =
            button_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref font_style) = button.font_style {
        button_elem = button_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref text_decoration) = button.text_decoration {
        button_elem = button_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = button.text_align {
        button_elem = button_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref text_background) = button.text_background {
        button_elem = button_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(ref vertical_text_align) = button.vertical_text_align {
        button_elem = button_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref text_padding) = button.text_padding {
        let padding_str = serialize_text_padding(text_padding);
        button_elem = button_elem.with_attribute(("text_padding", padding_str.as_str()));
    }
    if let Some(ref font_color) = button.font_color {
        button_elem =
            button_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_border_color) = button.text_border_color {
        button_elem = button_elem.with_attribute((
            "text_border_color",
            serialize_color(text_border_color).as_str(),
        ));
    }
    if let Some(ref text_border_width) = button.text_border_width {
        button_elem = button_elem.with_attribute((
            "text_border_width",
            serialize_border_width(text_border_width).as_str(),
        ));
    }
    if let Some(ref text_border_style) = button.text_border_style {
        button_elem = button_elem.with_attribute((
            "text_border_style",
            serialize_border_style(text_border_style),
        ));
    }

    // Serialize optional nested elements
    let has_content = button.image.is_some()
        || button.popup.is_some()
        || button.link.is_some()
        || button.menu_action.is_some()
        || button.switch_action.is_some();

    if has_content {
        button_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            if let Some(ref image) = button.image {
                let mut img_elem = writer.create_element("image");
                img_elem = img_elem.with_attribute((
                    "size_to_parent",
                    if image.size_to_parent {
                        "true"
                    } else {
                        "false"
                    },
                ));
                let width_str = format!("{}", image.width);
                img_elem = img_elem.with_attribute(("width", width_str.as_str()));
                let height_str = format!("{}", image.height);
                img_elem = img_elem.with_attribute(("height", height_str.as_str()));
                if let Some(ref resource) = image.resource {
                    img_elem = img_elem.with_attribute(("resource", resource.as_str()));
                }
                if let Some(ref data) = image.data {
                    img_elem = img_elem.with_attribute(("data", data.as_str()));
                }
                img_elem.write_empty()?;
            }
            if let Some(ref popup) = button.popup {
                serialize_popup_content(writer, popup)?;
            }
            if let Some(ref link) = button.link {
                serialize_link(writer, link)?;
            }
            if let Some(ref menu_action) = button.menu_action {
                serialize_menu_action(writer, menu_action)?;
            }
            if let Some(ref switch_action) = button.switch_action {
                serialize_switch_action(writer, switch_action)?;
            }
            Ok(())
        })?;
    } else {
        button_elem.write_empty()?;
    }

    Ok(())
}

/// Serialize file-level Dimensions to XML.
pub fn serialize_file_dimensions<W: Write>(
    writer: &mut Writer<W>,
    dimensions: &Dimensions,
) -> Result<(), SerializeError> {
    writer.create_element("dimensions").write_inner_content(
        |writer| -> Result<(), SerializeError> {
            for dim in &dimensions.dims {
                serialize_dimension(writer, dim)?;
            }
            Ok(())
        },
    )?;
    Ok(())
}

/// Serialize a Dimension to XML.
fn serialize_dimension<W: Write>(
    writer: &mut Writer<W>,
    dim: &Dimension,
) -> Result<(), SerializeError> {
    let mut dim_elem = writer.create_element("dim");
    dim_elem = dim_elem.with_attribute(("name", dim.name.as_str()));

    if let Some(size) = dim.size {
        let size_str = format!("{}", size);
        dim_elem = dim_elem.with_attribute(("size", size_str.as_str()));
    }

    if !dim.elements.is_empty() {
        dim_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            for elem in &dim.elements {
                writer
                    .create_element("elem")
                    .with_attribute(("name", elem.name.as_str()))
                    .write_empty()?;
            }
            Ok(())
        })?;
    } else {
        dim_elem.write_empty()?;
    }
    Ok(())
}

/// Serialize ModelUnits to XML.
pub fn serialize_model_units<W: Write>(
    writer: &mut Writer<W>,
    model_units: &ModelUnits,
) -> Result<(), SerializeError> {
    writer.create_element("model_units").write_inner_content(
        |writer| -> Result<(), SerializeError> {
            for unit in &model_units.units {
                serialize_unit_definition(writer, unit)?;
            }
            Ok(())
        },
    )?;
    Ok(())
}

/// Serialize a UnitDefinition to XML.
fn serialize_unit_definition<W: Write>(
    writer: &mut Writer<W>,
    unit: &UnitDefinition,
) -> Result<(), SerializeError> {
    let mut unit_elem = writer.create_element("unit");
    unit_elem = unit_elem.with_attribute(("name", unit.name.as_str()));

    if let Some(disabled) = unit.disabled {
        if disabled {
            unit_elem = unit_elem.with_attribute(("disabled", "true"));
        }
    }

    unit_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        if let Some(ref eqn) = unit.eqn {
            write_element_with_text(writer, "eqn", eqn)?;
        }
        for alias in &unit.aliases {
            write_element_with_text(writer, "alias", alias)?;
        }
        Ok(())
    })?;
    Ok(())
}

/// Serialize a Variables structure to XML.
///
/// Variables contains a mix of different variable types (stock, flow, aux, gf, module, group)
/// that need to be serialized with their respective tag names.
pub fn serialize_variables<W: Write>(
    writer: &mut Writer<W>,
    variables: &Variables,
) -> Result<(), SerializeError> {
    writer.create_element("variables").write_inner_content(
        |writer| -> Result<(), SerializeError> {
            for var in &variables.variables {
                match var {
                    Variable::Stock(stock) => {
                        use crate::model::vars::stock::Stock;
                        match stock {
                            Stock::Basic(basic) => {
                                serialize_basic_stock(writer, basic)?;
                            }
                            Stock::Conveyor(_) | Stock::Queue(_) => {
                                // TODO: Implement conveyor and queue serialization in later phases
                                // For now, serialize as basic stock
                                // This is a placeholder
                            }
                        }
                    }
                    Variable::Flow(flow) => {
                        // Variable::Flow contains BasicFlow directly
                        serialize_basic_flow(writer, flow)?;
                    }
                    Variable::Auxiliary(aux) => {
                        serialize_auxiliary(writer, aux)?;
                    }
                    Variable::GraphicalFunction(gf) => {
                        serialize_graphical_function(writer, gf)?;
                    }
                    #[cfg(feature = "submodels")]
                    Variable::Module(module) => {
                        serialize_module(writer, module)?;
                    }
                    Variable::Group(group) => {
                        serialize_group(writer, group)?;
                    }
                }
            }
            Ok(())
        },
    )?;

    Ok(())
}

/// Serialize a DeviceRange to XML.
pub fn serialize_range<W: Write>(
    writer: &mut Writer<W>,
    range: &DeviceRange,
) -> Result<(), SerializeError> {
    let min_str = format!("{}", range.min);
    let max_str = format!("{}", range.max);
    writer
        .create_element("range")
        .with_attribute(("min", min_str.as_str()))
        .with_attribute(("max", max_str.as_str()))
        .write_empty()?;
    Ok(())
}

/// Serialize a DeviceScale to XML.
pub fn serialize_scale<W: Write>(
    writer: &mut Writer<W>,
    scale: &DeviceScale,
) -> Result<(), SerializeError> {
    let mut scale_elem = writer.create_element("scale");

    match scale {
        crate::model::object::DeviceScale::MinMax { min, max } => {
            let min_str = format!("{}", min);
            let max_str = format!("{}", max);
            scale_elem = scale_elem
                .with_attribute(("min", min_str.as_str()))
                .with_attribute(("max", max_str.as_str()));
        }
        crate::model::object::DeviceScale::Auto(auto) => {
            scale_elem = scale_elem.with_attribute(("auto", if *auto { "true" } else { "false" }));
        }
        crate::model::object::DeviceScale::Group(group) => {
            let group_str = format!("{}", group);
            scale_elem = scale_elem.with_attribute(("group", group_str.as_str()));
        }
    }

    scale_elem.write_empty()?;
    Ok(())
}

/// Serialize FormatOptions to XML.
pub fn serialize_format<W: Write>(
    writer: &mut Writer<W>,
    format: &FormatOptions,
) -> Result<(), SerializeError> {
    let mut format_elem = writer.create_element("format");

    if let Some(precision) = format.precision {
        let precision_str = format!("{}", precision);
        format_elem = format_elem.with_attribute(("precision", precision_str.as_str()));
    }
    if let Some(scale_by) = format.scale_by {
        let scale_by_str = format!("{}", scale_by);
        format_elem = format_elem.with_attribute(("scale_by", scale_by_str.as_str()));
    }
    if let Some(display_as) = &format.display_as {
        let display_str = match display_as {
            DisplayAs::Number => "number",
            DisplayAs::Currency => "currency",
            DisplayAs::Percent => "percent",
        };
        format_elem = format_elem.with_attribute(("display_as", display_str));
    }
    if let Some(delimit_000s) = format.delimit_000s {
        format_elem = format_elem
            .with_attribute(("delimit_000s", if delimit_000s { "true" } else { "false" }));
    }

    format_elem.write_empty()?;
    Ok(())
}

/// Serialize an ArrayElement to XML.
#[cfg(feature = "arrays")]
pub fn serialize_array_element<W: Write>(
    writer: &mut Writer<W>,
    element: &ArrayElement,
) -> Result<(), SerializeError> {
    let mut elem = writer.create_element("element");
    elem = elem.with_attribute(("subscript", element.subscript.as_str()));

    elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        // Optional: equation or graphical function (mutually exclusive)
        if let Some(ref eqn) = element.eqn {
            write_expression(writer, eqn)?;
        } else if let Some(ref gf) = element.gf {
            serialize_graphical_function(writer, gf)?;
        }
        Ok(())
    })?;

    Ok(())
}

/// Serialize a Model structure to XML.
pub fn serialize_model<W: Write>(
    writer: &mut Writer<W>,
    model: &Model,
) -> Result<(), SerializeError> {
    let mut model_elem = writer.create_element("model");

    // Add optional attributes
    if let Some(ref name) = model.name {
        model_elem = model_elem.with_attribute(("name", name.as_str()));
    }
    if let Some(ref resource) = model.resource {
        model_elem = model_elem.with_attribute(("resource", resource.as_str()));
    }

    model_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        // Optional sim_specs
        if let Some(ref sim_specs) = model.sim_specs {
            serialize_sim_specs(writer, sim_specs)?;
        }

        // Optional behavior
        if let Some(ref behavior) = model.behavior {
            serialize_behavior(writer, behavior)?;
        }

        // Required: variables
        serialize_variables(writer, &model.variables)?;

        // Optional views
        if let Some(ref views) = model.views {
            serialize_views(writer, views)?;
        }

        Ok(())
    })?;

    Ok(())
}

/// Serialize a Module to XML.
#[cfg(feature = "submodels")]
fn serialize_module<W: Write>(
    writer: &mut Writer<W>,
    module: &Module,
) -> Result<(), SerializeError> {
    let mut module_elem = writer.create_element("module");

    // Required attribute: name
    module_elem = module_elem.with_attribute(("name", module.name.to_string().as_str()));

    // Optional attribute: resource
    if let Some(ref resource) = module.resource {
        module_elem = module_elem.with_attribute(("resource", resource.as_str()));
    }

    // Optional child elements: connections and documentation
    let has_content = !module.connections.is_empty() || module.documentation.is_some();

    if has_content {
        module_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            // Serialize connections
            for connection in &module.connections {
                let mut connect_elem = writer.create_element("connect");
                connect_elem = connect_elem.with_attribute(("to", connection.to.as_str()));
                connect_elem = connect_elem.with_attribute(("from", connection.from.as_str()));
                connect_elem.write_empty()?;
            }

            // Serialize documentation
            if let Some(ref doc) = module.documentation {
                let doc_str = match doc {
                    crate::model::object::Documentation::PlainText(text) => text,
                    crate::model::object::Documentation::Html(html) => html,
                };
                write_element_with_text(writer, "doc", doc_str)?;
            }

            Ok(())
        })?;
    } else {
        module_elem.write_empty()?;
    }

    Ok(())
}

/// Serialize a Group to XML.
fn serialize_group<W: Write>(writer: &mut Writer<W>, group: &Group) -> Result<(), SerializeError> {
    let mut group_elem = writer.create_element("group");

    // Required attribute: name
    group_elem = group_elem.with_attribute(("name", group.name.to_string().as_str()));

    // Optional child elements: doc and entities
    let has_content = group.doc.is_some() || !group.entities.is_empty();

    if has_content {
        group_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
            // Serialize documentation
            if let Some(ref doc) = group.doc {
                let doc_str = match doc {
                    crate::model::object::Documentation::PlainText(text) => text,
                    crate::model::object::Documentation::Html(html) => html,
                };
                write_element_with_text(writer, "doc", doc_str)?;
            }

            // Serialize entities
            for entity in &group.entities {
                let mut entity_elem = writer.create_element("entity");
                entity_elem =
                    entity_elem.with_attribute(("name", entity.name.to_string().as_str()));
                entity_elem =
                    entity_elem.with_attribute(("run", if entity.run { "true" } else { "false" }));
                entity_elem.write_empty()?;
            }

            Ok(())
        })?;
    } else {
        group_elem.write_empty()?;
    }

    Ok(())
}

/// Serialize a Macro to XML.
#[cfg(feature = "macros")]
pub fn serialize_macro<W: Write>(
    writer: &mut Writer<W>,
    macro_def: &Macro,
) -> Result<(), SerializeError> {
    let mut macro_elem = writer.create_element("macro");

    // Required attribute: name
    macro_elem = macro_elem.with_attribute(("name", macro_def.name.to_string().as_str()));

    // Optional attribute: namespace
    if let Some(ref namespace_vec) = macro_def.namespace {
        if !namespace_vec.is_empty() {
            let ns_str = Namespace::as_prefix(namespace_vec);
            macro_elem = macro_elem.with_attribute(("namespace", ns_str.as_str()));
        }
    }

    // Always need to write eqn (required), so always use inner_content
    macro_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        // Serialize parameters (should appear before eqn)
        for param in &macro_def.parameters {
            let mut parm_elem = writer.create_element("parm");

            // Optional attribute: default
            if let Some(ref default_expr) = param.default {
                let default_str = default_expr.to_string();
                parm_elem = parm_elem.with_attribute(("default", default_str.as_str()));
            }

            // Text content: parameter name
            parm_elem
                .write_text_content(quick_xml::events::BytesText::new(&param.name.to_string()))?;
        }

        // Required: equation
        write_expression(writer, &macro_def.eqn)?;

        // Optional: format
        if let Some(ref format) = macro_def.format {
            write_element_with_text(writer, "format", format)?;
        }

        // Optional: documentation
        if let Some(ref doc) = macro_def.doc {
            let doc_str = match doc {
                crate::model::object::Documentation::PlainText(text) => text,
                crate::model::object::Documentation::Html(html) => html,
            };
            write_element_with_text(writer, "doc", doc_str)?;
        }

        // Optional: sim_specs
        if let Some(ref sim_specs) = macro_def.sim_specs {
            serialize_sim_specs(writer, sim_specs)?;
        }

        // Optional: variables
        if let Some(ref variables) = macro_def.variables {
            use crate::xml::schema::Variables;
            let vars_wrapper = Variables {
                variables: variables.clone(),
            };
            serialize_variables(writer, &vars_wrapper)?;
        }

        // Optional: views (exactly one view within <views>)
        if let Some(ref view) = macro_def.views {
            writer.create_element("views").write_inner_content(
                |writer| -> Result<(), SerializeError> {
                    serialize_view(writer, view)?;
                    Ok(())
                },
            )?;
        }

        Ok(())
    })?;

    Ok(())
}

/// Serialize a Behavior structure to XML.
pub fn serialize_behavior<W: Write>(
    writer: &mut Writer<W>,
    behavior: &Behavior,
) -> Result<(), SerializeError> {
    let behavior_elem = writer.create_element("behavior");

    // Check if we have any content to write
    let has_global = behavior.global.non_negative.is_some();
    let has_entities = !behavior.entities.is_empty();

    if !has_global && !has_entities {
        // Empty behavior tag
        behavior_elem.write_empty()?;
        return Ok(());
    }

    behavior_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        // Serialize global non_negative if present and true
        if let Some(true) = behavior.global.non_negative {
            writer.create_element("non_negative").write_empty()?;
        }

        // Serialize entity-specific behaviors
        for entry in &behavior.entities {
            if let Some(true) = entry.behavior.non_negative {
                let entity_elem = writer.create_element(entry.entity_type.as_str());
                entity_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
                    writer.create_element("non_negative").write_empty()?;
                    Ok(())
                })?;
            }
        }

        Ok(())
    })?;

    Ok(())
}

/// Serialize a Style structure to XML.
pub fn serialize_style<W: Write>(
    writer: &mut Writer<W>,
    style: &Style,
) -> Result<(), SerializeError> {
    let mut style_elem = writer.create_element("style");

    // Add global style attributes
    if let Some(ref color) = style.color {
        style_elem = style_elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = style.background {
        style_elem =
            style_elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = style.z_index {
        style_elem = style_elem.with_attribute(("z_index", format!("{}", z_index).as_str()));
    }
    if let Some(ref border_width) = style.border_width {
        style_elem = style_elem.with_attribute((
            "border_width",
            serialize_border_width(border_width).as_str(),
        ));
    }
    if let Some(ref border_color) = style.border_color {
        style_elem =
            style_elem.with_attribute(("border_color", serialize_color(border_color).as_str()));
    }
    if let Some(ref border_style) = style.border_style {
        style_elem =
            style_elem.with_attribute(("border_style", serialize_border_style(border_style)));
    }
    if let Some(ref font_family) = style.font_family {
        style_elem = style_elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(ref font_style) = style.font_style {
        style_elem = style_elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref font_weight) = style.font_weight {
        style_elem = style_elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref text_decoration) = style.text_decoration {
        style_elem = style_elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = style.text_align {
        style_elem = style_elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref vertical_text_align) = style.vertical_text_align {
        style_elem = style_elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref font_color) = style.font_color {
        style_elem =
            style_elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_background) = style.text_background {
        style_elem = style_elem
            .with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(font_size) = style.font_size {
        style_elem = style_elem.with_attribute(("font_size", format!("{}pt", font_size).as_str()));
    }
    if let Some(ref padding) = style.padding {
        style_elem = style_elem.with_attribute(("padding", serialize_padding(padding).as_str()));
    }

    // Check if we have object-specific styles
    let has_object_styles = style.stock.is_some()
        || style.flow.is_some()
        || style.aux.is_some()
        || style.module.is_some()
        || style.group.is_some()
        || style.connector.is_some()
        || style.alias.is_some()
        || style.slider.is_some()
        || style.knob.is_some()
        || style.switch.is_some()
        || style.options.is_some()
        || style.numeric_input.is_some()
        || style.list_input.is_some()
        || style.graphical_input.is_some()
        || style.numeric_display.is_some()
        || style.lamp.is_some()
        || style.gauge.is_some()
        || style.graph.is_some()
        || style.table.is_some()
        || style.text_box.is_some()
        || style.graphics_frame.is_some()
        || style.button.is_some();

    if !has_object_styles {
        // No object-specific styles, write as empty or self-closing tag
        style_elem.write_empty()?;
        return Ok(());
    }

    // Write object-specific styles as child elements
    style_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        if let Some(ref obj_style) = style.stock {
            serialize_object_style(writer, "stock", obj_style)?;
        }
        if let Some(ref obj_style) = style.flow {
            serialize_object_style(writer, "flow", obj_style)?;
        }
        if let Some(ref obj_style) = style.aux {
            serialize_object_style(writer, "aux", obj_style)?;
        }
        if let Some(ref obj_style) = style.module {
            serialize_object_style(writer, "module", obj_style)?;
        }
        if let Some(ref obj_style) = style.group {
            serialize_object_style(writer, "group", obj_style)?;
        }
        if let Some(ref obj_style) = style.connector {
            serialize_object_style(writer, "connector", obj_style)?;
        }
        if let Some(ref obj_style) = style.alias {
            serialize_object_style(writer, "alias", obj_style)?;
        }
        if let Some(ref obj_style) = style.slider {
            serialize_object_style(writer, "slider", obj_style)?;
        }
        if let Some(ref obj_style) = style.knob {
            serialize_object_style(writer, "knob", obj_style)?;
        }
        if let Some(ref obj_style) = style.switch {
            serialize_object_style(writer, "switch", obj_style)?;
        }
        if let Some(ref obj_style) = style.options {
            serialize_object_style(writer, "options", obj_style)?;
        }
        if let Some(ref obj_style) = style.numeric_input {
            serialize_object_style(writer, "numeric_input", obj_style)?;
        }
        if let Some(ref obj_style) = style.list_input {
            serialize_object_style(writer, "list_input", obj_style)?;
        }
        if let Some(ref obj_style) = style.graphical_input {
            serialize_object_style(writer, "graphical_input", obj_style)?;
        }
        if let Some(ref obj_style) = style.numeric_display {
            serialize_object_style(writer, "numeric_display", obj_style)?;
        }
        if let Some(ref obj_style) = style.lamp {
            serialize_object_style(writer, "lamp", obj_style)?;
        }
        if let Some(ref obj_style) = style.gauge {
            serialize_object_style(writer, "gauge", obj_style)?;
        }
        if let Some(ref obj_style) = style.graph {
            serialize_object_style(writer, "graph", obj_style)?;
        }
        if let Some(ref obj_style) = style.table {
            serialize_object_style(writer, "table", obj_style)?;
        }
        if let Some(ref obj_style) = style.text_box {
            serialize_object_style(writer, "text_box", obj_style)?;
        }
        if let Some(ref obj_style) = style.graphics_frame {
            serialize_object_style(writer, "graphics_frame", obj_style)?;
        }
        if let Some(ref obj_style) = style.button {
            serialize_object_style(writer, "button", obj_style)?;
        }
        Ok(())
    })?;

    Ok(())
}

/// Serialize an ObjectStyle to XML as a child element.
fn serialize_object_style<W: Write>(
    writer: &mut Writer<W>,
    element_name: &str,
    obj_style: &ObjectStyle,
) -> Result<(), SerializeError> {
    let mut elem = writer.create_element(element_name);

    // Add all style attributes
    if let Some(ref color) = obj_style.color {
        elem = elem.with_attribute(("color", serialize_color(color).as_str()));
    }
    if let Some(ref background) = obj_style.background {
        elem = elem.with_attribute(("background", serialize_color(background).as_str()));
    }
    if let Some(z_index) = obj_style.z_index {
        elem = elem.with_attribute(("z_index", format!("{}", z_index).as_str()));
    }
    if let Some(ref border_width) = obj_style.border_width {
        elem = elem.with_attribute((
            "border_width",
            serialize_border_width(border_width).as_str(),
        ));
    }
    if let Some(ref border_color) = obj_style.border_color {
        elem = elem.with_attribute(("border_color", serialize_color(border_color).as_str()));
    }
    if let Some(ref border_style) = obj_style.border_style {
        elem = elem.with_attribute(("border_style", serialize_border_style(border_style)));
    }
    if let Some(ref font_family) = obj_style.font_family {
        elem = elem.with_attribute(("font_family", font_family.as_str()));
    }
    if let Some(ref font_style) = obj_style.font_style {
        elem = elem.with_attribute(("font_style", serialize_font_style(font_style)));
    }
    if let Some(ref font_weight) = obj_style.font_weight {
        elem = elem.with_attribute(("font_weight", serialize_font_weight(font_weight)));
    }
    if let Some(ref text_decoration) = obj_style.text_decoration {
        elem = elem.with_attribute((
            "text_decoration",
            serialize_text_decoration(text_decoration),
        ));
    }
    if let Some(ref text_align) = obj_style.text_align {
        elem = elem.with_attribute(("text_align", serialize_text_align(text_align)));
    }
    if let Some(ref vertical_text_align) = obj_style.vertical_text_align {
        elem = elem.with_attribute((
            "vertical_text_align",
            serialize_vertical_text_align(vertical_text_align),
        ));
    }
    if let Some(ref font_color) = obj_style.font_color {
        elem = elem.with_attribute(("font_color", serialize_color(font_color).as_str()));
    }
    if let Some(ref text_background) = obj_style.text_background {
        elem = elem.with_attribute(("text_background", serialize_color(text_background).as_str()));
    }
    if let Some(font_size) = obj_style.font_size {
        elem = elem.with_attribute(("font_size", format!("{}pt", font_size).as_str()));
    }
    if let Some(ref padding) = obj_style.padding {
        elem = elem.with_attribute(("padding", serialize_padding(padding).as_str()));
    }

    elem.write_empty()?;
    Ok(())
}

/// Serialize Padding to string (comma-separated values).
fn serialize_padding(padding: &Padding) -> String {
    let mut parts = vec![format!("{}", padding.top)];
    if let Some(right) = padding.right {
        parts.push(format!("{}", right));
    }
    if let Some(bottom) = padding.bottom {
        parts.push(format!("{}", bottom));
    }
    if let Some(left) = padding.left {
        parts.push(format!("{}", left));
    }
    parts.join(",")
}

/// Serialize a Data structure to XML.
pub fn serialize_data<W: Write>(writer: &mut Writer<W>, data: &Data) -> Result<(), SerializeError> {
    let data_elem = writer.create_element("data");

    if data.imports.is_empty() && data.exports.is_empty() {
        data_elem.write_empty()?;
        return Ok(());
    }

    data_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        // Serialize imports
        for import in &data.imports {
            serialize_data_import(writer, import)?;
        }

        // Serialize exports
        for export in &data.exports {
            serialize_data_export(writer, export)?;
        }

        Ok(())
    })?;

    Ok(())
}

/// Serialize a DataImport to XML.
fn serialize_data_import<W: Write>(
    writer: &mut Writer<W>,
    import: &DataImport,
) -> Result<(), SerializeError> {
    let mut import_elem = writer.create_element("import");

    if let Some(ref data_type) = import.data_type {
        import_elem = import_elem.with_attribute(("type", data_type.as_str()));
    }
    if let Some(enabled) = import.enabled {
        import_elem =
            import_elem.with_attribute(("enabled", if enabled { "true" } else { "false" }));
    }
    if let Some(ref frequency) = import.frequency {
        import_elem = import_elem.with_attribute(("frequency", frequency.as_str()));
    }
    if let Some(ref orientation) = import.orientation {
        import_elem = import_elem.with_attribute(("orientation", orientation.as_str()));
    }
    if let Some(ref resource) = import.resource {
        import_elem = import_elem.with_attribute(("resource", resource.as_str()));
    }
    if let Some(ref worksheet) = import.worksheet {
        import_elem = import_elem.with_attribute(("worksheet", worksheet.as_str()));
    }

    import_elem.write_empty()?;
    Ok(())
}

/// Serialize a DataExport to XML.
fn serialize_data_export<W: Write>(
    writer: &mut Writer<W>,
    export: &DataExport,
) -> Result<(), SerializeError> {
    let mut export_elem = writer.create_element("export");

    if let Some(ref data_type) = export.data_type {
        export_elem = export_elem.with_attribute(("type", data_type.as_str()));
    }
    if let Some(enabled) = export.enabled {
        export_elem =
            export_elem.with_attribute(("enabled", if enabled { "true" } else { "false" }));
    }
    if let Some(ref frequency) = export.frequency {
        export_elem = export_elem.with_attribute(("frequency", frequency.as_str()));
    }
    if let Some(ref orientation) = export.orientation {
        export_elem = export_elem.with_attribute(("orientation", orientation.as_str()));
    }
    if let Some(ref resource) = export.resource {
        export_elem = export_elem.with_attribute(("resource", resource.as_str()));
    }
    if let Some(ref worksheet) = export.worksheet {
        export_elem = export_elem.with_attribute(("worksheet", worksheet.as_str()));
    }
    if let Some(ref interval) = export.interval {
        export_elem = export_elem.with_attribute(("interval", interval.as_str()));
    }

    // Check if we have content (all or table)
    let has_content = export.export_all.is_some() || export.table_uid.is_some();

    if !has_content {
        export_elem.write_empty()?;
        return Ok(());
    }

    export_elem.write_inner_content(|writer| -> Result<(), SerializeError> {
        if export.export_all.is_some() {
            writer.create_element("all").write_empty()?;
        }
        if let Some(ref table) = export.table_uid {
            let mut table_elem = writer.create_element("table");
            table_elem = table_elem.with_attribute(("uid", table.uid.as_str()));
            if let Some(use_settings) = table.use_settings {
                table_elem = table_elem
                    .with_attribute(("use_settings", if use_settings { "true" } else { "false" }));
            }
            table_elem.write_empty()?;
        }
        Ok(())
    })?;

    Ok(())
}
