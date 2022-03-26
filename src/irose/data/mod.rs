use std::{path::Path, sync::Arc};

use rose_data::{CharacterMotionDatabaseOptions, NpcDatabaseOptions};
use rose_data_irose::{
    get_ai_database, get_character_motion_database, get_data_decoder, get_item_database,
    get_npc_database, get_quest_database, get_skill_database, get_status_effect_database,
    get_warp_gate_database, get_zone_database,
};
use rose_file_readers::VfsIndex;
use rose_game_irose::data::{get_ability_value_calculator, get_drop_table};

use crate::game::GameData;

mod character_creator;
use character_creator::get_character_creator;

pub fn get_game_data(data_idx_path: Option<&Path>, data_extracted_path: Option<&Path>) -> GameData {
    log::info!(
        "Loading irose game data from {}",
        &data_idx_path
            .unwrap_or_else(|| data_extracted_path.unwrap())
            .to_string_lossy()
    );

    let vfs_index =
        VfsIndex::with_paths(data_idx_path, data_extracted_path).expect("Failed to initialise VFS");

    let item_database =
        Arc::new(get_item_database(&vfs_index).expect("Failed to load item database"));
    let npc_database = Arc::new(
        get_npc_database(
            &vfs_index,
            &NpcDatabaseOptions {
                load_frame_data: true,
            },
        )
        .expect("Failed to load npc database"),
    );
    let skill_database =
        Arc::new(get_skill_database(&vfs_index).expect("Failed to load skill database"));
    let zone_database =
        Arc::new(get_zone_database(&vfs_index).expect("Failed to load zone database"));
    let drop_table = get_drop_table(&vfs_index, item_database.clone(), npc_database.clone())
        .expect("Failed to load drop table");

    GameData {
        character_creator: get_character_creator(
            &vfs_index,
            skill_database.clone(),
            &zone_database,
        )
        .expect("Failed to get character creator"),
        ability_value_calculator: get_ability_value_calculator(
            item_database.clone(),
            skill_database.clone(),
            npc_database.clone(),
        ),
        data_decoder: get_data_decoder(),
        drop_table,
        ai: Arc::new(get_ai_database(&vfs_index).expect("Failed to load AI database")),
        items: item_database,
        motions: Arc::new(
            get_character_motion_database(
                &vfs_index,
                &CharacterMotionDatabaseOptions {
                    load_frame_data: true,
                },
            )
            .expect("Failed to load motion database"),
        ),
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
