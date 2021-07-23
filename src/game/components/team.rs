#[derive(Clone, Debug)]
pub struct Team {
    pub id: u32,
}

impl Team {
    pub const DEFAULT_NPC_TEAM_ID: u32 = 1;
    pub const DEFAULT_CHARACTER_TEAM_ID: u32 = 2;
    pub const DEFAULT_MONSTER_TEAM_ID: u32 = 100;

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
}
