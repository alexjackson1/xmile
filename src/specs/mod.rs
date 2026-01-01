// Every XMILE file MUST contain at least one set of simulation specifications, either as a top-level tag under <xmile> or as a child of the root model. Note that simulation specifications can recur in each Model section to override specific global defaults. Great care should be taken in these situations to avoid nonsensical results.
// The simulation specifications block is defined with the tag <sim_specs>.  The following properties are REQUIRED:
// ·         Start time:  <start> w/time
// ·         Stop time:  <stop> w/time (after start time)
// There are several additional OPTIONAL attributes and properties with appropriate defaults:
// ·         Step size:  <dt> w/value (default: 1)
// Optionally specified as the integer reciprocal of DT (for DT <= 1 only) with an attribute of <dt>:  reciprocal="…" with true/false (default: false)
// ·         Integration method:  method="…" w/XMILE name (default: euler)
// ·         Unit of time:  time_units="…" w/Name (empty default)
// ·         Pause interval:  pause="…" w/interval (default: infinity – can be ignored)
// ·         Run selected groups or modules:  <run by="…"> with run type either:  all, group, or module (default: all, i.e., run whole-model).  Which groups or modules to run are identified by run attributes on the group or model.

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SimulationSpecs {
    /// The start time of the simulation.
    pub start: f64,
    /// The stop time of the simulation.
    pub stop: f64,
    /// The step size (DT) of the simulation.
    pub dt: Option<f64>,
    /// The integration method used in the simulation.
    pub method: Option<String>,
    /// The unit of time for the simulation.
    pub time_units: Option<String>,
    /// The pause interval for the simulation.
    pub pause: Option<f64>,
    /// The run type for the simulation (e.g., all, group, module).
    pub run_by: Option<String>,
}
