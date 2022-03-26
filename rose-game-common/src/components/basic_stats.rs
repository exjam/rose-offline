use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum BasicStatType {
    Strength,
    Dexterity,
    Intelligence,
    Concentration,
    Charm,
    Sense,
}

#[derive(Component, Clone, Debug, Deserialize, Serialize)]
pub struct BasicStats {
    pub strength: i32,
    pub dexterity: i32,
    pub intelligence: i32,
    pub concentration: i32,
    pub charm: i32,
    pub sense: i32,
}

impl Default for BasicStats {
    fn default() -> Self {
        Self {
            strength: 10,
            dexterity: 10,
            intelligence: 10,
            concentration: 10,
            charm: 10,
            sense: 10,
        }
    }
}
