use serde::{Deserialize, Serialize};

use crate::{
    model::object::{Document, Documentation, Object},
    Identifier, Uid,
};

/// Helper struct for deserializing entity tags within groups
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupEntity {
    #[serde(rename = "@name")]
    pub name: Identifier,
    #[serde(rename = "@run", default = "default_false")]
    pub run: bool,
}

fn default_false() -> bool {
    false
}

/// A group (sector) that collects related model structure together.
/// Groups REQUIRE a name and MAY have documentation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
    #[serde(rename = "@name")]
    pub name: Identifier,
    #[serde(rename = "doc", default)]
    pub doc: Option<Documentation>,
    #[serde(rename = "entity", default)]
    pub entities: Vec<GroupEntity>,
    // Display UIDs are handled separately in views
    #[serde(skip)]
    pub display: Vec<Uid>,
}

impl Object for Group {
    fn range(&self) -> Option<&crate::model::object::DeviceRange> {
        None
    }

    fn scale(&self) -> Option<&crate::model::object::DeviceScale> {
        None
    }

    fn format(&self) -> Option<&crate::model::object::FormatOptions> {
        None
    }
}

impl Document for Group {
    fn documentation(&self) -> Option<&Documentation> {
        self.doc.as_ref()
    }
}
