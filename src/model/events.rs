// Events based on entity values can be triggered while the model is being simulated. At the simulation level, these events are limited to pausing the simulation (default) or stopping the simulation. The model user can be informed of these events in various ways, as described in Chapter 6.
//
// All events appear in an event_poster block with min and max attributes specifying the lower and upper bounds for all posters (this is a user setting to help them decide where to place events). A series of threshold blocks then define the event triggers:
//
// <event_poster min="0" max="10">
//
//    <threshold value="5">
//
//      <event>
//
//         ...
//
//      </event>
//
//      ...
//
//    </threshold>
//
// </event_poster>
//
// The threshold has these additional OPTIONAL attributes:
//
//     Direction:  direction="…" w/valid XMILE event direction name – see Chapter 3
//     (default: increasing)
//     Frequency:  repeat="…" w/valid XMILE event frequency name – see Chapter 3 (default: each)
//     Repetition interval:  interval="…" w/number of unit times (default: disabled; only enabled if present)
//
// Each threshold block MUST have a unique value and direction (so there can be two threshold blocks at 5 as long as one is increasing and the other is decreasing). Within each threshold block, the actual events are defined, which MAY be either a single event that is used every time the threshold is exceeded (frequency of each REQUIRES there be only one event) or a sequence of events that are used one at a time in their specified order each time the threshold is exceeded (i.e., the first event is used the first time the threshold is exceeded, the second is used the second time, etc.). Events appear in an <event> tag which has one OPTIONAL attribute:
//
//     Action:  sim_action="…" w/valid XMILE event action name – see Chapter 3 (default: pause)

use serde::{Deserialize, Serialize};

pub trait Poster {
    fn poster(&self) -> Option<&EventPoster>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventPoster {
    #[serde(rename = "@min")]
    pub min: f64,
    #[serde(rename = "@max")]
    pub max: f64,
    #[serde(rename = "threshold", default)]
    pub thresholds: Vec<Threshold>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Threshold {
    #[serde(rename = "@value")]
    pub value: f64,
    #[serde(rename = "@direction")]
    pub direction: Option<String>,
    #[serde(rename = "@repeat")]
    pub repeat: Option<String>,
    #[serde(rename = "@interval")]
    pub interval: Option<f64>,
    #[serde(rename = "event", default)]
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "@sim_action")]
    pub sim_action: Option<String>,
    // Actions can be text content or child elements - for now, we'll handle as text
    #[serde(rename = "#text", default)]
    pub actions: Vec<String>, // Actions to be taken when the event is triggered
}
