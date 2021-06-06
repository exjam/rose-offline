use std::time::Duration;

use legion::{system, systems::CommandBuffer, world::SubWorld, Entity, EntityStore};

use crate::game::{
    components::{
        AbilityValues, ClientEntity, Command, CommandAttack, CommandData, CommandMove, Destination,
        MotionData, NextCommand, Position,
    },
    messages::server::{self, ServerMessage},
    resources::{DeltaTime, ServerMessages},
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

    *command = Command::new(CommandData::Stop, None);
}

#[system(for_each)]
#[read_component(ClientEntity)]
#[read_component(Position)]
pub fn command(
    world: &SubWorld,
    cmd: &mut CommandBuffer,
    entity: &Entity,
    entity_id: &ClientEntity,
    position: &Position,
    motion_data: &MotionData,
    command: &mut Command,
    ability_values: &AbilityValues,
    next_command: Option<&NextCommand>,
    #[resource] delta_time: &DeltaTime,
    #[resource] server_messages: &mut ServerMessages,
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

    if next_command.is_none() {
        // No next command
        return;
    }

    match next_command.unwrap().0 {
        CommandData::Stop => {
            set_command_stop(command, cmd, entity, entity_id, position, server_messages);
            cmd.remove_component::<NextCommand>(*entity);
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

            *command = Command::new(
                CommandData::Move(CommandMove {
                    destination,
                    target,
                }),
                None,
            );
            cmd.remove_component::<NextCommand>(*entity);
        }
        CommandData::Attack(CommandAttack { target }) => {
            let mut valid_attack_target = false;
            if let Ok(entry) = world.entry_ref(target) {
                if let Ok(target_client_entity) = entry.get_component::<ClientEntity>() {
                    if let Ok(target_position) = entry.get_component::<Position>() {
                        if target_position.zone == position.zone {
                            let distance = (target_position.position.xy() - position.position.xy())
                                .magnitude();

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
                                let attack_duration = motion_data
                                    .attack
                                    .as_ref()
                                    .map(|attack_motion| attack_motion.duration)
                                    .unwrap_or_else(|| Duration::from_secs(1));

                                *command = Command::new(
                                    CommandData::Attack(CommandAttack { target }),
                                    Some(attack_duration),
                                );
                                cmd.remove_component::<Destination>(*entity);
                            } else {
                                *command = Command::new(
                                    CommandData::Move(CommandMove {
                                        destination: target_position.position,
                                        target: Some(target),
                                    }),
                                    None,
                                );
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
            }

            if !valid_attack_target {
                set_command_stop(command, cmd, entity, entity_id, position, server_messages);
                cmd.remove_component::<NextCommand>(*entity);
            }
        }
    }
}
