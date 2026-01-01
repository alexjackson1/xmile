pub mod style;
pub use style::Style;

use crate::{Uid, Vendor};

pub mod objects;
pub use objects::*;

/// The type of a view determines what kind of display objects it can contain.
#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSequence {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageOrientation {
    Landscape,
    Portrait,
}
