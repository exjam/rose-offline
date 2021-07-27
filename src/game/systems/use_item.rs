use legion::{system, systems::CommandBuffer, world::SubWorld, Entity, Query};
use log::warn;

use crate::{
    data::{
        ability::AbilityType,
        item::{ItemClass, ItemType},
        SkillReference,
    },
    game::{
        bundles::{ability_values_add_value, ability_values_get_value, skill_list_try_learn_skill},
        components::{
            AbilityValues, BasicStats, CharacterInfo, ClientEntity, Equipment, ExperiencePoints,
            GameClient, Inventory, ItemSlot, Level, MoveSpeed, SkillList, SkillPoints, Stamina,
            StatPoints, Team, UnionMembership,
        },
        messages::server::{ServerMessage, UpdateInventory, UseItem},
        resources::{PendingUseItem, PendingUseItemList, ServerMessages},
        GameData,
    },
};

struct UseItemWorld<'a> {
    pub cmd: &'a mut CommandBuffer,
    pub game_data: &'a GameData,
    pub server_messages: &'a mut ServerMessages,
}

struct UseItemUser<'a> {
    pub entity: &'a Entity,
    pub ability_values: &'a AbilityValues,
    pub basic_stats: &'a mut BasicStats,
    pub character_info: &'a CharacterInfo,
    pub client_entity: &'a ClientEntity,
    pub experience_points: &'a ExperiencePoints,
    pub game_client: Option<&'a GameClient>,
    pub equipment: &'a Equipment,
    pub inventory: &'a mut Inventory,
    pub level: &'a Level,
    pub move_speed: &'a MoveSpeed,
    pub skill_list: &'a mut SkillList,
    pub skill_points: &'a mut SkillPoints,
    pub stamina: &'a mut Stamina,
    pub stat_points: &'a mut StatPoints,
    pub team_number: &'a Team,
    pub union_membership: &'a mut UnionMembership,
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
            Some(use_item_user.ability_values),
            Some(use_item_user.character_info),
            Some(use_item_user.experience_points),
            Some(use_item_user.inventory),
            Some(use_item_user.level),
            Some(use_item_user.move_speed),
            Some(use_item_user.skill_points),
            Some(use_item_user.stamina),
            Some(use_item_user.stat_points),
            Some(use_item_user.team_number),
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
        ItemClass::SkillBook => (
            skill_list_try_learn_skill(
                use_item_world.game_data.skills.as_ref(),
                SkillReference(item_data.learn_skill_id),
                use_item_user.skill_list,
                Some(use_item_user.skill_points),
                use_item_user.game_client,
            )
            .is_ok(),
            false,
        ),
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
        use_item_world.cmd.add_component(
            *use_item_user.entity,
            use_item_world.game_data.ability_value_calculator.calculate(
                use_item_user.character_info,
                use_item_user.level,
                use_item_user.equipment,
                use_item_user.inventory,
                use_item_user.basic_stats,
                use_item_user.skill_list,
            ),
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
#[system]
pub fn use_item(
    cmd: &mut CommandBuffer,
    world: &mut SubWorld,
    entity_query: &mut Query<(
        &AbilityValues,
        &mut BasicStats,
        &CharacterInfo,
        &ClientEntity,
        &ExperiencePoints,
        &Equipment,
        &mut Inventory,
        &Level,
        &MoveSpeed,
        &mut SkillList,
        &mut SkillPoints,
        &mut Stamina,
        &mut StatPoints,
        &Team,
        &mut UnionMembership,
        Option<&GameClient>,
    )>,
    #[resource] game_data: &GameData,
    #[resource] pending_use_item_list: &mut PendingUseItemList,
    #[resource] server_messages: &mut ServerMessages,
) {
    let mut use_item_world = UseItemWorld {
        cmd,
        game_data,
        server_messages,
    };

    for PendingUseItem {
        entity,
        item_slot,
        target_entity,
    } in pending_use_item_list.drain(..)
    {
        if let Ok((
            ability_values,
            basic_stats,
            character_info,
            client_entity,
            experience_points,
            equipment,
            inventory,
            level,
            move_speed,
            skill_list,
            skill_points,
            stamina,
            stat_points,
            team_number,
            union_membership,
            game_client,
        )) = entity_query.get_mut(world, entity)
        {
            let mut use_item_user = UseItemUser {
                entity: &entity,
                ability_values,
                basic_stats,
                character_info,
                client_entity,
                experience_points,
                equipment,
                inventory,
                level,
                move_speed,
                skill_list,
                skill_points,
                stamina,
                stat_points,
                team_number,
                union_membership,
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
