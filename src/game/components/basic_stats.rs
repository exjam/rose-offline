use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct BasicStats {
    pub strength: u16,
    pub dexterity: u16,
    pub intelligence: u16,
    pub concentration: u16,
    pub charm: u16,
    pub sense: u16,
}
