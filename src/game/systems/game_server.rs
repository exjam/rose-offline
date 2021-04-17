use std::{
    num::{ParseFloatError, ParseIntError},
};

use legion::systems::CommandBuffer;
use legion::*;
use nalgebra::Vector3;
use server::Whisper;

use crate::game::components::{
    CharacterInfo, ClientEntityId, Destination, GameClient, Level, MoveSpeed, Position, Target,
};
use crate::game::data::calculate_ability_values;
use crate::game::data::{account::AccountStorage, character::CharacterStorage};
use crate::game::messages::client::{
    ClientMessage, ConnectionRequestError, GameConnectionResponse, JoinZoneResponse,
};
use crate::game::messages::server;
use crate::game::messages::server::ServerMessage;
use crate::game::resources::{ClientEntityIdList, LoginTokens, ServerMessages, ZoneEntityId};

#[system(for_each)]
#[filter(!component::<CharacterInfo>())]
pub fn game_server_authentication(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    client: &mut GameClient,
    #[resource] login_tokens: &mut LoginTokens,
) {
    if let Ok(message) = client.client_message_rx.try_recv() {
        match message {
            ClientMessage::GameConnectionRequest(message) => {
                let response = login_tokens
                    .tokens
                    .iter()
                    .find(|t| t.token == message.login_token)
                    .ok_or(ConnectionRequestError::InvalidToken)
                    .and_then(|token| {
                        AccountStorage::try_load(&token.username, &message.password_md5)
                            .ok()
                            .ok_or(ConnectionRequestError::InvalidPassword)
                            .and_then(|_| {
                                CharacterStorage::try_load(&token.selected_character)
                                    .ok()
                                    .ok_or(ConnectionRequestError::Failed)
                            })
                            .and_then(|character| {
                                let ability_values = calculate_ability_values(
                                    &character.equipment,
                                    &character.inventory,
                                    &character.basic_stats,
                                );
                                cmd.add_component(
                                    *entity,
                                    MoveSpeed {
                                        speed: ability_values.run_speed,
                                    },
                                );
                                cmd.add_component(*entity, ability_values);
                                cmd.add_component(*entity, character.basic_stats.clone());
                                cmd.add_component(*entity, character.info.clone());
                                cmd.add_component(*entity, character.equipment.clone());
                                cmd.add_component(*entity, character.inventory.clone());
                                cmd.add_component(*entity, character.level.clone());
                                cmd.add_component(*entity, character.position.clone());

                                Ok(GameConnectionResponse {
                                    packet_sequence_id: 123,
                                    character_info: character.info,
                                    position: character.position,
                                    equipment: character.equipment,
                                    basic_stats: character.basic_stats,
                                    level: character.level,
                                    inventory: character.inventory,
                                })
                            })
                    });
                message.response_tx.send(response).ok();
            }
            _ => println!("Received unexpected client message"),
        }
    }
}

#[system(for_each)]
#[filter(!component::<ClientEntityId>())]
pub fn game_server_join(
    cmd: &mut CommandBuffer,
    client: &mut GameClient,
    entity: &Entity,
    level: &Level,
    position: &Position,
    #[resource] client_entity_id_list: &mut ClientEntityIdList,
) {
    if let Ok(message) = client.client_message_rx.try_recv() {
        match message {
            ClientMessage::JoinZoneRequest(message) => {
                let entity_id = client_entity_id_list
                    .get_zone_mut(position.zone as usize)
                    .allocate(*entity)
                    .unwrap();

                cmd.add_component(*entity, ClientEntityId { id: entity_id });

                message
                    .response_tx
                    .send(JoinZoneResponse {
                        entity_id: entity_id.0,
                        level: level.clone(),
                    })
                    .ok();
            }
            _ => println!("Received unexpected client message"),
        }
    }
}

use clap::{App, Arg};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref GM_COMMANDS: App<'static> = {
        App::new("GM Commands")
            .subcommand(App::new("help"))
            .subcommand(App::new("where"))
            .subcommand(
                App::new("mm")
                    .arg(Arg::new("zone").required(true))
                    .arg(Arg::new("x").required(true))
                    .arg(Arg::new("y").required(true)),
            )
    };
}

fn send_gm_commands_help(client: &mut GameClient) {
    for subcommand in GM_COMMANDS.get_subcommands() {
        let mut help_string = String::from(subcommand.get_name());
        for arg in subcommand.get_arguments() {
            help_string.push(' ');
            if !arg.is_set(clap::ArgSettings::Required) {
                help_string.push('[');
                help_string.push_str(arg.get_name());
                help_string.push(']');
            } else {
                help_string.push_str(arg.get_name());
            }
        }

        client
            .server_message_tx
            .send(ServerMessage::Whisper(Whisper {
                from: String::from("SERVER"),
                text: help_string,
            }))
            .ok();
    }
}

pub enum GMCommandError {
    InvalidCommand,
    InvalidArguments,
}

impl From<shellwords::MismatchedQuotes> for GMCommandError {
    fn from(_: shellwords::MismatchedQuotes) -> Self {
        Self::InvalidCommand
    }
}

impl From<clap::Error> for GMCommandError {
    fn from(error: clap::Error) -> Self {
        match error.kind {
            clap::ErrorKind::MissingRequiredArgument => Self::InvalidArguments,
            _ => Self::InvalidCommand,
        }
    }
}

impl From<ParseIntError> for GMCommandError {
    fn from(_: ParseIntError) -> Self {
        Self::InvalidArguments
    }
}

impl From<ParseFloatError> for GMCommandError {
    fn from(_: ParseFloatError) -> Self {
        Self::InvalidArguments
    }
}

fn handle_gm_command(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    client: &mut GameClient,
    text: &str,
    entity_id: &ClientEntityId,
    position: &Position,
) -> Result<(), GMCommandError> {
    let mut args = shellwords::split(text)?;
    args.insert(0, String::new()); // Clap expects arg[0] to be like executable name
    let matches = GM_COMMANDS.clone().try_get_matches_from(args)?;

    match matches.subcommand().ok_or(GMCommandError::InvalidCommand)? {
        ("where", _) => {
            client
                .server_message_tx
                .send(ServerMessage::Whisper(Whisper {
                    from: String::from("SERVER"),
                    text: format!(
                        "zone: {} x: {} y: {} z: {}",
                        position.zone,
                        position.position.x,
                        position.position.y,
                        position.position.z
                    ),
                }))
                .ok();
        }
        ("mm", matches) => {
            let zone = matches.value_of("zone").unwrap().parse::<u16>()?;
            let x = matches.value_of("x").unwrap().parse::<f32>()? * 1000.0;
            let y = matches.value_of("y").unwrap().parse::<f32>()? * 1000.0;

            cmd.add_component(
                *entity,
                Position {
                    position: Vector3::new(x, y, 0.0),
                    zone: zone,
                    respawn_zone: position.respawn_zone,
                },
            );
            cmd.remove_component::<ClientEntityId>(*entity);
            // TODO: Destroy entity for nearby players

            client
                .server_message_tx
                .send(ServerMessage::Teleport(server::Teleport {
                    entity_id: entity_id.id.0,
                    zone_no: zone,
                    x: x,
                    y: y,
                    run_mode: 1,
                    ride_mode: 0,
                }))
                .ok();
        }
        _ => return Err(GMCommandError::InvalidCommand),
    }

    Ok(())
}

#[system(for_each)]
pub fn game_server_move(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    client: &mut GameClient,
    entity_id: &ClientEntityId,
    position: &Position,
    #[resource] client_entity_id_list: &mut ClientEntityIdList,
    #[resource] server_messages: &mut ServerMessages,
) {
    if let Ok(message) = client.client_message_rx.try_recv() {
        match message {
            ClientMessage::Chat(text) => {
                if text.chars().nth(0).map_or(false, |c| c == '/') {
                    if handle_gm_command(cmd, entity, client, &text[1..], entity_id, position)
                        .is_err()
                    {
                        send_gm_commands_help(client);
                    }
                } else {
                    server_messages.send_nearby_message(
                        position.clone(),
                        ServerMessage::LocalChat(server::LocalChat {
                            entity_id: entity_id.id.0,
                            text: text,
                        }),
                    );
                }
            }
            ClientMessage::Move(message) => {
                let mut target_entity_id = 0;
                if message.target_entity_id > 0 {
                    if let Some(target_entity) = client_entity_id_list
                        .get_zone(position.zone as usize)
                        .get_entity(ZoneEntityId(message.target_entity_id))
                    {
                        target_entity_id = message.target_entity_id;
                        cmd.add_component(
                            *entity,
                            Target {
                                entity: target_entity,
                            },
                        );
                    } else {
                        cmd.remove_component::<Target>(*entity);
                    }
                } else {
                    cmd.remove_component::<Target>(*entity);
                }

                let destination = Vector3::new(message.x, message.y, message.z as f32);
                cmd.add_component(
                    *entity,
                    Destination {
                        position: destination,
                    },
                );

                let distance = destination.metric_distance(&position.position);
                server_messages.send_nearby_message(
                    position.clone(),
                    ServerMessage::MoveEntity(server::MoveEntity {
                        entity_id: entity_id.id.0,
                        target_entity_id: target_entity_id,
                        distance: distance as u16,
                        x: message.x,
                        y: message.y,
                        z: message.z,
                    }),
                );
            }
            _ => println!("Received unimplemented client message"),
        }
    }
}
