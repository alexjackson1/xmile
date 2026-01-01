use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Uid {
    #[serde(rename = "@uid")]
    pub value: i32,
}

impl Uid {
    pub fn new(value: i32) -> Self {
        Uid { value }
    }
}
