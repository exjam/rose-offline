use bevy::{ecs::prelude::Component, math::Vec3, reflect::Reflect};
use enum_map::Enum;
use serde::{Deserialize, Serialize};

use rose_data::ZoneId;

pub type CharacterUniqueId = u32;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Enum, PartialEq, Eq, Reflect)]
pub enum CharacterGender {
    Male,
    Female,
}

#[derive(Component, Clone, Debug, Deserialize, Serialize, Reflect)]
pub struct CharacterInfo {
    pub name: String,
    pub gender: CharacterGender,
    pub race: u8,
    pub birth_stone: u8,
    pub job: u16,
    pub face: u8,
    pub hair: u8,
    pub rank: u8,
    pub fame: u8,
    pub fame_b: u16,
    pub fame_g: u16,
    pub revive_zone_id: ZoneId,
    pub revive_position: Vec3,
    pub unique_id: CharacterUniqueId,
}
