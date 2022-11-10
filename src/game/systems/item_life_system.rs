use bevy::prelude::{EventReader, Query, Res};
use rose_data::VehiclePartIndex;
use rose_game_common::{components::ItemSlot, messages::server::ServerMessage};

use crate::game::{
    components::{AbilityValues, Equipment, GameClient},
    events::ItemLifeEvent,
    GameData,
};

pub fn item_life_system(
    mut item_life_events: EventReader<ItemLifeEvent>,
    mut query: Query<(&AbilityValues, &mut Equipment, Option<&GameClient>)>,
    game_data: Res<GameData>,
) {
    for event in item_life_events.iter() {
        match *event {
            ItemLifeEvent::DecreaseWeaponLife(entity) => {
                if let Ok((ability_values, mut equipment, game_client)) = query.get_mut(entity) {
                    if let Some(item_slot) = game_data
                        .ability_value_calculator
                        .calculate_decrease_weapon_life(
                            ability_values.is_driving,
                            equipment.as_ref(),
                        )
                    {
                        let equipment_slot = match item_slot {
                            ItemSlot::Equipment(index) => equipment.get_equipment_slot_mut(index),
                            ItemSlot::Vehicle(index) => equipment.get_vehicle_slot_mut(index),
                            _ => continue,
                        };

                        if let Some(equipment_item) = equipment_slot.as_mut() {
                            if equipment_item.life >= 1 {
                                equipment_item.life -= 1;

                                if let Some(game_client) = game_client {
                                    game_client
                                        .server_message_tx
                                        .send(ServerMessage::UpdateItemLife {
                                            item_slot,
                                            life: equipment_item.life,
                                        })
                                        .ok();
                                }
                            }
                        }
                    }
                }
            }
            ItemLifeEvent::DecreaseArmourLife(entity, damage) => {
                if let Ok((ability_values, mut equipment, game_client)) = query.get_mut(entity) {
                    if let Some(item_slot) = game_data
                        .ability_value_calculator
                        .calculate_decrease_armour_life(
                            ability_values.is_driving,
                            equipment.as_ref(),
                            &damage,
                        )
                    {
                        let equipment_slot = match item_slot {
                            ItemSlot::Equipment(index) => equipment.get_equipment_slot_mut(index),
                            ItemSlot::Vehicle(index) => equipment.get_vehicle_slot_mut(index),
                            _ => continue,
                        };

                        if let Some(equipment_item) = equipment_slot.as_mut() {
                            if equipment_item.life >= 1 {
                                equipment_item.life -= 1;

                                if let Some(game_client) = game_client {
                                    game_client
                                        .server_message_tx
                                        .send(ServerMessage::UpdateItemLife {
                                            item_slot,
                                            life: equipment_item.life,
                                        })
                                        .ok();
                                }
                            }
                        }
                    }
                }
            }
            ItemLifeEvent::DecreaseVehicleEngineLife(entity) => {
                if let Ok((_, mut equipment, game_client)) = query.get_mut(entity) {
                    let equipment_slot = equipment.get_vehicle_slot_mut(VehiclePartIndex::Engine);

                    if let Some(engine_item) = equipment_slot.as_mut() {
                        if let Some(item_data) = game_data
                            .items
                            .get_vehicle_item(engine_item.item.item_number)
                        {
                            if engine_item.life > 0 {
                                engine_item.life = engine_item
                                    .life
                                    .saturating_sub(item_data.fuel_use_rate as u16);

                                if let Some(game_client) = game_client {
                                    game_client
                                        .server_message_tx
                                        .send(ServerMessage::UpdateItemLife {
                                            item_slot: ItemSlot::Vehicle(VehiclePartIndex::Engine),
                                            life: engine_item.life,
                                        })
                                        .ok();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
