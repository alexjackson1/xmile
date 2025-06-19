use crate::{Identifier, Uid};

pub struct Group {
    pub name: Identifier,
    pub entities: Vec<Identifier>,
    pub display: Vec<Uid>,
}
