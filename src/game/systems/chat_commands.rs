use clap::{App, Arg};
use lazy_static::lazy_static;
use legion::{system, systems::CommandBuffer, world::SubWorld, Entity, Query};
use nalgebra::{Point2, Point3};
use num_traits::FromPrimitive;
use rand::prelude::SliceRandom;
use std::{
    f32::consts::PI,
    num::{ParseFloatError, ParseIntError},
};

use crate::{
    data::{
        item::{Item, ItemType},
        AbilityType, ItemReference, ZoneId,
    },
    game::{
        bundles::{
            ability_values_add_value, client_entity_join_zone, client_entity_teleport_zone,
            create_character_entity,
        },
        components::{
            AbilityValues, BasicStats, BotAi, ClientEntity, ClientEntityType, Command,
            EquipmentIndex, EquipmentItemDatabase, GameClient, Inventory, Level, Money, MoveSpeed,
            NextCommand, Owner, PersonalStore, Position, SkillPoints, Stamina, StatPoints, Team,
            UnionMembership, PERSONAL_STORE_ITEM_SLOTS,
        },
        messages::server::{ServerMessage, Whisper},
        resources::{
            BotList, BotListEntry, ClientEntityList, PendingChatCommandList, PendingXp,
            PendingXpList,
        },
        GameData,
    },
};

pub struct ChatCommandWorld<'a> {
    cmd: &'a mut CommandBuffer,
    bot_list: &'a mut BotList,
    client_entity_list: &'a mut ClientEntityList,
    game_data: &'a GameData,
    pending_xp_list: &'a mut PendingXpList,
}

pub struct ChatCommandUser<'a> {
    ability_values: &'a AbilityValues,
    client_entity: &'a ClientEntity,
    entity: &'a Entity,
    game_client: &'a GameClient,
    level: &'a Level,
    position: &'a Position,
    basic_stats: &'a mut BasicStats,
    inventory: &'a mut Inventory,
    skill_points: &'a mut SkillPoints,
    stamina: &'a mut Stamina,
    stat_points: &'a mut StatPoints,
    union_membership: &'a mut UnionMembership,
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
                    .arg(Arg::new("x"))
                    .arg(Arg::new("y")),
            )
            .subcommand(App::new("level").arg(Arg::new("level").required(true)))
            .subcommand(App::new("bot").arg(Arg::new("n").required(true)))
            .subcommand(App::new("shop").arg(Arg::new("item_type").required(true)))
            .subcommand(
                App::new("add")
                    .arg(Arg::new("ability_type").required(true))
                    .arg(Arg::new("value").required(true)),
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

fn create_bot_entity(
    chat_command_world: &mut ChatCommandWorld,
    name: String,
    gender: u8,
    face: u8,
    hair: u8,
    position: Position,
    owner: Entity,
) -> Option<Entity> {
    let mut bot_data = chat_command_world
        .game_data
        .character_creator
        .create(name, gender, 1, face, hair)
        .ok()?;

    let ability_values = chat_command_world
        .game_data
        .ability_value_calculator
        .calculate(
            &bot_data.info,
            &bot_data.level,
            &bot_data.equipment,
            &bot_data.inventory,
            &bot_data.basic_stats,
            &bot_data.skill_list,
        );

    let command = Command::default();
    let next_command = NextCommand::default();
    let move_speed = MoveSpeed::new(ability_values.run_speed as f32);
    let team = Team::default_character();

    let weapon_motion_type = chat_command_world
        .game_data
        .items
        .get_equipped_weapon_item_data(&bot_data.equipment, EquipmentIndex::WeaponRight)
        .map(|item_data| item_data.motion_type)
        .unwrap_or(0) as usize;

    let motion_data = chat_command_world
        .game_data
        .motions
        .get_character_action_motions(weapon_motion_type, bot_data.info.gender as usize);

    bot_data.position = position.clone();
    bot_data.health_points.hp = ability_values.max_health as u32;
    bot_data.mana_points.mp = ability_values.max_mana as u32;

    let entity = chat_command_world
        .cmd
        .push((BotAi::new(), Owner::new(owner)));

    create_character_entity(
        chat_command_world.cmd,
        &entity,
        ability_values,
        bot_data.basic_stats,
        command,
        bot_data.equipment,
        bot_data.experience_points,
        bot_data.health_points,
        bot_data.hotbar,
        bot_data.info,
        bot_data.inventory,
        bot_data.level,
        bot_data.mana_points,
        motion_data,
        move_speed,
        next_command,
        bot_data.position,
        bot_data.quest_state,
        bot_data.skill_list,
        bot_data.skill_points,
        bot_data.stamina,
        bot_data.stat_points,
        team,
        bot_data.union_membership,
    );

    client_entity_join_zone(
        chat_command_world.cmd,
        chat_command_world.client_entity_list,
        &entity,
        ClientEntityType::Character,
        &position,
    )
    .ok();

    Some(entity)
}

fn create_random_bot_entities(
    chat_command_world: &mut ChatCommandWorld,
    num_bots: usize,
    spacing: f32,
    origin: Position,
    owner: &Entity,
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
            chat_command_world,
            format!("Friend {}", chat_command_world.bot_list.len() as usize),
            *genders.choose(&mut rng).unwrap() as u8,
            *faces.choose(&mut rng).unwrap() as u8,
            *hair.choose(&mut rng).unwrap() as u8,
            bot_position,
            *owner,
        ) {
            chat_command_world
                .bot_list
                .push(BotListEntry::new(bot_entity));
            bot_entities.push(bot_entity);
        }
    }

    bot_entities
}

fn handle_gm_command(
    chat_command_world: &mut ChatCommandWorld,
    chat_command_user: &mut ChatCommandUser,
    command_text: &str,
) -> Result<(), GMCommandError> {
    let mut args = shellwords::split(command_text)?;
    args.insert(0, String::new()); // Clap expects arg[0] to be like executable name
    let command_matches = GM_COMMANDS.clone().try_get_matches_from(args)?;

    match command_matches
        .subcommand()
        .ok_or(GMCommandError::InvalidCommand)?
    {
        ("where", _) => {
            let sector = chat_command_world
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
            } else if let Some(zone_data) = chat_command_world.game_data.zones.get_zone(zone_id) {
                (zone_data.start_position.x, zone_data.start_position.y)
            } else {
                (520.0, 520.0)
            };

            if chat_command_world
                .client_entity_list
                .get_zone(zone_id)
                .is_some()
            {
                client_entity_teleport_zone(
                    chat_command_world.cmd,
                    chat_command_world.client_entity_list,
                    chat_command_user.entity,
                    chat_command_user.client_entity,
                    chat_command_user.position,
                    Position::new(Point3::new(x, y, 0.0), zone_id),
                    Some(chat_command_user.game_client),
                );
            } else {
                send_multiline_whisper(
                    chat_command_user.game_client,
                    &format!("Invalid zone id {}", zone_id.get()),
                );
            }
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
                required_xp += chat_command_world
                    .game_data
                    .ability_value_calculator
                    .calculate_levelup_require_xp(level);
            }

            chat_command_world.pending_xp_list.push(PendingXp::new(
                *chat_command_user.entity,
                required_xp,
                0,
                None,
            ));
        }
        ("bot", arg_matches) => {
            let num_bots = arg_matches.value_of("n").unwrap().parse::<usize>()?;
            create_random_bot_entities(
                chat_command_world,
                num_bots,
                15.0,
                chat_command_user.position.clone(),
                chat_command_user.entity,
            );
        }
        ("shop", arg_matches) => {
            let item_type: Option<ItemType> =
                FromPrimitive::from_i32(arg_matches.value_of("item_type").unwrap().parse::<i32>()?);
            if let Some(item_type) = item_type {
                let mut all_items: Vec<ItemReference> = chat_command_world
                    .game_data
                    .items
                    .iter_items(item_type)
                    .collect();
                all_items.sort_by(|a, b| a.item_number.cmp(&b.item_number));

                let num_bots =
                    (all_items.len() + PERSONAL_STORE_ITEM_SLOTS - 1) / PERSONAL_STORE_ITEM_SLOTS;
                let bot_entities = create_random_bot_entities(
                    chat_command_world,
                    num_bots,
                    30.0,
                    chat_command_user.position.clone(),
                    chat_command_user.entity,
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

                    chat_command_world.cmd.add_component(entity, store);
                    chat_command_world.cmd.add_component(entity, inventory);
                    chat_command_world
                        .cmd
                        .add_component(entity, NextCommand::with_personal_store());
                }
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
                _ => return Err(GMCommandError::InvalidArguments),
            };

            if ability_values_add_value(
                ability_type,
                value,
                Some(chat_command_user.basic_stats),
                Some(chat_command_user.inventory),
                Some(chat_command_user.skill_points),
                Some(chat_command_user.stamina),
                Some(chat_command_user.stat_points),
                Some(chat_command_user.union_membership),
                Some(chat_command_user.game_client),
            ) {
                send_multiline_whisper(
                    chat_command_user.game_client,
                    &format!("Success: /add {} {}", ability_type_str, value),
                );
            }
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
        &Level,
        &Position,
        &mut BasicStats,
        &mut Inventory,
        &mut SkillPoints,
        &mut Stamina,
        &mut StatPoints,
        &mut UnionMembership,
    )>,
    #[resource] bot_list: &mut BotList,
    #[resource] client_entity_list: &mut ClientEntityList,
    #[resource] game_data: &GameData,
    #[resource] pending_chat_commands: &mut PendingChatCommandList,
    #[resource] pending_xp_list: &mut PendingXpList,
) {
    let mut chat_command_world = ChatCommandWorld {
        cmd,
        bot_list,
        client_entity_list,
        game_data,
        pending_xp_list,
    };

    for (entity, command) in pending_chat_commands.iter() {
        if let Ok((
            ability_values,
            client_entity,
            game_client,
            level,
            position,
            basic_stats,
            inventory,
            skill_points,
            stamina,
            stat_points,
            union_membership,
        )) = user_query.get_mut(world, *entity)
        {
            let mut chat_command_user = ChatCommandUser {
                ability_values,
                client_entity,
                entity,
                game_client,
                level,
                position,
                basic_stats,
                inventory,
                skill_points,
                stamina,
                stat_points,
                union_membership,
            };

            if handle_gm_command(
                &mut chat_command_world,
                &mut chat_command_user,
                &command[1..],
            )
            .is_err()
            {
                send_gm_commands_help(chat_command_user.game_client);
            }
        }
    }

    pending_chat_commands.clear();
}
