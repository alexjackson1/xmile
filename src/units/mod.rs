// All user-specified model unit definitions are specified in the <model_units> tag as shown below:
// <model_units>
//    <unit name="models_per_person_per_year">
//      <eqn>models/person/year</eqn> <!-- name, equation -->
//    </unit>
//    <unit name="Rabbits">
//      <alias>Rabbit</alias>   <!-- name, alias -->
//    </unit>
//    <unit name="models_per_year">
//      <eqn>models/year</eqn>       <!-- name, eqn, alias -->
//      <alias>model_per_year</alias>
//      <alias>mpy</alias>
//    </unit>
//    <unit name="Joules" disabled="true"> <!-- disabled unit -->
//      <alias>J</alias>
//    </unit>
// </model_units>
// All unit definitions MUST contain a name, possibly an equation, and 0 or more aliases (Including a unit definition with only a name is valid but discouraged). Unit equations (<eqn> tag) are defined with XMILE unit expressions. One <alias> tag with the name of the alias appears for each distinct unit alias. A unit with the attribute disabled set to true MUST NOT be included in the unit substitution process. It is included to override a Unit Definition that may be built into the software or specified as a preference by the user.
// Vendor-provided unit definitions not used in a model are NOT REQUIRED to appear in the file, but SHOULD be made available in this same format in a vendor-specific library.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelUnits {
    /// A list of unit definitions in the XMILE file.
    #[serde(rename = "unit")]
    pub units: Vec<UnitDefinition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitDefinition {
    /// The name of the unit.
    #[serde(rename = "@name")]
    pub name: String,
    /// An optional equation defining the unit.
    pub eqn: Option<String>,
    /// A list of aliases for the unit.
    #[serde(rename = "alias", default)]
    pub aliases: Vec<String>,
    /// Indicates whether the unit is disabled.
    #[serde(rename = "@disabled")]
    pub disabled: Option<bool>,
}
