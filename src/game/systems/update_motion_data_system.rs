use bevy::{
    ecs::{prelude::Changed, query::WorldQuery, system::Query},
    prelude::{Entity, Or, Res},
};
use rose_data::{EquipmentIndex, VehiclePartIndex};

use crate::game::{
    components::{
        CharacterInfo, Equipment, HealthPoints, ManaPoints, MotionData, MoveMode, MoveSpeed, Npc,
    },
    GameData,
};

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct AbilityValuesChangedQuery<'w> {
    motion_data: &'w mut MotionData,
    health_points: &'w mut HealthPoints,
    mana_points: Option<&'w mut ManaPoints>,
    move_mode: &'w MoveMode,
    move_speed: &'w mut MoveSpeed,
}

pub fn update_character_motion_data_system(
    mut query: Query<
        (&CharacterInfo, &Equipment, &MoveMode, &mut MotionData),
        Or<(
            Changed<CharacterInfo>,
            Changed<Equipment>,
            Changed<MoveMode>,
        )>,
    >,
    game_data: Res<GameData>,
) {
    for (character_info, equipment, move_mode, mut motion_data) in query.iter_mut() {
        match &*motion_data {
            MotionData::Character(character_motion_data) => {
                let weapon_item_data = equipment
                    .get_equipment_item(EquipmentIndex::Weapon)
                    .and_then(|weapon_item| {
                        game_data
                            .items
                            .get_weapon_item(weapon_item.item.item_number)
                    });
                let weapon_motion_type = weapon_item_data
                    .map(|weapon_item_data| weapon_item_data.motion_type as usize)
                    .unwrap_or(0);
                let base_vehicle_motion_index = if matches!(move_mode, MoveMode::Drive) {
                    equipment.equipped_vehicle[VehiclePartIndex::Body]
                        .as_ref()
                        .and_then(|equipment_item| {
                            game_data
                                .items
                                .get_vehicle_item(equipment_item.item.item_number)
                        })
                        .map(|body_item_data| body_item_data.base_motion_index as usize)
                } else {
                    None
                };

                if character_motion_data.weapon_motion_type != weapon_motion_type
                    || character_motion_data.gender != character_info.gender
                    || character_motion_data.base_vehicle_motion_index != base_vehicle_motion_index
                {
                    if let Some(base_vehicle_motion_index) = base_vehicle_motion_index {
                        let weapon_motion_type = equipment.equipped_vehicle[VehiclePartIndex::Arms]
                            .as_ref()
                            .and_then(|equipment_item| {
                                game_data
                                    .items
                                    .get_vehicle_item(equipment_item.item.item_number)
                            })
                            .map_or(0, |vehicle_item_data| {
                                vehicle_item_data.base_motion_index as usize
                            });
                        *motion_data = MotionData::from_vehicle(
                            game_data.motions.as_ref(),
                            base_vehicle_motion_index,
                            weapon_motion_type,
                        );
                    } else {
                        *motion_data = MotionData::from_character(
                            game_data.motions.as_ref(),
                            weapon_motion_type,
                            character_info.gender,
                        );
                    }
                }
            }
            MotionData::Npc(_) => continue,
        }
    }
}

pub fn update_npc_motion_data_system(
    mut query: Query<(&Npc, &mut MotionData), Changed<Npc>>,
    game_data: Res<GameData>,
) {
    for (npc, mut motion_data) in query.iter_mut() {
        match &*motion_data {
            MotionData::Npc(npc_motion_data) => {
                if npc_motion_data.npc_id != npc.id {
                    *motion_data = MotionData::from_npc(game_data.npcs.as_ref(), npc.id);
                }
            }
            MotionData::Character(_) => continue,
        }
    }
}
