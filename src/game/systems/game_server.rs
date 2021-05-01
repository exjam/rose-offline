use std::num::{ParseFloatError, ParseIntError};

use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;
use nalgebra::Point3;
use server::Whisper;

use crate::game::components::{BasicStats, CharacterInfo, ClientEntity, ClientEntityVisibility, Destination, Equipment, GameClient, Hotbar, Inventory, Level, MoveSpeed, Position, SkillList, Target, Team};
use crate::game::data::calculate_ability_values;
use crate::game::data::{account::AccountStorage, character::CharacterStorage};
use crate::game::messages::client::{
    ClientMessage, ConnectionRequestError, GameConnectionResponse, JoinZoneResponse, SetHotbarSlot,
    SetHotbarSlotError,
};
use crate::game::messages::server;
use crate::game::messages::server::ServerMessage;
use crate::game::resources::{ClientEntityId, ClientEntityList, LoginTokens, ServerMessages};

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
                                cmd.add_component(*entity, character.skill_list.clone());
                                cmd.add_component(*entity, character.hotbar.clone());
                                cmd.add_component(*entity, Team::default_character());

                                Ok(GameConnectionResponse {
                                    packet_sequence_id: 123,
                                    character_info: character.info,
                                    position: character.position,
                                    equipment: character.equipment,
                                    basic_stats: character.basic_stats,
                                    level: character.level,
                                    inventory: character.inventory,
                                    skill_list: character.skill_list,
                                    hotbar: character.hotbar,
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
#[filter(!component::<ClientEntity>())]
pub fn game_server_join(
    cmd: &mut CommandBuffer,
    client: &mut GameClient,
    entity: &Entity,
    level: &Level,
    team: &Team,
    position: &Position,
    #[resource] client_entity_list: &mut ClientEntityList,
) {
    if let Ok(message) = client.client_message_rx.try_recv() {
        match message {
            ClientMessage::JoinZoneRequest(message) => {
                if let Some(zone) = client_entity_list.get_zone_mut(position.zone as usize) {
                    if let Some(client_entity) = zone.allocate(*entity, position.position) {
                        let entity_id = client_entity.id;
                        cmd.add_component(*entity, client_entity);
                        cmd.add_component(*entity, ClientEntityVisibility::new());

                        message
                            .response_tx
                            .send(JoinZoneResponse {
                                entity_id: entity_id.0,
                                level: level.clone(),
                                team: team.clone(),
                            })
                            .ok();
                    }
                }
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
    entity_id: &ClientEntity,
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

            cmd.add_component(*entity, Position::new(Point3::new(x, y, 0.0), zone));
            cmd.remove_component::<ClientEntity>(*entity);
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
pub fn game_server_main(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    client: &mut GameClient,
    entity_id: &ClientEntity,
    position: &Position,
    hotbar: &mut Hotbar,
    #[resource] client_entity_list: &mut ClientEntityList,
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
                    server_messages.send_entity_message(
                        entity.clone(),
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
                    if let Some(target_entity) = client_entity_list
                        .get_zone(position.zone as usize)
                        .and_then(|zone| zone.get_entity(ClientEntityId(message.target_entity_id)))
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

                let destination = Point3::new(message.x, message.y, message.z as f32);
                cmd.add_component(
                    *entity,
                    Destination {
                        position: destination,
                    },
                );

                let distance = (destination - position.position).magnitude();
                server_messages.send_entity_message(
                    entity.clone(),
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
            ClientMessage::SetHotbarSlot(SetHotbarSlot {
                slot_index,
                slot,
                response_tx,
            }) => {
                if let Some(_) = hotbar.set_slot(slot_index, slot) {
                    response_tx.send(Ok(())).ok();
                } else {
                    response_tx.send(Err(SetHotbarSlotError::InvalidSlot)).ok();
                }
            }
            _ => println!("Received unimplemented client message"),
        }
    }
}

#[system(for_each)]
#[read_component(BasicStats)]
#[read_component(Inventory)]
#[read_component(Equipment)]
#[read_component(Level)]
#[read_component(Position)]
#[read_component(SkillList)]
#[read_component(Hotbar)]
#[filter(!component::<GameClient>())]
pub fn game_server_disconnect_handler(
    world: &SubWorld,
    cmd: &mut CommandBuffer,
    entity: &Entity,
    client_entity: Option<&ClientEntity>,
    info: &CharacterInfo,
    position: &Position,
    #[resource] client_entity_list: &mut ClientEntityList,
) {
    if let Ok(entry) = world.entry_ref(*entity) {
        let basic_stats = entry.get_component::<BasicStats>();
        let inventory = entry.get_component::<Inventory>();
        let equipment = entry.get_component::<Equipment>();
        let level = entry.get_component::<Level>();
        let position = entry.get_component::<Position>();
        let skill_list = entry.get_component::<SkillList>();
        let hotbar = entry.get_component::<Hotbar>();
        let storage = CharacterStorage {
            info: info.clone(),
            basic_stats: basic_stats.unwrap().clone(),
            inventory: inventory.unwrap().clone(),
            equipment: equipment.unwrap().clone(),
            level: level.unwrap().clone(),
            position: position.unwrap().clone(),
            skill_list: skill_list.unwrap().clone(),
            hotbar: hotbar.unwrap().clone(),
            delete_time: None,
        };
        storage.save().ok();
    }

    if let Some(client_entity) = client_entity {
        client_entity_list
            .get_zone_mut(position.zone as usize)
            .unwrap()
            .free(ClientEntityId(client_entity.id.0));
    }

    cmd.remove(*entity);
}
