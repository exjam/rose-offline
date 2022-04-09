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

impl BasicStats {
    pub fn get(&self, basic_stat_type: BasicStatType) -> i32 {
        match basic_stat_type {
            BasicStatType::Strength => self.strength,
            BasicStatType::Dexterity => self.dexterity,
            BasicStatType::Intelligence => self.intelligence,
            BasicStatType::Concentration => self.concentration,
            BasicStatType::Charm => self.charm,
            BasicStatType::Sense => self.sense,
        }
    }

    pub fn set(&mut self, basic_stat_type: BasicStatType, value: i32) {
        match basic_stat_type {
            BasicStatType::Strength => self.strength = value,
            BasicStatType::Dexterity => self.dexterity = value,
            BasicStatType::Intelligence => self.intelligence = value,
            BasicStatType::Concentration => self.concentration = value,
            BasicStatType::Charm => self.charm = value,
            BasicStatType::Sense => self.sense = value,
        }
    }
}
