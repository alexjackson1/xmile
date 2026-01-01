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
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@x")]
    pub x: Option<f64>, // May be aliased
    #[serde(rename = "@y")]
    pub y: Option<f64>, // May be aliased
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
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
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowObject {
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@x")]
    pub x: Option<f64>, // May be aliased
    #[serde(rename = "@y")]
    pub y: Option<f64>, // May be aliased
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@label_side")]
    pub label_side: Option<String>,
    #[serde(rename = "@label_angle")]
    pub label_angle: Option<f64>,
    #[serde(rename = "pts")]
    pub pts: Vec<Point>,
}

// The <aux> tag in the context of a <view> tag is used to describe the appearance of an XMILE aux equation object.  Support is REQUIRED for any implementation supporting views.  An example tag is shown below:
// <aux name=”water flow rate” x=”50” y=”100” width=”45” height=”35” label_side=”top” color=”blue” background=”white” z_index=”1” font_family=”Arial” font_size=”9pt” font_weight=”bold” font_style=”italic” text_decoration=”underline” text_align=”center” vertical_text_align=”center” text_padding=”2px” font_color=”blue” text_border_color=”black” text_border_width=”1px” text_border_style=”solid”/>
// Descriptions of all the display attributes of an aux can be found in Section 6.1.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuxObject {
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@x")]
    pub x: Option<f64>, // May be aliased
    #[serde(rename = "@y")]
    pub y: Option<f64>, // May be aliased
    #[serde(rename = "@width")]
    pub width: Option<f64>,
    #[serde(rename = "@height")]
    pub height: Option<f64>,
    pub shape: Option<Shape>,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@label_side")]
    pub label_side: Option<String>,
    #[serde(rename = "@label_angle")]
    pub label_angle: Option<f64>,
}

// The <module> tag in the context of a <view> tag is used to describe the appearance of an XMILE module equation object.  Support is OPTIONAL for any implementation supporting views and modules.   An example tag is shown below:
// <module name=”Important_Module” x=”50” y=”100” width=”45” height=”35” label_side=”top” color=”blue” background=”white” z_index=”1” font_family=”Arial” font_size=”9pt” font_weight=”bold” font_style=”italic” text_decoration=”underline” text_align=”center” vertical_text_align=”center” text_padding=”2px” font_color=”blue” text_border_color=”black” text_border_width=”1px” text_border_style=”solid”/>
// Descriptions of all the display attributes of a module can be found in Section 6.1.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleObject {
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    pub shape: Option<Shape>,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@label_side")]
    pub label_side: Option<String>,
    #[serde(rename = "@label_angle")]
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
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@locked")]
    pub locked: bool,
    #[serde(rename = "item", default)]
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

/// Helper struct for deserializing alias tags within from/to
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AliasTag {
    #[serde(rename = "@uid")]
    pub uid: Uid,
}

/// A pointer to a model entity, either by alias or by name
#[derive(Debug, Clone, PartialEq)]
pub enum Pointer {
    Alias(Uid),
    Name(String),
}

// Custom deserialization for Pointer to handle both <alias uid="..."/> and text content
impl<'de> serde::Deserialize<'de> for Pointer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use std::fmt;

        struct PointerVisitor;

        impl<'de> Visitor<'de> for PointerVisitor {
            type Value = Pointer;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a pointer (alias tag or text content)")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                // Try to deserialize as an alias tag
                if let Ok(Some((key, tag))) = map.next_entry::<String, AliasTag>() {
                    if key == "alias" {
                        return Ok(Pointer::Alias(tag.uid));
                    }
                }
                Err(de::Error::custom("Expected alias tag or text content"))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Text content means it's a name
                Ok(Pointer::Name(v.to_string()))
            }
        }

        // Try to deserialize as a map first (for alias), then as string (for name)
        deserializer.deserialize_any(PointerVisitor)
    }
}

impl serde::Serialize for Pointer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Pointer::Alias(uid) => {
                use serde::ser::SerializeStruct;
                let mut state = serializer.serialize_struct("alias", 1)?;
                state.serialize_field("@uid", &uid.value)?;
                state.end()
            }
            Pointer::Name(name) => serializer.serialize_str(name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectorObject {
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@angle")]
    pub angle: f64,
    #[serde(rename = "@line_style")]
    pub line_style: Option<LineStyle>,
    #[serde(rename = "@delay_mark")]
    pub delay_mark: bool,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@polarity")]
    pub polarity: Option<Polarity>,
    #[serde(rename = "from")]
    pub from: Pointer,
    #[serde(rename = "to")]
    pub to: Pointer,
    #[serde(rename = "pts")]
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
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "of")]
    pub of: String,
    pub shape: Option<Shape>,
    // Additional properties to match the aliased object (optional overrides)
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@label_side")]
    pub label_side: Option<String>,
    #[serde(rename = "@label_angle")]
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
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    #[serde(rename = "@visible_index")]
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

/// Helper struct for deserializing entity tags
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct EntityTag {
    #[serde(rename = "@name")]
    name: String,
}

/// Helper struct for deserializing reset_to tags
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ResetToTag {
    #[serde(rename = "@after")]
    after: String,
    #[serde(rename = "#text")]
    value: f64,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct RawSliderObject {
    #[serde(rename = "@uid")]
    uid: Uid,
    #[serde(rename = "@x")]
    x: f64,
    #[serde(rename = "@y")]
    y: f64,
    #[serde(rename = "@width")]
    width: f64,
    #[serde(rename = "@height")]
    height: f64,
    #[serde(rename = "@color")]
    color: Option<Color>,
    #[serde(rename = "@background")]
    background: Option<Color>,
    #[serde(rename = "@z_index")]
    z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    font_family: Option<String>,
    #[serde(rename = "@font_size")]
    font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    text_border_style: Option<BorderStyle>,
    #[serde(rename = "@min")]
    min: f64,
    #[serde(rename = "@max")]
    max: f64,
    #[serde(rename = "@show_name", default = "default_true")]
    show_name: bool,
    #[serde(rename = "@show_number", default = "default_true")]
    show_number: bool,
    #[serde(rename = "@show_min_max", default = "default_true")]
    show_min_max: bool,
    #[serde(rename = "entity")]
    entity: Option<EntityTag>,
    #[serde(rename = "reset_to")]
    reset_to: Option<ResetToTag>,
}

impl From<RawSliderObject> for SliderObject {
    fn from(raw: RawSliderObject) -> Self {
        SliderObject {
            uid: raw.uid,
            x: raw.x,
            y: raw.y,
            width: raw.width,
            height: raw.height,
            color: raw.color,
            background: raw.background,
            z_index: raw.z_index,
            font_family: raw.font_family,
            font_size: raw.font_size,
            font_weight: raw.font_weight,
            font_style: raw.font_style,
            text_decoration: raw.text_decoration,
            text_align: raw.text_align,
            text_background: raw.text_background,
            vertical_text_align: raw.vertical_text_align,
            text_padding: raw.text_padding,
            font_color: raw.font_color,
            text_border_color: raw.text_border_color,
            text_border_width: raw.text_border_width,
            text_border_style: raw.text_border_style,
            entity_name: raw.entity.map(|e| e.name).unwrap_or_default(),
            min: raw.min,
            max: raw.max,
            show_name: raw.show_name,
            show_number: raw.show_number,
            show_min_max: raw.show_min_max,
            reset_to: raw.reset_to.map(|r| (r.value, r.after)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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

impl<'de> serde::Deserialize<'de> for SliderObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = RawSliderObject::deserialize(deserializer)?;
        Ok(SliderObject::from(raw))
    }
}

impl serde::Serialize for SliderObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("slider", 25)?;
        
        state.serialize_field("@uid", &self.uid.value)?;
        state.serialize_field("@x", &self.x)?;
        state.serialize_field("@y", &self.y)?;
        state.serialize_field("@width", &self.width)?;
        state.serialize_field("@height", &self.height)?;
        if let Some(color) = &self.color {
            state.serialize_field("@color", color)?;
        }
        if let Some(background) = &self.background {
            state.serialize_field("@background", background)?;
        }
        if let Some(z_index) = &self.z_index {
            state.serialize_field("@z_index", z_index)?;
        }
        if let Some(font_family) = &self.font_family {
            state.serialize_field("@font_family", font_family)?;
        }
        if let Some(font_size) = &self.font_size {
            state.serialize_field("@font_size", font_size)?;
        }
        if let Some(font_weight) = &self.font_weight {
            state.serialize_field("@font_weight", font_weight)?;
        }
        if let Some(font_style) = &self.font_style {
            state.serialize_field("@font_style", font_style)?;
        }
        if let Some(text_decoration) = &self.text_decoration {
            state.serialize_field("@text_decoration", text_decoration)?;
        }
        if let Some(text_align) = &self.text_align {
            state.serialize_field("@text_align", text_align)?;
        }
        if let Some(text_background) = &self.text_background {
            state.serialize_field("@text_background", text_background)?;
        }
        if let Some(vertical_text_align) = &self.vertical_text_align {
            state.serialize_field("@vertical_text_align", vertical_text_align)?;
        }
        if let Some(text_padding) = &self.text_padding {
            state.serialize_field("@text_padding", text_padding)?;
        }
        if let Some(font_color) = &self.font_color {
            state.serialize_field("@font_color", font_color)?;
        }
        if let Some(text_border_color) = &self.text_border_color {
            state.serialize_field("@text_border_color", text_border_color)?;
        }
        if let Some(text_border_width) = &self.text_border_width {
            state.serialize_field("@text_border_width", text_border_width)?;
        }
        if let Some(text_border_style) = &self.text_border_style {
            state.serialize_field("@text_border_style", text_border_style)?;
        }
        state.serialize_field("@min", &self.min)?;
        state.serialize_field("@max", &self.max)?;
        if !self.show_name {
            state.serialize_field("@show_name", &self.show_name)?;
        }
        if !self.show_number {
            state.serialize_field("@show_number", &self.show_number)?;
        }
        if !self.show_min_max {
            state.serialize_field("@show_min_max", &self.show_min_max)?;
        }
        
        // Serialize entity tag
        state.serialize_field("entity", &EntityTag { name: self.entity_name.clone() })?;
        
        // Serialize reset_to if present
        if let Some((value, after)) = &self.reset_to {
            state.serialize_field("reset_to", &ResetToTag { after: after.clone(), value: *value })?;
        }
        
        state.end()
    }
}

// Knobs are the same as sliders but for stocks
pub type KnobObject = SliderObject;

// Switches and Radio Buttons (Option Groups)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchObject {
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@label_side")]
    pub label_side: Option<String>,
    #[serde(rename = "@label_angle")]
    pub label_angle: Option<f64>,
    #[serde(rename = "@show_name")]
    pub show_name: bool,
    #[serde(rename = "@switch_style")]
    pub switch_style: SwitchStyle,
    #[serde(rename = "@clicking_sound")]
    pub clicking_sound: bool,
    #[serde(rename = "@entity_name")]
    pub entity_name: Option<String>,
    #[serde(rename = "@entity_value")]
    pub entity_value: Option<f64>,
    #[serde(rename = "@group_name")]
    pub group_name: Option<String>,
    #[serde(rename = "@module_name")]
    pub module_name: Option<String>,
    pub reset_to: Option<(f64, String)>, // (value, after) - handled via custom deserialization if needed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchStyle {
    Toggle,
    PushButton,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionsObject {
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@layout")]
    pub layout: OptionsLayout,
    #[serde(rename = "@horizontal_spacing")]
    pub horizontal_spacing: f64,
    #[serde(rename = "@vertical_spacing")]
    pub vertical_spacing: f64,
    #[serde(rename = "entity", default)]
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
    #[serde(rename = "@name")]
    pub entity_name: String,
    #[serde(rename = "@index")]
    pub index: Option<String>,
    #[serde(rename = "#text")]
    pub value: f64,
}

// Numeric Inputs and List Input Devices
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumericInputObject {
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@entity_name")]
    pub entity_name: String,
    #[serde(rename = "@entity_index")]
    pub entity_index: Option<String>,
    #[serde(rename = "@min")]
    pub min: f64,
    #[serde(rename = "@max")]
    pub max: f64,
    #[serde(rename = "@precision")]
    pub precision: Option<f64>,
    #[serde(rename = "@value")]
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListInputObject {
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@column_width")]
    pub column_width: f64,
    #[serde(rename = "numeric_input", default)]
    pub numeric_inputs: Vec<NumericInputObject>,
}

// Graphical Inputs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphicalInputObject {
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@entity_name")]
    pub entity_name: String,
    #[serde(rename = "gf")]
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
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@entity_name")]
    pub entity_name: String,
    #[serde(rename = "@show_name")]
    pub show_name: bool,
    #[serde(rename = "@retain_ending_value")]
    pub retain_ending_value: bool,
    #[serde(rename = "@precision")]
    pub precision: Option<f64>,
    #[serde(rename = "@delimit_000s")]
    pub delimit_000s: bool,
}

// Lamps and Gauges
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LampObject {
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@entity_name")]
    pub entity_name: String,
    #[serde(rename = "@show_name")]
    pub show_name: bool,
    #[serde(rename = "@retain_ending_value")]
    pub retain_ending_value: bool,
    #[serde(rename = "@flash_on_panic")]
    pub flash_on_panic: bool,
    #[serde(rename = "zone", default)]
    pub zones: Vec<Zone>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaugeObject {
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@entity_name")]
    pub entity_name: String,
    #[serde(rename = "@show_name")]
    pub show_name: bool,
    #[serde(rename = "@show_number")]
    pub show_number: bool,
    #[serde(rename = "@retain_ending_value")]
    pub retain_ending_value: bool,
    #[serde(rename = "zone", default)]
    pub zones: Vec<Zone>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    #[serde(rename = "@type")]
    pub zone_type: ZoneType,
    #[serde(rename = "@color")]
    pub color: Color,
    #[serde(rename = "@min")]
    pub min: f64,
    #[serde(rename = "@max")]
    pub max: f64,
    #[serde(rename = "@sound")]
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
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@graph_type")]
    pub graph_type: GraphType,
    #[serde(rename = "@title")]
    pub title: Option<String>,
    #[serde(rename = "@doc")]
    pub doc: Option<String>,
    #[serde(rename = "@show_grid")]
    pub show_grid: bool,
    #[serde(rename = "@num_x_grid_lines")]
    pub num_x_grid_lines: u32,
    #[serde(rename = "@num_y_grid_lines")]
    pub num_y_grid_lines: u32,
    #[serde(rename = "@num_x_labels")]
    pub num_x_labels: u32,
    #[serde(rename = "@num_y_labels")]
    pub num_y_labels: u32,
    #[serde(rename = "@x_axis_title")]
    pub x_axis_title: Option<String>,
    #[serde(rename = "@right_axis_title")]
    pub right_axis_title: Option<String>,
    #[serde(rename = "@right_axis_auto_scale")]
    pub right_axis_auto_scale: bool,
    #[serde(rename = "@right_axis_multi_scale")]
    pub right_axis_multi_scale: bool,
    #[serde(rename = "@left_axis_title")]
    pub left_axis_title: Option<String>,
    #[serde(rename = "@left_axis_auto_scale")]
    pub left_axis_auto_scale: bool,
    #[serde(rename = "@left_axis_multi_scale")]
    pub left_axis_multi_scale: bool,
    #[serde(rename = "@plot_numbers")]
    pub plot_numbers: bool,
    #[serde(rename = "@comparative")]
    pub comparative: bool,
    #[serde(rename = "@from")]
    pub from: Option<f64>,
    #[serde(rename = "@to")]
    pub to: Option<f64>,
    #[serde(rename = "plot", default)]
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
    #[serde(rename = "@index")]
    pub index: u32,
    #[serde(rename = "@pen_width")]
    pub pen_width: f64,
    #[serde(rename = "@pen_style")]
    pub pen_style: PenStyle,
    #[serde(rename = "@show_y_axis")]
    pub show_y_axis: bool,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "@right_axis")]
    pub right_axis: bool,
    #[serde(rename = "@entity_name")]
    pub entity_name: String,
    #[serde(rename = "@precision")]
    pub precision: Option<f64>,
    pub scale: Option<PlotScale>,
    #[serde(rename = "@color")]
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
    #[serde(rename = "@min")]
    pub min: f64,
    #[serde(rename = "@max")]
    pub max: f64,
}

// Tables
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableObject {
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@title")]
    pub title: Option<String>,
    #[serde(rename = "@doc")]
    pub doc: Option<String>,
    #[serde(rename = "@orientation")]
    pub orientation: TableOrientation,
    #[serde(rename = "@column_width")]
    pub column_width: f64,
    #[serde(rename = "@blank_column_width")]
    pub blank_column_width: Option<f64>,
    #[serde(rename = "@interval")]
    pub interval: String,
    #[serde(rename = "@report_balances")]
    pub report_balances: ReportBalances,
    #[serde(rename = "@report_flows")]
    pub report_flows: ReportFlows,
    #[serde(rename = "@comparative")]
    pub comparative: bool,
    #[serde(rename = "@wrap_text")]
    pub wrap_text: bool,
    #[serde(rename = "item", default)]
    pub items: Vec<TableItem>,
    // Header style attributes (prefixed with "header_")
    #[serde(rename = "@header_font_family")]
    pub header_font_family: Option<String>,
    #[serde(rename = "@header_font_size")]
    pub header_font_size: Option<f64>,
    #[serde(rename = "@header_font_weight")]
    pub header_font_weight: Option<FontWeight>,
    #[serde(rename = "@header_font_style")]
    pub header_font_style: Option<FontStyle>,
    #[serde(rename = "@header_text_decoration")]
    pub header_text_decoration: Option<TextDecoration>,
    #[serde(rename = "@header_text_align")]
    pub header_text_align: Option<TextAlign>,
    #[serde(rename = "@header_vertical_text_align")]
    pub header_vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@header_text_background")]
    pub header_text_background: Option<Color>,
    #[serde(rename = "@header_text_padding")]
    pub header_text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@header_font_color")]
    pub header_font_color: Option<Color>,
    #[serde(rename = "@header_text_border_color")]
    pub header_text_border_color: Option<Color>,
    #[serde(rename = "@header_text_border_width")]
    pub header_text_border_width: Option<BorderWidth>,
    #[serde(rename = "@header_text_border_style")]
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
    #[serde(rename = "@type")]
    pub item_type: TableItemType,
    #[serde(rename = "@entity_name")]
    pub entity_name: Option<String>,
    #[serde(rename = "@precision")]
    pub precision: Option<f64>,
    #[serde(rename = "@delimit_000s")]
    pub delimit_000s: bool,
    #[serde(rename = "@column_width")]
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
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@appearance")]
    pub appearance: TextBoxAppearance,
    #[serde(rename = "#text")]
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
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@border_color")]
    pub border_color: Option<Color>,
    #[serde(rename = "@border_style")]
    pub border_style: Option<BorderStyle>,
    #[serde(rename = "@border_width")]
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
    #[serde(rename = "@uid")]
    pub uid: Uid,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
    #[serde(rename = "@background")]
    pub background: Option<Color>,
    #[serde(rename = "@z_index")]
    pub z_index: Option<i32>,
    #[serde(rename = "@font_family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font_size")]
    pub font_size: Option<f64>,
    #[serde(rename = "@font_weight")]
    pub font_weight: Option<FontWeight>,
    #[serde(rename = "@font_style")]
    pub font_style: Option<FontStyle>,
    #[serde(rename = "@text_decoration")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(rename = "@text_align")]
    pub text_align: Option<TextAlign>,
    #[serde(rename = "@text_background")]
    pub text_background: Option<Color>,
    #[serde(rename = "@vertical_text_align")]
    pub vertical_text_align: Option<VerticalTextAlign>,
    #[serde(rename = "@text_padding")]
    pub text_padding: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)>,
    #[serde(rename = "@font_color")]
    pub font_color: Option<Color>,
    #[serde(rename = "@text_border_color")]
    pub text_border_color: Option<Color>,
    #[serde(rename = "@text_border_width")]
    pub text_border_width: Option<BorderWidth>,
    #[serde(rename = "@text_border_style")]
    pub text_border_style: Option<BorderStyle>,
    #[serde(rename = "@appearance")]
    pub appearance: ButtonAppearance,
    #[serde(rename = "@style")]
    pub style: ButtonStyle,
    #[serde(rename = "@label")]
    pub label: Option<String>,
    pub image: Option<ImageContent>,
    #[serde(rename = "@clicking_sound")]
    pub clicking_sound: bool,
    #[serde(rename = "@sound")]
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
