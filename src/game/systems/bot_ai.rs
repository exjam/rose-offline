use bevy_ecs::prelude::{Commands, Entity, EventWriter, Query, Res};
use rand::seq::SliceRandom;

use crate::game::{
    components::{
        BotAi, BotAiState, Command, CommandData, Destination, DroppedItem, Inventory,
        InventoryPageType, ItemSlot, NextCommand, Npc, Owner, Position, Team,
        BOT_IDLE_CHECK_DURATION,
    },
    events::UseItemEvent,
    resources::{ClientEntityList, ServerTime},
    GameData,
};

const BOT_SEARCH_ENTITY_DISTANCE: f32 = 1500.0f32;

pub fn bot_ai_system(
    mut commands: Commands,
    mut bot_query: Query<(
        Entity,
        &mut BotAi,
        &Command,
        &NextCommand,
        &Inventory,
        &Position,
        &Owner,
        &Team,
    )>,
    owner_query: Query<(&Position, Option<&Destination>)>,
    nearby_item_query: Query<(&Option<DroppedItem>, &Owner)>,
    nearby_enemy_query: Query<(Option<&Npc>, &Team)>,
    client_entity_list: Res<ClientEntityList>,
    game_data: Res<GameData>,
    server_time: Res<ServerTime>,
    mut use_item_events: EventWriter<UseItemEvent>,
) {
    bot_query.for_each_mut(
        |(entity, mut bot_ai, command, _next_command, inventory, position, owner, team)| {
            let _owner_components = owner_query.get(owner.entity);

            match command.command {
                CommandData::Stop(_) => {
                    bot_ai.time_since_last_idle_check += server_time.delta;
                    if bot_ai.time_since_last_idle_check < BOT_IDLE_CHECK_DURATION {
                        return;
                    }
                    bot_ai.time_since_last_idle_check -= BOT_IDLE_CHECK_DURATION;

                    match bot_ai.state {
                        BotAiState::SnowballFight => {
                            if let Some(zone_entities) =
                                client_entity_list.get_zone(position.zone_id)
                            {
                                let item_slot =
                                    ItemSlot::Inventory(InventoryPageType::Consumables, 0);
                                if inventory.get_item(item_slot).is_some() {
                                    let mut rng = rand::thread_rng();
                                    let mut nearby_targets = Vec::new();

                                    for (nearby_entity, _) in zone_entities
                                        .iter_entities_within_distance(
                                            position.position.xy(),
                                            BOT_SEARCH_ENTITY_DISTANCE,
                                        )
                                    {
                                        if let Ok((_, nearby_team)) =
                                            nearby_enemy_query.get(nearby_entity)
                                        {
                                            if nearby_team.id == team.id {
                                                nearby_targets.push(nearby_entity);
                                            }
                                        }
                                    }

                                    if let Some(target_entity) =
                                        nearby_targets.choose(&mut rng).copied()
                                    {
                                        use_item_events.send(UseItemEvent::new(
                                            entity,
                                            item_slot,
                                            Some(target_entity),
                                        ));

                                        // Speed up the snowball fight!
                                        bot_ai.time_since_last_idle_check +=
                                            (BOT_IDLE_CHECK_DURATION * 3) / 4;
                                    }
                                }
                            }
                        }
                        BotAiState::Farm => {
                            if let Some(zone_entities) =
                                client_entity_list.get_zone(position.zone_id)
                            {
                                let mut rng = rand::thread_rng();
                                let mut nearby_items = Vec::new();
                                let mut nearby_monsters = Vec::new();

                                for (nearby_entity, nearby_position) in zone_entities
                                    .iter_entities_within_distance(
                                        position.position.xy(),
                                        BOT_SEARCH_ENTITY_DISTANCE,
                                    )
                                {
                                    if let Ok((Some(dropped_item), dropped_item_owner)) =
                                        nearby_item_query.get(nearby_entity)
                                    {
                                        // Find any nearby dropped items that belong to us and that we have space to pick up
                                        if dropped_item_owner.entity == entity {
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
                                    } else if let Ok((nearby_npc, nearby_team)) =
                                        nearby_enemy_query.get(nearby_entity)
                                    {
                                        // Find valid nearby enemy entities that we can attack
                                        let is_untargetable = nearby_npc
                                            .and_then(|nearby_npc| {
                                                game_data.npcs.get_npc(nearby_npc.id)
                                            })
                                            .map_or(false, |nearby_npc_data| {
                                                nearby_npc_data.is_untargetable
                                            });

                                        if !is_untargetable
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
                                    commands.entity(entity).insert(NextCommand::with_move(
                                        *target_position,
                                        Some(*target),
                                        None,
                                    ));
                                    bot_ai.state = BotAiState::PickupItem(*target);
                                } else if let Some((target, _)) = nearby_monsters.choose(&mut rng) {
                                    commands
                                        .entity(entity)
                                        .insert(NextCommand::with_attack(*target));
                                }
                            }
                        }
                        BotAiState::PickupItem(target_item) => {
                            commands
                                .entity(entity)
                                .insert(NextCommand::with_pickup_dropped_item(target_item));
                            bot_ai.state = BotAiState::Farm;
                        }
                    }
                }
                CommandData::Die(_) => {
                    // TODO: Handle death by respawning, or disappearing?
                }
                _ => {}
            }
        },
    );
}
