use nalgebra::Point3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CharacterInfo {
    pub name: String,
    pub gender: u8,
    pub birth_stone: u8,
    pub job: u16,
    pub face: u8,
    pub hair: u8,
    pub revive_zone: u16,
    pub revive_position: Point3<f32>,
}
