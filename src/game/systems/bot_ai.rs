use legion::{systems::CommandBuffer, world::SubWorld, Entity, Query};
use rand::seq::SliceRandom;

use crate::game::{
    components::{
        BotAi, BotAiState, Command, CommandData, CommandDie, Destination, DroppedItem, Inventory,
        InventoryPageType, NextCommand, Npc, Owner, Position, Team,
    },
    resources::ClientEntityList,
    GameData,
};

#[allow(clippy::type_complexity)]
#[legion::system]
pub fn bot_ai(
    world: &mut SubWorld,
    cmd: &mut CommandBuffer,
    bot_query: &mut Query<(
        Entity,
        &mut BotAi,
        &Command,
        &NextCommand,
        &Inventory,
        &Position,
        &Owner,
        &Team,
    )>,
    owner_query: &mut Query<(&Position, Option<&Destination>)>,
    nearby_item_query: &mut Query<(&Option<DroppedItem>, &Owner)>,
    nearby_enemy_query: &mut Query<(Option<&Npc>, &Team)>,
    #[resource] client_entity_list: &mut ClientEntityList,
    #[resource] game_data: &GameData,
) {
    let (owner_query_world, mut world) = world.split_for_query(&owner_query);
    let (nearby_item_query_world, mut world) = world.split_for_query(&nearby_item_query);
    let (nearby_enemy_query_world, world) = world.split_for_query(&nearby_enemy_query);
    let mut bot_world = world;

    bot_query.for_each_mut(
        &mut bot_world,
        |(entity, bot_ai, command, _next_command, inventory, position, owner, team)| {
            let _owner_components = owner_query.get(&owner_query_world, owner.entity);

            match command.command {
                CommandData::Stop => {
                    match bot_ai.state {
                        BotAiState::Default => {
                            let search_distance = 2000.0f32;
                            let mut rng = rand::thread_rng();

                            if let Some(zone_entities) =
                                client_entity_list.get_zone(position.zone as usize)
                            {
                                let mut nearby_items = Vec::new();
                                let mut nearby_monsters = Vec::new();

                                for (nearby_entity, nearby_position) in zone_entities
                                    .iter_entities_within_distance(
                                        position.position.xy(),
                                        search_distance,
                                    )
                                {
                                    if let Ok((Some(dropped_item), dropped_item_owner)) =
                                        nearby_item_query
                                            .get(&nearby_item_query_world, nearby_entity)
                                    {
                                        // Find any nearby dropped items that belong to us and that we have space to pick up
                                        if dropped_item_owner.entity == *entity {
                                            let has_space = match dropped_item {
                                                DroppedItem::Item(item) => inventory
                                                    .has_empty_slot(
                                                        InventoryPageType::from_item_type(
                                                            item.get_item_type(),
                                                        ),
                                                    ),
                                                DroppedItem::Money(_) => true,
                                            };
                                            if has_space {
                                                nearby_items.push((nearby_entity, nearby_position));
                                            }
                                        }
                                    } else if let Ok((nearby_npc, nearby_team)) = nearby_enemy_query
                                        .get(&nearby_enemy_query_world, nearby_entity)
                                    {
                                        // Find valid nearby enemy entities that we can attack
                                        let is_targetable = nearby_npc
                                            .and_then(|nearby_npc| {
                                                game_data.npcs.get_npc(nearby_npc.id as usize)
                                            })
                                            .map_or(true, |nearby_npc_data| {
                                                nearby_npc_data.is_targetable
                                            });

                                        if is_targetable
                                            && nearby_team.id != Team::DEFAULT_NPC_TEAM_ID
                                            && nearby_team.id != team.id
                                        {
                                            nearby_monsters.push((nearby_entity, nearby_position));
                                        }
                                    }
                                }

                                if let Some((target, target_position)) =
                                    nearby_items.choose(&mut rng)
                                {
                                    // Move towards item to pickup
                                    cmd.add_component(
                                        *entity,
                                        NextCommand::with_move(*target_position, Some(*target)),
                                    );
                                    bot_ai.state = BotAiState::PickupItem(*target);
                                } else if let Some((target, _)) = nearby_monsters.choose(&mut rng) {
                                    cmd.add_component(*entity, NextCommand::with_attack(*target));
                                }
                            }
                        }
                        BotAiState::PickupItem(target_item) => {
                            cmd.add_component(
                                *entity,
                                NextCommand::with_pickup_dropped_item(target_item),
                            );
                            bot_ai.state = BotAiState::Default;
                        }
                    }
                }
                CommandData::Die(CommandDie {
                    killer: _killer_entity,
                }) => {
                    // TODO: Handle death by respawning, or disappearing?
                }
                _ => {}
            }
        },
    );
}
