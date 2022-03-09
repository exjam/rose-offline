use bevy_ecs::{
    prelude::{Commands, Entity, EventReader, EventWriter, Mut, Query, Res, ResMut},
    system::SystemParam,
};
use clap::{Arg, PossibleValue};
use lazy_static::lazy_static;
use nalgebra::{Point2, Point3};
use rand::prelude::SliceRandom;
use std::{
    f32::consts::PI,
    num::{ParseFloatError, ParseIntError},
};

use rose_data::{AbilityType, EquipmentIndex, Item, ItemReference, ItemType, SkillId, ZoneId};

use crate::game::{
    bundles::{
        ability_values_add_value, ability_values_set_value, client_entity_join_zone,
        client_entity_teleport_zone, CharacterBundle,
    },
    components::{
        AbilityValues, BasicStats, BotAi, BotAiState, CharacterInfo, ClientEntity,
        ClientEntitySector, ClientEntityType, Command, EquipmentItemDatabase, GameClient,
        Inventory, Level, Money, MotionData, MoveMode, MoveSpeed, NextCommand, PartyMembership,
        PassiveRecoveryTime, PersonalStore, Position, SkillList, SkillPoints, Stamina, StatPoints,
        StatusEffects, StatusEffectsRegen, Team, UnionMembership, PERSONAL_STORE_ITEM_SLOTS,
    },
    events::{ChatCommandEvent, RewardXpEvent},
    messages::server::{LearnSkillSuccess, ServerMessage, UpdateSpeed, Whisper},
    resources::{BotList, BotListEntry, ClientEntityList, ServerMessages},
    GameData,
};

#[derive(SystemParam)]
pub struct ChatCommandParams<'w, 's> {
    commands: Commands<'w, 's>,
    bot_list: ResMut<'w, BotList>,
    client_entity_list: ResMut<'w, ClientEntityList>,
    game_data: Res<'w, GameData>,
    reward_xp_events: EventWriter<'w, 's, RewardXpEvent>,
    server_messages: ResMut<'w, ServerMessages>,
}

pub struct ChatCommandUser<'w, 's> {
    entity: Entity,
    ability_values: &'s AbilityValues,
    client_entity: &'s ClientEntity,
    client_entity_sector: &'s ClientEntitySector,
    game_client: &'s GameClient,
    level: &'s Level,
    position: &'s Position,
    basic_stats: &'s mut Mut<'w, BasicStats>,
    character_info: &'s mut Mut<'w, CharacterInfo>,
    inventory: &'s mut Mut<'w, Inventory>,
    skill_list: &'s mut Mut<'w, SkillList>,
    skill_points: &'s mut Mut<'w, SkillPoints>,
    stamina: &'s mut Mut<'w, Stamina>,
    stat_points: &'s mut Mut<'w, StatPoints>,
    union_membership: &'s mut Mut<'w, UnionMembership>,
}

lazy_static! {
    pub static ref CHAT_COMMANDS: clap::Command<'static> = {
        clap::Command::new("Chat Commands")
            .subcommand(clap::Command::new("help"))
            .subcommand(clap::Command::new("where"))
            .subcommand(clap::Command::new("ability_values"))
            .subcommand(
                clap::Command::new("mm")
                    .arg(Arg::new("zone").required(true))
                    .arg(Arg::new("x"))
                    .arg(Arg::new("y")),
            )
            .subcommand(clap::Command::new("level").arg(Arg::new("level").required(true)))
            .subcommand(clap::Command::new("bot").arg(Arg::new("n").required(true)))
            .subcommand(clap::Command::new("snowball_fight").arg(Arg::new("n").required(true)))
            .subcommand(clap::Command::new("shop").arg(Arg::new("item_type").required(true)))
            .subcommand(
                clap::Command::new("add")
                    .arg(Arg::new("ability_type").required(true))
                    .arg(Arg::new("value").required(true)),
            )
            .subcommand(
                clap::Command::new("set")
                    .arg(Arg::new("ability_type").required(true))
                    .arg(Arg::new("value").required(true)),
            )
            .subcommand(clap::Command::new("speed").arg(Arg::new("speed").required(true)))
            .subcommand(
                clap::Command::new("skill")
                    .arg(
                        Arg::new("cmd")
                            .possible_values([
                                PossibleValue::new("add"),
                                PossibleValue::new("remove"),
                            ])
                            .required(true),
                    )
                    .arg(Arg::new("id").required(true)),
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

fn send_chat_commands_help(client: &GameClient) {
    for subcommand in CHAT_COMMANDS.get_subcommands() {
        let mut help_string = String::from(subcommand.get_name());
        for arg in subcommand.get_arguments() {
            if arg.get_id() == "help" || arg.get_id() == "version" {
                continue;
            }

            help_string.push(' ');
            if !arg.is_required_set() {
                help_string.push('[');
                help_string.push_str(arg.get_id());
                help_string.push(']');
            } else {
                help_string.push_str(arg.get_id());
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

pub enum ChatCommandError {
    InvalidCommand,
    InvalidArguments,
    WithMessage(String),
}

impl From<shellwords::MismatchedQuotes> for ChatCommandError {
    fn from(_: shellwords::MismatchedQuotes) -> Self {
        Self::InvalidCommand
    }
}

impl From<clap::Error> for ChatCommandError {
    fn from(error: clap::Error) -> Self {
        match error.kind() {
            clap::ErrorKind::MissingRequiredArgument => Self::InvalidArguments,
            _ => Self::InvalidCommand,
        }
    }
}

impl From<ParseIntError> for ChatCommandError {
    fn from(_: ParseIntError) -> Self {
        Self::InvalidArguments
    }
}

impl From<ParseFloatError> for ChatCommandError {
    fn from(_: ParseFloatError) -> Self {
        Self::InvalidArguments
    }
}

fn create_bot_entity(
    chat_command_params: &mut ChatCommandParams,
    name: String,
    gender: u8,
    face: u8,
    hair: u8,
    position: Position,
) -> Option<Entity> {
    let mut bot_data = chat_command_params
        .game_data
        .character_creator
        .create(name, gender, 1, face, hair)
        .ok()?;

    let status_effects = StatusEffects::new();
    let status_effects_regen = StatusEffectsRegen::new();

    let ability_values = chat_command_params
        .game_data
        .ability_value_calculator
        .calculate(
            &bot_data.info,
            &bot_data.level,
            &bot_data.equipment,
            &bot_data.basic_stats,
            &bot_data.skill_list,
            &status_effects,
        );

    let move_speed = MoveSpeed::new(ability_values.get_run_speed() as f32);

    let weapon_motion_type = chat_command_params
        .game_data
        .items
        .get_equipped_weapon_item_data(&bot_data.equipment, EquipmentIndex::WeaponRight)
        .map(|item_data| item_data.motion_type)
        .unwrap_or(0) as usize;

    let motion_data = MotionData::from_character(
        chat_command_params.game_data.motions.as_ref(),
        weapon_motion_type,
        bot_data.info.gender as usize,
    );

    bot_data.position = position.clone();
    bot_data.health_points.hp = ability_values.get_max_health();
    bot_data.mana_points.mp = ability_values.get_max_mana();

    let entity = chat_command_params
        .commands
        .spawn()
        .insert(BotAi::new(BotAiState::Farm))
        .insert_bundle(CharacterBundle {
            ability_values,
            basic_stats: bot_data.basic_stats,
            command: Command::default(),
            equipment: bot_data.equipment,
            experience_points: bot_data.experience_points,
            health_points: bot_data.health_points,
            hotbar: bot_data.hotbar,
            info: bot_data.info,
            inventory: bot_data.inventory,
            level: bot_data.level,
            mana_points: bot_data.mana_points,
            motion_data,
            move_mode: MoveMode::Run,
            move_speed,
            next_command: NextCommand::default(),
            party_membership: PartyMembership::default(),
            passive_recovery_time: PassiveRecoveryTime::default(),
            position: bot_data.position,
            quest_state: bot_data.quest_state,
            skill_list: bot_data.skill_list,
            skill_points: bot_data.skill_points,
            stamina: bot_data.stamina,
            stat_points: bot_data.stat_points,
            status_effects,
            status_effects_regen,
            team: Team::default_character(),
            union_membership: bot_data.union_membership,
        })
        .id();

    client_entity_join_zone(
        &mut chat_command_params.commands,
        &mut chat_command_params.client_entity_list,
        entity,
        ClientEntityType::Character,
        &position,
    )
    .ok();

    Some(entity)
}

fn create_random_bot_entities(
    chat_command_params: &mut ChatCommandParams,
    num_bots: usize,
    spacing: f32,
    origin: Position,
) -> Vec<Entity> {
    let mut rng = rand::thread_rng();
    let genders = [0, 1];
    let faces = [1, 8, 15, 22, 29, 36, 43];
    let hair = [0, 5, 10, 15, 20];

    let spawn_radius = f32::max(num_bots as f32 * spacing, 100.0);
    let mut bot_entities = Vec::new();

    for i in 0..num_bots {
        let angle = (i as f32 * (2.0 * PI)) / num_bots as f32;
        let mut bot_position = origin.clone();
        bot_position.position.x += spawn_radius * angle.cos();
        bot_position.position.y += spawn_radius * angle.sin();

        if let Some(bot_entity) = create_bot_entity(
            chat_command_params,
            format!("Friend {}", chat_command_params.bot_list.len() as usize),
            *genders.choose(&mut rng).unwrap() as u8,
            *faces.choose(&mut rng).unwrap() as u8,
            *hair.choose(&mut rng).unwrap() as u8,
            bot_position,
        ) {
            chat_command_params
                .bot_list
                .push(BotListEntry::new(bot_entity));
            bot_entities.push(bot_entity);
        }
    }

    bot_entities
}

fn handle_chat_command(
    chat_command_params: &mut ChatCommandParams,
    chat_command_user: &mut ChatCommandUser,
    command_text: &str,
) -> Result<(), ChatCommandError> {
    let mut args = shellwords::split(command_text)?;
    args.insert(0, String::new()); // Clap expects arg[0] to be like executable name
    let command_matches = CHAT_COMMANDS.clone().try_get_matches_from(args)?;

    match command_matches
        .subcommand()
        .ok_or(ChatCommandError::InvalidCommand)?
    {
        ("where", _) => {
            let sector = chat_command_params
                .client_entity_list
                .get_zone(chat_command_user.position.zone_id)
                .map(|client_entity_zone| {
                    client_entity_zone.calculate_sector(chat_command_user.position.position.xy())
                })
                .unwrap_or_else(|| Point2::new(0u32, 0u32));

            chat_command_user
                .game_client
                .server_message_tx
                .send(ServerMessage::Whisper(Whisper {
                    from: String::from("SERVER"),
                    text: format!(
                        "zone: {} position: ({}, {}, {}) sector: ({}, {})",
                        chat_command_user.position.zone_id.get(),
                        chat_command_user.position.position.x,
                        chat_command_user.position.position.y,
                        chat_command_user.position.position.z,
                        sector.x,
                        sector.y,
                    ),
                }))
                .ok();
        }
        ("mm", arg_matches) => {
            let zone_id = arg_matches.value_of("zone").unwrap().parse::<ZoneId>()?;
            let (x, y) = if let (Some(x), Some(y)) =
                (arg_matches.value_of("x"), arg_matches.value_of("y"))
            {
                (x.parse::<f32>()? * 1000.0, y.parse::<f32>()? * 1000.0)
            } else if let Some(zone_data) = chat_command_params.game_data.zones.get_zone(zone_id) {
                (zone_data.start_position.x, zone_data.start_position.y)
            } else {
                (520.0, 520.0)
            };

            let _zone = chat_command_params
                .client_entity_list
                .get_zone(zone_id)
                .ok_or_else(|| {
                    ChatCommandError::WithMessage(format!("Invalid zone id {}", zone_id.get()))
                })?;

            client_entity_teleport_zone(
                &mut chat_command_params.commands,
                &mut chat_command_params.client_entity_list,
                chat_command_user.entity,
                chat_command_user.client_entity,
                chat_command_user.client_entity_sector,
                chat_command_user.position,
                Position::new(Point3::new(x, y, 0.0), zone_id),
                Some(chat_command_user.game_client),
            );
        }
        ("ability_values", _) => {
            send_multiline_whisper(
                chat_command_user.game_client,
                &format!("{:?}", chat_command_user.ability_values),
            );
        }
        ("level", arg_matches) => {
            let target_level = arg_matches.value_of("level").unwrap().parse::<u32>()?;
            let current_level = chat_command_user.level.level;
            let mut required_xp = 0;

            for level in current_level..target_level {
                required_xp += chat_command_params
                    .game_data
                    .ability_value_calculator
                    .calculate_levelup_require_xp(level);
            }

            chat_command_params
                .reward_xp_events
                .send(RewardXpEvent::new(
                    chat_command_user.entity,
                    required_xp,
                    0,
                    None,
                ));
        }
        ("bot", arg_matches) => {
            let num_bots = arg_matches.value_of("n").unwrap().parse::<usize>()?;
            create_random_bot_entities(
                chat_command_params,
                num_bots,
                15.0,
                chat_command_user.position.clone(),
            );
        }
        ("snowball_fight", arg_matches) => {
            let num_bots = arg_matches.value_of("n").unwrap().parse::<usize>()?;
            let bot_entities = create_random_bot_entities(
                chat_command_params,
                num_bots,
                15.0,
                chat_command_user.position.clone(),
            );

            for entity in bot_entities.into_iter() {
                let mut inventory = Inventory::new();
                inventory
                    .try_add_item(
                        Item::new(&ItemReference::new(ItemType::Consumable, 326), 999).unwrap(),
                    )
                    .ok();

                chat_command_params
                    .commands
                    .entity(entity)
                    .insert(inventory)
                    .insert(BotAi::new(BotAiState::SnowballFight));
            }
        }
        ("shop", arg_matches) => {
            let item_type_id = arg_matches
                .value_of("item_type")
                .unwrap()
                .parse::<usize>()?;
            let item_type: ItemType = chat_command_params
                .game_data
                .data_decoder
                .decode_item_type(item_type_id)
                .ok_or_else(|| {
                    ChatCommandError::WithMessage(format!("Invalid item type {}", item_type_id))
                })?;

            let mut all_items: Vec<ItemReference> = chat_command_params
                .game_data
                .items
                .iter_items(item_type)
                .collect();
            all_items.sort_by(|a, b| a.item_number.cmp(&b.item_number));

            let num_bots =
                (all_items.len() + PERSONAL_STORE_ITEM_SLOTS - 1) / PERSONAL_STORE_ITEM_SLOTS;
            let bot_entities = create_random_bot_entities(
                chat_command_params,
                num_bots,
                30.0,
                chat_command_user.position.clone(),
            );
            let mut index = 0usize;

            for (shop_index, entity) in bot_entities.into_iter().enumerate() {
                let mut store = PersonalStore::new(format!("Shop {}", shop_index), 0);
                let mut inventory = Inventory::new();

                for i in index..index + PERSONAL_STORE_ITEM_SLOTS {
                    if let Some(item) = all_items.get(i).and_then(|item| Item::new(item, 999)) {
                        if let Ok((slot, _)) = inventory.try_add_item(item) {
                            store.add_sell_item(slot, Money(1)).ok();
                        }
                    }
                }

                index += PERSONAL_STORE_ITEM_SLOTS;

                chat_command_params
                    .commands
                    .entity(entity)
                    .insert(store)
                    .insert(inventory)
                    .insert(NextCommand::with_personal_store());
            }
        }
        ("add", arg_matches) => {
            let ability_type_str = arg_matches.value_of("ability_type").unwrap();
            let value = arg_matches.value_of("value").unwrap().parse::<i32>()?;
            let ability_type = match ability_type_str {
                "strength" => AbilityType::Strength,
                "dexterity" => AbilityType::Dexterity,
                "intelligence" => AbilityType::Intelligence,
                "concentration" => AbilityType::Concentration,
                "charm" => AbilityType::Charm,
                "sense" => AbilityType::Sense,
                "bonus_point" => AbilityType::BonusPoint,
                "skillpoint" => AbilityType::Skillpoint,
                "money" => AbilityType::Money,
                "stamina" => AbilityType::Stamina,
                "health" => AbilityType::Health,
                "mana" => AbilityType::Mana,
                "experience" => AbilityType::Experience,
                "level" => AbilityType::Level,
                "union_point1" => AbilityType::UnionPoint1,
                "union_point2" => AbilityType::UnionPoint2,
                "union_point3" => AbilityType::UnionPoint3,
                "union_point4" => AbilityType::UnionPoint4,
                "union_point5" => AbilityType::UnionPoint5,
                "union_point6" => AbilityType::UnionPoint6,
                "union_point7" => AbilityType::UnionPoint7,
                "union_point8" => AbilityType::UnionPoint8,
                "union_point9" => AbilityType::UnionPoint9,
                "union_point10" => AbilityType::UnionPoint10,
                _ => {
                    return Err(ChatCommandError::WithMessage(format!(
                        "Invalid ability type {}",
                        ability_type_str
                    )))
                }
            };

            ability_values_add_value(
                ability_type,
                value,
                Some(chat_command_user.basic_stats),
                Some(chat_command_user.inventory),
                Some(chat_command_user.skill_points),
                Some(chat_command_user.stamina),
                Some(chat_command_user.stat_points),
                Some(chat_command_user.union_membership),
                Some(chat_command_user.game_client),
            );
        }
        ("set", arg_matches) => {
            let ability_type_str = arg_matches.value_of("ability_type").unwrap();
            let value = arg_matches.value_of("value").unwrap().parse::<i32>()?;
            let ability_type = match ability_type_str {
                "gender" => AbilityType::Gender,
                "face" => AbilityType::Face,
                "hair" => AbilityType::Hair,
                "class" => AbilityType::Class,
                "strength" => AbilityType::Strength,
                "dexterity" => AbilityType::Dexterity,
                "intelligence" => AbilityType::Intelligence,
                "concentration" => AbilityType::Concentration,
                "charm" => AbilityType::Charm,
                "sense" => AbilityType::Sense,
                "health" => AbilityType::Health,
                "mana" => AbilityType::Mana,
                "experience" => AbilityType::Experience,
                "level" => AbilityType::Level,
                "pvp_flag" => AbilityType::PvpFlag,
                "team_number" => AbilityType::TeamNumber,
                "union" => AbilityType::Union,
                "union_point1" => AbilityType::UnionPoint1,
                "union_point2" => AbilityType::UnionPoint2,
                "union_point3" => AbilityType::UnionPoint3,
                "union_point4" => AbilityType::UnionPoint4,
                "union_point5" => AbilityType::UnionPoint5,
                "union_point6" => AbilityType::UnionPoint6,
                "union_point7" => AbilityType::UnionPoint7,
                "union_point8" => AbilityType::UnionPoint8,
                "union_point9" => AbilityType::UnionPoint9,
                "union_point10" => AbilityType::UnionPoint10,
                _ => {
                    return Err(ChatCommandError::WithMessage(format!(
                        "Invalid ability type {}",
                        ability_type_str
                    )))
                }
            };

            ability_values_set_value(
                ability_type,
                value,
                Some(chat_command_user.basic_stats),
                Some(chat_command_user.character_info),
                Some(chat_command_user.union_membership),
                Some(chat_command_user.game_client),
            );
        }
        ("speed", arg_matches) => {
            let value = arg_matches.value_of("speed").unwrap().parse::<i32>()?;

            chat_command_params
                .commands
                .entity(chat_command_user.entity)
                .insert(MoveSpeed::new(value as f32));
            chat_command_params.server_messages.send_entity_message(
                chat_command_user.client_entity,
                ServerMessage::UpdateSpeed(UpdateSpeed {
                    entity_id: chat_command_user.client_entity.id,
                    run_speed: value,
                    passive_attack_speed: chat_command_user
                        .ability_values
                        .get_passive_attack_speed(),
                }),
            );
        }
        ("skill", arg_matches) => {
            let cmd = arg_matches.value_of("cmd").unwrap();
            let id = arg_matches.value_of("id").unwrap().parse::<SkillId>()?;
            let skill_data = chat_command_params
                .game_data
                .skills
                .get_skill(id)
                .ok_or_else(|| {
                    ChatCommandError::WithMessage(format!("Invalid skill id {}", id.get()))
                })?;

            if matches!(cmd, "add")
                && chat_command_user
                    .skill_list
                    .find_skill(skill_data)
                    .is_some()
            {
                return Err(ChatCommandError::WithMessage(format!(
                    "Already have skill {}",
                    cmd
                )));
            } else if let Some((skill_slot, skill_id)) = match cmd {
                "add" => chat_command_user
                    .skill_list
                    .add_skill(skill_data)
                    .map(|(skill_slot, skill_id)| (skill_slot, Some(skill_id))),
                "remove" => chat_command_user
                    .skill_list
                    .remove_skill(skill_data)
                    .map(|skill_slot| (skill_slot, None)),
                _ => None,
            } {
                chat_command_user
                    .game_client
                    .server_message_tx
                    .send(ServerMessage::LearnSkillResult(Ok(LearnSkillSuccess {
                        skill_slot,
                        skill_id,
                        updated_skill_points: **chat_command_user.skill_points,
                    })))
                    .ok();
            } else {
                return Err(ChatCommandError::WithMessage(format!(
                    "Invalid skill command {}",
                    cmd
                )));
            }
        }
        _ => return Err(ChatCommandError::InvalidCommand),
    }

    Ok(())
}

pub fn chat_commands_system(
    mut chat_command_params: ChatCommandParams,
    mut user_query: Query<(
        &AbilityValues,
        &ClientEntity,
        &ClientEntitySector,
        &GameClient,
        &Level,
        &Position,
        &mut BasicStats,
        &mut CharacterInfo,
        &mut Inventory,
        &mut SkillList,
        &mut SkillPoints,
        &mut Stamina,
        &mut StatPoints,
        &mut UnionMembership,
    )>,
    mut chat_command_events: EventReader<ChatCommandEvent>,
) {
    for &ChatCommandEvent {
        entity,
        ref command,
    } in chat_command_events.iter()
    {
        if let Ok((
            ability_values,
            client_entity,
            client_entity_sector,
            game_client,
            level,
            position,
            mut basic_stats,
            mut character_info,
            mut inventory,
            mut skill_list,
            mut skill_points,
            mut stamina,
            mut stat_points,
            mut union_membership,
        )) = user_query.get_mut(entity)
        {
            let mut chat_command_user = ChatCommandUser {
                entity,
                ability_values,
                client_entity,
                client_entity_sector,
                game_client,
                level,
                position,
                basic_stats: &mut basic_stats,
                character_info: &mut character_info,
                inventory: &mut inventory,
                skill_list: &mut skill_list,
                skill_points: &mut skill_points,
                stamina: &mut stamina,
                stat_points: &mut stat_points,
                union_membership: &mut union_membership,
            };

            match handle_chat_command(
                &mut chat_command_params,
                &mut chat_command_user,
                &command[1..],
            ) {
                Ok(_) => {
                    send_multiline_whisper(
                        chat_command_user.game_client,
                        &format!("Success: {}", command),
                    );
                }
                Err(error) => {
                    send_multiline_whisper(
                        chat_command_user.game_client,
                        &format!("Failed: {}", command),
                    );

                    match error {
                        ChatCommandError::InvalidCommand => {
                            send_multiline_whisper(chat_command_user.game_client, "Invalid command")
                        }
                        ChatCommandError::InvalidArguments => send_multiline_whisper(
                            chat_command_user.game_client,
                            "Invalid argument",
                        ),
                        ChatCommandError::WithMessage(message) => {
                            send_multiline_whisper(chat_command_user.game_client, &message)
                        }
                    };
                }
            }
        }
    }
}
