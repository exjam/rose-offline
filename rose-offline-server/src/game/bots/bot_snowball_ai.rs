use std::time::Duration;

use bevy::{
    ecs::query::WorldQuery,
    math::Vec3Swizzles,
    prelude::{Component, Entity, EventWriter, Query, Res},
    time::Time,
};
use rand::seq::SliceRandom;

use rose_data::{ItemReference, StackableItem};
use rose_game_common::components::{Inventory, Team};

use crate::game::{
    components::{Command, Position},
    events::UseItemEvent,
    resources::ClientEntityList,
};

const BOT_SNOWBALL_INTERVAL: Duration = Duration::from_millis(250);
const BOT_SNOWBALL_TARGET_DISTANCE: f32 = 2000.0f32;
const BOT_SNOWBALL_USE_ITEM: ItemReference = ItemReference::consumable(326); // Snowball

#[derive(Component, Default)]
pub struct BotSnowballAi {
    time_since_last_snowball: Duration,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct SnowballBotQuery<'w> {
    entity: Entity,
    snowball_ai: &'w mut BotSnowballAi,
    command: &'w Command,
    inventory: &'w mut Inventory,
    position: &'w Position,
    team: &'w Team,
}

pub fn bot_snowball_ai_system(
    mut query_snowball_bots: Query<SnowballBotQuery>,
    query_team: Query<&Team>,
    client_entity_list: Res<ClientEntityList>,
    time: Res<Time>,
    mut use_item_events: EventWriter<UseItemEvent>,
) {
    let mut rng = rand::thread_rng();

    for mut bot in query_snowball_bots.iter_mut() {
        if !bot.command.is_stop() {
            // Wait until bot is idle
            continue;
        }

        bot.snowball_ai.time_since_last_snowball += time.delta();
        if bot.snowball_ai.time_since_last_snowball < BOT_SNOWBALL_INTERVAL {
            continue;
        }
        bot.snowball_ai.time_since_last_snowball -= BOT_SNOWBALL_INTERVAL;

        let Some(zone_entities) =
            client_entity_list.get_zone(bot.position.zone_id) else {
                continue;
            };

        let Some(item_slot) =
            bot.inventory
                .find_item(BOT_SNOWBALL_USE_ITEM)
                .or_else(||
                    bot.inventory.try_add_item(
                        StackableItem::new(BOT_SNOWBALL_USE_ITEM, 999).unwrap().into())
                    .ok().map(|(item_slot, _)| item_slot)
                ) else {
                    continue;
                };

        // Find all nearby entities on our team
        let mut nearby_targets = Vec::new();

        for (nearby_entity, _) in zone_entities
            .iter_entities_within_distance(bot.position.position.xy(), BOT_SNOWBALL_TARGET_DISTANCE)
        {
            if let Ok(nearby) = query_team.get(nearby_entity) {
                if nearby.id == bot.team.id {
                    nearby_targets.push(nearby_entity);
                }
            }
        }

        // Throw snowball at random nearby target
        if let Some(target_entity) = nearby_targets.choose(&mut rng).copied() {
            use_item_events.send(UseItemEvent::from_inventory(
                bot.entity,
                item_slot,
                Some(target_entity),
            ));
        }
    }
}
