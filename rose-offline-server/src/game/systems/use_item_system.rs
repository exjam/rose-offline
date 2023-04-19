use std::time::Duration;

use bevy::{
    ecs::{
        prelude::{Commands, Entity, EventReader, Query, Res, ResMut},
        query::WorldQuery,
        system::SystemParam,
    },
    math::Vec3,
    time::Time,
};
use log::warn;

use rose_data::{AbilityType, ItemClass, ItemType, SkillType, VehiclePartIndex};
use rose_game_common::components::{Equipment, HealthPoints, ManaPoints};

use crate::game::{
    bundles::{
        ability_values_add_value, ability_values_get_value, client_entity_teleport_zone,
        skill_list_try_learn_skill, SkillListBundle,
    },
    components::{
        AbilityValues, BasicStats, CharacterInfo, ClientEntity, ClientEntitySector,
        ExperiencePoints, GameClient, Inventory, ItemSlot, Level, MoveSpeed, NextCommand, Position,
        SkillList, SkillPoints, Stamina, StatPoints, StatusEffects, StatusEffectsRegen, Team,
        UnionMembership,
    },
    events::UseItemEvent,
    messages::server::{ServerMessage, UseInventoryItem, UseItem},
    resources::{ClientEntityList, ServerMessages},
    GameData,
};

#[derive(SystemParam)]
pub struct UseItemSystemParameters<'w, 's> {
    commands: Commands<'w, 's>,
    game_data: Res<'w, GameData>,
    client_entity_list: ResMut<'w, ClientEntityList>,
    server_messages: ResMut<'w, ServerMessages>,
    time: Res<'w, Time>,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct UseItemUserQuery<'w> {
    entity: Entity,
    ability_values: &'w AbilityValues,
    basic_stats: &'w mut BasicStats,
    character_info: &'w CharacterInfo,
    client_entity: &'w ClientEntity,
    client_entity_sector: &'w ClientEntitySector,
    experience_points: &'w mut ExperiencePoints,
    equipment: &'w mut Equipment,
    game_client: Option<&'w GameClient>,
    health_points: &'w mut HealthPoints,
    inventory: &'w mut Inventory,
    level: &'w Level,
    mana_points: &'w mut ManaPoints,
    move_speed: &'w MoveSpeed,
    position: &'w Position,
    skill_list: &'w mut SkillList,
    skill_points: &'w mut SkillPoints,
    stamina: &'w mut Stamina,
    stat_points: &'w mut StatPoints,
    status_effects: &'w mut StatusEffects,
    status_effects_regen: &'w mut StatusEffectsRegen,
    team: &'w Team,
    union_membership: &'w mut UnionMembership,
}

enum UseItemError {
    InvalidItem,
    AbilityRequirement,
}

fn apply_item_effect(
    use_item_system_parameters: &UseItemSystemParameters,
    use_item_user: &mut UseItemUserQueryItem,
    item_data: &rose_data::ConsumableItemData,
) {
    if let Some((base_status_effect_id, total_potion_value)) = item_data.apply_status_effect {
        if let Some(base_status_effect) = use_item_system_parameters
            .game_data
            .status_effects
            .get_status_effect(base_status_effect_id)
        {
            for (status_effect_data, &potion_value_per_second) in base_status_effect
                .apply_status_effects
                .iter()
                .filter_map(|(id, value)| {
                    use_item_system_parameters
                        .game_data
                        .status_effects
                        .get_status_effect(*id)
                        .map(|data| (data, value))
                })
            {
                if use_item_user
                    .status_effects
                    .can_apply(status_effect_data, status_effect_data.id.get() as i32)
                {
                    use_item_user.status_effects.apply_potion(
                        &mut use_item_user.status_effects_regen,
                        status_effect_data,
                        use_item_system_parameters.time.last_update().unwrap()
                            + Duration::from_micros(
                                total_potion_value as u64 * 1000000
                                    / potion_value_per_second as u64,
                            ),
                        total_potion_value,
                        potion_value_per_second,
                    );
                }
            }
        }
    } else if let Some((add_ability_type, add_ability_value)) = item_data.add_ability {
        ability_values_add_value(
            add_ability_type,
            add_ability_value,
            Some(use_item_user.ability_values),
            Some(&mut use_item_user.basic_stats),
            Some(&mut use_item_user.experience_points),
            Some(&mut use_item_user.health_points),
            Some(&mut use_item_user.inventory),
            Some(&mut use_item_user.mana_points),
            Some(&mut use_item_user.skill_points),
            Some(&mut use_item_user.stamina),
            Some(&mut use_item_user.stat_points),
            Some(&mut use_item_user.union_membership),
            use_item_user.game_client,
        );
    }
}

fn use_inventory_item(
    use_item_system_parameters: &mut UseItemSystemParameters,
    use_item_user: &mut UseItemUserQueryItem,
    item_slot: ItemSlot,
    target_entity: Option<Entity>,
    _repair_item_slot: Option<ItemSlot>,
) -> Result<(), UseItemError> {
    let item = use_item_user
        .inventory
        .get_item(item_slot)
        .ok_or(UseItemError::InvalidItem)?;

    if item.get_item_type() != ItemType::Consumable {
        return Err(UseItemError::InvalidItem);
    }

    let item_data = use_item_system_parameters
        .game_data
        .items
        .get_consumable_item(item.get_item_number())
        .ok_or(UseItemError::InvalidItem)?;

    // TODO: Check use item cooldown

    if let Some((require_ability_type, require_ability_value)) = item_data.ability_requirement {
        let ability_value = ability_values_get_value(
            require_ability_type,
            Some(use_item_user.ability_values),
            Some(use_item_user.level),
            Some(use_item_user.move_speed),
            Some(use_item_user.team),
            Some(use_item_user.character_info),
            Some(&use_item_user.experience_points),
            Some(&use_item_user.inventory),
            Some(&use_item_user.skill_points),
            Some(&use_item_user.stamina),
            Some(&use_item_user.stat_points),
            Some(&use_item_user.union_membership),
            Some(&use_item_user.health_points),
            Some(&use_item_user.mana_points),
        )
        .unwrap_or(0);

        // For planet we compare with !=, everything else we compare with <
        if matches!(require_ability_type, AbilityType::CurrentPlanet) {
            if ability_value != require_ability_value {
                return Err(UseItemError::AbilityRequirement);
            }
        } else if ability_value < require_ability_value {
            return Err(UseItemError::AbilityRequirement);
        }
    }

    let item = use_item_user
        .inventory
        .try_take_quantity(item_slot, 1)
        .ok_or(UseItemError::InvalidItem)?;

    let (consume_item, message_to_nearby) = match item_data.item_data.class {
        ItemClass::MagicItem => {
            if let Some((skill_id, skill_data)) = item_data.use_skill_id.and_then(|skill_id| {
                use_item_system_parameters
                    .game_data
                    .skills
                    .get_skill(skill_id)
                    .map(|skill_data| (skill_id, skill_data))
            }) {
                if skill_data.skill_type.is_self_skill() {
                    use_item_system_parameters
                        .commands
                        .entity(use_item_user.entity)
                        .insert(NextCommand::with_cast_skill_target_self(
                            skill_id,
                            Some((item_slot, item.clone())),
                        ));
                    (false, false)
                } else if skill_data.skill_type.is_target_skill() && target_entity.is_some() {
                    use_item_system_parameters
                        .commands
                        .entity(use_item_user.entity)
                        .insert(NextCommand::with_cast_skill_target_entity(
                            skill_id,
                            target_entity.unwrap(),
                            Some((item_slot, item.clone())),
                        ));
                    (false, false)
                } else if matches!(skill_data.skill_type, SkillType::Warp) {
                    if let Some(zone_id) = skill_data.warp_zone_id {
                        // TODO: Check skill_data.required_planet

                        // We need to send an update inventory packet before teleporting, otherwise it is lost
                        if let Some(game_client) = use_item_user.game_client {
                            game_client
                                .server_message_tx
                                .send(ServerMessage::UpdateInventory {
                                    items: vec![(
                                        item_slot,
                                        use_item_user.inventory.get_item(item_slot).cloned(),
                                    )],
                                    money: None,
                                })
                                .ok();
                        }

                        client_entity_teleport_zone(
                            &mut use_item_system_parameters.commands,
                            &mut use_item_system_parameters.client_entity_list,
                            use_item_user.entity,
                            use_item_user.client_entity,
                            use_item_user.client_entity_sector,
                            use_item_user.position,
                            Position::new(
                                Vec3::new(skill_data.warp_zone_x, skill_data.warp_zone_y, 0.0),
                                zone_id,
                            ),
                            use_item_user.game_client,
                        );
                    }
                    (true, false)
                } else {
                    (false, false)
                }
            } else {
                (false, false)
            }
        }
        ItemClass::SkillBook => {
            if let Some(skill_id) = item_data.learn_skill_id {
                (
                    skill_list_try_learn_skill(
                        &use_item_system_parameters.game_data,
                        &mut SkillListBundle {
                            skill_list: &mut use_item_user.skill_list,
                            skill_points: Some(&mut use_item_user.skill_points),
                            game_client: use_item_user.game_client,
                            ability_values: use_item_user.ability_values,
                            level: use_item_user.level,
                            move_speed: Some(use_item_user.move_speed),
                            team: Some(use_item_user.team),
                            character_info: Some(use_item_user.character_info),
                            experience_points: Some(&use_item_user.experience_points),
                            inventory: Some(&use_item_user.inventory),
                            stamina: Some(&use_item_user.stamina),
                            stat_points: Some(&use_item_user.stat_points),
                            union_membership: Some(&use_item_user.union_membership),
                            health_points: Some(&use_item_user.health_points),
                            mana_points: Some(&use_item_user.mana_points),
                        },
                        skill_id,
                    )
                    .is_ok(),
                    false,
                )
            } else {
                (false, false)
            }
        }
        ItemClass::EngineFuel => {
            if let Some(engine_item) = use_item_user
                .equipment
                .get_vehicle_item_mut(VehiclePartIndex::Engine)
            {
                engine_item.life = engine_item
                    .life
                    .saturating_add(item_data.add_fuel as u16 * 10)
                    .min(1000);

                if let Some(game_client) = use_item_user.game_client {
                    game_client
                        .server_message_tx
                        .send(ServerMessage::UpdateItemLife {
                            item_slot: ItemSlot::Vehicle(VehiclePartIndex::Engine),
                            life: engine_item.life,
                        })
                        .ok();
                }

                (true, false)
            } else {
                (false, false)
            }
        }
        ItemClass::RepairTool | ItemClass::TimeCoupon => {
            warn!(
                "Unimplemented use item ItemClass {:?} with item {:?}",
                item_data.item_data.class, item
            );
            (false, false)
        }
        _ => {
            apply_item_effect(use_item_system_parameters, use_item_user, item_data);
            (true, true)
        }
    };

    if consume_item {
        if let Some(game_client) = use_item_user.game_client {
            if message_to_nearby {
                use_item_system_parameters
                    .server_messages
                    .send_entity_message(
                        use_item_user.client_entity,
                        ServerMessage::UseItem(UseItem {
                            entity_id: use_item_user.client_entity.id,
                            item: item.get_item_reference(),
                        }),
                    );
            }

            match use_item_user.inventory.get_item(item_slot) {
                None => {
                    // When the item has been fully consumed we send UpdateInventory packet
                    game_client
                        .server_message_tx
                        .send(ServerMessage::UpdateInventory {
                            items: vec![(item_slot, None)],
                            money: None,
                        })
                        .ok();
                }
                Some(item) => {
                    // When there is still remaining quantity we send UseInventoryItem packet
                    game_client
                        .server_message_tx
                        .send(ServerMessage::UseInventoryItem(UseInventoryItem {
                            entity_id: use_item_user.client_entity.id,
                            item: item.get_item_reference(),
                            inventory_slot: item_slot,
                        }))
                        .ok();
                }
            }
        }
    } else {
        use_item_user
            .inventory
            .try_stack_with_item(item_slot, item)
            .expect("Unexpected error returning unconsumed item to inventory");
    }

    Ok(())
}

pub fn use_item_system(
    mut use_item_system_parameters: UseItemSystemParameters,
    mut query_user: Query<UseItemUserQuery>,
    mut use_item_events: EventReader<UseItemEvent>,
) {
    for event in use_item_events.iter() {
        match *event {
            UseItemEvent::Inventory {
                entity,
                item_slot,
                target_entity,
            } => {
                if let Ok(mut use_item_user) = query_user.get_mut(entity) {
                    use_inventory_item(
                        &mut use_item_system_parameters,
                        &mut use_item_user,
                        item_slot,
                        target_entity,
                        None, // TODO: Support repair item use
                    )
                    .ok();
                }
            }
            UseItemEvent::Item { entity, ref item } => {
                if let Ok(mut use_item_user) = query_user.get_mut(entity) {
                    if let Some(item_data) = use_item_system_parameters
                        .game_data
                        .items
                        .get_consumable_item(item.get_item_number())
                    {
                        apply_item_effect(
                            &use_item_system_parameters,
                            &mut use_item_user,
                            item_data,
                        );

                        use_item_system_parameters
                            .server_messages
                            .send_entity_message(
                                use_item_user.client_entity,
                                ServerMessage::UseItem(UseItem {
                                    entity_id: use_item_user.client_entity.id,
                                    item: item.get_item_reference(),
                                }),
                            );
                    }
                }
            }
        }
    }
}
