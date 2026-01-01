pub mod style;
pub use style::Style;

use crate::{Uid, Vendor};

pub mod objects;

pub enum ViewType {
    StockFlow,
    Interface,
    Popup,
    VendorSpecific(Vendor, String),
}

pub trait DisplayObject {
    fn uid(&self) -> Uid;
    fn x(&self) -> f64;
    fn y(&self) -> f64;
    fn width(&self) -> f64;
    fn height(&self) -> f64;
}

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
}

pub enum PageSequence {
    Row,
    Column,
}

pub enum PageOrientation {
    Landscape,
    Portrait,
}
