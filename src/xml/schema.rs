use crate::model::variables::Variable;

/// A XMILE file contains information about a whole-model, with a
/// well-specified structure. The file MUST be encoded in UTF-8. The entire
/// XMILE file is enclosed within a <xmile> tag as follows:
///
/// ```xml
/// <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
///    ...
/// </xmile>
/// ```
///
/// The version number MUST refer to the version of XMILE used (presently 1.0).
/// The XML namespace refers to tags and attributes used in this specification.
/// Both of these attributes are REQUIRED. Inside of the <xmile> tag are a
/// number of top-level tags, listed below. These tags are marked req (a single
/// instance is REQUIRED), opt (a single instance is OPTIONAL), * (zero or more
/// tags MAY occur) and + (one or more tags MAY occur). Top level tags MAY
/// occur in any order, but are RECOMMENDED to occur in the following order:
///
/// - `<header>` (req) - information about the origin of the file and required
///   capabilities.
/// - `<sim_specs>` (opt) - default simulation specifications for this file.
/// - `<model_units>` (opt) - definitions of model units used in this file.
/// - `<dimensions>` (opt) - definitions of array dimensions specific to this
///   file.
/// - `<behavior>` (opt) - simulation style definitions that are
///   inherited/cascaded through all models defined in this XMILE file.
/// - `<style>` (opt) - display style definitions that are inherited/cascaded
///   through all views defined in this XMILE file.
/// - `<data>` (opt) - definitions of persistent data import/export
///   connections.
/// - `<model>+` - definition of model equations and (optionally) diagrams.
/// - `<macro>*` - definition of macros that can be used in model equations.
///
/// These tags are specified in the subsequent sections of this chapter, after XMILE namespaces are discussed.
///
/// When an XMILE file includes references to models contained in separate files or at a specific URL, each such file may contain overlapping information, most commonly in sim_specs, model_units and dimensions. When such overlap is consistent, combining parts is done by taking the union of the different component files. When an inconsistency is found, (for example, a dimension with two distinct definitions) software reading the files MUST resolve the inconsistency and SHOULD provide user feedback in doing so. Some inconsistencies, such as conflicting Macro or Model names MUST be resolved as detailed in section 2.11.3.
pub struct File {
    /// The version of the XMILE specification used in this file.
    pub version: String,
    /// The XML namespace for XMILE.
    pub xmlns: String,
    /// The header information for the XMILE file.
    pub header: Header,
    /// Optional simulation specifications for the XMILE file.
    pub sim_specs: Option<SimulationSpecs>,
    /// Optional model units defined in the XMILE file.
    pub model_units: Option<ModelUnits>,
    /// Optional dimensions defined in the XMILE file.
    pub dimensions: Option<Dimensions>,
    /// Optional behavior specifications for the XMILE file.
    pub behavior: Option<Behavior>,
    /// Optional style definitions for the XMILE file.
    pub style: Option<Style>,
    /// Optional data definitions for the XMILE file.
    pub data: Option<Data>,
    /// A list of models defined in the XMILE file.
    pub models: Vec<Model>,
    /// A list of macros defined in the XMILE file.
    pub macros: Vec<Macro>,
}

/// The XML tag for the file header is <header>. The REQUIRED sub-tags are:
///
/// ·         Vendor name:  <vendor> w/company name
/// ·         Product name:  <product version="…" lang="…"> w/product name – the product version number is REQUIRED. The language code is optional (default: English) and describes the language used for variable names and comments. Language codes are described by ISO 639-1 unless the language is not there, in which case the ISO 639-2 code should be used (e.g., for Hawaiian).
///
/// OPTIONAL sub-tags include:
///
/// ·         XMILE options: <options> (defined below)
/// ·         Model name:     <name> w/name
/// ·         Model version:  <version> w/version information
/// ·         Model caption:  <caption> w/caption
/// ·         Picture of the model in JPG, GIF, TIF, or PNG format:  <image resource=””>. The resource attribute is optional and may specify a relative file path, an absolute file path, or an URL.  The picture data may also be embedded inside the <image> tag in Data URI format, using base64 encoding.
/// ·         Author name:  <author> w/author name
/// ·         Company name:  <affiliation> w/company name
/// ·         Client name:  <client> w/client name
/// ·         Copyright notice:  <copyright> w/copyright information
/// ·         Contact information (e-mail, phone, mailing address, web site):
///             <contact> block w/contact information broken into <address>,
///             <phone>, <fax>, <email>, and <website>, all optional
/// ·         Date created:  <created> whose contents MUST be in ISO 8601 format, e.g. “ 2014-08-10”.
/// ·         Date modified:  <modified>  whose contents MUST be in ISO 8601 format, as well
/// ·         Model universally unique ID:  <uuid> where the ID MUST be in IETF RFC4122 format (84-4-4-12 hex digits with the dashes)
/// ·         Includes: <includes> section with a list of included files or URLs. This is specified in more detail in Section 2.11.

#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    vendor: String,
    product: String,
    version: String,
    lang: Option<String>,
    options: Option<Options>,
    name: Option<String>,
    version_info: Option<String>,
    caption: Option<String>,
    image: Option<String>, // Resource path or Data URI
    author: Option<String>,
    affiliation: Option<String>,
    client: Option<String>,
    copyright: Option<String>,
    contact: Option<Contact>,
    created: Option<String>,       // ISO 8601 format
    modified: Option<String>,      // ISO 8601 format
    uuid: Option<String>,          // IETF RFC4122 format
    includes: Option<Vec<String>>, // List of included files or URLs
}
/// The overall structure of a <model> tag appears below (sub-tags MUST appear in this order):
///
/// ```xml
/// <model>
///    <sim_specs>    <!-- OPTIONAL – see Chapter 2 -->
///      ...
///    </sim_specs>
///    <behavior>     <!-- OPTIONAL – see Chapter 2 -->
///      ...
///    </behavior>
///    <variables>    <!-- REQUIRED -->
///      ...
///    </variables>
///    <views>        <!-- OPTIONAL – see Chapters 5 & 6 -->
///      ...
///    </views>
/// </model>
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Model {
    sim_specs: Option<SimulationSpecs>,
    behavior: Option<Behavior>,
    variables: Vec<Variable>,
    views: Option<Views>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SimulationSpecs {}

#[derive(Debug, PartialEq, Clone)]
pub struct Behavior {}

#[derive(Debug, PartialEq, Clone)]
pub struct Views {}

pub struct AuxiliaryVariable {
    name: String,
    expression: String,
}

pub struct StockVariable {
    name: String,
    initial_value: f64,
}
