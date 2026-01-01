pub mod style;
pub use style::Style;

use serde::{Deserialize, Deserializer, Serialize};

use crate::{Uid, Vendor};

pub mod objects;
pub use objects::*;

/// The type of a view determines what kind of display objects it can contain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewType {
    StockFlow,
    Interface,
    Popup,
    VendorSpecific(Vendor, String),
}

/// A view contains XMILE display objects and represents a page or screen
/// of a model's stock and flow diagram, or its interface.
#[derive(Debug, Clone, PartialEq)]
pub struct View {
    pub uid: Uid,
    pub view_type: ViewType,
    pub order: Option<u32>,
    pub width: f64,
    pub height: f64,
    pub zoom: Option<f64>,
    pub scroll_x: Option<f64>,
    pub scroll_y: Option<f64>,
    pub background: Option<String>,
    pub page_width: f64,
    pub page_height: f64,
    pub page_sequence: PageSequence,
    pub page_orientation: PageOrientation,
    pub show_pages: bool,
    pub home_page: u32,
    pub home_view: bool,
    /// Optional style definitions that apply to all objects within this view.
    pub style: Option<Style>,
    /// Stock and flow diagram objects
    pub stocks: Vec<StockObject>,
    pub flows: Vec<FlowObject>,
    pub auxes: Vec<AuxObject>,
    pub modules: Vec<ModuleObject>,
    pub groups: Vec<GroupObject>,
    pub connectors: Vec<ConnectorObject>,
    pub aliases: Vec<AliasObject>,
    /// Container objects
    pub stacked_containers: Vec<StackedContainerObject>,
    /// Input objects
    pub sliders: Vec<SliderObject>,
    pub knobs: Vec<KnobObject>,
    pub switches: Vec<SwitchObject>,
    pub options: Vec<OptionsObject>,
    pub numeric_inputs: Vec<NumericInputObject>,
    pub list_inputs: Vec<ListInputObject>,
    pub graphical_inputs: Vec<GraphicalInputObject>,
    /// Output objects
    pub numeric_displays: Vec<NumericDisplayObject>,
    pub lamps: Vec<LampObject>,
    pub gauges: Vec<GaugeObject>,
    pub graphs: Vec<GraphObject>,
    pub tables: Vec<TableObject>,
    /// Annotation objects
    pub text_boxes: Vec<TextBoxObject>,
    pub graphics_frames: Vec<GraphicsFrameObject>,
    pub buttons: Vec<ButtonObject>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PageSequence {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PageOrientation {
    Landscape,
    Portrait,
}

/// Raw view structure for deserialization from XML.
/// Handles mixed content within a <view> tag.
#[derive(Debug, Clone, PartialEq, Deserialize)]
struct RawView {
    #[serde(rename = "@uid")]
    uid: i32,
    #[serde(rename = "@type", default)]
    r#type: Option<String>,
    #[serde(rename = "@order")]
    order: Option<u32>,
    #[serde(rename = "@width")]
    width: f64,
    #[serde(rename = "@height")]
    height: f64,
    #[serde(rename = "@zoom")]
    zoom: Option<f64>,
    #[serde(rename = "@scroll_x")]
    scroll_x: Option<f64>,
    #[serde(rename = "@scroll_y")]
    scroll_y: Option<f64>,
    #[serde(rename = "@background")]
    background: Option<String>,
    #[serde(rename = "@page_width")]
    page_width: f64,
    #[serde(rename = "@page_height")]
    page_height: f64,
    #[serde(rename = "@page_sequence", default = "default_page_sequence")]
    page_sequence: PageSequence,
    #[serde(rename = "@page_orientation", default = "default_page_orientation")]
    page_orientation: PageOrientation,
    #[serde(rename = "@show_pages", default = "default_false")]
    show_pages: bool,
    #[serde(rename = "@home_page", default = "default_zero")]
    home_page: u32,
    #[serde(rename = "@home_view", default = "default_false")]
    home_view: bool,
    #[serde(rename = "style")]
    style: Option<Style>,
    // Stock and flow diagram objects
    #[serde(rename = "stock", default)]
    stocks: Vec<StockObject>,
    #[serde(rename = "flow", default)]
    flows: Vec<FlowObject>,
    #[serde(rename = "aux", default)]
    auxes: Vec<AuxObject>,
    #[serde(rename = "module", default)]
    modules: Vec<ModuleObject>,
    #[serde(rename = "group", default)]
    groups: Vec<GroupObject>,
    #[serde(rename = "connector", default)]
    connectors: Vec<ConnectorObject>,
    #[serde(rename = "alias", default)]
    aliases: Vec<AliasObject>,
    // Container objects
    #[serde(rename = "stacked_container", default)]
    stacked_containers: Vec<StackedContainerObject>,
    // Input objects
    #[serde(rename = "slider", default)]
    sliders: Vec<SliderObject>,
    #[serde(rename = "knob", default)]
    knobs: Vec<KnobObject>,
    #[serde(rename = "switch", default)]
    switches: Vec<SwitchObject>,
    #[serde(rename = "options", default)]
    options: Vec<OptionsObject>,
    #[serde(rename = "numeric_input", default)]
    numeric_inputs: Vec<NumericInputObject>,
    #[serde(rename = "list_input", default)]
    list_inputs: Vec<ListInputObject>,
    #[serde(rename = "graphical_input", default)]
    graphical_inputs: Vec<GraphicalInputObject>,
    // Output objects
    #[serde(rename = "numeric_display", default)]
    numeric_displays: Vec<NumericDisplayObject>,
    #[serde(rename = "lamp", default)]
    lamps: Vec<LampObject>,
    #[serde(rename = "gauge", default)]
    gauges: Vec<GaugeObject>,
    #[serde(rename = "graph", default)]
    graphs: Vec<GraphObject>,
    #[serde(rename = "table", default)]
    tables: Vec<TableObject>,
    // Annotation objects
    #[serde(rename = "text_box", default)]
    text_boxes: Vec<TextBoxObject>,
    #[serde(rename = "graphics_frame", default)]
    graphics_frames: Vec<GraphicsFrameObject>,
    #[serde(rename = "button", default)]
    buttons: Vec<ButtonObject>,
}

fn default_page_sequence() -> PageSequence {
    PageSequence::Row
}

fn default_page_orientation() -> PageOrientation {
    PageOrientation::Landscape
}

fn default_false() -> bool {
    false
}

fn default_zero() -> u32 {
    0
}

fn parse_vendor(s: &str) -> Vendor {
    match s.to_lowercase().as_str() {
        "anylogic" => Vendor::Anylogic,
        "forio" => Vendor::Forio,
        "insightmaker" => Vendor::Insightmaker,
        "isee" => Vendor::Isee,
        "powersim" => Vendor::Powersim,
        "simanticssd" => Vendor::Simanticssd,
        "simile" => Vendor::Simile,
        "sysdea" => Vendor::Sysdea,
        "vensim" => Vendor::Vensim,
        "simlab" => Vendor::SimLab,
        _ => Vendor::Other,
    }
}

impl From<RawView> for View {
    fn from(raw: RawView) -> Self {
        // Parse view_type from type attribute
        let view_type = if let Some(type_str) = raw.r#type {
            // Try to parse as ViewType
            match type_str.as_str() {
                "stock_flow" => ViewType::StockFlow,
                "interface" => ViewType::Interface,
                "popup" => ViewType::Popup,
                _ => {
                    // Try to parse as vendor-specific
                    // Format: "vendor:type" or just use as-is
                    if let Some((vendor_str, type_part)) = type_str.split_once(':') {
                        let vendor = parse_vendor(vendor_str);
                        ViewType::VendorSpecific(vendor, type_part.to_string())
                    } else {
                        ViewType::StockFlow // Default fallback
                    }
                }
            }
        } else {
            ViewType::StockFlow // Default when type is not specified
        };

        View {
            uid: Uid::new(raw.uid),
            view_type,
            order: raw.order,
            width: raw.width,
            height: raw.height,
            zoom: raw.zoom,
            scroll_x: raw.scroll_x,
            scroll_y: raw.scroll_y,
            background: raw.background,
            page_width: raw.page_width,
            page_height: raw.page_height,
            page_sequence: raw.page_sequence,
            page_orientation: raw.page_orientation,
            show_pages: raw.show_pages,
            home_page: raw.home_page,
            home_view: raw.home_view,
            style: raw.style,
            stocks: raw.stocks,
            flows: raw.flows,
            auxes: raw.auxes,
            modules: raw.modules,
            groups: raw.groups,
            connectors: raw.connectors,
            aliases: raw.aliases,
            stacked_containers: raw.stacked_containers,
            sliders: raw.sliders,
            knobs: raw.knobs,
            switches: raw.switches,
            options: raw.options,
            numeric_inputs: raw.numeric_inputs,
            list_inputs: raw.list_inputs,
            graphical_inputs: raw.graphical_inputs,
            numeric_displays: raw.numeric_displays,
            lamps: raw.lamps,
            gauges: raw.gauges,
            graphs: raw.graphs,
            tables: raw.tables,
            text_boxes: raw.text_boxes,
            graphics_frames: raw.graphics_frames,
            buttons: raw.buttons,
        }
    }
}

impl<'de> Deserialize<'de> for View {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawView::deserialize(deserializer)?;
        Ok(View::from(raw))
    }
}

impl Serialize for View {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("view", 22)?;
        
        state.serialize_field("@uid", &self.uid)?;
        
        // Serialize view_type
        let type_str = match &self.view_type {
            ViewType::StockFlow => "stock_flow",
            ViewType::Interface => "interface",
            ViewType::Popup => "popup",
            ViewType::VendorSpecific(_vendor, _type_part) => {
                // For vendor-specific, we'd need to serialize as "vendor:type"
                // For now, serialize as stock_flow and note this might need adjustment
                "stock_flow"
            }
        };
        state.serialize_field("@type", type_str)?;
        
        if let Some(order) = &self.order {
            state.serialize_field("@order", order)?;
        }
        state.serialize_field("@width", &self.width)?;
        state.serialize_field("@height", &self.height)?;
        if let Some(zoom) = &self.zoom {
            state.serialize_field("@zoom", zoom)?;
        }
        if let Some(scroll_x) = &self.scroll_x {
            state.serialize_field("@scroll_x", scroll_x)?;
        }
        if let Some(scroll_y) = &self.scroll_y {
            state.serialize_field("@scroll_y", scroll_y)?;
        }
        if let Some(background) = &self.background {
            state.serialize_field("@background", background)?;
        }
        state.serialize_field("@page_width", &self.page_width)?;
        state.serialize_field("@page_height", &self.page_height)?;
        state.serialize_field("@page_sequence", &self.page_sequence)?;
        state.serialize_field("@page_orientation", &self.page_orientation)?;
        state.serialize_field("@show_pages", &self.show_pages)?;
        state.serialize_field("@home_page", &self.home_page)?;
        state.serialize_field("@home_view", &self.home_view)?;
        
        if let Some(style) = &self.style {
            state.serialize_field("style", style)?;
        }
        
        // Serialize all object vectors
        if !self.stocks.is_empty() {
            state.serialize_field("stock", &self.stocks)?;
        }
        if !self.flows.is_empty() {
            state.serialize_field("flow", &self.flows)?;
        }
        if !self.auxes.is_empty() {
            state.serialize_field("aux", &self.auxes)?;
        }
        if !self.modules.is_empty() {
            state.serialize_field("module", &self.modules)?;
        }
        if !self.groups.is_empty() {
            state.serialize_field("group", &self.groups)?;
        }
        if !self.connectors.is_empty() {
            state.serialize_field("connector", &self.connectors)?;
        }
        if !self.aliases.is_empty() {
            state.serialize_field("alias", &self.aliases)?;
        }
        if !self.stacked_containers.is_empty() {
            state.serialize_field("stacked_container", &self.stacked_containers)?;
        }
        if !self.sliders.is_empty() {
            state.serialize_field("slider", &self.sliders)?;
        }
        if !self.knobs.is_empty() {
            state.serialize_field("knob", &self.knobs)?;
        }
        if !self.switches.is_empty() {
            state.serialize_field("switch", &self.switches)?;
        }
        if !self.options.is_empty() {
            state.serialize_field("options", &self.options)?;
        }
        if !self.numeric_inputs.is_empty() {
            state.serialize_field("numeric_input", &self.numeric_inputs)?;
        }
        if !self.list_inputs.is_empty() {
            state.serialize_field("list_input", &self.list_inputs)?;
        }
        if !self.graphical_inputs.is_empty() {
            state.serialize_field("graphical_input", &self.graphical_inputs)?;
        }
        if !self.numeric_displays.is_empty() {
            state.serialize_field("numeric_display", &self.numeric_displays)?;
        }
        if !self.lamps.is_empty() {
            state.serialize_field("lamp", &self.lamps)?;
        }
        if !self.gauges.is_empty() {
            state.serialize_field("gauge", &self.gauges)?;
        }
        if !self.graphs.is_empty() {
            state.serialize_field("graph", &self.graphs)?;
        }
        if !self.tables.is_empty() {
            state.serialize_field("table", &self.tables)?;
        }
        if !self.text_boxes.is_empty() {
            state.serialize_field("text_box", &self.text_boxes)?;
        }
        if !self.graphics_frames.is_empty() {
            state.serialize_field("graphics_frame", &self.graphics_frames)?;
        }
        if !self.buttons.is_empty() {
            state.serialize_field("button", &self.buttons)?;
        }
        
        state.end()
    }
}
