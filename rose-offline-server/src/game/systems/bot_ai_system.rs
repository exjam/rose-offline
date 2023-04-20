use bevy::{
    ecs::{
        prelude::{Commands, Entity, EventWriter, Query, Res},
        query::WorldQuery,
    },
    math::{Vec3, Vec3Swizzles},
    time::Time,
};

use rand::{seq::SliceRandom, Rng};

use crate::game::{
    components::{
        BotAi, BotAiState, BotMessage, Command, CommandData, DroppedItem, Inventory,
        InventoryPageType, ItemDrop, NextCommand, Npc, Owner, Party, PartyMembership, Position,
        Team, BOT_IDLE_CHECK_DURATION,
    },
    events::PartyEvent,
    resources::ClientEntityList,
    GameData,
};

const BOT_SEARCH_ENTITY_DISTANCE: f32 = 3000.0f32;
const BOT_PARTY_OWNER_MAX_DISTANCE: f32 = 500.0f32;

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct BotQuery<'w> {
    entity: Entity,
    ai: &'w mut BotAi,
    command: &'w Command,
    next_command: &'w NextCommand,
    inventory: &'w Inventory,
    party_membership: &'w PartyMembership,
    position: &'w Position,
    team: &'w Team,
}

#[derive(WorldQuery)]
pub struct ItemDropQuery<'w> {
    drop: &'w ItemDrop,
    owner: Option<&'w Owner>,
}

#[derive(WorldQuery)]
pub struct TeamQuery<'w> {
    position: &'w Position,
    team: &'w Team,
    npc: Option<&'w Npc>,
}

pub fn bot_ai_system(
    mut commands: Commands,
    mut bot_query: Query<BotQuery>,
    item_query: Query<ItemDropQuery>,
    team_query: Query<TeamQuery>,
    party_query: Query<&Party>,
    client_entity_list: Res<ClientEntityList>,
    game_data: Res<GameData>,
    time: Res<Time>,
    mut party_events: EventWriter<PartyEvent>,
) {
    let mut rng = rand::thread_rng();

    for mut bot in bot_query.iter_mut() {
        for message in bot.ai.messages.iter() {
            let BotMessage::PartyInvite(owner_entity) = *message;
            if bot.party_membership.is_none() {
                party_events.send(PartyEvent::AcceptInvite {
                    owner_entity,
                    invited_entity: bot.entity,
                });
            }
        }
        bot.ai.messages.clear();

        match bot.command.command {
            CommandData::Stop(_) => {
                bot.ai.time_since_last_idle_check += time.delta();
                if bot.ai.time_since_last_idle_check < BOT_IDLE_CHECK_DURATION {
                    continue;
                }
                bot.ai.time_since_last_idle_check -= BOT_IDLE_CHECK_DURATION;

                match bot.ai.state {
                    BotAiState::Farm => {
                        if let Some(zone_entities) =
                            client_entity_list.get_zone(bot.position.zone_id)
                        {
                            let mut nearby_items = Vec::new();
                            let mut nearby_enemies = Vec::new();

                            for (nearby_entity, nearby_position) in zone_entities
                                .iter_entities_within_distance(
                                    bot.position.position.xy(),
                                    BOT_SEARCH_ENTITY_DISTANCE,
                                )
                            {
                                if let Ok(nearby_item) = item_query.get(nearby_entity) {
                                    if let Some(dropped_item) = nearby_item.drop.item.as_ref() {
                                        // Pick up any valid nearby dropped items
                                        if nearby_item
                                            .owner
                                            .map_or(true, |owner| owner.entity == bot.entity)
                                        {
                                            let has_space = match dropped_item {
                                                DroppedItem::Item(item) => {
                                                    bot.inventory.has_empty_slot(
                                                        InventoryPageType::from_item_type(
                                                            item.get_item_type(),
                                                        ),
                                                    )
                                                }
                                                DroppedItem::Money(_) => true,
                                            };
                                            if has_space {
                                                nearby_items.push((nearby_entity, nearby_position));
                                            }
                                        }
                                    }
                                } else if let Ok(nearby_enemy) = team_query.get(nearby_entity) {
                                    // Find valid nearby enemy entities that we can attack
                                    let is_untargetable = nearby_enemy
                                        .npc
                                        .and_then(|nearby_npc| {
                                            game_data.npcs.get_npc(nearby_npc.id)
                                        })
                                        .map_or(false, |nearby_npc_data| {
                                            nearby_npc_data.is_untargetable
                                        });

                                    if !is_untargetable
                                        && nearby_enemy.team.id != Team::DEFAULT_NPC_TEAM_ID
                                        && nearby_enemy.team.id != bot.team.id
                                    {
                                        nearby_enemies.push((nearby_entity, nearby_position));
                                    }
                                }
                            }

                            if let Some((target, target_position)) = nearby_items.choose(&mut rng) {
                                // Pick up item
                                commands.entity(bot.entity).insert(NextCommand::with_move(
                                    *target_position,
                                    Some(*target),
                                    None,
                                ));
                                bot.ai.state = BotAiState::PickupItem(*target);
                                bot.ai.time_since_last_idle_check += BOT_IDLE_CHECK_DURATION;
                            } else {
                                //  Move near party owner if we are too far away
                                if let Some(party_entity) = bot.party_membership.party() {
                                    if let Ok(party) = party_query.get(party_entity) {
                                        if let Ok(owner) = team_query.get(party.owner) {
                                            if bot.position.zone_id == owner.position.zone_id
                                                && bot
                                                    .position
                                                    .position
                                                    .xy()
                                                    .distance_squared(owner.position.position.xy())
                                                    > BOT_PARTY_OWNER_MAX_DISTANCE
                                                        * BOT_PARTY_OWNER_MAX_DISTANCE
                                            {
                                                let x_distance = rng.gen_range(50.0..=350.0);
                                                let y_distance = rng.gen_range(50.0..=350.0);

                                                let move_offset = Vec3::new(
                                                    if rng.gen_bool(0.5) {
                                                        -x_distance
                                                    } else {
                                                        x_distance
                                                    },
                                                    if rng.gen_bool(0.5) {
                                                        -y_distance
                                                    } else {
                                                        y_distance
                                                    },
                                                    0.0,
                                                );

                                                commands.entity(bot.entity).insert(
                                                    NextCommand::with_move(
                                                        owner.position.position + move_offset,
                                                        None,
                                                        None,
                                                    ),
                                                );
                                                bot.ai.time_since_last_idle_check +=
                                                    BOT_IDLE_CHECK_DURATION;
                                                continue;
                                            }
                                        }
                                    }
                                }

                                if let Some((target, _)) = nearby_enemies.choose(&mut rng) {
                                    commands
                                        .entity(bot.entity)
                                        .insert(NextCommand::with_attack(*target));
                                    bot.ai.time_since_last_idle_check += BOT_IDLE_CHECK_DURATION;
                                }
                            }
                        }
                    }
                    BotAiState::PickupItem(target_item) => {
                        commands
                            .entity(bot.entity)
                            .insert(NextCommand::with_pickup_item_drop(target_item));
                        bot.ai.state = BotAiState::Farm;
                        bot.ai.time_since_last_idle_check += BOT_IDLE_CHECK_DURATION;
                    }
                }
            }
            CommandData::Die(_) => {
                // TODO: Handle death by respawning, or disappearing?
            }
            _ => {}
        }
    }
}
