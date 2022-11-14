use bevy::prelude::Resource;

#[derive(Resource)]
pub struct GameConfig {
    pub enable_npc_spawns: bool,
    pub enable_monster_spawns: bool,
}

impl GameConfig {
    pub fn default() -> Self {
        Self {
            enable_monster_spawns: true,
            enable_npc_spawns: true,
        }
    }
}
