use std::time::{Duration, Instant};

use bevy::{
    math::Vec3Swizzles,
    prelude::{Component, Entity, EventWriter, Query, Res, With, Without},
    time::Time,
};
use big_brain::{
    prelude::{ActionBuilder, ActionState},
    thinker::Actor,
};
use rand::seq::SliceRandom;

use rose_data::{ItemReference, StackableItem};
use rose_game_common::components::Inventory;

use crate::game::{
    components::{ClientEntity, ClientEntityType, Command, Dead, Position},
    events::{RewardItemEvent, UseItemEvent},
    resources::ClientEntityList,
};

const SNOWBALL_THROW_SEARCH_DISTANCE: f32 = 1000.0f32;
const SNOWBALL_THROW_INTERVAL: Duration = Duration::from_secs(1);
const SNOWBALL_ITEM_REFERENCE: ItemReference = ItemReference::consumable(326); // Snowball

#[derive(Debug, Default, Clone, Component, ActionBuilder)]
pub struct SnowballFight {
    last_throw_time: Option<Instant>,
}

pub fn action_snowball_fight(
    mut query: Query<(&Actor, &mut ActionState, &mut SnowballFight)>,
    query_entity: Query<(&Command, &Inventory, &Position), (With<ClientEntity>, Without<Dead>)>,
    client_entity_list: Res<ClientEntityList>,
    time: Res<Time>,
    mut use_item_events: EventWriter<UseItemEvent>,
    mut reward_item_events: EventWriter<RewardItemEvent>,
) {
    let now = time.last_update();
    let mut rng = rand::thread_rng();

    for (&Actor(entity), mut state, mut snowball_fight) in query.iter_mut() {
        let Ok((command, inventory, position)) = query_entity.get(entity) else {
            continue;
        };

        match *state {
            ActionState::Requested => {
                let Some(zone_entities) =
                    client_entity_list.get_zone(position.zone_id) else {
                        *state = ActionState::Failure;
                        continue;
                    };

                if command.is_dead() {
                    // Cannot throw snowballs whilst dead
                    *state = ActionState::Failure;
                    continue;
                }

                let Some(item_slot) = inventory.find_item(SNOWBALL_ITEM_REFERENCE) else {
                    // We do not have a snowball in our inventory, try reward a stack and then set the
                    // state to Success so we go on to wait before next execution.
                    reward_item_events
                    .send(RewardItemEvent::new(
                        entity,
                        StackableItem::new(SNOWBALL_ITEM_REFERENCE, 999)
                            .unwrap()
                            .into(),
                        false,
                    ));

                    *state = ActionState::Success;
                    continue;
                };

                // Choose random nearby character to throw snowball at
                let nearby_target = zone_entities
                    .iter_entity_type_within_distance(
                        position.position.xy(),
                        SNOWBALL_THROW_SEARCH_DISTANCE,
                        &[ClientEntityType::Character],
                    )
                    .map(|(nearby_entity, _)| nearby_entity)
                    .collect::<Vec<Entity>>()
                    .choose(&mut rng)
                    .cloned();

                if let Some(target_entity) = nearby_target {
                    use_item_events.send(UseItemEvent::from_inventory(
                        entity,
                        item_slot,
                        Some(target_entity),
                    ));

                    snowball_fight.last_throw_time = now;
                    *state = ActionState::Executing;
                } else {
                    *state = ActionState::Failure;
                }
            }
            ActionState::Cancelled | ActionState::Executing => {
                if command.is_dead() {
                    // Cannot throw snowballs whilst dead
                    *state = ActionState::Failure;
                    continue;
                }

                // Throttle snowball fight by waiting for THROW_SNOWBALL_INTERVAL
                if now.unwrap() - snowball_fight.last_throw_time.unwrap() < SNOWBALL_THROW_INTERVAL
                {
                    continue;
                }

                // Ensure we are idle before completeing this action.
                if command.is_stop() {
                    *state = ActionState::Success;
                }
            }
            _ => {}
        }
    }
}
