use std::time::Duration;

use legion::{system, systems::CommandBuffer, world::SubWorld, Entity, EntityStore};

use crate::game::{
    components::{
        AbilityValues, ClientEntity, Command, CommandAttack, CommandData, CommandMove, Destination,
        HealthPoints, MotionData, NextCommand, Position,
    },
    messages::server::{self, ServerMessage},
    resources::{DeltaTime, GameData, PendingDamage, PendingDamageList, ServerMessages},
};

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

    server_messages.send_entity_message(
        *entity,
        ServerMessage::StopMoveEntity(server::StopMoveEntity {
            entity_id: entity_id.id.0,
            x: position.position.x,
            y: position.position.y,
            z: position.position.z as u16,
        }),
    );

    *command = Command::with_stop();
}

#[system(for_each)]
#[read_component(ClientEntity)]
#[read_component(Position)]
#[read_component(AbilityValues)]
#[read_component(HealthPoints)]
pub fn command(
    world: &SubWorld,
    cmd: &mut CommandBuffer,
    entity: &Entity,
    entity_id: &ClientEntity,
    position: &Position,
    motion_data: &MotionData,
    command: &mut Command,
    ability_values: &AbilityValues,
    next_command: &NextCommand,
    #[resource] delta_time: &DeltaTime,
    #[resource] pending_damage_list: &mut PendingDamageList,
    #[resource] server_messages: &mut ServerMessages,
    #[resource] game_data: &GameData,
) {
    command.duration += delta_time.delta;

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

    if next_command.0.is_none() {
        // No next command
        return;
    }

    match *next_command.0.as_ref().unwrap() {
        CommandData::Stop => {
            set_command_stop(command, cmd, entity, entity_id, position, server_messages);
            cmd.add_component(*entity, NextCommand::default());
        }
        CommandData::Move(CommandMove {
            destination,
            target,
        }) => {
            cmd.add_component(
                *entity,
                Destination {
                    position: destination,
                },
            );

            let mut target_entity_id = 0;
            if let Some(target_entity) = target {
                if let Ok(entry) = world.entry_ref(target_entity) {
                    if let Ok(target_client_entity) = entry.get_component::<ClientEntity>() {
                        target_entity_id = target_client_entity.id.0;
                    }
                }
            }

            let distance = (destination.xy() - position.position.xy()).magnitude();
            server_messages.send_entity_message(
                *entity,
                ServerMessage::MoveEntity(server::MoveEntity {
                    entity_id: entity_id.id.0,
                    target_entity_id,
                    distance: distance as u16,
                    x: destination.x,
                    y: destination.y,
                    z: destination.z as u16,
                }),
            );

            *command = Command::with_move(destination, target);
            cmd.add_component(*entity, NextCommand::default());
        }
        CommandData::Attack(CommandAttack { target }) => {
            let mut valid_attack_target = false;
            if let Ok(entry) = world.entry_ref(target) {
                if let (
                    Ok(target_client_entity),
                    Ok(target_position),
                    Ok(target_ability_values),
                    Ok(target_health),
                ) = (
                    entry.get_component::<ClientEntity>(),
                    entry.get_component::<Position>(),
                    entry.get_component::<AbilityValues>(),
                    entry.get_component::<HealthPoints>(),
                ) {
                    if target_position.zone == position.zone && target_health.hp > 0 {
                        let distance =
                            (target_position.position.xy() - position.position.xy()).magnitude();

                        // Check if we have just started attacking this target
                        let attack_started = match command.command {
                            CommandData::Attack(CommandAttack {
                                target: current_attack_target,
                                ..
                            }) => current_attack_target != target,
                            CommandData::Move(CommandMove {
                                target: Some(current_attack_target),
                                ..
                            }) => current_attack_target != target,
                            _ => true,
                        };

                        // TODO: This needs to use ability values which include +/- from buffs,
                        //       the current ability_values component does not do that.
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
                            *command = Command::with_attack(target, attack_duration);

                            // Remove our destination component, as we have reached it!
                            cmd.remove_component::<Destination>(*entity);

                            // Spawn an entity for DamageSystem to apply damage
                            pending_damage_list.push(PendingDamage {
                                attacker: *entity,
                                defender: target,
                                damage: game_data.ability_value_calculator.calculate_damage(
                                    ability_values,
                                    target_ability_values,
                                    hit_count as i32,
                                ),
                            });
                        } else {
                            // Not in range, set current command to move
                            *command = Command::with_move(target_position.position, Some(target));

                            // Set destination to move towards
                            cmd.add_component(
                                *entity,
                                Destination {
                                    position: target_position.position,
                                },
                            );
                        }

                        if attack_started {
                            server_messages.send_entity_message(
                                *entity,
                                ServerMessage::AttackEntity(server::AttackEntity {
                                    entity_id: entity_id.id.0,
                                    target_entity_id: target_client_entity.id.0,
                                    distance: distance as u16,
                                    x: target_position.position.x,
                                    y: target_position.position.y,
                                    z: target_position.position.z as u16,
                                }),
                            );
                        }

                        valid_attack_target = true;
                    }
                }
            }

            if !valid_attack_target {
                set_command_stop(command, cmd, entity, entity_id, position, server_messages);
                cmd.add_component(*entity, NextCommand::default());
            }
        }
        _ => {}
    }
}
