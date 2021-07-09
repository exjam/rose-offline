use legion::{
    component, system, systems::CommandBuffer, world::SubWorld, Entity, EntityStore, Query,
};
use nalgebra::Point3;
use std::num::{ParseFloatError, ParseIntError};

use crate::{
    data::{account::AccountStorage, character::CharacterStorage, item::Item},
    game::{
        components::{
            AbilityValues, BasicStatType, BasicStats, CharacterInfo, ClientEntity,
            ClientEntityType, ClientEntityVisibility, Command, Equipment, EquipmentIndex,
            EquipmentItemDatabase, ExperiencePoints, GameClient, HealthPoints, Hotbar, Inventory,
            ItemSlot, Level, ManaPoints, MoveSpeed, NextCommand, Position, SkillList, SkillPoints,
            StatPoints, Team, WorldClient,
        },
        messages::{
            client::{
                ChangeEquipment, ClientMessage, ConnectionRequestError, GameConnectionResponse,
                JoinZoneResponse, SetHotbarSlot, SetHotbarSlotError,
            },
            server::{self, LogoutReply, ServerMessage, UpdateBasicStat, UpdateInventory, Whisper},
        },
        resources::{ClientEntityList, GameData, LoginTokens, ServerMessages},
    },
};

#[system(for_each)]
#[filter(!component::<CharacterInfo>())]
pub fn game_server_authentication(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    client: &mut GameClient,
    #[resource] login_tokens: &mut LoginTokens,
    #[resource] game_data: &GameData,
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
                            .map(|character| {
                                let ability_values = game_data.ability_value_calculator.calculate(
                                    &character.info,
                                    &character.level,
                                    &character.equipment,
                                    &character.inventory,
                                    &character.basic_stats,
                                    &character.skill_list,
                                );

                                let weapon_motion_type = game_data
                                    .items
                                    .get_equipped_weapon_item_data(
                                        &character.equipment,
                                        EquipmentIndex::WeaponRight,
                                    )
                                    .map(|item_data| item_data.motion_type)
                                    .unwrap_or(0)
                                    as usize;

                                cmd.add_component(
                                    *entity,
                                    game_data.motions.get_character_motions(
                                        weapon_motion_type,
                                        character.info.gender as usize,
                                    ),
                                );
                                cmd.add_component(
                                    *entity,
                                    MoveSpeed {
                                        speed: ability_values.run_speed,
                                    },
                                );
                                cmd.add_component(*entity, Command::default());
                                cmd.add_component(*entity, NextCommand::default());
                                cmd.add_component(*entity, ability_values);
                                cmd.add_component(*entity, character.info.clone());
                                cmd.add_component(*entity, character.basic_stats.clone());
                                cmd.add_component(*entity, character.inventory.clone());
                                cmd.add_component(*entity, character.equipment.clone());
                                cmd.add_component(*entity, character.level.clone());
                                cmd.add_component(*entity, character.experience_points.clone());
                                cmd.add_component(*entity, character.position.clone());
                                cmd.add_component(*entity, character.skill_list.clone());
                                cmd.add_component(*entity, character.hotbar.clone());
                                cmd.add_component(*entity, character.health_points.clone());
                                cmd.add_component(*entity, character.mana_points.clone());
                                cmd.add_component(*entity, character.skill_points.clone());
                                cmd.add_component(*entity, character.stat_points.clone());
                                cmd.add_component(*entity, Team::default_character());

                                GameConnectionResponse {
                                    packet_sequence_id: 123,
                                    character_info: character.info,
                                    position: character.position,
                                    equipment: character.equipment,
                                    basic_stats: character.basic_stats,
                                    level: character.level,
                                    experience_points: character.experience_points,
                                    inventory: character.inventory,
                                    skill_list: character.skill_list,
                                    hotbar: character.hotbar,
                                    health_points: character.health_points,
                                    mana_points: character.mana_points,
                                    stat_points: character.stat_points,
                                    skill_points: character.skill_points,
                                }
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
    experience_points: &ExperiencePoints,
    team: &Team,
    health_points: &HealthPoints,
    mana_points: &ManaPoints,
    position: &Position,
    #[resource] client_entity_list: &mut ClientEntityList,
) {
    if let Ok(message) = client.client_message_rx.try_recv() {
        match message {
            ClientMessage::JoinZoneRequest(message) => {
                if let Some(zone) = client_entity_list.get_zone_mut(position.zone as usize) {
                    if let Some(client_entity) =
                        zone.allocate(ClientEntityType::Character, *entity, position.position)
                    {
                        let entity_id = client_entity.id;
                        cmd.add_component(*entity, client_entity);
                        cmd.add_component(*entity, ClientEntityVisibility::new());

                        message
                            .response_tx
                            .send(JoinZoneResponse {
                                entity_id,
                                level: level.clone(),
                                experience_points: experience_points.clone(),
                                team: team.clone(),
                                health_points: health_points.clone(),
                                mana_points: mana_points.clone(),
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
            .subcommand(App::new("ability_values"))
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

fn send_multiline_whisper(client: &mut GameClient, str: &str) {
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
    ability_values: &AbilityValues,
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

            client
                .server_message_tx
                .send(ServerMessage::Teleport(server::Teleport {
                    entity_id: entity_id.id,
                    zone_no: zone,
                    x,
                    y,
                    run_mode: 1,
                    ride_mode: 0,
                }))
                .ok();
        }
        ("ability_values", _) => {
            send_multiline_whisper(client, &format!("{:?}", ability_values));
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
    basic_stats: &mut BasicStats,
    stat_points: &mut StatPoints,
    hotbar: &mut Hotbar,
    equipment: &mut Equipment,
    inventory: &mut Inventory,
    ability_values: &AbilityValues,
    #[resource] client_entity_list: &mut ClientEntityList,
    #[resource] server_messages: &mut ServerMessages,
    #[resource] game_data: &GameData,
) {
    if let Ok(message) = client.client_message_rx.try_recv() {
        match message {
            ClientMessage::Chat(text) => {
                if text.chars().next().map_or(false, |c| c == '/') {
                    if handle_gm_command(
                        cmd,
                        entity,
                        client,
                        &text[1..],
                        entity_id,
                        position,
                        ability_values,
                    )
                    .is_err()
                    {
                        send_gm_commands_help(client);
                    }
                } else {
                    server_messages.send_entity_message(
                        *entity,
                        ServerMessage::LocalChat(server::LocalChat {
                            entity_id: entity_id.id,
                            text,
                        }),
                    );
                }
            }
            ClientMessage::Move(message) => {
                let mut move_target_entity = None;
                if let Some(target_entity_id) = message.target_entity_id {
                    if let Some(target_entity) = client_entity_list
                        .get_zone(position.zone as usize)
                        .and_then(|zone| zone.get_entity(target_entity_id))
                    {
                        move_target_entity = Some(target_entity);
                    }
                }

                let destination = Point3::new(message.x, message.y, message.z as f32);
                cmd.add_component(
                    *entity,
                    NextCommand::with_move(destination, move_target_entity),
                );
            }
            ClientMessage::Attack(message) => {
                if let Some(target_entity) = client_entity_list
                    .get_zone(position.zone as usize)
                    .and_then(|zone| zone.get_entity(message.target_entity_id))
                {
                    cmd.add_component(*entity, NextCommand::with_attack(target_entity));
                } else {
                    cmd.add_component(*entity, NextCommand::with_stop());
                }
            }
            ClientMessage::SetHotbarSlot(SetHotbarSlot {
                slot_index,
                slot,
                response_tx,
            }) => {
                if hotbar.set_slot(slot_index, slot).is_some() {
                    response_tx.send(Ok(())).ok();
                } else {
                    response_tx.send(Err(SetHotbarSlotError::InvalidSlot)).ok();
                }
            }
            ClientMessage::ChangeEquipment(ChangeEquipment {
                equipment_index,
                item_slot,
            }) => {
                // TODO: Cannot change equipment whilst casting spell
                // TODO: Cannot change equipment whilst stunned

                if let Some(item_slot) = item_slot {
                    // TODO: Check if satisfy equipment requirements
                    // TODO: Handle 2 handed weapons
                    // Try equip item from inventory
                    if let Some(inventory_slot) = inventory.get_item_slot_mut(item_slot) {
                        let equipment_slot = equipment.get_equipment_slot_mut(equipment_index);

                        if let Some(Item::Equipment(equipment_item)) = inventory_slot {
                            let previous = equipment_slot.take();
                            *equipment_slot = Some(equipment_item.clone());
                            *inventory_slot = previous.map(Item::Equipment);

                            client
                                .server_message_tx
                                .send(ServerMessage::UpdateInventory(UpdateInventory {
                                    items: vec![
                                        (
                                            ItemSlot::Equipped(equipment_index),
                                            equipment_slot.clone().map(Item::Equipment),
                                        ),
                                        (item_slot, inventory_slot.clone()),
                                    ],
                                }))
                                .ok();

                            server_messages.send_entity_message(
                                *entity,
                                ServerMessage::UpdateEquipment(server::UpdateEquipment {
                                    entity_id: entity_id.id,
                                    equipment_index,
                                    item: equipment_slot.clone(),
                                }),
                            );
                        }
                    }
                } else {
                    // Try unequip to inventory
                    let equipment_slot = equipment.get_equipment_slot_mut(equipment_index);
                    let item = equipment_slot.take();
                    if let Some(item) = item {
                        match inventory.try_add_equipment_item(item) {
                            Ok((inventory_slot, item)) => {
                                *equipment_slot = None;

                                client
                                    .server_message_tx
                                    .send(ServerMessage::UpdateInventory(UpdateInventory {
                                        items: vec![
                                            (ItemSlot::Equipped(equipment_index), None),
                                            (inventory_slot, Some(item.clone())),
                                        ],
                                    }))
                                    .ok();

                                server_messages.send_entity_message(
                                    *entity,
                                    ServerMessage::UpdateEquipment(server::UpdateEquipment {
                                        entity_id: entity_id.id,
                                        equipment_index,
                                        item: None,
                                    }),
                                );
                            }
                            Err(item) => {
                                *equipment_slot = Some(item);
                            }
                        }
                    }
                }
            }
            ClientMessage::IncreaseBasicStat(basic_stat_type) => {
                if let Some(cost) = game_data
                    .ability_value_calculator
                    .calculate_basic_stat_increase_cost(basic_stats, basic_stat_type)
                {
                    if cost < stat_points.points {
                        let value = match basic_stat_type {
                            BasicStatType::Strength => &mut basic_stats.strength,
                            BasicStatType::Dexterity => &mut basic_stats.dexterity,
                            BasicStatType::Intelligence => &mut basic_stats.intelligence,
                            BasicStatType::Concentration => &mut basic_stats.concentration,
                            BasicStatType::Charm => &mut basic_stats.charm,
                            BasicStatType::Sense => &mut basic_stats.sense,
                        };

                        stat_points.points -= cost;
                        *value += 1;

                        client
                            .server_message_tx
                            .send(ServerMessage::UpdateBasicStat(UpdateBasicStat {
                                basic_stat_type,
                                value: *value,
                            }))
                            .ok();
                    }
                }
            }
            ClientMessage::PickupDroppedItem(message) => {
                if let Some(target_entity) = client_entity_list
                    .get_zone(position.zone as usize)
                    .and_then(|zone| zone.get_entity(message.target_entity_id))
                {
                    cmd.add_component(
                        *entity,
                        NextCommand::with_pickup_dropped_item(target_entity),
                    );
                } else {
                    cmd.add_component(*entity, NextCommand::with_stop());
                }
            }
            ClientMessage::LogoutRequest(_) => {
                client
                    .server_message_tx
                    .send(ServerMessage::LogoutReply(LogoutReply { result: Ok(()) }))
                    .ok();
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
#[read_component(ExperiencePoints)]
#[read_component(Position)]
#[read_component(SkillList)]
#[read_component(Hotbar)]
#[read_component(HealthPoints)]
#[read_component(ManaPoints)]
#[read_component(StatPoints)]
#[read_component(SkillPoints)]
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
        let experience_points = entry.get_component::<ExperiencePoints>();
        let position = entry.get_component::<Position>();
        let skill_list = entry.get_component::<SkillList>();
        let hotbar = entry.get_component::<Hotbar>();
        let health_points = entry.get_component::<HealthPoints>();
        let mana_points = entry.get_component::<ManaPoints>();
        let stat_points = entry.get_component::<StatPoints>();
        let skill_points = entry.get_component::<SkillPoints>();
        let storage = CharacterStorage {
            info: info.clone(),
            basic_stats: basic_stats.unwrap().clone(),
            inventory: inventory.unwrap().clone(),
            equipment: equipment.unwrap().clone(),
            level: level.unwrap().clone(),
            experience_points: experience_points.unwrap().clone(),
            position: position.unwrap().clone(),
            skill_list: skill_list.unwrap().clone(),
            hotbar: hotbar.unwrap().clone(),
            delete_time: None,
            health_points: health_points.unwrap().clone(),
            mana_points: mana_points.unwrap().clone(),
            stat_points: stat_points.unwrap().clone(),
            skill_points: skill_points.unwrap().clone(),
        };
        storage.save().ok();
    }

    if let Some(client_entity) = client_entity {
        client_entity_list
            .get_zone_mut(position.zone as usize)
            .unwrap()
            .free(client_entity.id);
    }

    cmd.remove(*entity);
}
