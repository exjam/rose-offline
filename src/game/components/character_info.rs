use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct CharacterInfo {
    pub name: String,
    pub gender: u8,
    pub birth_stone: u8,
    pub job: u16,
    pub face: u8,
    pub hair: u8,
}
