mod ability_values;
mod ai_database;
mod character_creator;
mod item_database;
mod motion_database;
mod npc_database;
mod skill_database;
mod zone_database;

use ability_values::get_ability_value_calculator;
use ai_database::get_ai_database;
use character_creator::get_character_creator;
use item_database::get_item_database;
use motion_database::get_motion_database;
use npc_database::get_npc_database;
use skill_database::get_skill_database;
use zone_database::get_zone_database;

use crate::{data::formats::VfsIndex, game::GameData};
use std::{path::Path, sync::Arc};

pub fn get_game_data() -> GameData {
    let vfs_index = VfsIndex::load(&Path::new("data.idx")).expect("Failed reading data.idx");

    let item_database =
        Arc::new(get_item_database(&vfs_index).expect("Failed to load item database"));
    let npc_database = Arc::new(get_npc_database(&vfs_index).expect("Failed to load npc database"));
    let skill_database =
        Arc::new(get_skill_database(&vfs_index).expect("Failed to load skill database"));

    let ability_value_calculator = get_ability_value_calculator(
        item_database.clone(),
        skill_database.clone(),
        npc_database.clone(),
    )
    .expect("Failed to get ability value calculator");

    GameData {
        character_creator: get_character_creator(&vfs_index, &skill_database)
            .expect("Failed to get character creator"),
        ability_value_calculator,
        ai: Arc::new(get_ai_database(&vfs_index).expect("Failed to load AI database")),
        items: item_database,
        motions: Arc::new(get_motion_database(&vfs_index).expect("Failed to load motion database")),
        npcs: npc_database,
        skills: skill_database,
        zones: Arc::new(get_zone_database(&vfs_index).expect("Failed to load zone database")),
    }
}
