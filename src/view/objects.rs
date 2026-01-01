// Attributes common to objects described in Sections 6.1.1 through 6.1.7:

//     x,y – defined in Section 5.1.2

// ·         color - defined in Section 5.2 and specified in Section 5.2.2

//     background – defined in Section 5.2 and specified in Section 5.2.2
//     z_index - defined in Section 5.2
//     font_family - defined in Section 5.2
//     font_size - defined in Section 5.2
//     font_weight - defined in Section 5.2
//     font_style - defined in Section 5.2
//     text_decoration - defined in Section 5.2
//     text_align - defined in Section 5.2
//     text_background - defined in Section 5.2
//     vertical_text_align – defined in Section 5.2

// ·         text_padding - same as padding as defined in Section 5.2

// ·         font_color - defined in Section 5.2

// ·         text_border_color - same as border_color as defined in Section 5.2

// ·         text_border_width - same as border_width as defined in Section 5.2

//     text_border_style – same as border_style as defined in Section 5.2

// Additional attributes common to objects described in Sections 6.1.1 through 6.1.5:

//     name – described in Section 5.1.1, but in short it is the local name of the model entity represented by this tag.

// Additional attributes common to objects described in Sections 6.1.1 through 6.1.4:

//     width, height – defined in Section 5.1.2

// ·         label_side – This is the side of the symbol that the nameplate appears on.  Valid values are top|left|center|bottom|right

// ·         label_angle – This is the precise angle (in degrees where 0 is at 3 o’clock, increasing counter-clockwise) of the nameplate on the widget.  This is always specified in conjunction with label_side.

use serde::{Deserialize, Serialize};

use crate::Uid;

use super::style::{
    BorderStyle, BorderWidth, Color, FontStyle, FontWeight, TextAlign, TextDecoration,
    VerticalTextAlign,
};

/// Shape tags allow stock, auxiliary, module, or alias objects to be represented
/// using a different symbol than the default.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Shape {
    Rectangle {
        width: f64,
        height: f64,
        corner_radius: Option<f64>,
    },
    Circle {
        radius: f64,
    },
    NameOnly {
        width: Option<f64>,
        height: Option<f64>,
    },
}

// The <stock> tag in the context of a <view> tag is used to describe the appearance of an XMILE stock equation object.  Support is REQUIRED for any implementation supporting views.  An example tag is shown below:
// <stock name=”Bathtub” x=”50” y=”100” width=”45” height=”35” label_side=”top” color=”blue” background=”white” z_index=”1” font_family=”Arial” font_size=”9pt” font_weight=”bold” font_style=”italic” text_decoration=”underline” text_align=”center” vertical_text_align=”center” text_padding=”2px” font_color=”blue” text_border_color=”black” text_border_width=”1px” text_border_style=”solid”/>
// Descriptions of all the display attributes of a stock can be found in Section 6.1.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockObject {
    pub uid: Uid,
    pub name: String,
    pub x: Option<f64>, // May be aliased
    pub y: Option<f64>, // May be aliased
    pub width: f64,
    pub height: f64,
    pub shape: Option<Shape>,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub label_side: Option<String>,
    pub label_angle: Option<f64>,
}

// The <flow> tag in the context of a <view> tag is used to describe the appearance of an XMILE flow equation object. Support is REQUIRED for any implementation supporting views.  An example tag is shown below:
// <flow name=”faucet” x=”50” y=”100” width=”18” height=”18” label_side=”top” color=”blue” background=”white” z_index=”1” font_family=”Arial” font_size=”9pt” font_weight=”bold” font_style=”italic” text_decoration=”underline” text_align=”center” vertical_text_align=”center” text_padding=”2px” font_color=”blue” text_border_color=”black” text_border_width=”1px” text_border_style=”solid”>
//       <pts>
//             <pt x=”0” y=”100”/>
//             <pt x=”150 y=”100”/>
//       </pts>
// </flow>
//     pts REQUIRED – These are the anchor points for the flow specified in model coordinates.  Flows can have any arbitrary number of points, but those points MUST form right angles.
// Descriptions of all other display attributes of a flow can be found in Section 6.1.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowObject {
    pub uid: Uid,
    pub name: String,
    pub x: Option<f64>, // May be aliased
    pub y: Option<f64>, // May be aliased
    pub width: f64,
    pub height: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub label_side: Option<String>,
    pub label_angle: Option<f64>,
    pub pts: Vec<Point>,
}

// The <aux> tag in the context of a <view> tag is used to describe the appearance of an XMILE aux equation object.  Support is REQUIRED for any implementation supporting views.  An example tag is shown below:
// <aux name=”water flow rate” x=”50” y=”100” width=”45” height=”35” label_side=”top” color=”blue” background=”white” z_index=”1” font_family=”Arial” font_size=”9pt” font_weight=”bold” font_style=”italic” text_decoration=”underline” text_align=”center” vertical_text_align=”center” text_padding=”2px” font_color=”blue” text_border_color=”black” text_border_width=”1px” text_border_style=”solid”/>
// Descriptions of all the display attributes of an aux can be found in Section 6.1.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuxObject {
    pub uid: Uid,
    pub name: String,
    pub x: Option<f64>, // May be aliased
    pub y: Option<f64>, // May be aliased
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub shape: Option<Shape>,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub label_side: Option<String>,
    pub label_angle: Option<f64>,
}

// The <module> tag in the context of a <view> tag is used to describe the appearance of an XMILE module equation object.  Support is OPTIONAL for any implementation supporting views and modules.   An example tag is shown below:
// <module name=”Important_Module” x=”50” y=”100” width=”45” height=”35” label_side=”top” color=”blue” background=”white” z_index=”1” font_family=”Arial” font_size=”9pt” font_weight=”bold” font_style=”italic” text_decoration=”underline” text_align=”center” vertical_text_align=”center” text_padding=”2px” font_color=”blue” text_border_color=”black” text_border_width=”1px” text_border_style=”solid”/>
// Descriptions of all the display attributes of a module can be found in Section 6.1.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleObject {
    pub uid: Uid,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub shape: Option<Shape>,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub label_side: Option<String>,
    pub label_angle: Option<f64>,
}

// The <group> tag in the context of a <view> tag is used to describe the appearance of an XMILE group object. Support in the view is RECOMMENDED. A <group> display object differs from all other display objects used to represent model section objects in that there is a one-to-one relationship between group objects in the model section and group objects in the display section. This means that you can only have one <group> tag in the <views> tag that represents the <group> tag in the <variables> tag. All XMILE model objects which appear in the group within the model section are implicitly contained within the group object in the display section, but groups can also contain objects which are not present within the model section. Those objects are included within the group using an <item> tag. An example is shown below:
// <group name=”Major_Group” x=”50” y=”100” color=”blue” background=”white” z_index=”1” font_family=”Arial” font_size=”9pt” font_weight=”bold” font_style=”italic” text_decoration=”underline” text_align=”center” vertical_text_align=”center” text_padding=”2px” font_color=”blue” text_border_color=”black” text_border_width=”1px” text_border_style=”solid” locked=”true”>
//       <item uid=”1”/>
// </group>
//     <item uid=”*”/> OPTIONAL - A tag representing an un-named object which is present inside of this group.  The * is a UID for an object in the group’s model.  These are objects like aliases, tables, graphs, buttons etc.  Note: <item> tags representing connector objects are NOT REQUIRED to be present in the tag to be considered a part of the group.  Connector objects are automatically considered a part of the group if one end of the connector is contained within the group.
//     locked="…" with true/false (default: false) REQUIRED – When a group is locked, all entities in that group move with the group.  When not locked, moving the group adjusts the items inside of the group (both model and display section objects).
// Descriptions of all other display attributes of a group can be found in Section 6.1.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupObject {
    pub uid: Uid,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub locked: bool,
    pub items: Vec<Uid>,
}

// The <connector> tag is used to describe the visual appearance of the relationships between XMILE model objects.  Support is REQUIRED for any implementation supporting views.  A connector is an arrow which only appears between two display objects.  An example tag is shown below:
// <connector uid=”1” x=”50” y=”100” angle=”32” line_style=”solid” delay_mark=”false” color=”blue” background=”white” z_index=”1” font_family=”Arial” font_size=”9pt” font_weight=”bold” font_style=”italic” text_decoration=”underline” text_align=”center” vertical_text_align=”center” text_padding=”2px” font_color=”blue” text_border_color=”black” text_border_width=”1px” text_border_style=”solid” polarity=””>
//       <from>
//             <alias uid=”2”/>
//       </from>
//       <to>faucet</to>
// <pts>
//       <pt x=”50” y=”50”/>
//       <pt x=”100” y=”100”/>
//       <pt x=”150” y=”75”/>
// </pts>
// </connector>
//     polarity – “+ | - | none” – default option = “none”  OPTIONAL - The polarity is drawn as a symbol at the end of a connector arrowhead representing the type of the relationship between the variables connected by the connector.
//     from REQUIRED – The name of (or an alias tag pointing to) the model entity from which this connector starts.  The entity needs to be in the same model as the connector.
//     to REQUIRED – The name of the model entity to which this connector ends, the entity needs to be in the same model as the connector.
//     pts REQUIRED – These are the anchor points for the connector specified in model coordinates for when there are more than two points (the start and end point).  Connectors MAY  have any number of points (greater than two) and those points are RECOMMENDED to be connected using an arc when there are two points and Bezier curves if three or more points are present.  If a vendor does not support multipoint connectors, this information is ignored and the angle attribute is used instead to calculate the connector start and end points.
//     angle REQUIRED – The angle in degrees of the takeoff point from the center of the start object.  0 is 3 o’clock and angles increase counter-clockwise.
//     line_style “solid|dashed|vendor specific” – default option=”solid” OPTIONAL - Describes whether the connector is dashed or not.  For the vendor specific options, SVG line-style types are suggested.
//     delay_mark ="…" with true/false (default: false) OPTIONAL - Describes whether or not this connector is marked with a symbol signifying a delay.
// Descriptions of all other display attributes of a connector can be found in Section 6.1.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Polarity {
    Positive,
    Negative,
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LineStyle {
    Solid,
    Dashed,
    VendorSpecific(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pointer {
    Alias(Uid),
    Name(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectorObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub angle: f64,
    pub line_style: Option<LineStyle>,
    pub delay_mark: bool,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub polarity: Option<Polarity>,
    pub from: Pointer,
    pub to: Pointer,
    pub pts: Vec<Point>,
}

// The <alias> tag is used to describe the visual appearance of an alias of an XMILE model object.  Support is REQUIRED for any implementation supporting views.  An alias is a symbol representing a “portal” to the display of another XMILE model object in the same view.  Aliases are only valid for stocks, flows, and auxiliaries.  It is RECOMMENDED for aliases to take on all the same styles as the object they represent with only the differences being written to the <alias> tag.  Aliases MAY have connectors leaving them but MAY NOT have connectors pointing to them.  An example tag is shown below:
// <alias uid=”1” x=”50” y=”100”>
//       <of>faucet</of>
// </alias>
//     uid REQUIRED – defined in Section 5.1.3
//     x,y REQUIRED – defined in Section 5.1.2
//     of REQUIRED – The name of the model entity which this alias represents.  The model entity must be in the same model as the alias.
// The other attributes of an alias are the same as the object to which the alias refers.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AliasObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub of: String,
    pub shape: Option<Shape>,
    // Additional properties to match the aliased object (optional overrides)
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub label_side: Option<String>,
    pub label_angle: Option<f64>,
}

// A stacked container is used to allow XMILE display objects to be stacked on top of one another in flipbook form. Support for this tag is OPTIONAL. This allows model creators to create pages of tables or graphs.  Any display object may be placed within a stacked container, but typical objects are graphs and tables.  An example tag is shown below:

// <stacked_container x="92" y="114" height="282" width="492" uid="0" visible_index=”0”>

// </stacked_container>

//     visible_index REQUIRED – Integer 0 based index of which content object to display.

// Descriptions of all other display attributes of a stacked container can be found in Section 6.1.

// Stacked container objects are REQUIRED to have ONLY the five properties shown above.  Any borders, backgrounds etc. are supplied by their contents.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StackedContainerObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub visible_index: usize,
}

// Sliders and knobs are used to change the value of a variable in the model from the interface.  Support for these tags is OPTIONAL. Stocks can only be manipulated by knobs. Iin this case, knobs can only change the stock’s initial value, i.e., knobs attached to stocks MUST NOT be changed in the middle of a simulation run.  Sliders are defined with the <slider> tag and knobs are defined with the <knob> tag; they are otherwise the same.  An example slider tag is shown below:

// <slider x="172" y="114" color="black" width="197" height="43" min="7" max="9" background=”white” z_index=”1” font_family=”Arial” font_size=”9pt” font_weight=”bold” font_style=”italic” text_decoration=”underline” text_align=”center” vertical_text_align=”center” text_padding=”2px” font_color=”blue” text_border_color=”black” text_border_width=”1px” text_border_style=”solid”>

//       <entity name="Converter_1" />

// <reset_to after="one_time_unit">7</reset_to>

// </slider>

//     Show name: OPTIONAL  show_name="…" with true/false (default: true)
//     Show number:  OPTIONAL show_number="…" with true/false; when the number is visible it MUST be directly editable (default: true)
//     Show input range: OPTIONAL  show_min_max="…" with true/false (default: true)
//     Input range REQUIRED:  min="…" and max="…", overriding entity’s input range setting (default:  entity’s setting)
//     OPTIONAL reset (slider only):  <reset_to> with the value to reset the entity to; it has one attribute that define when to reset the entity’s value:  after="…" with either one_time_unit or one_dt.

// Descriptions of all other display attributes of a slider or knob can be found in Section 6.1.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SliderObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub entity_name: String,
    pub min: f64,
    pub max: f64,
    pub show_name: bool,
    pub show_number: bool,
    pub show_min_max: bool,
    pub reset_to: Option<(f64, String)>, // (value, after)
}

// Knobs are the same as sliders but for stocks
pub type KnobObject = SliderObject;

// Switches and Radio Buttons (Option Groups)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub label_side: Option<String>,
    pub label_angle: Option<f64>,
    pub show_name: bool,
    pub switch_style: SwitchStyle,
    pub clicking_sound: bool,
    pub entity_name: Option<String>,
    pub entity_value: Option<f64>,
    pub group_name: Option<String>,
    pub module_name: Option<String>,
    pub reset_to: Option<(f64, String)>, // (value, after)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchStyle {
    Toggle,
    PushButton,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionsObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub layout: OptionsLayout,
    pub horizontal_spacing: f64,
    pub vertical_spacing: f64,
    pub entities: Vec<OptionEntity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionsLayout {
    Vertical,
    Horizontal,
    Grid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionEntity {
    pub entity_name: String,
    pub index: Option<String>,
    pub value: f64,
}

// Numeric Inputs and List Input Devices
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumericInputObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub entity_name: String,
    pub entity_index: Option<String>,
    pub min: f64,
    pub max: f64,
    pub precision: Option<f64>,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListInputObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub name: String,
    pub column_width: f64,
    pub numeric_inputs: Vec<NumericInputObject>,
}

// Graphical Inputs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphicalInputObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub entity_name: String,
    pub graphical_function: Option<GraphicalFunctionData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphicalFunctionData {
    pub xscale_min: f64,
    pub xscale_max: f64,
    pub ypts: Vec<f64>,
}

// Numeric Displays
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumericDisplayObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub entity_name: String,
    pub show_name: bool,
    pub retain_ending_value: bool,
    pub precision: Option<f64>,
    pub delimit_000s: bool,
}

// Lamps and Gauges
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LampObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub entity_name: String,
    pub show_name: bool,
    pub retain_ending_value: bool,
    pub flash_on_panic: bool,
    pub zones: Vec<Zone>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaugeObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub entity_name: String,
    pub show_name: bool,
    pub show_number: bool,
    pub retain_ending_value: bool,
    pub zones: Vec<Zone>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub zone_type: ZoneType,
    pub color: Color,
    pub min: f64,
    pub max: f64,
    pub sound: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneType {
    Normal,
    Caution,
    Panic,
}

// Graphs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub graph_type: GraphType,
    pub title: Option<String>,
    pub doc: Option<String>,
    pub show_grid: bool,
    pub num_x_grid_lines: u32,
    pub num_y_grid_lines: u32,
    pub num_x_labels: u32,
    pub num_y_labels: u32,
    pub x_axis_title: Option<String>,
    pub right_axis_title: Option<String>,
    pub right_axis_auto_scale: bool,
    pub right_axis_multi_scale: bool,
    pub left_axis_title: Option<String>,
    pub left_axis_auto_scale: bool,
    pub left_axis_multi_scale: bool,
    pub plot_numbers: bool,
    pub comparative: bool,
    pub from: Option<f64>,
    pub to: Option<f64>,
    pub plots: Vec<Plot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphType {
    TimeSeries,
    Scatter,
    Bar,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plot {
    pub index: u32,
    pub pen_width: f64,
    pub pen_style: PenStyle,
    pub show_y_axis: bool,
    pub title: String,
    pub right_axis: bool,
    pub entity_name: String,
    pub precision: Option<f64>,
    pub scale: Option<PlotScale>,
    pub color: Option<Color>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PenStyle {
    Solid,
    Dotted,
    Dashed,
    DotDashed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotScale {
    pub min: f64,
    pub max: f64,
}

// Tables
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub title: Option<String>,
    pub doc: Option<String>,
    pub orientation: TableOrientation,
    pub column_width: f64,
    pub blank_column_width: Option<f64>,
    pub interval: String,
    pub report_balances: ReportBalances,
    pub report_flows: ReportFlows,
    pub comparative: bool,
    pub wrap_text: bool,
    pub items: Vec<TableItem>,
    // Header style attributes (prefixed with "header_")
    pub header_font_family: Option<String>,
    pub header_font_size: Option<f64>,
    pub header_font_weight: Option<FontWeight>,
    pub header_font_style: Option<FontStyle>,
    pub header_text_decoration: Option<TextDecoration>,
    pub header_text_align: Option<TextAlign>,
    pub header_vertical_text_align: Option<VerticalTextAlign>,
    pub header_text_background: Option<Color>,
    pub header_text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub header_font_color: Option<Color>,
    pub header_text_border_color: Option<Color>,
    pub header_text_border_width: Option<BorderWidth>,
    pub header_text_border_style: Option<BorderStyle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportBalances {
    Beginning,
    Ending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFlows {
    Instantaneous,
    Summed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableItem {
    pub item_type: TableItemType,
    pub entity_name: Option<String>,
    pub precision: Option<f64>,
    pub delimit_000s: bool,
    pub column_width: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableItemType {
    Time,
    Variable,
    Blank,
}

// Text Boxes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextBoxObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub appearance: TextBoxAppearance,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextBoxAppearance {
    Transparent,
    Normal,
}

// Graphics Frames
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphicsFrameObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub border_color: Option<Color>,
    pub border_style: Option<BorderStyle>,
    pub border_width: Option<BorderWidth>,
    pub content: GraphicsFrameContent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GraphicsFrameContent {
    Image(ImageContent),
    Video(VideoContent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageContent {
    pub size_to_parent: bool,
    pub width: f64,
    pub height: f64,
    pub resource: Option<String>,
    pub data: Option<String>, // base64 encoded data URI
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoContent {
    pub size_to_parent: bool,
    pub width: f64,
    pub height: f64,
    pub resource: String,
}

// Buttons
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonObject {
    pub uid: Uid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub z_index: Option<i32>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_background: Option<Color>,
    pub vertical_text_align: Option<VerticalTextAlign>,
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    pub font_color: Option<Color>,
    pub text_border_color: Option<Color>,
    pub text_border_width: Option<BorderWidth>,
    pub text_border_style: Option<BorderStyle>,
    pub appearance: ButtonAppearance,
    pub style: ButtonStyle,
    pub label: Option<String>,
    pub image: Option<ImageContent>,
    pub clicking_sound: bool,
    pub sound: Option<String>,
    pub popup: Option<PopupContent>,
    pub link: Option<Link>,
    pub menu_action: Option<MenuAction>,
    pub switch_action: Option<SwitchAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonAppearance {
    Opaque,
    Transparent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonStyle {
    Square,
    Rounded,
    Capsule,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PopupContent {
    TextBox(TextBoxObject),
    Image(ImageContent),
    Video(VideoContent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
    pub effect: Option<LinkEffect>,
    pub to_black: bool,
    pub target: LinkTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkEffect {
    Dissolve,
    Checkerboard,
    Bars,
    WipeLeft,
    WipeRight,
    WipeTop,
    WipeBottom,
    WipeClockwise,
    WipeCounterclockwise,
    IrisIn,
    IrisOut,
    DoorsClose,
    DoorsOpen,
    VenetianLeft,
    VenetianRight,
    VenetianTop,
    VenetianBottom,
    PushBottom,
    PushTop,
    PushLeft,
    PushRight,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LinkTarget {
    View { view_type: String, order: String },
    Page { view_type: String, order: String, page: String },
    NextPage,
    PreviousPage,
    HomePage,
    NextView,
    PreviousView,
    HomeView,
    BackPage,
    BackView,
    Url(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MenuAction {
    File(FileAction),
    Printing(PrintingAction),
    Simulation(SimulationAction),
    Restore(RestoreAction),
    Data(DataAction),
    Miscellaneous(MiscellaneousAction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileAction {
    Open,
    Close,
    Save,
    SaveAs,
    SaveAsImage,
    Revert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrintingAction {
    PrintSetup,
    Print,
    PrintScreen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimulationAction {
    Run,
    Pause,
    Resume,
    Stop,
    RunRestore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestoreAction {
    RestoreAll,
    RestoreSliders,
    RestoreKnobs,
    RestoreListInputs,
    RestoreGraphicalInputs,
    RestoreSwitches,
    RestoreNumericDisplays,
    RestoreGraphsTables,
    RestoreLampsGauges,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataAction {
    DataManager,
    SaveDataNow { run_name: String },
    ImportNow { resource: String, worksheet: Option<String>, all: bool },
    ExportNow { resource: String, worksheet: Option<String>, all: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiscellaneousAction {
    Exit,
    Find,
    RunSpecs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchAction {
    pub entity_name: Option<String>,
    pub group_name: Option<String>,
    pub module_name: Option<String>,
    pub value: f64,
}
