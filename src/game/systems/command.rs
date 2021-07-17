use std::time::Duration;

use legion::{system, systems::CommandBuffer, world::SubWorld, Entity, Query};

use crate::game::{
    components::{
        AbilityValues, ClientEntity, ClientEntityType, Command, CommandAttack, CommandData,
        CommandMove, CommandPickupDroppedItem, Destination, DroppedItem, GameClient, HealthPoints,
        Inventory, MotionData, NextCommand, Owner, Position, Target,
    },
    messages::server::{
        self, PickupDroppedItemContent, PickupDroppedItemError, PickupDroppedItemResult,
        ServerMessage,
    },
    resources::{GameData, PendingDamage, PendingDamageList, ServerMessages, ServerTime},
};

const NPC_MOVE_TO_DISTANCE: f32 = 250.0;
const CHARACTER_MOVE_TO_DISTANCE: f32 = 1000.0;
const DROPPED_ITEM_MOVE_TO_DISTANCE: f32 = 150.0;
const DROPPED_ITEM_PICKUP_DISTANCE: f32 = 200.0;

fn set_command_stop(
    command: &mut Command,
    cmd: &mut CommandBuffer,
    entity: &Entity,
    entity_id: &ClientEntity,
    position: &Position,
    server_messages: &mut ServerMessages,
) {
    // Remove all components associated with other actions
    cmd.remove_component::<Destination>(*entity);
    cmd.remove_component::<Target>(*entity);

    server_messages.send_entity_message(
        *entity,
        ServerMessage::StopMoveEntity(server::StopMoveEntity {
            entity_id: entity_id.id,
            x: position.position.x,
            y: position.position.y,
            z: position.position.z as u16,
        }),
    );

    *command = Command::with_stop();
}

fn is_valid_move_target<'a>(
    position: &Position,
    target_entity: &Entity,
    move_target_query: &mut Query<(&ClientEntity, &Position)>,
    move_target_query_world: &'a SubWorld,
) -> Option<(&'a ClientEntity, &'a Position)> {
    if let Ok((target_client_entity, target_position)) =
        move_target_query.get(move_target_query_world, *target_entity)
    {
        if target_position.zone == position.zone {
            return Some((target_client_entity, target_position));
        }
    }

    None
}

fn is_valid_attack_target<'a>(
    position: &Position,
    target_entity: &Entity,
    attack_target_query: &mut Query<(&ClientEntity, &Position, &AbilityValues, &HealthPoints)>,
    attack_target_query_world: &'a SubWorld,
) -> Option<(&'a ClientEntity, &'a Position, &'a AbilityValues)> {
    if let Ok((target_client_entity, target_position, target_ability_values, target_health)) =
        attack_target_query.get(attack_target_query_world, *target_entity)
    {
        if target_position.zone == position.zone && target_health.hp > 0 {
            return Some((target_client_entity, target_position, target_ability_values));
        }
    }

    None
}

fn is_valid_pickup_target<'a>(
    position: &Position,
    target_entity: &Entity,
    query: &mut Query<(
        &ClientEntity,
        &Position,
        &mut Option<DroppedItem>,
        Option<&Owner>,
    )>,
    world: &'a mut SubWorld,
) -> Option<(
    &'a ClientEntity,
    &'a mut Option<DroppedItem>,
    Option<&'a Owner>,
)> {
    if let Ok((target_client_entity, target_position, target_dropped_item, target_owner)) =
        query.get_mut(world, *target_entity)
    {
        // Check distance to target
        let distance = (position.position.xy() - target_position.position.xy()).magnitude();
        if position.zone == target_position.zone && distance <= DROPPED_ITEM_PICKUP_DISTANCE {
            return Some((target_client_entity, target_dropped_item, target_owner));
        }
    }

    None
}

#[allow(clippy::clippy::type_complexity)]
#[system]
pub fn command(
    world: &mut SubWorld,
    cmd: &mut CommandBuffer,
    query: &mut Query<(
        Entity,
        &ClientEntity,
        &Position,
        &MotionData,
        &AbilityValues,
        &mut Command,
        &mut NextCommand,
        Option<&GameClient>,
        Option<&mut Inventory>,
    )>,
    move_target_query: &mut Query<(&ClientEntity, &Position)>,
    attack_target_query: &mut Query<(&ClientEntity, &Position, &AbilityValues, &HealthPoints)>,
    pickup_dropped_item_target_query: &mut Query<(
        &ClientEntity,
        &Position,
        &mut Option<DroppedItem>,
        Option<&Owner>,
    )>,
    #[resource] server_time: &ServerTime,
    #[resource] pending_damage_list: &mut PendingDamageList,
    #[resource] server_messages: &mut ServerMessages,
    #[resource] game_data: &GameData,
) {
    let (move_target_query_world, mut world) = world.split_for_query(&move_target_query);
    let (attack_target_query_world, mut world) = world.split_for_query(&attack_target_query);
    let (mut pickup_dropped_item_target_query_world, mut world) =
        world.split_for_query(&pickup_dropped_item_target_query);

    query.for_each_mut(
        &mut world,
        |(
            entity,
            client_entity,
            position,
            motion_data,
            ability_values,
            command,
            next_command,
            game_client,
            inventory,
        )| {
            if !next_command.has_sent_server_message && next_command.command.is_some() {
                // Send any server message required for update client next command
                match next_command.command.as_mut().unwrap() {
                    CommandData::Die(_) => {
                        panic!("Next command should never be set to die, set current command")
                    }
                    CommandData::Stop => {}
                    CommandData::PickupDroppedItem(_) => {}
                    CommandData::Move(CommandMove {
                        destination,
                        target,
                    }) => {
                        let mut target_entity_id = None;
                        if let Some(target_entity) = target {
                            if let Some((target_client_entity, target_position)) =
                                is_valid_move_target(
                                    position,
                                    target_entity,
                                    move_target_query,
                                    &move_target_query_world,
                                )
                            {
                                *destination = target_position.position;
                                target_entity_id = Some(target_client_entity.id);
                            } else {
                                *target = None;
                            }
                        }

                        let distance = (destination.xy() - position.position.xy()).magnitude();
                        server_messages.send_entity_message(
                            *entity,
                            ServerMessage::MoveEntity(server::MoveEntity {
                                entity_id: client_entity.id,
                                target_entity_id,
                                distance: distance as u16,
                                x: destination.x,
                                y: destination.y,
                                z: destination.z as u16,
                            }),
                        );
                    }
                    CommandData::Attack(CommandAttack {
                        target: target_entity,
                    }) => {
                        if let Some((target_client_entity, target_position, _)) =
                            is_valid_attack_target(
                                position,
                                target_entity,
                                attack_target_query,
                                &attack_target_query_world,
                            )
                        {
                            let distance = (target_position.position.xy() - position.position.xy())
                                .magnitude();

                            server_messages.send_entity_message(
                                *entity,
                                ServerMessage::AttackEntity(server::AttackEntity {
                                    entity_id: client_entity.id,
                                    target_entity_id: target_client_entity.id,
                                    distance: distance as u16,
                                    x: target_position.position.x,
                                    y: target_position.position.y,
                                    z: target_position.position.z as u16,
                                }),
                            );
                        } else {
                            next_command.command = Some(CommandData::Stop);
                        }
                    }
                }

                next_command.has_sent_server_message = true;
            }

            command.duration += server_time.delta;

            let required_duration = match command.command {
                CommandData::Attack(_) => {
                    let attack_speed = i32::max(ability_values.attack_speed, 30) as f32 / 100.0;
                    command
                        .required_duration
                        .map(|duration| duration.div_f32(attack_speed))
                }
                _ => command.required_duration,
            };

            let command_complete =
                required_duration.map_or_else(|| true, |duration| command.duration > duration);
            if !command_complete {
                // Current command still in animation
                return;
            }

            if next_command.command.is_none() {
                // No next command
                return;
            }

            match next_command.command.as_mut().unwrap() {
                CommandData::Stop => {
                    set_command_stop(
                        command,
                        cmd,
                        entity,
                        client_entity,
                        position,
                        server_messages,
                    );
                    *next_command = NextCommand::default();
                }
                CommandData::Move(CommandMove {
                    destination,
                    target,
                }) => {
                    let mut required_distance = 0.1;
                    if let Some(target_entity) = target {
                        if let Some((target_client_entity, target_position)) = is_valid_move_target(
                            position,
                            target_entity,
                            move_target_query,
                            &move_target_query_world,
                        ) {
                            *destination = target_position.position;
                            match target_client_entity.entity_type {
                                ClientEntityType::Character => {
                                    required_distance = CHARACTER_MOVE_TO_DISTANCE
                                }
                                ClientEntityType::Npc => required_distance = NPC_MOVE_TO_DISTANCE,
                                ClientEntityType::DroppedItem => {
                                    required_distance = DROPPED_ITEM_MOVE_TO_DISTANCE
                                }
                                _ => {}
                            }
                        } else {
                            *target = None;
                            cmd.remove_component::<Target>(*entity);
                        }
                    }

                    let distance = (destination.xy() - position.position.xy()).magnitude_squared();
                    if distance < required_distance {
                        *command = Command::with_stop();
                        cmd.remove_component::<Destination>(*entity);
                        cmd.remove_component::<Target>(*entity);
                    } else {
                        cmd.add_component(*entity, Destination::new(*destination));

                        if let Some(target_entity) = target {
                            cmd.add_component(*entity, Target::new(*target_entity));
                        }
                    }
                }
                CommandData::PickupDroppedItem(CommandPickupDroppedItem {
                    target: target_entity,
                }) => {
                    if let Some(inventory) = inventory {
                        if let Some((target_client_entity, target_dropped_item, target_owner)) =
                            is_valid_pickup_target(
                                position,
                                target_entity,
                                pickup_dropped_item_target_query,
                                &mut pickup_dropped_item_target_query_world,
                            )
                        {
                            let result = if !target_owner
                                .map_or(true, |owner| owner.entity == *entity)
                            {
                                // Not owner
                                Err(PickupDroppedItemError::NoPermission)
                            } else {
                                // Try add to inventory
                                match target_dropped_item.take() {
                                    None => Err(PickupDroppedItemError::NotExist),
                                    Some(DroppedItem::Item(item)) => {
                                        match inventory.try_add_item(item) {
                                            Ok((slot, item)) => Ok(PickupDroppedItemContent::Item(
                                                slot,
                                                item.clone(),
                                            )),
                                            Err(item) => {
                                                *target_dropped_item =
                                                    Some(DroppedItem::Item(item));
                                                Err(PickupDroppedItemError::InventoryFull)
                                            }
                                        }
                                    }
                                    Some(DroppedItem::Money(money)) => {
                                        if inventory.try_add_money(money).is_ok() {
                                            Ok(PickupDroppedItemContent::Money(money))
                                        } else {
                                            *target_dropped_item = Some(DroppedItem::Money(money));
                                            Err(PickupDroppedItemError::InventoryFull)
                                        }
                                    }
                                }
                            };

                            if result.is_ok() {
                                // Delete picked up item
                                cmd.remove(*target_entity);

                                // Update our current command
                                let motion_duration =
                                    motion_data.pickup_dropped_item.as_ref().map_or_else(
                                        || Duration::from_secs(1),
                                        |motion| motion.duration,
                                    );

                                *command = Command::with_pickup_dropped_item(
                                    *target_entity,
                                    motion_duration,
                                );
                                cmd.remove_component::<Destination>(*entity);
                                cmd.remove_component::<Target>(*entity);
                            }

                            // Send message to client with result
                            if let Some(game_client) = game_client {
                                game_client
                                    .server_message_tx
                                    .send(ServerMessage::PickupDroppedItemResult(
                                        PickupDroppedItemResult {
                                            item_entity_id: target_client_entity.id,
                                            result,
                                        },
                                    ))
                                    .ok();
                            }
                        }

                        *next_command = NextCommand::default();
                    }
                }
                CommandData::Attack(CommandAttack {
                    target: target_entity,
                }) => {
                    if let Some((_, target_position, target_ability_values)) =
                        is_valid_attack_target(
                            position,
                            target_entity,
                            attack_target_query,
                            &attack_target_query_world,
                        )
                    {
                        let distance =
                            (target_position.position.xy() - position.position.xy()).magnitude();

                        // Check if we are in attack range
                        let attack_range = ability_values.attack_range as f32;
                        if distance < attack_range {
                            let (attack_duration, hit_count) = motion_data
                                .attack
                                .as_ref()
                                .map(|attack_motion| {
                                    (attack_motion.duration, attack_motion.total_attack_frames)
                                })
                                .unwrap_or_else(|| (Duration::from_secs(1), 1));

                            // In range, set current command to attack
                            *command = Command::with_attack(*target_entity, attack_duration);

                            // Remove our destination component, as we have reached it!
                            cmd.remove_component::<Destination>(*entity);

                            // Update target
                            cmd.add_component(*entity, Target::new(*target_entity));

                            // Spawn an entity for DamageSystem to apply damage
                            pending_damage_list.push(PendingDamage {
                                attacker: *entity,
                                defender: *target_entity,
                                damage: game_data.ability_value_calculator.calculate_damage(
                                    ability_values,
                                    target_ability_values,
                                    hit_count as i32,
                                ),
                            });
                        } else {
                            // Not in range, set current command to move
                            *command =
                                Command::with_move(target_position.position, Some(*target_entity));

                            // Set destination to move towards
                            cmd.add_component(*entity, Destination::new(target_position.position));

                            // Update target
                            cmd.add_component(*entity, Target::new(*target_entity));
                        }
                    } else {
                        set_command_stop(
                            command,
                            cmd,
                            entity,
                            client_entity,
                            position,
                            server_messages,
                        );
                        *next_command = NextCommand::default();
                    }
                }
                _ => {}
            }
        },
    );
}
