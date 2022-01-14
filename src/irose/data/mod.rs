use crate::{
    data::{formats::VfsIndex, AbilityType},
    game::GameData,
};
use std::{path::Path, str::FromStr, sync::Arc};

impl FromStr for AbilityType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u32>().map_err(|_| ())?;
        FromPrimitive::from_u32(value).ok_or(())
    }
}

mod ability_values;
mod ai_database;
mod character_creator;
mod drop_table;
mod item_database;
mod motion_database;
mod npc_database;
mod quest_database;
mod skill_database;
mod status_effect_database;
mod warp_gate_database;
mod zone_database;

use ability_values::get_ability_value_calculator;
use ai_database::get_ai_database;
use character_creator::get_character_creator;
use drop_table::get_drop_table;
use item_database::get_item_database;
use motion_database::get_motion_database;
use npc_database::get_npc_database;
use num_traits::FromPrimitive;
use quest_database::get_quest_database;
use skill_database::get_skill_database;
use status_effect_database::get_status_effect_database;
use warp_gate_database::get_warp_gate_database;
use zone_database::get_zone_database;

pub fn get_game_data(data_idx_path: &Path) -> GameData {
    let vfs_index = VfsIndex::load(data_idx_path).expect("Failed reading data.idx");

    let item_database =
        Arc::new(get_item_database(&vfs_index).expect("Failed to load item database"));
    let npc_database = Arc::new(get_npc_database(&vfs_index).expect("Failed to load npc database"));
    let skill_database =
        Arc::new(get_skill_database(&vfs_index).expect("Failed to load skill database"));
    let zone_database =
        Arc::new(get_zone_database(&vfs_index).expect("Failed to load zone database"));
    let drop_table = get_drop_table(&vfs_index, item_database.clone(), npc_database.clone())
        .expect("Failed to load drop table");

    let ability_value_calculator = get_ability_value_calculator(
        item_database.clone(),
        skill_database.clone(),
        npc_database.clone(),
    )
    .expect("Failed to get ability value calculator");

    GameData {
        character_creator: get_character_creator(
            &vfs_index,
            skill_database.clone(),
            &zone_database,
        )
        .expect("Failed to get character creator"),
        ability_value_calculator,
        drop_table,
        ai: Arc::new(get_ai_database(&vfs_index).expect("Failed to load AI database")),
        items: item_database,
        motions: Arc::new(get_motion_database(&vfs_index).expect("Failed to load motion database")),
        npcs: npc_database,
        quests: Arc::new(get_quest_database(&vfs_index).expect("Failed to load quest database")),
        skills: skill_database,
        status_effects: Arc::new(
            get_status_effect_database(&vfs_index).expect("Failed to load status effect database"),
        ),
        warp_gates: Arc::new(
            get_warp_gate_database(&vfs_index).expect("Failed to load warp gate database"),
        ),
        zones: zone_database,
    }
}
