use crate::{
    behavior::Behavior, data::Data, dimensions::Dimensions, header::Header, model::vars::Variable,
    specs::SimulationSpecs, units::ModelUnits, view::Style,
};

#[cfg(feature = "macros")]
use crate::r#macro::Macro;

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
/// These tags are specified in the subsequent sections of this chapter, after
/// XMILE namespaces are discussed.
///
/// When an XMILE file includes references to models contained in separate files
/// or at a specific URL, each such file may contain overlapping information,
/// most commonly in sim_specs, model_units and dimensions. When such overlap
/// is consistent, combining parts is done by taking the union of the different
/// component files. When an inconsistency is found, (for example, a dimension
/// with two distinct definitions) software reading the files MUST resolve the
/// inconsistency and SHOULD provide user feedback in doing so. Some
/// inconsistencies, such as conflicting Macro or Model names MUST be resolved
/// as detailed in section 2.11.3.
pub struct XmileFile {
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
    #[cfg(feature = "macros")]
    pub macros: Vec<Macro>,
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
pub struct Views {}
