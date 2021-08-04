use bevy_ecs::prelude::{Commands, Entity, Mut, Query, Res, ResMut, EventReader};
use clap::{App, Arg};
use lazy_static::lazy_static;
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
            CharacterBundle,
        },
        components::{
            AbilityValues, BasicStats, BotAi, ClientEntity, ClientEntityType, Command,
            EquipmentIndex, EquipmentItemDatabase, GameClient, Inventory, Level, Money, MoveMode,
            MoveSpeed, NextCommand, Owner, PersonalStore, Position, SkillPoints, Stamina,
            StatPoints, StatusEffects, Team, UnionMembership, PERSONAL_STORE_ITEM_SLOTS,
        },
        events::ChatCommandEvent,
        messages::server::{ServerMessage, UpdateSpeed, Whisper},
        resources::{
            BotList, BotListEntry, ClientEntityList, PendingXp,
            PendingXpList, ServerMessages,
        },
        GameData,
    },
};

pub struct ChatCommandWorld<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i, 'j, 'k, 'l> {
    commands: &'a mut Commands<'b>,
    bot_list: &'c mut ResMut<'d, BotList>,
    client_entity_list: &'e mut ResMut<'f, ClientEntityList>,
    game_data: &'g Res<'h, GameData>,
    pending_xp_list: &'i mut ResMut<'j, PendingXpList>,
    server_messages: &'k mut ResMut<'l, ServerMessages>,
}

pub struct ChatCommandUser<'world, 'a> {
    entity: Entity,
    ability_values: &'world AbilityValues,
    client_entity: &'world ClientEntity,
    game_client: &'world GameClient,
    level: &'world Level,
    position: &'world Position,
    basic_stats: &'a mut Mut<'world, BasicStats>,
    inventory: &'a mut Mut<'world, Inventory>,
    skill_points: &'a mut Mut<'world, SkillPoints>,
    stamina: &'a mut Mut<'world, Stamina>,
    stat_points: &'a mut Mut<'world, StatPoints>,
    union_membership: &'a mut Mut<'world, UnionMembership>,
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
            .subcommand(App::new("speed").arg(Arg::new("speed").required(true)))
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
            &bot_data.basic_stats,
            &bot_data.skill_list,
        );

    let move_speed = MoveSpeed::new(ability_values.run_speed as f32);

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
        .commands
        .spawn()
        .insert(BotAi::new())
        .insert(Owner::new(owner))
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
            position: bot_data.position,
            quest_state: bot_data.quest_state,
            skill_list: bot_data.skill_list,
            skill_points: bot_data.skill_points,
            stamina: bot_data.stamina,
            stat_points: bot_data.stat_points,
            status_effects: StatusEffects::new(),
            team: Team::default_character(),
            union_membership: bot_data.union_membership,
        })
        .id();

    client_entity_join_zone(
        chat_command_world.commands,
        chat_command_world.client_entity_list,
        entity,
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
    owner: Entity,
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
            owner,
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
                    chat_command_world.commands,
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
                chat_command_user.entity,
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

                    chat_command_world
                        .commands
                        .entity(entity)
                        .insert(store)
                        .insert(inventory)
                        .insert(NextCommand::with_personal_store());
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
        ("speed", arg_matches) => {
            let value = arg_matches.value_of("speed").unwrap().parse::<i32>()?;

            chat_command_world
                .commands
                .entity(chat_command_user.entity)
                .insert(MoveSpeed::new(value as f32));
            chat_command_world.server_messages.send_entity_message(
                chat_command_user.client_entity,
                ServerMessage::UpdateSpeed(UpdateSpeed {
                    entity_id: chat_command_user.client_entity.id,
                    run_speed: value,
                    passive_attack_speed: chat_command_user.ability_values.passive_attack_speed,
                }),
            );
        }
        _ => return Err(GMCommandError::InvalidCommand),
    }

    Ok(())
}

#[allow(clippy::type_complexity)]
pub fn chat_commands_system(
    mut commands: Commands,
    mut user_query: Query<(
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
    mut bot_list: ResMut<BotList>,
    mut client_entity_list: ResMut<ClientEntityList>,
    game_data: Res<GameData>,
    mut chat_command_events: EventReader<ChatCommandEvent>,
    mut pending_xp_list: ResMut<PendingXpList>,
    mut server_messages: ResMut<ServerMessages>,
) {
    let mut chat_command_world = ChatCommandWorld {
        commands: &mut commands,
        bot_list: &mut bot_list,
        client_entity_list: &mut client_entity_list,
        game_data: &game_data,
        pending_xp_list: &mut pending_xp_list,
        server_messages: &mut server_messages,
    };

    for &ChatCommandEvent { entity, ref command } in chat_command_events.iter() {
        if let Ok((
            ability_values,
            client_entity,
            game_client,
            level,
            position,
            mut basic_stats,
            mut inventory,
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
                game_client,
                level,
                position,
                basic_stats: &mut basic_stats,
                inventory: &mut inventory,
                skill_points: &mut skill_points,
                stamina: &mut stamina,
                stat_points: &mut stat_points,
                union_membership: &mut union_membership,
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
}
