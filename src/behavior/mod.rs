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

// TODO: This is too vague for me to understand how to implement properly right now.

#[derive(Debug, PartialEq, Clone)]
pub struct Behavior {
    pub entries: Vec<BehaviorEntry>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BehaviorEntry {
    pub entity: String,
    pub entry_properties: Vec<BehaviorProperty>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BehaviorProperty {
    NonNegative,
}
