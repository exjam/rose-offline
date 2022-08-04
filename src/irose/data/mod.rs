use std::sync::Arc;

use rose_data::{CharacterMotionDatabaseOptions, NpcDatabaseOptions};
use rose_data_irose::{
    get_ai_database, get_character_motion_database, get_data_decoder, get_item_database,
    get_npc_database, get_quest_database, get_skill_database, get_status_effect_database,
    get_warp_gate_database, get_zone_database,
};
use rose_file_readers::VirtualFilesystem;
use rose_game_irose::data::{get_ability_value_calculator, get_drop_table};

use crate::game::GameData;

mod character_creator;
use character_creator::get_character_creator;

pub fn get_game_data(virtual_filesystem: &VirtualFilesystem) -> GameData {
    let item_database =
        Arc::new(get_item_database(virtual_filesystem).expect("Failed to load item database"));
    let npc_database = Arc::new(
        get_npc_database(
            virtual_filesystem,
            &NpcDatabaseOptions {
                load_frame_data: true,
            },
        )
        .expect("Failed to load npc database"),
    );
    let skill_database =
        Arc::new(get_skill_database(virtual_filesystem).expect("Failed to load skill database"));
    let zone_database =
        Arc::new(get_zone_database(virtual_filesystem).expect("Failed to load zone database"));
    let drop_table = get_drop_table(
        virtual_filesystem,
        item_database.clone(),
        npc_database.clone(),
    )
    .expect("Failed to load drop table");

    GameData {
        character_creator: get_character_creator(
            virtual_filesystem,
            item_database.clone(),
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
        ai: Arc::new(get_ai_database(virtual_filesystem).expect("Failed to load AI database")),
        items: item_database,
        motions: Arc::new(
            get_character_motion_database(
                virtual_filesystem,
                &CharacterMotionDatabaseOptions {
                    load_frame_data: true,
                },
            )
            .expect("Failed to load motion database"),
        ),
        npcs: npc_database,
        quests: Arc::new(
            get_quest_database(virtual_filesystem).expect("Failed to load quest database"),
        ),
        skills: skill_database,
        status_effects: Arc::new(
            get_status_effect_database(virtual_filesystem)
                .expect("Failed to load status effect database"),
        ),
        warp_gates: Arc::new(
            get_warp_gate_database(virtual_filesystem).expect("Failed to load warp gate database"),
        ),
        zones: zone_database,
    }
}
