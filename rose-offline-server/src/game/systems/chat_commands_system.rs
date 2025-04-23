use std::{
    f32::consts::PI,
    num::{ParseFloatError, ParseIntError},
};

use bevy::{
    ecs::{
        prelude::{Commands, Entity, EventReader, EventWriter, Query, Res, ResMut},
        query::WorldQuery,
        system::SystemParam,
    },
    math::{UVec2, Vec3, Vec3Swizzles},
    time::Time,
    utils::HashSet,
};
use clap::{Arg, PossibleValue};
use lazy_static::lazy_static;
use rand::Rng;

use rose_data::{
    AbilityType, EquipmentIndex, EquipmentItem, Item, ItemReference, ItemType, NpcId, SkillId,
    StackableItem, ZoneId,
};
use rose_game_common::{
    components::{BasicStatType, ClanLevel, ClanPoints, DroppedItem, ExperiencePoints, SkillSlot},
    data::Damage,
};

use crate::game::{
    bots::{
        bot_build_artisan, bot_build_bourgeois, bot_build_champion, bot_build_cleric,
        bot_build_knight, bot_build_mage, bot_build_raider, bot_build_scout,
        bot_create_random_build, bot_create_with_build, bot_snowball_fight, bot_thinker,
    },
    bundles::{
        ability_values_add_value, ability_values_set_value, client_entity_teleport_zone,
        CharacterBundle, ItemDropBundle, MonsterBundle,
    },
    components::{
        AbilityValues, BasicStats, CharacterInfo, ClanMembership, ClientEntity, ClientEntitySector,
        ClientEntityType, Command, Cooldowns, DamageSources, EquipmentItemDatabase, GameClient,
        HealthPoints, Inventory, Level, ManaPoints, Money, MotionData, MoveMode, MoveSpeed,
        NextCommand, PartyMembership, PassiveRecoveryTime, PersonalStore, Position, SkillList,
        SkillPoints, SpawnOrigin, Stamina, StatPoints, StatusEffects, StatusEffectsRegen, Team,
        UnionMembership, PERSONAL_STORE_ITEM_SLOTS,
    },
    events::{ChatCommandEvent, ClanEvent, DamageEvent, RewardItemEvent, RewardXpEvent},
    messages::server::ServerMessage,
    resources::{BotList, BotListEntry, ClientEntityList, ServerMessages, WorldRates},
    GameData,
};

#[derive(SystemParam)]
pub struct ChatCommandParams<'w, 's> {
    commands: Commands<'w, 's>,
    bot_list: ResMut<'w, BotList>,
    client_entity_list: ResMut<'w, ClientEntityList>,
    game_data: Res<'w, GameData>,
    clan_events: EventWriter<'w, ClanEvent>,
    reward_xp_events: EventWriter<'w, RewardXpEvent>,
    damage_events: EventWriter<'w, DamageEvent>,
    reward_item_events: EventWriter<'w, RewardItemEvent>,
    server_messages: ResMut<'w, ServerMessages>,
    time: Res<'w, Time>,
    world_rates: ResMut<'w, WorldRates>,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct ChatCommandUserQuery<'w> {
    entity: Entity,
    ability_values: &'w AbilityValues,
    client_entity: &'w ClientEntity,
    client_entity_sector: &'w ClientEntitySector,
    game_client: &'w GameClient,
    level: &'w mut Level,
    position: &'w Position,
    basic_stats: &'w mut BasicStats,
    character_info: &'w mut CharacterInfo,
    experience_points: &'w mut ExperiencePoints,
    health_points: &'w mut HealthPoints,
    inventory: &'w mut Inventory,
    mana_points: &'w mut ManaPoints,
    skill_list: &'w mut SkillList,
    skill_points: &'w mut SkillPoints,
    stamina: &'w mut Stamina,
    stat_points: &'w mut StatPoints,
    union_membership: &'w mut UnionMembership,
    clan_membership: &'w ClanMembership,
}

lazy_static! {
    pub static ref CHAT_COMMANDS: clap::Command<'static> = {
        clap::Command::new("Chat Commands")
            .subcommand(clap::Command::new("help"))
            .subcommand(clap::Command::new("where"))
            .subcommand(clap::Command::new("ability_values"))
            .subcommand(
                clap::Command::new("damage")
                    .arg(Arg::new("amount").required(true))
                    .arg(Arg::new("distance").required(true))
                    .arg(Arg::new("type").required(false)),
            )
            .subcommand(
                clap::Command::new("search")
                    .arg(Arg::new("type").required(true))
                    .arg(Arg::new("query")),
            )
            .subcommand(
                clap::Command::new("drop")
                    .arg(Arg::new("type").required(true))
                    .arg(Arg::new("id").required(true))
                    .arg(Arg::new("quantity").required(false))
                    .arg(Arg::new("socket").required(false))
                    .arg(Arg::new("gem").required(false))
                    .arg(Arg::new("grade").required(false)),
            )
            .subcommand(
                clap::Command::new("item")
                    .arg(Arg::new("type").required(true))
                    .arg(Arg::new("id").required(true))
                    .arg(Arg::new("quantity").required(false))
                    .arg(Arg::new("socket").required(false))
                    .arg(Arg::new("gem").required(false))
                    .arg(Arg::new("grade").required(false)),
            )
            .subcommand(
                clap::Command::new("mm")
                    .arg(Arg::new("zone").required(true))
                    .arg(Arg::new("x"))
                    .arg(Arg::new("y")),
            )
            .subcommand(
                clap::Command::new("mon")
                    .arg(Arg::new("id").required(true))
                    .arg(Arg::new("count").required(true))
                    .arg(Arg::new("distance").required(false))
                    .arg(Arg::new("team").required(false)),
            )
            .subcommand(clap::Command::new("level").arg(Arg::new("level").required(true)))
            .subcommand(clap::Command::new("bot").arg(Arg::new("n").required(true)))
            .subcommand(
                clap::Command::new("build")
                    .arg(Arg::new("name").required(true))
                    .arg(Arg::new("level").required(false)),
            )
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
            .subcommand(
                clap::Command::new("clan")
                    .subcommand(
                        clap::Command::new("level")
                            .arg(
                                Arg::new("cmd")
                                    .possible_values([
                                        PossibleValue::new("add"),
                                        PossibleValue::new("set"),
                                    ])
                                    .required(true),
                            )
                            .arg(Arg::new("value").required(true)),
                    )
                    .subcommand(
                        clap::Command::new("points")
                            .arg(
                                Arg::new("cmd")
                                    .possible_values([
                                        PossibleValue::new("add"),
                                        PossibleValue::new("set"),
                                    ])
                                    .required(true),
                            )
                            .arg(Arg::new("value").required(true)),
                    )
                    .subcommand(
                        clap::Command::new("money")
                            .arg(
                                Arg::new("cmd")
                                    .possible_values([
                                        PossibleValue::new("add"),
                                        PossibleValue::new("set"),
                                    ])
                                    .required(true),
                            )
                            .arg(Arg::new("value").required(true)),
                    )
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
                            .arg(Arg::new("value").required(true)),
                    ),
            )
            .subcommand(
                clap::Command::new("rate")
                    .arg(
                        Arg::new("type")
                            .possible_values([
                                "xp",
                                "drop",
                                "money",
                                "reward",
                                "stamina",
                                "craft",
                                "world_price",
                                "item_price",
                                "town_price",
                            ])
                            .required(true),
                    )
                    .arg(Arg::new("value").required(true)),
            )
    };
}

fn send_multiline_whisper(client: &GameClient, str: &str) {
    for line in str.lines() {
        client
            .server_message_tx
            .send(ServerMessage::Whisper {
                from: String::from("SERVER"),
                text: line.to_string(),
            })
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
            .send(ServerMessage::Whisper {
                from: String::from("SERVER"),
                text: help_string,
            })
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
    position: Position,
    level: u32,
) -> Option<Entity> {
    let (bot_build, mut bot_data) =
        bot_create_random_build(&chat_command_params.game_data, name, level);

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

    let move_mode = MoveMode::Run;
    let move_speed = MoveSpeed::new(ability_values.get_move_speed(&move_mode));

    let weapon_motion_type = chat_command_params
        .game_data
        .items
        .get_equipped_weapon_item_data(&bot_data.equipment, EquipmentIndex::Weapon)
        .map(|item_data| item_data.motion_type)
        .unwrap_or(0) as usize;

    let motion_data = MotionData::from_character(
        chat_command_params.game_data.motions.as_ref(),
        weapon_motion_type,
        bot_data.info.gender,
    );

    bot_data.position = position;
    bot_data.health_points.hp = ability_values.get_max_health();
    bot_data.mana_points.mp = ability_values.get_max_mana();

    let entity = chat_command_params
        .commands
        .spawn((
            bot_build,
            bot_thinker(),
            CharacterBundle {
                ability_values,
                basic_stats: bot_data.basic_stats,
                bank: Default::default(),
                cooldowns: Cooldowns::default(),
                command: Command::default(),
                damage_sources: DamageSources::default_character(),
                equipment: bot_data.equipment,
                experience_points: bot_data.experience_points,
                health_points: bot_data.health_points,
                hotbar: bot_data.hotbar,
                info: bot_data.info,
                inventory: bot_data.inventory,
                level: bot_data.level,
                mana_points: bot_data.mana_points,
                motion_data,
                move_mode,
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
                clan_membership: ClanMembership::default(),
            },
        ))
        .id();

    Some(entity)
}

fn create_random_bot_entities(
    chat_command_params: &mut ChatCommandParams,
    num_bots: usize,
    spacing: f32,
    origin: Position,
) -> Vec<Entity> {
    let mut rng = rand::thread_rng();
    let spawn_radius = f32::max(num_bots as f32 * spacing, 100.0);
    let mut bot_entities = Vec::new();

    // Try to calculate an ideal bot level range based on the current zone
    let mut bot_level_range = 1..=170;
    if let Some(zone_data) = chat_command_params.game_data.zones.get_zone(origin.zone_id) {
        // Find all monsters which spawn in this zone
        let mut monster_ids = HashSet::new();
        for spawn in zone_data.monster_spawns.iter() {
            for npc_id in spawn.basic_spawns.iter().map(|(npc_id, _)| *npc_id) {
                monster_ids.insert(npc_id);
            }

            for npc_id in spawn.tactic_spawns.iter().map(|(npc_id, _)| *npc_id) {
                monster_ids.insert(npc_id);
            }
        }

        // Create a sorted list of the levels of the monsters in this zone
        let mut monster_levels: Vec<_> = monster_ids
            .iter()
            .filter_map(|npc_id| chat_command_params.game_data.npcs.get_npc(*npc_id))
            .map(|npc_data| npc_data.level)
            .collect();
        monster_levels.sort();

        // Calculate a bot level range based on the median monster level +/- 20%
        if let Some(median_monster_level) = monster_levels.get((monster_levels.len() + 1) / 2) {
            // Ensure delta_level is mininum of 2 to ensure at least some variation in bot levels
            let delta_level = (*median_monster_level / 5).max(2);

            bot_level_range = (median_monster_level - delta_level).max(1)
                ..=(median_monster_level + delta_level).min(170);
        }
    }

    for i in 0..num_bots {
        let angle = (i as f32 * (2.0 * PI)) / num_bots as f32;
        let mut bot_position = origin.clone();
        bot_position.position.x += spawn_radius * angle.cos();
        bot_position.position.y += spawn_radius * angle.sin();

        if let Some(bot_entity) = create_bot_entity(
            chat_command_params,
            format!("Friend {}", chat_command_params.bot_list.len()),
            bot_position,
            rng.gen_range::<i32, _>(bot_level_range.clone()) as u32,
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
    chat_command_user: &mut ChatCommandUserQueryItem,
    command_text: &str,
) -> Result<(), ChatCommandError> {
    let mut args = shellwords::split(command_text)?;
    args.insert(0, String::new()); // Clap expects arg[0] to be like executable name
    let command_matches = CHAT_COMMANDS.clone().try_get_matches_from(args)?;

    match command_matches
        .subcommand()
        .ok_or(ChatCommandError::InvalidCommand)?
    {
        ("help", _) => {
            send_chat_commands_help(chat_command_user.game_client);
        }
        ("where", _) => {
            let sector = chat_command_params
                .client_entity_list
                .get_zone(chat_command_user.position.zone_id)
                .map(|client_entity_zone| {
                    client_entity_zone.calculate_sector(chat_command_user.position.position.xy())
                })
                .unwrap_or_else(|| UVec2::new(0u32, 0u32));

            chat_command_user
                .game_client
                .server_message_tx
                .send(ServerMessage::Whisper {
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
                })
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
                Position::new(Vec3::new(x, y, 0.0), zone_id),
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
                    false,
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
        ("build", arg_matches) => {
            let name = arg_matches.value_of("name").unwrap();
            let bot_build = match name {
                "knight" => bot_build_knight(),
                "champion" => bot_build_champion(),
                "cleric" => bot_build_cleric(),
                "mage" => bot_build_mage(),
                "scout" => bot_build_scout(),
                "raider" => bot_build_raider(),
                "artisan" => bot_build_artisan(),
                "bourgeois" => bot_build_bourgeois(),
                _ => {
                    return Err(ChatCommandError::WithMessage(format!(
                        "Invalid build {}",
                        name
                    )));
                }
            };
            let level = arg_matches
                .value_of("level")
                .and_then(|str| str.parse::<u32>().ok())
                .unwrap_or(chat_command_user.level.level);

            let bot_data = bot_create_with_build(
                &chat_command_params.game_data,
                name.into(),
                level,
                &bot_build,
            );

            chat_command_user.character_info.job = bot_data.info.job;
            chat_command_user
                .game_client
                .server_message_tx
                .send(ServerMessage::UpdateAbilityValueSet {
                    ability_type: AbilityType::Job,
                    value: chat_command_user.character_info.job as i32,
                })
                .ok();

            *chat_command_user.basic_stats = bot_data.basic_stats;
            for basic_stat_type in [
                BasicStatType::Strength,
                BasicStatType::Dexterity,
                BasicStatType::Intelligence,
                BasicStatType::Concentration,
                BasicStatType::Charm,
                BasicStatType::Sense,
            ] {
                chat_command_user
                    .game_client
                    .server_message_tx
                    .send(ServerMessage::UpdateBasicStat {
                        basic_stat_type,
                        value: chat_command_user.basic_stats.get(basic_stat_type),
                    })
                    .ok();
            }

            chat_command_user.level.level = level;
            *chat_command_user.stat_points = bot_data.stat_points;
            *chat_command_user.skill_points = bot_data.skill_points;
            *chat_command_user.experience_points = bot_data.experience_points;
            chat_command_user
                .game_client
                .server_message_tx
                .send(ServerMessage::UpdateLevel {
                    entity_id: chat_command_user.client_entity.id,
                    level: *chat_command_user.level,
                    experience_points: *chat_command_user.experience_points,
                    stat_points: *chat_command_user.stat_points,
                    skill_points: *chat_command_user.skill_points,
                })
                .ok();

            *chat_command_user.skill_list = bot_data.skill_list;
            for skill_page in chat_command_user.skill_list.pages.iter() {
                for (page_index, skill_slot) in skill_page.skills.iter().enumerate() {
                    chat_command_user
                        .game_client
                        .server_message_tx
                        .send(ServerMessage::LearnSkillSuccess {
                            skill_slot: SkillSlot(skill_page.page_type, page_index),
                            skill_id: *skill_slot,
                            updated_skill_points: *chat_command_user.skill_points,
                        })
                        .ok();
                }
            }

            let ability_values = chat_command_params
                .game_data
                .ability_value_calculator
                .calculate(
                    &chat_command_user.character_info,
                    &chat_command_user.level,
                    &bot_data.equipment,
                    &chat_command_user.basic_stats,
                    &chat_command_user.skill_list,
                    &StatusEffects::default(),
                );

            chat_command_user.health_points.hp = ability_values.get_max_health();
            chat_command_user
                .game_client
                .server_message_tx
                .send(ServerMessage::UpdateAbilityValueSet {
                    ability_type: AbilityType::Health,
                    value: chat_command_user.health_points.hp,
                })
                .ok();

            chat_command_user.mana_points.mp = ability_values.get_max_mana();
            chat_command_user
                .game_client
                .server_message_tx
                .send(ServerMessage::UpdateAbilityValueSet {
                    ability_type: AbilityType::Mana,
                    value: chat_command_user.mana_points.mp,
                })
                .ok();

            // TODO: Equipment ?
            // TODO: Remove unlearned skills from hotbar ?
        }
        ("snowball_fight", arg_matches) => {
            let num_bots = arg_matches.value_of("n").unwrap().parse::<usize>()?;
            let bot_entities = create_random_bot_entities(
                chat_command_params,
                num_bots,
                30.0,
                chat_command_user.position.clone(),
            );

            for entity in bot_entities.into_iter() {
                chat_command_params
                    .commands
                    .entity(entity)
                    .insert(bot_snowball_fight());
            }
        }
        ("shop", arg_matches) => {
            let item_type_str = arg_matches.value_of("item_type").unwrap();
            let item_type: ItemType = ItemType::try_from_id_str(
                item_type_str,
                &chat_command_params.game_data.data_decoder,
            )
            .ok_or_else(|| {
                ChatCommandError::WithMessage(format!("Invalid item type {}", item_type_str))
            })?;

            let mut all_items: Vec<(ItemReference, u8)> = chat_command_params
                .game_data
                .items
                .iter_items(item_type)
                .map(|item| {
                    (
                        item,
                        chat_command_params
                            .game_data
                            .items
                            .get_base_item(item)
                            .map(|x| x.durability)
                            .unwrap_or(0),
                    )
                })
                .collect();
            all_items.sort_by(|(a, _), (b, _)| a.item_number.cmp(&b.item_number));

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
                    if let Some(item) = all_items.get(i).and_then(|(item, durability)| {
                        if item_type.is_stackable_item() {
                            StackableItem::new(*item, 999).map(Item::from)
                        } else {
                            EquipmentItem::new(*item, *durability).map(Item::from)
                        }
                    }) {
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
                "str" | "strength" => AbilityType::Strength,
                "dex" | "dexterity" => AbilityType::Dexterity,
                "int" | "intelligence" => AbilityType::Intelligence,
                "con" | "concentration" => AbilityType::Concentration,
                "cha" | "charm" => AbilityType::Charm,
                "sen" | "sense" => AbilityType::Sense,
                "bonus_point" => AbilityType::BonusPoint,
                "skillpoint" => AbilityType::Skillpoint,
                "money" => AbilityType::Money,
                "stamina" => AbilityType::Stamina,
                "hp" | "health" => AbilityType::Health,
                "mp" | "mana" => AbilityType::Mana,
                "xp" | "experience" => AbilityType::Experience,
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
                Some(chat_command_user.ability_values),
                Some(&mut chat_command_user.basic_stats),
                Some(&mut chat_command_user.experience_points),
                Some(&mut chat_command_user.health_points),
                Some(&mut chat_command_user.inventory),
                Some(&mut chat_command_user.mana_points),
                Some(&mut chat_command_user.skill_points),
                Some(&mut chat_command_user.stamina),
                Some(&mut chat_command_user.stat_points),
                Some(&mut chat_command_user.union_membership),
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
                "job" => AbilityType::Job,
                "str" | "strength" => AbilityType::Strength,
                "dex" | "dexterity" => AbilityType::Dexterity,
                "int" | "intelligence" => AbilityType::Intelligence,
                "con" | "concentration" => AbilityType::Concentration,
                "cha" | "charm" => AbilityType::Charm,
                "sen" | "sense" => AbilityType::Sense,
                "hp" | "health" => AbilityType::Health,
                "mp" | "mana" => AbilityType::Mana,
                "xp" | "experience" => AbilityType::Experience,
                "level" => AbilityType::Level,
                "pvp_flag" => AbilityType::PvpFlag,
                "team" | "team_number" => AbilityType::TeamNumber,
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
                Some(chat_command_user.ability_values),
                Some(&mut chat_command_user.basic_stats),
                Some(&mut chat_command_user.character_info),
                Some(&mut chat_command_user.experience_points),
                Some(&mut chat_command_user.health_points),
                Some(&mut chat_command_user.mana_points),
                Some(&mut chat_command_user.union_membership),
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
                ServerMessage::UpdateSpeed {
                    entity_id: chat_command_user.client_entity.id,
                    run_speed: value,
                    passive_attack_speed: chat_command_user
                        .ability_values
                        .get_passive_attack_speed(),
                },
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
                    .find_skill_exact(skill_data)
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
                    .send(ServerMessage::LearnSkillSuccess {
                        skill_slot,
                        skill_id,
                        updated_skill_points: *chat_command_user.skill_points,
                    })
                    .ok();
            } else {
                return Err(ChatCommandError::WithMessage(format!(
                    "Invalid skill command {}",
                    cmd
                )));
            }
        }
        ("damage", arg_matches) => {
            let amount = arg_matches.value_of("amount").unwrap().parse::<u32>()?;
            let distance = arg_matches.value_of("distance").unwrap().parse::<f32>()?;
            let damage = Damage {
                amount,
                is_critical: arg_matches
                    .value_of("type")
                    .map_or(false, |str| str == "crit"),
                apply_hit_stun: arg_matches
                    .value_of("type")
                    .map_or(false, |str| str == "hit"),
            };

            if let Some(client_entity_zone) = chat_command_params
                .client_entity_list
                .get_zone(chat_command_user.position.zone_id)
            {
                for (defender, _) in client_entity_zone.iter_entity_type_within_distance(
                    chat_command_user.position.position.xy(),
                    distance,
                    &[ClientEntityType::Character, ClientEntityType::Monster],
                ) {
                    if chat_command_user.entity != defender {
                        chat_command_params
                            .damage_events
                            .send(DamageEvent::Immediate {
                                attacker: chat_command_user.entity,
                                defender,
                                damage,
                            });
                    }
                }
            }
        }
        ("mon", arg_matches) => {
            let npc_id = NpcId::new(arg_matches.value_of("id").unwrap().parse::<u16>()?)
                .ok_or(ChatCommandError::InvalidArguments)?;
            let count = arg_matches.value_of("count").unwrap().parse::<usize>()?;
            let spawn_range = arg_matches
                .value_of("distance")
                .and_then(|str| str.parse::<i32>().ok())
                .unwrap_or(250);
            let team = arg_matches
                .value_of("team")
                .and_then(|x| x.parse::<u32>().ok())
                .map_or_else(Team::default_monster, Team::new);

            for _ in 0..count {
                MonsterBundle::spawn(
                    &mut chat_command_params.commands,
                    &mut chat_command_params.client_entity_list,
                    &chat_command_params.game_data,
                    npc_id,
                    chat_command_user.position.zone_id,
                    SpawnOrigin::Summoned(
                        chat_command_user.entity,
                        chat_command_user.position.position,
                    ),
                    spawn_range,
                    team.clone(),
                    None,
                    None,
                );
            }
        }
        ("search", arg_matches) => {
            let item_type_str = arg_matches.value_of("type").unwrap();

            let item_type: ItemType = ItemType::try_from_id_str(
                item_type_str,
                &chat_command_params.game_data.data_decoder,
            )
            .ok_or_else(|| {
                ChatCommandError::WithMessage(format!("Invalid item type {}", item_type_str))
            })?;

            let query = arg_matches.value_of("query");

            if let Some(query) = query {
                let query = query.to_lowercase();
                let mut lines = String::new();
                let mut count = 0;

                for item in chat_command_params.game_data.items.iter_items(item_type) {
                    let item_data = chat_command_params
                        .game_data
                        .items
                        .get_base_item(item)
                        .ok_or_else(|| {
                            ChatCommandError::WithMessage(format!("Invalid item {:?}", item))
                        })?;

                    let name = item_data.name.to_lowercase();

                    if !name.contains(&query) {
                        continue;
                    }

                    lines.push_str(&format!(
                        "{:?}: {:?} (q={})\n",
                        item_data.id.item_number, item_data.name, item_data.quality
                    ));
                    count += 1;
                }

                lines.push_str(&format!(
                    "Done; found {count} items of type {:?} that match '{}'",
                    item_type, query
                ));

                send_multiline_whisper(chat_command_user.game_client, &lines);
            } else {
                let mut lines = String::new();
                let mut count = 0;

                for item in chat_command_params.game_data.items.iter_items(item_type) {
                    let item_data = chat_command_params
                        .game_data
                        .items
                        .get_base_item(item)
                        .ok_or_else(|| {
                            ChatCommandError::WithMessage(format!("Invalid item {:?}", item))
                        })?;

                    lines.push_str(&format!(
                        "{:?}: {:?} (q={})\n",
                        item_data.id.item_number, item_data.name, item_data.quality
                    ));
                    count += 1;
                }

                lines.push_str(&format!(
                    "Done; found {count} items of type {:?}",
                    item_type
                ));

                send_multiline_whisper(chat_command_user.game_client, &lines);
            }
        }
        ("item", arg_matches) | ("drop", arg_matches) => {
            let is_drop = command_matches.subcommand().unwrap().0 == "drop";

            let item_type_str = arg_matches.value_of("type").unwrap();
            let item_type: ItemType = ItemType::try_from_id_str(
                item_type_str,
                &chat_command_params.game_data.data_decoder,
            )
            .ok_or_else(|| {
                ChatCommandError::WithMessage(format!("Invalid item type {}", item_type_str))
            })?;

            let item_number = arg_matches.value_of("id").unwrap().parse::<usize>()?;

            let quantity = arg_matches
                .value_of("quantity")
                .and_then(|str| str.parse::<u32>().ok())
                .unwrap_or(1);

            let has_socket = arg_matches
                .value_of("socket")
                .and_then(|str| str.parse::<u8>().ok())
                .unwrap_or(0)
                != 0;

            let gem = arg_matches
                .value_of("gem")
                .and_then(|str| str.parse::<u16>().ok())
                .unwrap_or(0);

            let grade = arg_matches
                .value_of("grade")
                .and_then(|str| str.parse::<u8>().ok())
                .unwrap_or(0);

            let item_reference = ItemReference::new(item_type, item_number);
            let item_data = chat_command_params
                .game_data
                .items
                .get_base_item(item_reference)
                .ok_or_else(|| {
                    ChatCommandError::WithMessage(format!("Invalid item {:?}", item_reference))
                })?;

            let mut item = Item::from_item_data(item_data, quantity)
                .ok_or(ChatCommandError::InvalidArguments)?;

            match &mut item {
                Item::Equipment(equipment_item) => {
                    equipment_item.has_socket = has_socket;
                    equipment_item.gem = gem;
                    equipment_item.grade = grade;
                }
                Item::Stackable(_) => {}
            }

            if is_drop {
                ItemDropBundle::spawn(
                    &mut chat_command_params.commands,
                    &mut chat_command_params.client_entity_list,
                    DroppedItem::Item(item),
                    chat_command_user.position,
                    None,
                    None,
                    &chat_command_params.time,
                );
            } else {
                chat_command_params
                    .reward_item_events
                    .send(RewardItemEvent::new(chat_command_user.entity, item, true));
            }
        }
        ("clan", arg_matches) => {
            if let Some(sub_matches) = arg_matches.subcommand_matches("level") {
                let cmd = sub_matches.value_of("cmd").unwrap();
                let value = sub_matches.value_of("value").unwrap().parse::<i32>()?;

                if let Some(clan_entity) = chat_command_user.clan_membership.clan() {
                    match cmd {
                        "add" => {
                            chat_command_params.clan_events.send(ClanEvent::AddLevel {
                                clan_entity,
                                level: value,
                            });
                        }
                        "set" => {
                            chat_command_params.clan_events.send(ClanEvent::SetLevel {
                                clan_entity,
                                level: ClanLevel::new(value as u32)
                                    .ok_or(ChatCommandError::InvalidArguments)?,
                            });
                        }
                        _ => return Err(ChatCommandError::InvalidArguments),
                    }
                }
            } else if let Some(sub_matches) = arg_matches.subcommand_matches("points") {
                let cmd = sub_matches.value_of("cmd").unwrap();
                let value = sub_matches.value_of("value").unwrap().parse::<i64>()?;

                if let Some(clan_entity) = chat_command_user.clan_membership.clan() {
                    match cmd {
                        "add" => {
                            chat_command_params.clan_events.send(ClanEvent::AddPoints {
                                clan_entity,
                                points: value,
                            });
                        }
                        "set" => {
                            chat_command_params.clan_events.send(ClanEvent::SetPoints {
                                clan_entity,
                                points: ClanPoints(value as u64),
                            });
                        }
                        _ => return Err(ChatCommandError::InvalidArguments),
                    }
                }
            } else if let Some(sub_matches) = arg_matches.subcommand_matches("money") {
                let cmd = sub_matches.value_of("cmd").unwrap();
                let value = sub_matches.value_of("value").unwrap().parse::<i64>()?;

                if let Some(clan_entity) = chat_command_user.clan_membership.clan() {
                    match cmd {
                        "add" => {
                            chat_command_params.clan_events.send(ClanEvent::AddMoney {
                                clan_entity,
                                money: value,
                            });
                        }
                        "set" => {
                            chat_command_params.clan_events.send(ClanEvent::SetMoney {
                                clan_entity,
                                money: Money(value),
                            });
                        }
                        _ => return Err(ChatCommandError::InvalidArguments),
                    }
                }
            } else if let Some(sub_matches) = arg_matches.subcommand_matches("skill") {
                let cmd = sub_matches.value_of("cmd").unwrap();
                let value = sub_matches.value_of("value").unwrap().parse::<u16>()?;

                if let Some(clan_entity) = chat_command_user.clan_membership.clan() {
                    match cmd {
                        "add" => {
                            chat_command_params.clan_events.send(ClanEvent::AddSkill {
                                clan_entity,
                                skill_id: SkillId::new(value)
                                    .ok_or(ChatCommandError::InvalidArguments)?,
                            });
                        }
                        "remove" => {
                            chat_command_params
                                .clan_events
                                .send(ClanEvent::RemoveSkill {
                                    clan_entity,
                                    skill_id: SkillId::new(value)
                                        .ok_or(ChatCommandError::InvalidArguments)?,
                                });
                        }
                        _ => return Err(ChatCommandError::InvalidArguments),
                    }
                }
            }
        }
        ("rate", arg_matches) => {
            let rate_type = arg_matches.value_of("type").unwrap();
            let value = arg_matches.value_of("value").unwrap().parse::<i32>()?;

            match rate_type {
                "xp" => chat_command_params.world_rates.xp_rate = value,
                "drop" => chat_command_params.world_rates.drop_rate = value,
                "money" => chat_command_params.world_rates.drop_money_rate = value,
                "reward" => chat_command_params.world_rates.reward_rate = value,
                "stamina" => chat_command_params.world_rates.stamina_rate = value,
                "craft" => chat_command_params.world_rates.craft_rate = value,
                "world_price" => chat_command_params.world_rates.world_price_rate = value,
                "item_price" => chat_command_params.world_rates.item_price_rate = value,
                "town_price" => chat_command_params.world_rates.town_price_rate = value,
                _ => return Err(ChatCommandError::InvalidArguments),
            }

            chat_command_user
                .game_client
                .server_message_tx
                .send(ServerMessage::Whisper {
                    from: String::from("SERVER"),
                    text: format!("Updated {} rate to {}", rate_type, value),
                })
                .ok();
        }
        _ => return Err(ChatCommandError::InvalidCommand),
    }

    Ok(())
}

pub fn chat_commands_system(
    mut chat_command_params: ChatCommandParams,
    mut user_query: Query<ChatCommandUserQuery>,
    mut chat_command_events: EventReader<ChatCommandEvent>,
) {
    for &ChatCommandEvent {
        entity,
        ref command,
    } in chat_command_events.iter()
    {
        if let Ok(mut chat_command_user) = user_query.get_mut(entity) {
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
