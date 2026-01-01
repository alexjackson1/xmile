// 2.7 Style Section
// Every XMILE file MAY include style information to set default options for display objects. When <has_model_view> is set in the <options> block, support for styles is REQUIRED. Being style information, this mostly belongs to the Presentation section (5.3), which describes the display and layout of XMILE files.
// The style information is cascading across five levels from the entity outwards, with the actual entity style defined by the first occurrence of a style definition for that style property:
// 1.     Styles for the given entity
// 2.     Styles for all entities in a specific view
// 3.     Styles for all entities in a collection of views
// 4.     Styles for all entities in the XMILE file
// 5.     Default XMILE-defined styles when a default appears in this specification
// The style information usually includes program defaults when they differ from the standard, though it can also be used for file-specific file-wide settings. Whenever possible, style information uses standard CSS syntax and keywords recast into XML attributes and nodes.
// The style block begins with the <style> tag. Within this block, any known object can have its attributes set globally (but overridden locally) using its own modifier tags. Global settings that apply to everything are specified directly on the <style> tag or in nodes below it; this is true for <style> tags that appear within the <views> tag as well. For example, the following sets the color of objects within all views in the file to blue and the background to white:
// <style color="blue" background="white"/>
// Unless otherwise indicated or specified, style information appears in XML attributes. For example, font_family would be an attribute.
// These changes can also be applied directly to objects (again as a child to a <style> tag), e.g.,
// <style color="blue" background="white">
//    <connector color="magenta">
// </style>
// Note that when style information applies to a specific object, that style cannot be overridden at a lower level (e.g., within a view) by a change to the overall style (i.e., by the options on the <style> tag). Using the example above, to override the color of connectors at a lower level (e.g., the Display), the <connector> tag must explicitly appear in that level’s style block. If it does not appear there, connectors will be magenta at that level by default, even if the style block at that level sets the default color of all objects to green. In other words, object-specific styles at any level above an object take precedence over an overall style defined at any lower level.

use serde::{Deserialize, Serialize};

/// Style information that cascades across multiple levels:
/// 1. Styles for the given entity
/// 2. Styles for all entities in a specific view
/// 3. Styles for all entities in a collection of views
/// 4. Styles for all entities in the XMILE file
/// 5. Default XMILE-defined styles
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Style {
    /// Global style attributes that apply to all objects
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub border_width: Option<BorderWidth>,
    pub border_color: Option<Color>,
    pub border_style: Option<BorderStyle>,
    pub font_family: Option<String>,
    pub font_style: Option<FontStyle>,
    pub font_weight: Option<FontWeight>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub font_color: Option<Color>,
    pub text_background: Option<Color>,
    pub font_size: Option<f64>,
    pub padding: Option<Padding>,
    /// Object-specific style overrides
    pub stock: Option<ObjectStyle>,
    pub flow: Option<ObjectStyle>,
    pub aux: Option<ObjectStyle>,
    pub module: Option<ObjectStyle>,
    pub group: Option<ObjectStyle>,
    pub connector: Option<ObjectStyle>,
    pub alias: Option<ObjectStyle>,
    pub slider: Option<ObjectStyle>,
    pub knob: Option<ObjectStyle>,
    pub switch: Option<ObjectStyle>,
    pub options: Option<ObjectStyle>,
    pub numeric_input: Option<ObjectStyle>,
    pub list_input: Option<ObjectStyle>,
    pub graphical_input: Option<ObjectStyle>,
    pub numeric_display: Option<ObjectStyle>,
    pub lamp: Option<ObjectStyle>,
    pub gauge: Option<ObjectStyle>,
    pub graph: Option<ObjectStyle>,
    pub table: Option<ObjectStyle>,
    pub text_box: Option<ObjectStyle>,
    pub graphics_frame: Option<ObjectStyle>,
    pub button: Option<ObjectStyle>,
}

/// Style attributes for a specific object type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectStyle {
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub border_width: Option<BorderWidth>,
    pub border_color: Option<Color>,
    pub border_style: Option<BorderStyle>,
    pub font_family: Option<String>,
    pub font_style: Option<FontStyle>,
    pub font_weight: Option<FontWeight>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub font_color: Option<Color>,
    pub text_background: Option<Color>,
    pub font_size: Option<f64>,
    pub padding: Option<Padding>,
}

/// Padding specification supporting 1-4 values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Padding {
    pub top: f64,
    pub right: Option<f64>,
    pub bottom: Option<f64>,
    pub left: Option<f64>,
}

// All XMILE display objects provide attributes which describe their look and feel or style. Styles applied to visual XMILE objects are composed of attributes of the following core style objects plus any specific attributes available to that specific type of object.

// Border

//     border_width="thick | thin | <double>" –default=1px  –thick=3px  –thin=1px
//     border_color="<hex code> | predefined color*"
//     border_style="none | solid" –default=none

// Text Style

//     font_family="<string>"
//     font_style="normal | italic" –default=normal
//     font_weight="normal | bold" –default=normal
//     text_decoration="normal | underline" –default=normal
//     text_align="left | right | center"
//     vertical_text_align=”top | bottom | center"
//     font_color="<hex code> | predefined color*"
//     text_background="<hex code> | predefined color*"
//     font_size="<double>pt"
//     padding="<comma separated list of no more than 4 doubles and no fewer than 1 double>"**
//     any attributes of a Border object

// All visual XMILE objects allow control over the following style attributes:

//     color="<hex code> | predefined color*"
//     background="<hex code> | predefined color*"
//     z_index="<int>" –default=-1 (-1 is bottom-most, top-most is INT32_MAX ((1 << 31) – 1))
//     any attributes of a Text Style object

// * The list of predefined colors and their definitions appear in Section 5.2.2

// ** The specification for the padding attributes appears in Section 5.2.1

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StyleTag {
    Color(Color),
    Background(Color),
    ZIndex(i32),
    BorderWidth(BorderWidth),
    BorderColor(Color),
    BorderStyle(BorderStyle),
    FontFamily(String),
    FontStyle(FontStyle),
    FontWeight(FontWeight),
    TextDecoration(TextDecoration),
    TextAlign(TextAlign),
    VerticalTextAlign(VerticalTextAlign),
    FontColor(Color),
    TextBackground(Color),
    FontSize(f64),
    Padding {
        top: Option<f64>,
        right: Option<f64>,
        bottom: Option<f64>,
        left: Option<f64>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Color {
    Hex(String),
    Predefined(PredefinedColor),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredefinedColor {
    Aqua,
    Black,
    Blue,
    Fuchsia,
    Gray,
    Green,
    Lime,
    Maroon,
    Navy,
    Olive,
    Purple,
    Red,
    Silver,
    Teal,
    White,
    Yellow,
}

impl PredefinedColor {
    pub fn to_hex(&self) -> &str {
        match self {
            PredefinedColor::Aqua => "#00FFFF",
            PredefinedColor::Black => "#000000",
            PredefinedColor::Blue => "#0000FF",
            PredefinedColor::Fuchsia => "#FF00FF",
            PredefinedColor::Gray => "#808080",
            PredefinedColor::Green => "#008000",
            PredefinedColor::Lime => "#00FF00",
            PredefinedColor::Maroon => "#800000",
            PredefinedColor::Navy => "#000080",
            PredefinedColor::Olive => "#808000",
            PredefinedColor::Purple => "#800080",
            PredefinedColor::Red => "#FF0000",
            PredefinedColor::Silver => "#C0C0C0",
            PredefinedColor::Teal => "#008080",
            PredefinedColor::White => "#FFFFFF",
            PredefinedColor::Yellow => "#FFFF00",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BorderWidth {
    Thick,
    Thin,
    Px(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BorderStyle {
    None,
    Solid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontStyle {
    Normal,
    Italic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontWeight {
    Normal,
    Bold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextDecoration {
    Normal,
    Underline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerticalTextAlign {
    Top,
    Bottom,
    Center,
}
