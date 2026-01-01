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
