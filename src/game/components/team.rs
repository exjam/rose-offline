#[derive(Clone)]
pub struct Team {
    pub id: u32,
}

impl Team {
    pub fn default_npc() -> Self {
        Self { id: 1 }
    }

    pub fn default_character() -> Self {
        Self { id: 2 }
    }

    pub fn default_monster() -> Self {
        Self { id: 100 }
    }
}
