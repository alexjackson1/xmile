// 2.6 Behavior Section
// Every XMILE file MAY include behavior information to set default options that affect the simulation of model entities. Support for behaviors is REQUIRED. This is usually used in combination with macros to change some aspect of a given type of entity’s performance, for example, setting all stocks to be non-negative.
// The behavior information cascades across four levels from the entity outwards, with the actual entity behavior defined by the first occurrence of a behavior definition for that behavior property:
// 1.     Behaviors for a given entity
// 2.     Behaviors for all entities in a model (affects only that Model section)
// 3.     Behaviors for all entities in all models in the file (affects all Model sections)
// 4.     Default XMILE-defined behaviors when a default appears in this specification
// The behavior block begins with the <behavior> tag. Within this block, any known object can have its attributes set globally (but overridden locally) using its own modifier tags. Global settings that apply to everything are specified directly on the <behavior> tag or in nodes below it. This is true for <behavior> tags that appear within the <model> tag as well. For example, all entities (particularly stocks and flows) can be set to be non-negative by default:
// <behavior>
//    <non_negative/>
// </behavior>
// Only stocks or only flows can also be set to non-negative by default (flows in this example):
// <behavior>
//    <flow>
//      <non_negative/>
//    </flow>
// </behavior>

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::types::{Validate, ValidationResult};

/// Behavior information that cascades across four levels:
/// 1. Behaviors for a given entity
/// 2. Behaviors for all entities in a model (affects only that Model section)
/// 3. Behaviors for all entities in all models in the file (affects all Model sections)
/// 4. Default XMILE-defined behaviors when a default appears in this specification
///
/// The behavior block can contain:
/// - Global properties directly on the <behavior> tag (e.g., <non_negative/>)
/// - Entity-specific properties (e.g., <flow><non_negative/></flow>)
#[derive(Debug, PartialEq, Clone)]
pub struct Behavior {
    /// Global behavior properties that apply to all entities
    pub global: EntityBehavior,
    /// Entity-specific behavior properties
    pub entities: Vec<EntityBehaviorEntry>,
}

/// Behavior properties for a specific entity type or globally
#[derive(Debug, PartialEq, Clone)]
pub struct EntityBehavior {
    /// Whether entities should be non-negative by default
    pub non_negative: Option<bool>,
}

impl Default for EntityBehavior {
    fn default() -> Self {
        EntityBehavior {
            non_negative: None,
        }
    }
}

/// Entity-specific behavior entry (e.g., <flow><non_negative/></flow>)
#[derive(Debug, PartialEq, Clone)]
pub struct EntityBehaviorEntry {
    /// The entity type (e.g., "stock", "flow", "aux")
    pub entity_type: String,
    /// The behavior properties for this entity type
    pub behavior: EntityBehavior,
}

/// Raw XML structure for deserialization
#[derive(Debug, Clone, PartialEq, Deserialize)]
struct RawBehavior {
    #[serde(rename = "non_negative", default)]
    non_negative: Option<NonNegativeFlag>,
    #[serde(rename = "stock", default)]
    stock: Option<EntityBehaviorTag>,
    #[serde(rename = "flow", default)]
    flow: Option<EntityBehaviorTag>,
    #[serde(rename = "aux", default)]
    aux: Option<EntityBehaviorTag>,
    #[serde(rename = "gf", default)]
    gf: Option<EntityBehaviorTag>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct NonNegativeFlag {
    #[serde(rename = "#text", default = "default_true")]
    value: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct EntityBehaviorTag {
    #[serde(rename = "non_negative", default)]
    non_negative: Option<NonNegativeFlag>,
}

impl<'de> Deserialize<'de> for Behavior {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw: RawBehavior = Deserialize::deserialize(deserializer)?;
        
        let global = EntityBehavior {
            non_negative: raw.non_negative.map(|nn| nn.value),
        };
        
        let mut entities = Vec::new();
        
        if let Some(stock) = raw.stock {
            entities.push(EntityBehaviorEntry {
                entity_type: "stock".to_string(),
                behavior: EntityBehavior {
                    non_negative: stock.non_negative.map(|nn| nn.value),
                },
            });
        }
        
        if let Some(flow) = raw.flow {
            entities.push(EntityBehaviorEntry {
                entity_type: "flow".to_string(),
                behavior: EntityBehavior {
                    non_negative: flow.non_negative.map(|nn| nn.value),
                },
            });
        }
        
        if let Some(aux) = raw.aux {
            entities.push(EntityBehaviorEntry {
                entity_type: "aux".to_string(),
                behavior: EntityBehavior {
                    non_negative: aux.non_negative.map(|nn| nn.value),
                },
            });
        }
        
        if let Some(gf) = raw.gf {
            entities.push(EntityBehaviorEntry {
                entity_type: "gf".to_string(),
                behavior: EntityBehavior {
                    non_negative: gf.non_negative.map(|nn| nn.value),
                },
            });
        }
        
        Ok(Behavior {
            global,
            entities,
        })
    }
}

impl Serialize for Behavior {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("behavior", 5)?;
        
        if let Some(nn) = self.global.non_negative {
            if nn {
                state.serialize_field("non_negative", &NonNegativeFlag { value: true })?;
            }
        }
        
        for entry in &self.entities {
            match entry.entity_type.as_str() {
                "stock" => {
                    let mut tag = EntityBehaviorTag { non_negative: None };
                    if let Some(nn) = entry.behavior.non_negative {
                        if nn {
                            tag.non_negative = Some(NonNegativeFlag { value: true });
                        }
                    }
                    state.serialize_field("stock", &tag)?;
                }
                "flow" => {
                    let mut tag = EntityBehaviorTag { non_negative: None };
                    if let Some(nn) = entry.behavior.non_negative {
                        if nn {
                            tag.non_negative = Some(NonNegativeFlag { value: true });
                        }
                    }
                    state.serialize_field("flow", &tag)?;
                }
                "aux" => {
                    let mut tag = EntityBehaviorTag { non_negative: None };
                    if let Some(nn) = entry.behavior.non_negative {
                        if nn {
                            tag.non_negative = Some(NonNegativeFlag { value: true });
                        }
                    }
                    state.serialize_field("aux", &tag)?;
                }
                "gf" => {
                    let mut tag = EntityBehaviorTag { non_negative: None };
                    if let Some(nn) = entry.behavior.non_negative {
                        if nn {
                            tag.non_negative = Some(NonNegativeFlag { value: true });
                        }
                    }
                    state.serialize_field("gf", &tag)?;
                }
                _ => {}
            }
        }
        
        state.end()
    }
}

impl Behavior {
    /// Resolves behavior for a specific entity type using cascading rules.
    /// 
    /// The behavior information cascades across four levels from the entity outwards,
    /// with the actual entity behavior defined by the first occurrence of a behavior
    /// definition for that behavior property:
    /// 1. Behaviors for a given entity (passed as `entity_behavior`)
    /// 2. Behaviors for all entities in a model (passed as `model_behavior`)
    /// 3. Behaviors for all entities in all models in the file (passed as `file_behavior`)
    /// 4. Default XMILE-defined behaviors (hardcoded defaults)
    /// 
    /// # Arguments
    /// 
    /// * `entity_type` - The type of entity ("stock", "flow", "aux", "gf")
    /// * `entity_behavior` - Optional behavior defined directly on the entity
    /// * `model_behavior` - Optional behavior defined at the model level
    /// * `file_behavior` - Optional behavior defined at the file level
    /// 
    /// # Returns
    /// 
    /// The resolved `EntityBehavior` with values from the first level that defines them.
    pub fn resolve_for_entity(
        entity_type: &str,
        entity_behavior: Option<&EntityBehavior>,
        model_behavior: Option<&Behavior>,
        file_behavior: Option<&Behavior>,
    ) -> EntityBehavior {
        // Level 1: Entity-specific behavior (highest priority)
        if let Some(eb) = entity_behavior {
            return eb.clone();
        }
        
        // Level 2: Model-level behavior for this entity type
        if let Some(mb) = model_behavior {
            // Check for entity-specific behavior in model
            if let Some(entry) = mb.entities.iter().find(|e| e.entity_type == entity_type) {
                return entry.behavior.clone();
            }
            // Check for global behavior in model
            if mb.global.non_negative.is_some() {
                return mb.global.clone();
            }
        }
        
        // Level 3: File-level behavior for this entity type
        if let Some(fb) = file_behavior {
            // Check for entity-specific behavior in file
            if let Some(entry) = fb.entities.iter().find(|e| e.entity_type == entity_type) {
                return entry.behavior.clone();
            }
            // Check for global behavior in file
            if fb.global.non_negative.is_some() {
                return fb.global.clone();
            }
        }
        
        // Level 4: Default XMILE-defined behaviors
        // Currently, there are no default behaviors specified in the XMILE spec
        // for non_negative, so we return the default (None)
        EntityBehavior::default()
    }
    
    /// Gets behavior for a specific entity type from this behavior block.
    /// 
    /// Returns entity-specific behavior if present, otherwise global behavior.
    pub fn get_for_entity_type(&self, entity_type: &str) -> EntityBehavior {
        // First check for entity-specific behavior
        if let Some(entry) = self.entities.iter().find(|e| e.entity_type == entity_type) {
            entry.behavior.clone()
        } else {
            // Fall back to global behavior
            self.global.clone()
        }
    }
}

impl EntityBehavior {
    /// Merges this behavior with another, with `other` taking precedence.
    /// 
    /// Values from `other` override values in `self` when `other` has `Some`.
    pub fn merge(&self, other: &EntityBehavior) -> EntityBehavior {
        EntityBehavior {
            non_negative: other.non_negative.or(self.non_negative),
        }
    }
}

impl Validate for Behavior {
    fn validate(&self) -> ValidationResult {
        let warnings = Vec::new();
        let mut errors = Vec::new();
        
        // Validate entity types are valid
        let valid_entity_types = ["stock", "flow", "aux", "gf"];
        for entry in &self.entities {
            if !valid_entity_types.contains(&entry.entity_type.as_str()) {
                errors.push(format!(
                    "Invalid entity type '{}' in behavior. Valid types are: {:?}",
                    entry.entity_type, valid_entity_types
                ));
            }
        }
        
        // Check for duplicate entity type entries
        let mut seen_types = std::collections::HashSet::new();
        for entry in &self.entities {
            if !seen_types.insert(&entry.entity_type) {
                errors.push(format!(
                    "Duplicate entity type '{}' in behavior block",
                    entry.entity_type
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

impl Validate for EntityBehavior {
    fn validate(&self) -> ValidationResult {
        // Currently, EntityBehavior only has non_negative which is always valid
        // (it's either Some(bool) or None)
        ValidationResult::Valid(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_cascading_entity_first() {
        let entity_behavior = EntityBehavior {
            non_negative: Some(true),
        };
        let model_behavior = Behavior {
            global: EntityBehavior { non_negative: Some(false) },
            entities: vec![],
        };
        let file_behavior = Behavior {
            global: EntityBehavior { non_negative: Some(false) },
            entities: vec![],
        };
        
        let resolved = Behavior::resolve_for_entity(
            "stock",
            Some(&entity_behavior),
            Some(&model_behavior),
            Some(&file_behavior),
        );
        
        // Entity behavior should take precedence
        assert_eq!(resolved.non_negative, Some(true));
    }

    #[test]
    fn test_behavior_cascading_model_second() {
        let model_behavior = Behavior {
            global: EntityBehavior { non_negative: Some(true) },
            entities: vec![],
        };
        let file_behavior = Behavior {
            global: EntityBehavior { non_negative: Some(false) },
            entities: vec![],
        };
        
        let resolved = Behavior::resolve_for_entity(
            "stock",
            None,
            Some(&model_behavior),
            Some(&file_behavior),
        );
        
        // Model behavior should take precedence over file
        assert_eq!(resolved.non_negative, Some(true));
    }

    #[test]
    fn test_behavior_cascading_file_third() {
        let file_behavior = Behavior {
            global: EntityBehavior { non_negative: Some(true) },
            entities: vec![],
        };
        
        let resolved = Behavior::resolve_for_entity(
            "stock",
            None,
            None,
            Some(&file_behavior),
        );
        
        // File behavior should be used
        assert_eq!(resolved.non_negative, Some(true));
    }

    #[test]
    fn test_behavior_cascading_entity_specific() {
        let model_behavior = Behavior {
            global: EntityBehavior { non_negative: Some(false) },
            entities: vec![EntityBehaviorEntry {
                entity_type: "stock".to_string(),
                behavior: EntityBehavior { non_negative: Some(true) },
            }],
        };
        
        let resolved = Behavior::resolve_for_entity(
            "stock",
            None,
            Some(&model_behavior),
            None,
        );
        
        // Entity-specific behavior in model should take precedence over global
        assert_eq!(resolved.non_negative, Some(true));
    }

    #[test]
    fn test_behavior_default() {
        let resolved = Behavior::resolve_for_entity(
            "stock",
            None,
            None,
            None,
        );
        
        // Should return default (None for non_negative)
        assert_eq!(resolved.non_negative, None);
    }

    #[test]
    fn test_entity_behavior_merge() {
        let base = EntityBehavior {
            non_negative: Some(false),
        };
        let other = EntityBehavior {
            non_negative: Some(true),
        };
        
        let merged = base.merge(&other);
        assert_eq!(merged.non_negative, Some(true));
    }

    #[test]
    fn test_entity_behavior_merge_none_preserves() {
        let base = EntityBehavior {
            non_negative: Some(true),
        };
        let other = EntityBehavior {
            non_negative: None,
        };
        
        let merged = base.merge(&other);
        assert_eq!(merged.non_negative, Some(true));
    }
}
