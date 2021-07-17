use clap::{App, Arg};
use lazy_static::lazy_static;
use legion::{system, systems::CommandBuffer, world::SubWorld, Entity, Query};
use nalgebra::Point3;
use std::num::{ParseFloatError, ParseIntError};

use crate::game::{
    bundles::client_entity_teleport_zone,
    components::{AbilityValues, ClientEntity, GameClient, Position},
    messages::server::{ServerMessage, Whisper},
    resources::{ClientEntityList, PendingChatCommandList},
};

pub struct ChatCommandWorld<'a> {
    cmd: &'a mut CommandBuffer,
    client_entity_list: &'a mut ClientEntityList,
}

pub struct ChatCommandUser<'a> {
    ability_values: &'a AbilityValues,
    client_entity: &'a ClientEntity,
    entity: &'a Entity,
    game_client: &'a GameClient,
    position: &'a Position,
}

lazy_static! {
    pub static ref GM_COMMANDS: App<'static> = {
        App::new("GM Commands")
            .subcommand(App::new("help"))
            .subcommand(App::new("where"))
            .subcommand(App::new("ability_values"))
            .subcommand(
                App::new("mm")
                    .arg(Arg::new("zone").required(true))
                    .arg(Arg::new("x").required(true))
                    .arg(Arg::new("y").required(true)),
            )
    };
}

fn send_multiline_whisper(client: &GameClient, str: &str) {
    for line in str.lines() {
        client
            .server_message_tx
            .send(ServerMessage::Whisper(Whisper {
                from: String::from("SERVER"),
                text: line.to_string(),
            }))
            .ok();
    }
}

fn send_gm_commands_help(client: &GameClient) {
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
    chat_command_world: &mut ChatCommandWorld,
    chat_command_user: &ChatCommandUser,
    command_text: &str,
) -> Result<(), GMCommandError> {
    let mut args = shellwords::split(command_text)?;
    args.insert(0, String::new()); // Clap expects arg[0] to be like executable name
    let matches = GM_COMMANDS.clone().try_get_matches_from(args)?;

    match matches.subcommand().ok_or(GMCommandError::InvalidCommand)? {
        ("where", _) => {
            chat_command_user
                .game_client
                .server_message_tx
                .send(ServerMessage::Whisper(Whisper {
                    from: String::from("SERVER"),
                    text: format!(
                        "zone: {} x: {} y: {} z: {}",
                        chat_command_user.position.zone,
                        chat_command_user.position.position.x,
                        chat_command_user.position.position.y,
                        chat_command_user.position.position.z
                    ),
                }))
                .ok();
        }
        ("mm", matches) => {
            let zone = matches.value_of("zone").unwrap().parse::<u16>()?;
            let x = matches.value_of("x").unwrap().parse::<f32>()? * 1000.0;
            let y = matches.value_of("y").unwrap().parse::<f32>()? * 1000.0;

            if chat_command_world
                .client_entity_list
                .get_zone(zone as usize)
                .is_some()
            {
                client_entity_teleport_zone(
                    chat_command_world.cmd,
                    chat_command_world.client_entity_list,
                    chat_command_user.entity,
                    chat_command_user.client_entity,
                    chat_command_user.position,
                    Position::new(Point3::new(x, y, 0.0), zone),
                    Some(chat_command_user.game_client),
                );
            } else {
                send_multiline_whisper(
                    chat_command_user.game_client,
                    &format!("Invalid zone id {}", zone),
                );
            }
        }
        ("ability_values", _) => {
            send_multiline_whisper(
                chat_command_user.game_client,
                &format!("{:?}", chat_command_user.ability_values),
            );
        }
        _ => return Err(GMCommandError::InvalidCommand),
    }

    Ok(())
}

#[allow(clippy::type_complexity)]
#[system]
pub fn chat_commands(
    world: &mut SubWorld,
    cmd: &mut CommandBuffer,
    user_query: &mut Query<(
        &AbilityValues,
        &ClientEntity,
        &GameClient,
        &Position,
    )>,
    #[resource] client_entity_list: &mut ClientEntityList,
    #[resource] pending_chat_commands: &mut PendingChatCommandList,
) {
    let mut chat_command_world = ChatCommandWorld {
        cmd,
        client_entity_list,
    };

    for (entity, command) in pending_chat_commands.iter() {
        if let Ok((ability_values, client_entity, game_client, position)) =
            user_query.get(world, *entity)
        {
            let chat_command_user = ChatCommandUser {
                ability_values,
                client_entity,
                entity,
                game_client,
                position,
            };

            if handle_gm_command(&mut chat_command_world, &chat_command_user, &command[1..]).is_err() {
                send_gm_commands_help(chat_command_user.game_client);
            }
        }
    }

    pending_chat_commands.clear();
}
