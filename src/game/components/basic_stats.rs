use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug)]
pub enum BasicStatType {
    Strength,
    Dexterity,
    Intelligence,
    Concentration,
    Charm,
    Sense,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BasicStats {
    pub strength: i32,
    pub dexterity: i32,
    pub intelligence: i32,
    pub concentration: i32,
    pub charm: i32,
    pub sense: i32,
}
