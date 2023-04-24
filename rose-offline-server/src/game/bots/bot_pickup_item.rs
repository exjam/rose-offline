use bevy::{
    math::Vec3Swizzles,
    prelude::{Commands, Component, Entity, Query, Res, With, Without},
};
use big_brain::{
    prelude::{ActionBuilder, ActionState, ScorerBuilder},
    scorers::Score,
    thinker::Actor,
};

use rose_game_common::components::{Inventory, InventoryPageType};

use crate::game::{
    bots::IDLE_DURATION,
    components::{ClientEntity, ClientEntityType, Command, Dead, NextCommand, Owner, Position},
    resources::ClientEntityList,
};

const ITEM_DROP_SEARCH_DISTANCE: f32 = 1000.0f32;

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct FindNearbyItemDrop {
    pub score: f32,
}

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct PickupNearestItemDrop;

#[derive(Component)]
pub struct PickupItemDrop {
    entity: Entity,
}

pub fn score_find_nearby_item_drop_system(
    mut query: Query<(&FindNearbyItemDrop, &Actor, &mut Score)>,
    query_entity: Query<(&Command, &Inventory, &Position), (With<ClientEntity>, Without<Dead>)>,
    query_owner: Query<&Owner>,
    client_entity_list: Res<ClientEntityList>,
) {
    for (scorer, &Actor(entity), mut score) in query.iter_mut() {
        score.set(0.0);

        let Ok((command, inventory, position)) = query_entity.get(entity) else {
            continue;
        };

        if command.is_dead() {
            // Cannot pick up items whilst dead
            continue;
        }

        if !inventory.has_empty_slot(InventoryPageType::Equipment)
            || !inventory.has_empty_slot(InventoryPageType::Consumables)
            || !inventory.has_empty_slot(InventoryPageType::Materials)
            || !inventory.has_empty_slot(InventoryPageType::Vehicles)
        {
            // Only pickup items if we have a space
            continue;
        }

        let Some(zone_entities) =
            client_entity_list.get_zone(position.zone_id) else {
                continue;
            };

        // Find any item drop nearby which we own, or has no owner
        for (nearby_entity, _) in zone_entities.iter_entity_type_within_distance(
            position.position.xy(),
            ITEM_DROP_SEARCH_DISTANCE,
            &[ClientEntityType::ItemDrop],
        ) {
            if query_owner
                .get(nearby_entity)
                .map_or(true, |owner| owner.entity == entity)
            {
                score.set(scorer.score);
                break;
            }
        }
    }
}

pub fn action_pickup_nearest_item_drop(
    mut commands: Commands,
    mut query: Query<(&Actor, &mut ActionState), With<PickupNearestItemDrop>>,
    query_entity: Query<(&Command, &Position, Option<&PickupItemDrop>)>,
    query_owner: Query<&Owner>,
    client_entity_list: Res<ClientEntityList>,
) {
    for (&Actor(entity), mut state) in query.iter_mut() {
        let Ok((command, position, pickup_item_drop)) = query_entity.get(entity) else {
            continue;
        };

        match *state {
            ActionState::Requested => {
                let Some(zone_entities) =
                    client_entity_list.get_zone(position.zone_id) else {
                        continue;
                    };

                let mut nearest_item_drop = None;
                for (nearby_entity, nearby_position) in zone_entities
                    .iter_entity_type_within_distance(
                        position.position.xy(),
                        ITEM_DROP_SEARCH_DISTANCE,
                        &[ClientEntityType::ItemDrop],
                    )
                {
                    if query_owner
                        .get(nearby_entity)
                        .map_or(true, |owner| owner.entity == entity)
                    {
                        let distance = position
                            .position
                            .xy()
                            .distance_squared(nearby_position.xy());

                        if nearest_item_drop
                            .map_or(true, |(nearest_distance, _, _)| distance < nearest_distance)
                        {
                            nearest_item_drop = Some((distance, nearby_position, nearby_entity));
                        }
                    }
                }

                if let Some((_, nearest_position, nearest_entity)) = nearest_item_drop {
                    commands
                        .entity(entity)
                        .insert(NextCommand::with_move(
                            nearest_position,
                            Some(nearest_entity),
                            None,
                        ))
                        .insert(PickupItemDrop {
                            entity: nearest_entity,
                        });

                    *state = ActionState::Executing;
                } else {
                    *state = ActionState::Failure;
                }
            }
            ActionState::Executing => {
                if command.is_stop_for(IDLE_DURATION) {
                    // Wait until we are idle
                    continue;
                }

                if let Some(pickup_item_drop) = pickup_item_drop {
                    // We must have finished moving to the item drop, start a pickup
                    commands
                        .entity(entity)
                        .insert(NextCommand::with_pickup_item_drop(pickup_item_drop.entity))
                        .remove::<PickupItemDrop>();
                    continue;
                } else {
                    // We must have finished picking the item up
                    *state = ActionState::Success;
                }
            }
            ActionState::Cancelled => {
                commands
                    .entity(entity)
                    .insert(NextCommand::with_stop(true))
                    .remove::<PickupItemDrop>();

                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
