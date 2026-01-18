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
use std::collections::HashSet;

use crate::types::{Validate, ValidationResult};

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

/// Valid event action names according to XMILE spec
const VALID_SIM_ACTIONS: &[&str] = &["pause", "stop", "message"];

/// Valid event direction names according to XMILE spec
const VALID_DIRECTIONS: &[&str] = &["increasing", "decreasing"];

/// Valid event frequency/repeat names according to XMILE spec
const VALID_REPEAT: &[&str] = &["each"]; // Add more as needed based on spec

impl Validate for EventPoster {
    fn validate(&self) -> ValidationResult {
        let warnings = Vec::new();
        let mut errors = Vec::new();

        // Validate min <= max
        if self.min > self.max {
            errors.push(format!(
                "EventPoster min ({}) must be <= max ({})",
                self.min, self.max
            ));
        }

        // Validate thresholds
        for (idx, threshold) in self.thresholds.iter().enumerate() {
            match threshold.validate() {
                ValidationResult::Valid(_) => {}
                ValidationResult::Warnings(_, ws) => {
                    for w in ws {
                        errors.push(format!("Threshold {}: {}", idx, w));
                    }
                }
                ValidationResult::Invalid(ws, es) => {
                    for w in ws {
                        errors.push(format!("Threshold {}: {}", idx, w));
                    }
                    for e in es {
                        errors.push(format!("Threshold {}: {}", idx, e));
                    }
                }
            }
        }

        // Validate that each threshold has a unique (value, direction) combination
        // Use string key since f64 doesn't implement Hash
        let mut seen_combinations = HashSet::new();
        for (idx, threshold) in self.thresholds.iter().enumerate() {
            let direction = threshold.direction.as_deref().unwrap_or("increasing");
            // Format value with enough precision to distinguish thresholds
            let key = format!("{:.15}:{}", threshold.value, direction);
            if !seen_combinations.insert(key.clone()) {
                errors.push(format!(
                    "Threshold {}: Duplicate threshold with value {} and direction '{}'. Each threshold must have a unique (value, direction) combination.",
                    idx, threshold.value, direction
                ));
            }
        }

        if errors.is_empty() {
            ValidationResult::Valid(())
        } else {
            ValidationResult::Invalid(warnings, errors)
        }
    }
}

impl Validate for Threshold {
    fn validate(&self) -> ValidationResult {
        let warnings = Vec::new();
        let mut errors = Vec::new();

        // Validate direction if present
        if let Some(ref dir) = self.direction
            && !VALID_DIRECTIONS.contains(&dir.as_str())
        {
            errors.push(format!(
                "Invalid direction '{}'. Valid directions are: {:?}",
                dir, VALID_DIRECTIONS
            ));
        }

        // Validate repeat if present
        if let Some(ref repeat) = self.repeat
            && !VALID_REPEAT.contains(&repeat.as_str())
        {
            errors.push(format!(
                "Invalid repeat '{}'. Valid repeat values are: {:?}",
                repeat, VALID_REPEAT
            ));
        }

        // Validate that if repeat="each", there must be exactly one event
        let repeat_value = self.repeat.as_deref().unwrap_or("each");
        if repeat_value == "each" && self.events.len() != 1 {
            errors.push(format!(
                "Threshold with repeat='each' must have exactly one event, but has {}",
                self.events.len()
            ));
        }

        // Validate events
        for (idx, event) in self.events.iter().enumerate() {
            match event.validate() {
                ValidationResult::Valid(_) => {}
                ValidationResult::Warnings(_, ws) => {
                    for w in ws {
                        errors.push(format!("Event {}: {}", idx, w));
                    }
                }
                ValidationResult::Invalid(ws, es) => {
                    for w in ws {
                        errors.push(format!("Event {}: {}", idx, w));
                    }
                    for e in es {
                        errors.push(format!("Event {}: {}", idx, e));
                    }
                }
            }
        }

        // Validate that threshold has at least one event
        if self.events.is_empty() {
            errors.push("Threshold must have at least one event".to_string());
        }

        if errors.is_empty() {
            ValidationResult::Valid(())
        } else {
            ValidationResult::Invalid(warnings, errors)
        }
    }
}

impl Validate for Event {
    fn validate(&self) -> ValidationResult {
        let warnings = Vec::new();
        let mut errors = Vec::new();

        // Validate sim_action if present
        if let Some(ref action) = self.sim_action
            && !VALID_SIM_ACTIONS.contains(&action.as_str())
        {
            errors.push(format!(
                "Invalid sim_action '{}'. Valid actions are: {:?}",
                action, VALID_SIM_ACTIONS
            ));
        }

        if errors.is_empty() {
            ValidationResult::Valid(())
        } else {
            ValidationResult::Invalid(warnings, errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_poster_validation_min_max() {
        let poster = EventPoster {
            min: 10.0,
            max: 5.0, // Invalid: min > max
            thresholds: vec![],
        };

        match poster.validate() {
            ValidationResult::Invalid(_, errors) => {
                assert!(
                    errors
                        .iter()
                        .any(|e| e.contains("min") && e.contains("max"))
                );
            }
            _ => panic!("Expected validation error for min > max"),
        }
    }

    #[test]
    fn test_threshold_unique_value_direction() {
        let poster = EventPoster {
            min: 0.0,
            max: 10.0,
            thresholds: vec![
                Threshold {
                    value: 5.0,
                    direction: Some("increasing".to_string()),
                    repeat: None,
                    interval: None,
                    events: vec![Event {
                        sim_action: None,
                        actions: vec![],
                    }],
                },
                Threshold {
                    value: 5.0,
                    direction: Some("increasing".to_string()), // Duplicate
                    repeat: None,
                    interval: None,
                    events: vec![Event {
                        sim_action: None,
                        actions: vec![],
                    }],
                },
            ],
        };

        match poster.validate() {
            ValidationResult::Invalid(_, errors) => {
                assert!(errors.iter().any(|e| e.contains("Duplicate threshold")));
            }
            _ => panic!("Expected validation error for duplicate threshold"),
        }
    }

    #[test]
    fn test_threshold_same_value_different_direction() {
        let poster = EventPoster {
            min: 0.0,
            max: 10.0,
            thresholds: vec![
                Threshold {
                    value: 5.0,
                    direction: Some("increasing".to_string()),
                    repeat: None,
                    interval: None,
                    events: vec![Event {
                        sim_action: None,
                        actions: vec![],
                    }],
                },
                Threshold {
                    value: 5.0,
                    direction: Some("decreasing".to_string()), // Different direction - OK
                    repeat: None,
                    interval: None,
                    events: vec![Event {
                        sim_action: None,
                        actions: vec![],
                    }],
                },
            ],
        };

        // Should be valid - same value but different directions
        assert!(poster.validate().is_valid());
    }

    #[test]
    fn test_threshold_repeat_each_requires_one_event() {
        let threshold = Threshold {
            value: 5.0,
            direction: None,
            repeat: Some("each".to_string()),
            interval: None,
            events: vec![
                Event {
                    sim_action: None,
                    actions: vec![],
                },
                Event {
                    sim_action: None,
                    actions: vec![],
                },
            ],
        };

        match threshold.validate() {
            ValidationResult::Invalid(_, errors) => {
                assert!(
                    errors
                        .iter()
                        .any(|e| e.contains("repeat='each'") && e.contains("exactly one event"))
                );
            }
            _ => panic!("Expected validation error for repeat='each' with multiple events"),
        }
    }

    #[test]
    fn test_event_invalid_sim_action() {
        let event = Event {
            sim_action: Some("invalid_action".to_string()),
            actions: vec![],
        };

        match event.validate() {
            ValidationResult::Invalid(_, errors) => {
                assert!(errors.iter().any(|e| e.contains("Invalid sim_action")));
            }
            _ => panic!("Expected validation error for invalid sim_action"),
        }
    }

    #[test]
    fn test_threshold_must_have_at_least_one_event() {
        let threshold = Threshold {
            value: 5.0,
            direction: None,
            repeat: None,
            interval: None,
            events: vec![], // No events
        };

        match threshold.validate() {
            ValidationResult::Invalid(_, errors) => {
                assert!(errors.iter().any(|e| e.contains("at least one event")));
            }
            _ => panic!("Expected validation error for threshold with no events"),
        }
    }
}
