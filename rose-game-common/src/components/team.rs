use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    pub id: u32,
}

impl Team {
    pub const DEFAULT_NPC_TEAM_ID: u32 = 1;
    pub const DEFAULT_CHARACTER_TEAM_ID: u32 = 2;
    pub const DEFAULT_MONSTER_TEAM_ID: u32 = 100;
    pub const UNIQUE_TEAM_ID_BASE: u32 = 100;

    pub fn new(id: u32) -> Self {
        Self { id }
    }

    pub fn default_npc() -> Self {
        Self {
            id: Self::DEFAULT_NPC_TEAM_ID,
        }
    }

    pub fn default_character() -> Self {
        Self {
            id: Self::DEFAULT_CHARACTER_TEAM_ID,
        }
    }

    pub fn default_monster() -> Self {
        Self {
            id: Self::DEFAULT_MONSTER_TEAM_ID,
        }
    }

    pub fn with_unique_id(id: u32) -> Self {
        Self {
            id: Self::UNIQUE_TEAM_ID_BASE + id,
        }
    }
}
