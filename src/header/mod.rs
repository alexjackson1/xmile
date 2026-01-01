// 2.2 Header Section
// The XML tag for the file header is <header>. The REQUIRED sub-tags are:
// ·         Vendor name:  <vendor> w/company name
// ·         Product name:  <product version="…" lang="…"> w/product name – the product version number is REQUIRED. The language code is optional (default: English) and describes the language used for variable names and comments. Language codes are described by ISO 639-1 unless the language is not there, in which case the ISO 639-2 code should be used (e.g., for Hawaiian).
// OPTIONAL sub-tags include:
// ·         XMILE options: <options> (defined below)
// ·         Model name:     <name> w/name
// ·         Model version:  <version> w/version information
// ·         Model caption:  <caption> w/caption
// ·         Picture of the model in JPG, GIF, TIF, or PNG format:  <image resource=””>. The resource attribute is optional and may specify a relative file path, an absolute file path, or an URL.  The picture data may also be embedded inside the <image> tag in Data URI format, using base64 encoding.
// ·         Author name:  <author> w/author name
// ·         Company name:  <affiliation> w/company name
// ·         Client name:  <client> w/client name
// ·         Copyright notice:  <copyright> w/copyright information
// ·         Contact information (e-mail, phone, mailing address, web site):
//             <contact> block w/contact information broken into <address>,
//             <phone>, <fax>, <email>, and <website>, all optional
// ·         Date created:  <created> whose contents MUST be in ISO 8601 format, e.g. “ 2014-08-10”.
// ·         Date modified:  <modified>  whose contents MUST be in ISO 8601 format, as well
// ·         Model universally unique ID:  <uuid> where the ID MUST be in IETF RFC4122 format (84-4-4-12 hex digits with the dashes)
// ·         Includes: <includes> section with a list of included files or URLs. This is specified in more detail in Section 2.11.
// 2.2.1 XMILE Options
// The XMILE options appear under the tag <options>. This is a list of functionality that is used in the file that may not be included in all implementations. If a file makes use of any of the following functionality[1], it MUST be listed under the <options> tag. The available options are:
// <uses_conveyor/>
// <uses_queue/>
// <uses_arrays/>
// <uses_submodels/>
// <uses_macros/>
// <uses_event_posters/>
// <has_model_view/>
// <uses_outputs/>
// <uses_inputs/>
// <uses_annotation/>
// There is one OPTIONAL attribute for the <options> tag:
// ·         Namespace:  namespace="…" with XMILE namespaces, separated by commas. For example, namespace="std, isee" means try to resolve unrecognized identifiers against the std namespace first, and then against the isee namespace. (default: std)
// The <uses_arrays> tag has one REQUIRED attribute and OPTIONAL attribute:
// ·         Required: Specify the maximum dimensions used by any variable in the whole-model: maximum_dimensions.
// ·         Optional: Specify the value returned when an index is invalid:  invalid_index_value="…" with NaN/0 (default: 0)
// The <uses_macros> tag has two REQUIRED attributes:
// ·         Has macros which are recursive (directly or indirectly):  recursive_macros="…" with true/false.
// ·         Defines option filters:  option_filters="…" with true/false.
// The <uses_conveyor> tag has two OPTIONAL attributes:
// ·         Has conveyors that arrest:  arrest="…" with true/false (default: false)
// ·         Has conveyor leakages:  leak="…" with true/false (default: false)
// The <uses_queue> tag has one OPTIONAL attribute:
// ·         Has queue overflows:  overflow="…" with true/false (default: false)
// The <uses_event_posters> tag has one OPTIONAL attribute:
// ·         Has messages:  messages="…" with true/false (default: false)
// The <has_model_view> tag notes whether the XMILE file contains one or more <view> sections containing a visual representation of one or more models. Note that any software which supports XMILE should be able to simulate all whole-models, even those without diagrams.
// The <uses_outputs> tag implies both time-series graphs and tables are included. It has three OPTIONAL attributes:
// ·         Has numeric display:  numeric_display="…" with true/false (default: false)
// ·         Has lamp:  lamp="…" with true/false (default: false)
// ·         Has gauge:  gauge="…" with true/false (default: false)
// The <uses_inputs> tag implies sliders, knobs, switches, and option groups are included. It has three OPTIONAL attributes:
// ·         Has numeric input:  numeric_input="…" with true/false (default: false)
// ·         Has list input:  list="…" with true/false (default: false)
// ·         Has graphical input:  graphical_input="…" with true/false (default: false)
// The <uses_annotation> tag implies text boxes, graphics frames, and buttons are included.
// A sample options block appears below:
// <options namespace="std, isee">
//    <uses_conveyors leak="true"/>        <!-- has conveyors, some leak -->
//    <uses_arrays maximum_dimensions=”2”/>   <!-- has 2D arrays -->
//    <has_model_view/>                    <!-- has diagram of model -->
// </options>

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Header {
    /// The vendor/company name.
    pub vendor: String,
    /// The product information (name, version, and language).
    pub product: Product,
    /// The options for the header.
    pub options: Option<Options>,
    /// The name of the model.
    pub name: Option<String>,
    /// The version information for the model.
    pub version_info: Option<String>,
    /// The caption for the model.
    pub caption: Option<String>,
    /// The image for the model.
    pub image: Option<String>, // Resource path or Data URI
    /// The author of the model.
    pub author: Option<String>,
    /// The affiliation of the model.
    pub affiliation: Option<String>,
    /// The client of the model.
    pub client: Option<String>,
    /// The copyright information for the model.
    pub copyright: Option<String>,
    /// The contact information for the model.
    pub contact: Option<Contact>,
    /// The creation date of the model.
    pub created: Option<String>, // ISO 8601 format
    /// The last modified date of the model.
    pub modified: Option<String>, // ISO 8601 format
    /// The universally unique ID of the model.
    pub uuid: Option<String>, // IETF RFC4122 format
    /// The list of included files or URLs.
    pub includes: Option<Includes>,
}

/// A list of included files or URLs.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Includes {
    /// List of include resources (URLs, relative paths, or absolute paths).
    #[serde(rename = "include", default)]
    pub includes: Vec<Include>,
}

/// An included file or URL resource.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Include {
    /// The resource path (URL, relative path, or absolute path).
    /// Can include wildcards (e.g., "macros/*.xml").
    #[serde(rename = "@resource")]
    pub resource: String,
}

/// Product information from the <product> tag.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Product {
    /// The product version (REQUIRED attribute).
    #[serde(rename = "@version")]
    pub version: String,
    /// The language code (optional attribute).
    #[serde(rename = "@lang")]
    pub lang: Option<String>,
    /// The product name (text content of the tag).
    /// In serde-xml-rs, text content is typically the field name or can be accessed via #text
    #[serde(rename = "#text")]
    pub name: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Options {
    /// The namespace for the options.
    pub namespace: Option<String>,
    /// Indicates whether conveyors are used.
    pub uses_conveyor: Option<UsesConveyor>,
    /// Indicates whether queues are used.
    pub uses_queue: Option<UsesQueue>,
    /// Indicates whether arrays are used.
    pub uses_arrays: Option<UsesArrays>,
    /// Indicates whether submodels are used.
    pub uses_submodels: Option<bool>,
    /// Indicates whether macros are used.
    pub uses_macros: Option<UsesMacros>,
    /// Indicates whether event posters are used.
    pub uses_event_posters: Option<UsesEventPosters>,
    /// Indicates whether model views are present.
    pub has_model_view: Option<bool>,
    /// Indicates whether outputs are used.
    pub uses_outputs: Option<UsesOutputs>,
    /// Indicates whether inputs are used.
    pub uses_inputs: Option<UsesInputs>,
    /// Indicates whether annotations are used.
    pub uses_annotation: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct UsesConveyor {
    /// Indicates whether arrest is used.
    pub arrest: Option<bool>,
    /// Indicates whether leakages are used.
    pub leak: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct UsesQueue {
    /// Indicates whether overflow is used.
    pub overflow: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct UsesArrays {
    /// The maximum dimensions used by any variable in the whole-model.
    pub maximum_dimensions: usize,
    /// The value returned when an index is invalid.
    pub invalid_index_value: Option<String>, // NaN/0
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct UsesMacros {
    /// Indicates whether recursive macros are used.
    pub recursive_macros: bool,
    /// Indicates whether option filters are defined.
    pub option_filters: bool,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct UsesEventPosters {
    /// Indicates whether messages are used.
    pub messages: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct UsesOutputs {
    /// Indicates whether numeric display is used.
    pub numeric_display: Option<bool>,
    /// Indicates whether lamps are used.
    pub lamp: Option<bool>,
    /// Indicates whether gauges are used.
    pub gauge: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct UsesInputs {
    /// Indicates whether numeric input is used.
    pub numeric_input: Option<bool>,
    /// Indicates whether list input is used.
    pub list: Option<bool>,
    /// Indicates whether graphical input is used.
    pub graphical_input: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Contact {
    /// The address of the contact.
    pub address: Option<String>,
    /// The phone number of the contact.
    pub phone: Option<String>,
    /// The fax number of the contact.
    pub fax: Option<String>,
    /// The email of the contact.
    pub email: Option<String>,
    /// The website of the contact.
    pub website: Option<String>,
}
