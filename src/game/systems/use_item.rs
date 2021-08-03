use bevy_ecs::prelude::{Commands, Entity, Mut, Query, Res, ResMut};
use log::warn;

use crate::{
    data::{
        item::{ItemClass, ItemType},
        AbilityType,
    },
    game::{
        bundles::{
            ability_values_add_value, ability_values_get_value,
            client_entity_recalculate_ability_values, skill_list_try_learn_skill,
        },
        components::{
            AbilityValues, BasicStats, CharacterInfo, ClientEntity, Equipment, ExperiencePoints,
            GameClient, HealthPoints, Inventory, ItemSlot, Level, ManaPoints, MoveMode, MoveSpeed,
            SkillList, SkillPoints, Stamina, StatPoints, StatusEffects, Team, UnionMembership,
        },
        messages::server::{ServerMessage, UpdateInventory, UseItem},
        resources::{PendingUseItem, PendingUseItemList, ServerMessages},
        GameData,
    },
};

struct UseItemWorld<'a, 'b, 'c, 'd, 'e> {
    pub commands: &'a mut Commands<'b>,
    pub game_data: &'c GameData,
    pub server_messages: &'d mut ResMut<'e, ServerMessages>,
}

struct UseItemUser<'a, 'world> {
    pub entity: Entity,
    pub ability_values: &'a AbilityValues,
    pub basic_stats: &'a mut Mut<'world, BasicStats>,
    pub character_info: &'a CharacterInfo,
    pub client_entity: &'a ClientEntity,
    pub experience_points: &'a ExperiencePoints,
    pub game_client: Option<&'a GameClient>,
    pub equipment: &'a Equipment,
    pub health_points: &'a mut Mut<'world, HealthPoints>,
    pub inventory: &'a mut Mut<'world, Inventory>,
    pub level: &'a Level,
    pub mana_points: &'a mut Mut<'world, ManaPoints>,
    pub move_mode: &'a MoveMode,
    pub move_speed: &'a MoveSpeed,
    pub skill_list: &'a mut Mut<'world, SkillList>,
    pub skill_points: &'a mut Mut<'world, SkillPoints>,
    pub stamina: &'a mut Mut<'world, Stamina>,
    pub stat_points: &'a mut Mut<'world, StatPoints>,
    pub status_effects: &'a StatusEffects,
    pub team_number: &'a Team,
    pub union_membership: &'a mut Mut<'world, UnionMembership>,
}

enum UseItemError {
    InvalidItem,
    AbilityRequirement,
}

fn use_inventory_item(
    use_item_world: &mut UseItemWorld,
    use_item_user: &mut UseItemUser,
    item_slot: ItemSlot,
    _target_entity: Option<Entity>,
    _repair_item_slot: Option<ItemSlot>,
) -> Result<(), UseItemError> {
    let item = use_item_user
        .inventory
        .get_item(item_slot)
        .ok_or(UseItemError::InvalidItem)?;

    if item.get_item_type() != ItemType::Consumable {
        return Err(UseItemError::InvalidItem);
    }

    let item_data = use_item_world
        .game_data
        .items
        .get_consumable_item(item.get_item_number())
        .ok_or(UseItemError::InvalidItem)?;

    // TODO: Check use item cooldown

    if let Some((require_ability_type, require_ability_value)) = item_data.ability_requirement {
        let ability_value = ability_values_get_value(
            require_ability_type,
            (use_item_user.ability_values, use_item_user.status_effects),
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
        /*ItemClass::MagicItem => {
            let _skill = item_data.use_skill_id;
            // TODO: Use skill skill_index
            // TODO: Set NextCommand to UseSkill(UseSkill::ItemSelf(item_slot)) / UseSkill(UseSkill::ItemTarget(item_slot))
            (false, false)
        }*/
        ItemClass::SkillBook => {
            if let Some(skill_id) = item_data.learn_skill_id {
                (
                    skill_list_try_learn_skill(
                        use_item_world.game_data.skills.as_ref(),
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
        ItemClass::MagicItem
        | ItemClass::RepairTool
        | ItemClass::EngineFuel
        | ItemClass::TimeCoupon => {
            warn!(
                "Unimplemented use item ItemClass {:?} with item {:?}",
                item_data.item_data.class, item
            );
            (false, false)
        }
        _ => {
            if let Some(_apply_status_effect_id) = item_data.apply_status_effect_id {
                // TODO: Implement status effects
            }

            if let Some((add_ability_type, add_ability_value)) = item_data.add_ability {
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
        client_entity_recalculate_ability_values(
            use_item_world.commands,
            use_item_world.game_data.ability_value_calculator.as_ref(),
            use_item_user.client_entity,
            use_item_user.entity,
            use_item_user.status_effects,
            Some(use_item_user.basic_stats),
            Some(use_item_user.character_info),
            Some(use_item_user.equipment),
            Some(use_item_user.level),
            Some(use_item_user.move_mode),
            Some(use_item_user.skill_list),
            None,
            Some(use_item_user.health_points),
            Some(use_item_user.mana_points),
        );

        if let Some(game_client) = use_item_user.game_client {
            if message_to_nearby {
                use_item_world.server_messages.send_entity_message(
                    use_item_user.client_entity,
                    ServerMessage::UseItem(UseItem {
                        entity_id: use_item_user.client_entity.id,
                        item: item.get_item_reference(),
                        inventory_slot: item_slot,
                    }),
                );
            }

            match use_item_user.inventory.get_item(item_slot) {
                None => {
                    // When the item has been fully consumed we send UpdateInventory packet
                    game_client
                        .server_message_tx
                        .send(ServerMessage::UpdateInventory(UpdateInventory {
                            is_reward: false,
                            items: vec![(item_slot, None)],
                        }))
                        .ok();
                }
                Some(item) => {
                    // When there is still remaining quantity we send UseItem packet
                    if !message_to_nearby {
                        game_client
                            .server_message_tx
                            .send(ServerMessage::UseItem(UseItem {
                                entity_id: use_item_user.client_entity.id,
                                item: item.get_item_reference(),
                                inventory_slot: item_slot,
                            }))
                            .ok();
                    }
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

#[allow(clippy::type_complexity)]
pub fn use_item_system(
    mut commands: Commands,
    mut query: Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &ExperiencePoints,
        &Equipment,
        &Level,
        &MoveMode,
        &MoveSpeed,
        &StatusEffects,
        &Team,
        (
            &mut BasicStats,
            &mut HealthPoints,
            &mut Inventory,
            &mut ManaPoints,
            &mut SkillList,
            &mut SkillPoints,
            &mut Stamina,
            &mut StatPoints,
            &mut UnionMembership,
        ),
        Option<&GameClient>,
    )>,
    game_data: Res<GameData>,
    mut pending_use_item_list: ResMut<PendingUseItemList>,
    mut server_messages: ResMut<ServerMessages>,
) {
    let mut use_item_world = UseItemWorld {
        commands: &mut commands,
        game_data: &game_data,
        server_messages: &mut server_messages,
    };

    for PendingUseItem {
        entity,
        item_slot,
        target_entity,
    } in pending_use_item_list.drain(..)
    {
        if let Ok((
            ability_values,
            character_info,
            client_entity,
            experience_points,
            equipment,
            level,
            move_mode,
            move_speed,
            status_effects,
            team_number,
            (
                mut basic_stats,
                mut health_points,
                mut inventory,
                mut mana_points,
                mut skill_list,
                mut skill_points,
                mut stamina,
                mut stat_points,
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
                experience_points,
                equipment,
                health_points: &mut health_points,
                inventory: &mut inventory,
                level,
                mana_points: &mut mana_points,
                move_mode,
                move_speed,
                skill_list: &mut skill_list,
                skill_points: &mut skill_points,
                stamina: &mut stamina,
                stat_points: &mut stat_points,
                status_effects,
                team_number,
                union_membership: &mut union_membership,
                game_client,
            };

            use_inventory_item(
                &mut use_item_world,
                &mut use_item_user,
                item_slot,
                target_entity,
                None, // TODO: Support repair item use
            )
            .ok();
        }
    }
}
