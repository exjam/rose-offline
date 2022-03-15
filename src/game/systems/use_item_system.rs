use bevy_ecs::{
    prelude::{Commands, Entity, EventReader, Mut, Query, Res, ResMut},
    system::SystemParam,
};
use bevy_math::Vec3;
use log::warn;
use std::time::Duration;

use rose_data::{AbilityType, ItemClass, ItemType, SkillType};

use crate::game::{
    bundles::{
        ability_values_add_value, ability_values_get_value, client_entity_teleport_zone,
        skill_list_try_learn_skill,
    },
    components::{
        AbilityValues, BasicStats, CharacterInfo, ClientEntity, ClientEntitySector,
        ExperiencePoints, GameClient, Inventory, ItemSlot, Level, MoveSpeed, NextCommand, Position,
        SkillList, SkillPoints, Stamina, StatPoints, StatusEffects, StatusEffectsRegen, Team,
        UnionMembership,
    },
    events::UseItemEvent,
    messages::server::{ServerMessage, UseInventoryItem, UseItem},
    resources::{ClientEntityList, ServerMessages, ServerTime},
    GameData,
};

#[derive(SystemParam)]
pub struct UseItemSystemParameters<'w, 's> {
    commands: Commands<'w, 's>,
    game_data: Res<'w, GameData>,
    client_entity_list: ResMut<'w, ClientEntityList>,
    server_messages: ResMut<'w, ServerMessages>,
    server_time: Res<'w, ServerTime>,
}

struct UseItemUser<'a, 'world> {
    entity: Entity,
    ability_values: &'a AbilityValues,
    basic_stats: &'a mut Mut<'world, BasicStats>,
    character_info: &'a CharacterInfo,
    client_entity: &'a ClientEntity,
    client_entity_sector: &'a ClientEntitySector,
    experience_points: &'a ExperiencePoints,
    game_client: Option<&'a GameClient>,
    inventory: &'a mut Mut<'world, Inventory>,
    level: &'a Level,
    move_speed: &'a MoveSpeed,
    position: &'a Position,
    skill_list: &'a mut Mut<'world, SkillList>,
    skill_points: &'a mut Mut<'world, SkillPoints>,
    stamina: &'a mut Mut<'world, Stamina>,
    stat_points: &'a mut Mut<'world, StatPoints>,
    status_effects: &'a mut Mut<'world, StatusEffects>,
    status_effects_regen: &'a mut Mut<'world, StatusEffectsRegen>,
    team_number: &'a Team,
    union_membership: &'a mut Mut<'world, UnionMembership>,
}

enum UseItemError {
    InvalidItem,
    AbilityRequirement,
}

fn use_inventory_item(
    use_item_system_parameters: &mut UseItemSystemParameters,
    use_item_user: &mut UseItemUser,
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
            use_item_user.ability_values,
            use_item_user.level,
            use_item_user.move_speed,
            use_item_user.team_number,
            Some(use_item_user.character_info),
            Some(use_item_user.experience_points),
            Some(use_item_user.inventory),
            Some(use_item_user.skill_points),
            Some(use_item_user.stamina),
            Some(use_item_user.stat_points),
            Some(use_item_user.union_membership),
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
                                .send(ServerMessage::UpdateInventory(
                                    vec![(
                                        item_slot,
                                        use_item_user.inventory.get_item(item_slot).cloned(),
                                    )],
                                    None,
                                ))
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
                        use_item_system_parameters.game_data.skills.as_ref(),
                        skill_id,
                        use_item_user.skill_list,
                        Some(use_item_user.skill_points),
                        use_item_user.game_client,
                    )
                    .is_ok(),
                    false,
                )
            } else {
                (false, false)
            }
        }
        ItemClass::RepairTool | ItemClass::EngineFuel | ItemClass::TimeCoupon => {
            warn!(
                "Unimplemented use item ItemClass {:?} with item {:?}",
                item_data.item_data.class, item
            );
            (false, false)
        }
        _ => {
            if let Some((base_status_effect_id, total_potion_value)) = item_data.apply_status_effect
            {
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
                                use_item_user.status_effects_regen,
                                status_effect_data,
                                use_item_system_parameters.server_time.now
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
                    Some(use_item_user.basic_stats),
                    Some(use_item_user.inventory),
                    Some(use_item_user.skill_points),
                    Some(use_item_user.stamina),
                    Some(use_item_user.stat_points),
                    Some(use_item_user.union_membership),
                    use_item_user.game_client,
                );
            }

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
                        .send(ServerMessage::UpdateInventory(
                            vec![(item_slot, None)],
                            None,
                        ))
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
    mut query: Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &ClientEntitySector,
        &ExperiencePoints,
        &Level,
        &MoveSpeed,
        &Position,
        &Team,
        (
            &mut BasicStats,
            &mut Inventory,
            &mut SkillList,
            &mut SkillPoints,
            &mut Stamina,
            &mut StatPoints,
            &mut StatusEffects,
            &mut StatusEffectsRegen,
            &mut UnionMembership,
        ),
        Option<&GameClient>,
    )>,
    mut use_item_events: EventReader<UseItemEvent>,
) {
    for &UseItemEvent {
        entity,
        item_slot,
        target_entity,
    } in use_item_events.iter()
    {
        if let Ok((
            ability_values,
            character_info,
            client_entity,
            client_entity_sector,
            experience_points,
            level,
            move_speed,
            position,
            team_number,
            (
                mut basic_stats,
                mut inventory,
                mut skill_list,
                mut skill_points,
                mut stamina,
                mut stat_points,
                mut status_effects,
                mut status_effects_regen,
                mut union_membership,
            ),
            game_client,
        )) = query.get_mut(entity)
        {
            let mut use_item_user = UseItemUser {
                entity,
                ability_values,
                basic_stats: &mut basic_stats,
                character_info,
                client_entity,
                client_entity_sector,
                experience_points,
                inventory: &mut inventory,
                level,
                move_speed,
                position,
                skill_list: &mut skill_list,
                skill_points: &mut skill_points,
                stamina: &mut stamina,
                stat_points: &mut stat_points,
                status_effects: &mut status_effects,
                status_effects_regen: &mut status_effects_regen,
                team_number,
                union_membership: &mut union_membership,
                game_client,
            };

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
}
