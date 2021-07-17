use clap::{App, Arg};
use lazy_static::lazy_static;
use legion::{
    component, system, systems::CommandBuffer, world::SubWorld, Entity, EntityStore, Query,
};
use log::warn;
use nalgebra::Point3;
use std::num::{ParseFloatError, ParseIntError};

use crate::{
    data::{account::AccountStorage, character::CharacterStorage, item::Item},
    game::{
        bundles::{client_entity_leave_zone, client_entity_teleport_zone},
        components::{
            AbilityValues, BasicStatType, BasicStats, CharacterInfo, ClientEntity,
            ClientEntityType, ClientEntityVisibility, Command, Equipment, EquipmentIndex,
            EquipmentItemDatabase, ExperiencePoints, GameClient, HealthPoints, Hotbar, Inventory,
            ItemSlot, Level, ManaPoints, MoveSpeed, NextCommand, Position, QuestState, SkillList,
            SkillPoints, StatPoints, Team, UnionMembership, WorldClient,
        },
        messages::{
            client::{
                ChangeEquipment, ClientMessage, ConnectionRequestError, GameConnectionResponse,
                JoinZoneResponse, LogoutRequest, ReviveRequestType, SetHotbarSlot,
                SetHotbarSlotError,
            },
            server::{self, LogoutReply, ServerMessage, UpdateBasicStat, UpdateInventory, Whisper},
        },
        resources::{
            ClientEntityList, GameData, LoginTokens, PendingQuestTrigger, PendingQuestTriggerList,
            ServerMessages, WorldTime,
        },
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
                        client.login_token = message.login_token;
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

                                let (health_points, mana_points, position) =
                                    if character.health_points.hp == 0 {
                                        (
                                            HealthPoints::new(ability_values.max_health as u32),
                                            ManaPoints::new(ability_values.max_mana as u32),
                                            Position::new(
                                                character.info.revive_position,
                                                character.info.revive_zone,
                                            ),
                                        )
                                    } else {
                                        (
                                            character.health_points.clone(),
                                            character.mana_points.clone(),
                                            character.position.clone(),
                                        )
                                    };

                                cmd.add_component(*entity, character.info.clone());
                                cmd.add_component(*entity, character.basic_stats.clone());
                                cmd.add_component(*entity, character.inventory.clone());
                                cmd.add_component(*entity, character.equipment.clone());
                                cmd.add_component(*entity, character.level.clone());
                                cmd.add_component(*entity, character.experience_points.clone());
                                cmd.add_component(*entity, character.skill_list.clone());
                                cmd.add_component(*entity, character.hotbar.clone());
                                cmd.add_component(*entity, character.skill_points.clone());
                                cmd.add_component(*entity, character.stat_points.clone());
                                cmd.add_component(*entity, character.quest_state.clone());
                                cmd.add_component(*entity, character.union_membership.clone());
                                cmd.add_component(
                                    *entity,
                                    game_data.motions.get_character_motions(
                                        weapon_motion_type,
                                        character.info.gender as usize,
                                    ),
                                );
                                cmd.add_component(
                                    *entity,
                                    MoveSpeed::new(ability_values.run_speed),
                                );
                                cmd.add_component(*entity, Command::default());
                                cmd.add_component(*entity, NextCommand::default());
                                cmd.add_component(*entity, Team::default_character());
                                cmd.add_component(*entity, health_points.clone());
                                cmd.add_component(*entity, mana_points.clone());
                                cmd.add_component(*entity, position.clone());
                                cmd.add_component(*entity, ability_values);

                                GameConnectionResponse {
                                    packet_sequence_id: 123,
                                    character_info: character.info,
                                    position,
                                    equipment: character.equipment,
                                    basic_stats: character.basic_stats,
                                    level: character.level,
                                    experience_points: character.experience_points,
                                    inventory: character.inventory,
                                    skill_list: character.skill_list,
                                    hotbar: character.hotbar,
                                    health_points,
                                    mana_points,
                                    stat_points: character.stat_points,
                                    skill_points: character.skill_points,
                                    quest_state: character.quest_state,
                                    union_membership: character.union_membership,
                                }
                            })
                    });
                message.response_tx.send(response).ok();
            }
            _ => warn!("Received unexpected client message {:?}", message),
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
    #[resource] world_time: &WorldTime,
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
                                world_time: world_time.now,
                            })
                            .ok();
                    }
                }
            }
            _ => warn!("Received unexpected client message {:?}", message),
        }
    }
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
    client_entity: &ClientEntity,
    position: &Position,
    ability_values: &AbilityValues,
    client_entity_list: &mut ClientEntityList,
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

            if client_entity_list.get_zone(zone as usize).is_some() {
                client_entity_teleport_zone(
                    cmd,
                    client_entity_list,
                    entity,
                    client_entity,
                    position,
                    Position::new(Point3::new(x, y, 0.0), zone),
                    Some(client),
                );
            } else {
                send_multiline_whisper(client, &format!("Invalid zone id {}", zone));
            }
        }
        ("ability_values", _) => {
            send_multiline_whisper(client, &format!("{:?}", ability_values));
        }
        _ => return Err(GMCommandError::InvalidCommand),
    }

    Ok(())
}

#[allow(clippy::type_complexity)]
#[system]
pub fn game_server_main(
    cmd: &mut CommandBuffer,
    world: &mut SubWorld,
    game_client_query: &mut Query<(
        Entity,
        &mut GameClient,
        &ClientEntity,
        &Position,
        &mut BasicStats,
        &mut StatPoints,
        &mut Hotbar,
        &mut Equipment,
        &mut Inventory,
        &AbilityValues,
        &Command,
        &CharacterInfo,
        &Level,
        &SkillList,
    )>,
    world_client_query: &mut Query<&WorldClient>,
    #[resource] client_entity_list: &mut ClientEntityList,
    #[resource] pending_quest_trigger_list: &mut PendingQuestTriggerList,
    #[resource] server_messages: &mut ServerMessages,
    #[resource] game_data: &GameData,
) {
    let (world_client_query_world, mut world) = world.split_for_query(world_client_query);

    game_client_query.for_each_mut(
        &mut world,
        |(
            entity,
            client,
            client_entity,
            position,
            basic_stats,
            stat_points,
            hotbar,
            equipment,
            inventory,
            ability_values,
            command,
            character_info,
            level,
            skill_list,
        )| {
            if let Ok(message) = client.client_message_rx.try_recv() {
                match message {
                    ClientMessage::Chat(text) => {
                        if text.chars().next().map_or(false, |c| c == '/') {
                            if handle_gm_command(
                                cmd,
                                entity,
                                client,
                                &text[1..],
                                client_entity,
                                position,
                                ability_values,
                                client_entity_list,
                            )
                            .is_err()
                            {
                                send_gm_commands_help(client);
                            }
                        } else {
                            server_messages.send_entity_message(
                                *entity,
                                ServerMessage::LocalChat(server::LocalChat {
                                    entity_id: client_entity.id,
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
                                let equipment_slot =
                                    equipment.get_equipment_slot_mut(equipment_index);

                                if let Some(Item::Equipment(equipment_item)) = inventory_slot {
                                    let previous = equipment_slot.take();
                                    *equipment_slot = Some(equipment_item.clone());
                                    *inventory_slot = previous.map(Item::Equipment);

                                    client
                                        .server_message_tx
                                        .send(ServerMessage::UpdateInventory(UpdateInventory {
                                            is_reward: false,
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
                                            entity_id: client_entity.id,
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
                                                is_reward: false,
                                                items: vec![
                                                    (ItemSlot::Equipped(equipment_index), None),
                                                    (inventory_slot, Some(item.clone())),
                                                ],
                                            }))
                                            .ok();

                                        server_messages.send_entity_message(
                                            *entity,
                                            ServerMessage::UpdateEquipment(
                                                server::UpdateEquipment {
                                                    entity_id: client_entity.id,
                                                    equipment_index,
                                                    item: None,
                                                },
                                            ),
                                        );
                                    }
                                    Err(item) => {
                                        *equipment_slot = Some(item);
                                    }
                                }
                            }
                        }

                        cmd.add_component(
                            *entity,
                            game_data.ability_value_calculator.calculate(
                                character_info,
                                level,
                                equipment,
                                inventory,
                                basic_stats,
                                skill_list,
                            ),
                        );
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

                                cmd.add_component(
                                    *entity,
                                    game_data.ability_value_calculator.calculate(
                                        character_info,
                                        level,
                                        equipment,
                                        inventory,
                                        basic_stats,
                                        skill_list,
                                    ),
                                );
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
                    ClientMessage::LogoutRequest(request) => {
                        if let LogoutRequest::ReturnToCharacterSelect = request {
                            // Send ReturnToCharacterSelect via world_client
                            world_client_query.for_each(
                                &world_client_query_world,
                                |world_client| {
                                    if world_client.login_token == client.login_token {
                                        world_client
                                            .server_message_tx
                                            .send(ServerMessage::ReturnToCharacterSelect)
                                            .ok();
                                    }
                                },
                            );
                        }

                        client
                            .server_message_tx
                            .send(ServerMessage::LogoutReply(LogoutReply { result: Ok(()) }))
                            .ok();

                        client_entity_leave_zone(
                            cmd,
                            client_entity_list,
                            entity,
                            client_entity,
                            position,
                        );
                    }
                    ClientMessage::ReviveRequest(revive_request_type) => {
                        if command.is_dead() {
                            let new_position = match revive_request_type {
                                ReviveRequestType::RevivePosition => {
                                    let revive_position = if let Some(zone_data) =
                                        game_data.zones.get_zone(position.zone as usize)
                                    {
                                        if let Some(revive_position) =
                                            zone_data.get_closest_revive_position(position.position)
                                        {
                                            revive_position
                                        } else {
                                            zone_data.start_position
                                        }
                                    } else {
                                        position.position
                                    };

                                    Position::new(revive_position, position.zone)
                                }
                                ReviveRequestType::SavePosition => Position::new(
                                    character_info.revive_position,
                                    character_info.revive_zone,
                                ),
                            };

                            cmd.add_component(
                                *entity,
                                HealthPoints::new(ability_values.max_health as u32),
                            );
                            cmd.add_component(
                                *entity,
                                ManaPoints::new(ability_values.max_mana as u32),
                            );
                            client_entity_teleport_zone(
                                cmd,
                                client_entity_list,
                                entity,
                                client_entity,
                                position,
                                new_position,
                                Some(client),
                            );
                        }
                    }
                    ClientMessage::QuestTrigger(trigger_hash) => {
                        pending_quest_trigger_list.push(PendingQuestTrigger {
                            trigger_entity: *entity,
                            trigger_hash,
                        });
                    }
                    _ => warn!("Received unimplemented client message {:?}", message),
                }
            }
        },
    );
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
#[read_component(QuestState)]
#[read_component(UnionMembership)]
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
        let quest_state = entry.get_component::<QuestState>();
        let union_membership = entry.get_component::<UnionMembership>();
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
            quest_state: quest_state.unwrap().clone(),
            union_membership: union_membership.unwrap().clone(),
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
