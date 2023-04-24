use bevy::{
    ecs::{
        prelude::{Commands, Entity, EventWriter, Query, Res, ResMut, Without},
        query::WorldQuery,
        system::SystemParam,
    },
    math::{Vec3, Vec3Swizzles},
    time::Time,
};
use log::warn;

use rose_data::{EquipmentIndex, Item, ItemClass, ItemSlotBehaviour, ItemType};
use rose_game_common::{
    data::Password,
    messages::server::{CharacterData, CharacterDataItems, CraftInsertGemError},
};

use crate::game::{
    bundles::{
        client_entity_join_zone, client_entity_leave_zone, client_entity_teleport_zone,
        skill_list_try_level_up_skill, CharacterBundle, ItemDropBundle, SkillListBundle,
    },
    components::{
        AbilityValues, Account, Bank, BasicStatType, BasicStats, CharacterInfo, Clan, ClanMember,
        ClanMembership, ClientEntity, ClientEntitySector, ClientEntityType, ClientEntityVisibility,
        Command, CommandData, Cooldowns, DamageSources, Dead, DrivingTime, DroppedItem, Equipment,
        EquipmentItemDatabase, ExperiencePoints, GameClient, HealthPoints, Hotbar, Inventory,
        ItemSlot, Level, ManaPoints, Money, MotionData, MoveMode, MoveSpeed, NextCommand, Party,
        PartyMember, PartyMembership, PassiveRecoveryTime, Position, QuestState, SkillList,
        SkillPoints, StatPoints, StatusEffects, StatusEffectsRegen, Team, WorldClient,
    },
    events::{
        BankEvent, ChatCommandEvent, ClanEvent, EquipmentEvent, ItemLifeEvent, NpcStoreEvent,
        PartyEvent, PartyMemberEvent, PersonalStoreEvent, QuestTriggerEvent, ReviveEvent,
        RevivePosition, UseItemEvent,
    },
    messages::{
        client::ClientMessage,
        server::{ConnectionRequestError, ServerMessage},
    },
    resources::{ClientEntityList, GameData, LoginTokens, ServerMessages, WorldRates, WorldTime},
    storage::{account::AccountStorage, bank::BankStorage, character::CharacterStorage},
};

fn handle_game_connection_request(
    commands: &mut Commands,
    game_data: &GameData,
    login_tokens: &mut LoginTokens,
    entity: Entity,
    game_client: &mut GameClient,
    token_id: u32,
    password: &Password,
    query_world_client: &mut Query<&mut WorldClient>,
    query_clans: &mut Query<(Entity, &mut Clan)>,
) -> Result<
    (
        u32,
        Box<CharacterData>,
        Box<CharacterDataItems>,
        Box<QuestState>,
    ),
    ConnectionRequestError,
> {
    // Verify token
    let login_token = login_tokens
        .get_token_mut(token_id)
        .ok_or(ConnectionRequestError::InvalidToken)?;
    if login_token.world_client.is_none() || login_token.game_client.is_some() {
        return Err(ConnectionRequestError::InvalidToken);
    }

    let mut world_client =
        if let Ok(world_client) = query_world_client.get_mut(login_token.world_client.unwrap()) {
            world_client
        } else {
            return Err(ConnectionRequestError::InvalidToken);
        };

    // Verify account password
    let account: Account = AccountStorage::try_load(&login_token.username, password)
        .map_err(|error| {
            log::error!(
                "Failed to load account {} with error {:?}",
                &login_token.username,
                error
            );
            ConnectionRequestError::InvalidPassword
        })?
        .into();

    // Try load bank
    let bank = match BankStorage::try_load(&login_token.username) {
        Ok(bank_storage) => Bank::from(bank_storage),
        Err(_) => match BankStorage::create(&login_token.username) {
            Ok(bank_storage) => {
                log::info!("Created bank storage for account {}", &login_token.username);
                Bank::from(bank_storage)
            }
            Err(error) => {
                log::error!(
                    "Failed to create bank storage for account {} with error {}",
                    &login_token.username,
                    error
                );
                return Err(ConnectionRequestError::Failed);
            }
        },
    };

    // Try load character
    let character =
        CharacterStorage::try_load(&login_token.selected_character).map_err(|error| {
            log::error!(
                "Failed to load character {} with error {:?}",
                &login_token.selected_character,
                error
            );
            ConnectionRequestError::Failed
        })?;

    // Try find clan membership
    let mut clan_membership = ClanMembership(None);
    for (clan_entity, mut clan) in query_clans.iter_mut() {
        if let Some(clan_member) = clan.find_offline_member_mut(&character.info.name) {
            let &mut ClanMember::Offline { position, contribution, .. } = clan_member else {
                unreachable!();
            };

            *clan_member = ClanMember::Online {
                entity,
                position,
                contribution,
            };
            clan_membership = ClanMembership::new(clan_entity);
            break;
        }
    }

    // Update token
    login_token.game_client = Some(entity);
    game_client.login_token = login_token.token;

    // Associate world / game clients
    game_client.world_client_entity = login_token.world_client;
    world_client.game_client_entity = Some(entity);

    let status_effects = StatusEffects::new();
    let status_effects_regen = StatusEffectsRegen::new();

    let ability_values = game_data.ability_value_calculator.calculate(
        &character.info,
        &character.level,
        &character.equipment,
        &character.basic_stats,
        &character.skill_list,
        &status_effects,
    );

    // If the character was saved as dead, we must respawn them!
    let (health_points, mana_points, position) = if character.health_points.hp == 0 {
        (
            HealthPoints::new((3 * ability_values.get_max_health()) / 10),
            ManaPoints::new((3 * ability_values.get_max_mana()) / 10),
            Position::new(
                character.info.revive_position,
                character.info.revive_zone_id,
            ),
        )
    } else {
        (
            character.health_points,
            character.mana_points,
            character.position.clone(),
        )
    };

    let weapon_motion_type = game_data
        .items
        .get_equipped_weapon_item_data(&character.equipment, EquipmentIndex::Weapon)
        .map(|item_data| item_data.motion_type)
        .unwrap_or(0) as usize;

    let motion_data = MotionData::from_character(
        game_data.motions.as_ref(),
        weapon_motion_type,
        character.info.gender,
    );

    let move_mode = MoveMode::Run;
    let move_speed = MoveSpeed::new(ability_values.get_move_speed(&move_mode));

    commands.entity(entity).insert((
        account,
        CharacterBundle {
            ability_values,
            basic_stats: character.basic_stats.clone(),
            bank,
            command: Command::default(),
            cooldowns: Cooldowns::default(),
            damage_sources: DamageSources::new(1),
            equipment: character.equipment.clone(),
            experience_points: character.experience_points.clone(),
            health_points,
            hotbar: character.hotbar.clone(),
            info: character.info.clone(),
            inventory: character.inventory.clone(),
            level: character.level,
            mana_points,
            motion_data,
            move_mode,
            move_speed,
            next_command: NextCommand::default(),
            party_membership: PartyMembership::default(),
            passive_recovery_time: PassiveRecoveryTime::default(),
            position: position.clone(),
            quest_state: character.quest_state.clone(),
            skill_list: character.skill_list.clone(),
            skill_points: character.skill_points,
            stamina: character.stamina,
            stat_points: character.stat_points,
            status_effects,
            status_effects_regen,
            team: Team::default_character(),
            union_membership: character.union_membership.clone(),
            clan_membership,
        },
    ));

    Ok((
        123,
        Box::new(CharacterData {
            character_info: character.info,
            position: position.position,
            zone_id: position.zone_id,
            basic_stats: character.basic_stats,
            level: character.level,
            equipment: character.equipment.clone(),
            experience_points: character.experience_points,
            skill_list: character.skill_list,
            hotbar: character.hotbar,
            health_points,
            mana_points,
            stat_points: character.stat_points,
            skill_points: character.skill_points,
            union_membership: character.union_membership,
            stamina: character.stamina,
        }),
        Box::new(CharacterDataItems {
            inventory: character.inventory,
            equipment: character.equipment,
        }),
        Box::new(character.quest_state),
    ))
}

pub fn game_server_authentication_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut GameClient), Without<CharacterInfo>>,
    mut query_world_client: Query<&mut WorldClient>,
    mut query_clans: Query<(Entity, &mut Clan)>,
    mut login_tokens: ResMut<LoginTokens>,
    game_data: Res<GameData>,
) {
    query.for_each_mut(|(entity, mut game_client)| {
        if let Ok(message) = game_client.client_message_rx.try_recv() {
            match message {
                ClientMessage::GameConnectionRequest {
                    login_token,
                    password,
                } => {
                    match handle_game_connection_request(
                        &mut commands,
                        game_data.as_ref(),
                        login_tokens.as_mut(),
                        entity,
                        game_client.as_mut(),
                        login_token,
                        &password,
                        &mut query_world_client,
                        &mut query_clans,
                    ) {
                        Ok((
                            packet_sequence_id,
                            character_data,
                            character_data_items,
                            character_data_quest,
                        )) => {
                            game_client
                                .server_message_tx
                                .send(ServerMessage::ConnectionRequestSuccess {
                                    packet_sequence_id,
                                })
                                .ok();
                            game_client
                                .server_message_tx
                                .send(ServerMessage::CharacterData {
                                    data: character_data,
                                })
                                .ok();
                            game_client
                                .server_message_tx
                                .send(ServerMessage::CharacterDataItems {
                                    data: character_data_items,
                                })
                                .ok();
                            game_client
                                .server_message_tx
                                .send(ServerMessage::CharacterDataQuest {
                                    quest_state: character_data_quest,
                                })
                                .ok();
                        }
                        Err(error) => {
                            game_client
                                .server_message_tx
                                .send(ServerMessage::ConnectionRequestError { error })
                                .ok();
                        }
                    }
                }
                _ => warn!("Received unexpected client message {:?}", message),
            }
        }
    });
}

pub fn game_server_join_system(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &GameClient,
            &CharacterInfo,
            &ExperiencePoints,
            &Team,
            &HealthPoints,
            &ManaPoints,
            &Position,
        ),
        Without<ClientEntity>,
    >,
    mut client_entity_list: ResMut<ClientEntityList>,
    world_rates: Res<WorldRates>,
    world_time: Res<WorldTime>,
    mut party_query: Query<(Entity, &mut Party)>,
    mut party_member_events: EventWriter<PartyMemberEvent>,
) {
    query.for_each(
        |(
            entity,
            game_client,
            character_info,
            experience_points,
            team,
            health_points,
            mana_points,
            position,
        )| {
            if let Ok(message) = game_client.client_message_rx.try_recv() {
                match message {
                    ClientMessage::JoinZoneRequest => {
                        if let Ok(entity_id) = client_entity_join_zone(
                            &mut commands,
                            &mut client_entity_list,
                            entity,
                            ClientEntityType::Character,
                            position,
                        ) {
                            // See if we are in a party as an offline member
                            let mut party_membership = PartyMembership::default();
                            for (party_entity, mut party) in party_query.iter_mut() {
                                for party_member in party.members.iter_mut() {
                                    if let PartyMember::Offline(
                                        party_member_character_id,
                                        party_member_name,
                                    ) = party_member
                                    {
                                        if *party_member_character_id == character_info.unique_id
                                            && party_member_name == &character_info.name
                                        {
                                            *party_member = PartyMember::Online(entity);
                                            party_membership = PartyMembership::new(party_entity);
                                            party_member_events.send(PartyMemberEvent::Reconnect {
                                                party_entity,
                                                reconnect_entity: entity,
                                                character_id: character_info.unique_id,
                                                name: character_info.name.clone(),
                                            });
                                            break;
                                        }
                                    }
                                }
                            }

                            commands
                                .entity(entity)
                                .insert(party_membership)
                                .insert(ClientEntityVisibility::new())
                                .insert(PassiveRecoveryTime::default());

                            game_client
                                .server_message_tx
                                .send(ServerMessage::JoinZone {
                                    entity_id,
                                    experience_points: experience_points.clone(),
                                    team: team.clone(),
                                    health_points: *health_points,
                                    mana_points: *mana_points,
                                    world_ticks: world_time.ticks,
                                    craft_rate: world_rates.craft_rate,
                                    world_price_rate: world_rates.world_price_rate,
                                    item_price_rate: world_rates.item_price_rate,
                                    town_price_rate: world_rates.town_price_rate,
                                })
                                .ok();
                        }
                    }
                    _ => warn!("Received unexpected client message {:?}", message),
                }
            }
        },
    );
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct GameClientQuery<'w> {
    entity: Entity,
    game_client: &'w GameClient,
    client_entity: &'w ClientEntity,
    client_entity_sector: &'w ClientEntitySector,
    position: &'w Position,
    ability_values: &'w AbilityValues,
    command: &'w Command,
    dead: Option<&'w Dead>,
    level: &'w Level,
    move_speed: &'w MoveSpeed,
    team: &'w Team,
    basic_stats: &'w mut BasicStats,
    character_info: &'w mut CharacterInfo,
    stat_points: &'w mut StatPoints,
    skill_points: &'w mut SkillPoints,
    skill_list: &'w mut SkillList,
    hotbar: &'w mut Hotbar,
    equipment: &'w mut Equipment,
    inventory: &'w mut Inventory,
    quest_state: &'w mut QuestState,
    move_mode: &'w mut MoveMode,
}

#[derive(SystemParam)]
pub struct GameEvents<'w> {
    bank_events: EventWriter<'w, BankEvent>,
    chat_command_events: EventWriter<'w, ChatCommandEvent>,
    clan_events: EventWriter<'w, ClanEvent>,
    equipment_events: EventWriter<'w, EquipmentEvent>,
    item_life_events: EventWriter<'w, ItemLifeEvent>,
    npc_store_events: EventWriter<'w, NpcStoreEvent>,
    party_events: EventWriter<'w, PartyEvent>,
    personal_store_events: EventWriter<'w, PersonalStoreEvent>,
    quest_trigger_events: EventWriter<'w, QuestTriggerEvent>,
    revive_events: EventWriter<'w, ReviveEvent>,
    use_item_events: EventWriter<'w, UseItemEvent>,
}

pub fn game_server_main_system(
    mut commands: Commands,
    mut events: GameEvents,
    mut game_client_query: Query<GameClientQuery>,
    world_client_query: Query<&WorldClient>,
    mut client_entity_list: ResMut<ClientEntityList>,
    mut server_messages: ResMut<ServerMessages>,
    game_data: Res<GameData>,
    time: Res<Time>,
) {
    for mut game_client in game_client_query.iter_mut() {
        let mut entity_commands = commands.entity(game_client.entity);

        if let Ok(message) = game_client.game_client.client_message_rx.try_recv() {
            match message {
                ClientMessage::Chat { text } => {
                    if text.chars().next().map_or(false, |c| c == '/') {
                        events
                            .chat_command_events
                            .send(ChatCommandEvent::new(game_client.entity, text));
                    } else {
                        server_messages.send_entity_message(
                            game_client.client_entity,
                            ServerMessage::LocalChat {
                                entity_id: game_client.client_entity.id,
                                text,
                            },
                        );
                    }
                }
                ClientMessage::Move {
                    target_entity_id,
                    x,
                    y,
                    z,
                } => {
                    let mut move_target_entity = None;
                    if let Some(target_entity_id) = target_entity_id {
                        if let Some((target_entity, _, _)) = client_entity_list
                            .get_zone(game_client.position.zone_id)
                            .and_then(|zone| zone.get_entity(target_entity_id))
                        {
                            move_target_entity = Some(*target_entity);
                        }
                    }

                    let destination = Vec3::new(x, y, z as f32);
                    entity_commands.insert(NextCommand::with_move(
                        destination,
                        move_target_entity,
                        None,
                    ));
                }
                ClientMessage::Attack { target_entity_id } => {
                    if let Some((target_entity, _, _)) = client_entity_list
                        .get_zone(game_client.position.zone_id)
                        .and_then(|zone| zone.get_entity(target_entity_id))
                    {
                        entity_commands.insert(NextCommand::with_attack(*target_entity));
                    } else {
                        entity_commands.insert(NextCommand::with_stop(true));
                    }
                }
                ClientMessage::SetHotbarSlot { slot_index, slot } => {
                    if game_client
                        .hotbar
                        .set_slot(slot_index, slot.clone())
                        .is_some()
                    {
                        game_client
                            .game_client
                            .server_message_tx
                            .send(ServerMessage::SetHotbarSlot { slot_index, slot })
                            .ok();
                    }
                }
                ClientMessage::ChangeEquipment {
                    equipment_index,
                    item_slot,
                } => {
                    events
                        .equipment_events
                        .send(EquipmentEvent::ChangeEquipment {
                            entity: game_client.entity,
                            equipment_index,
                            item_slot,
                        });
                }
                ClientMessage::ChangeVehiclePart {
                    vehicle_part_index,
                    item_slot,
                } => {
                    events
                        .equipment_events
                        .send(EquipmentEvent::ChangeVehiclePart {
                            entity: game_client.entity,
                            vehicle_part_index,
                            item_slot,
                        });
                }
                ClientMessage::ChangeAmmo {
                    ammo_index,
                    item_slot,
                } => {
                    events.equipment_events.send(EquipmentEvent::ChangeAmmo {
                        entity: game_client.entity,
                        ammo_index,
                        item_slot,
                    });
                }
                ClientMessage::IncreaseBasicStat { basic_stat_type } => {
                    if let Some(cost) = game_data
                        .ability_value_calculator
                        .calculate_basic_stat_increase_cost(
                            &game_client.basic_stats,
                            basic_stat_type,
                        )
                    {
                        if cost < game_client.stat_points.points {
                            let value = match basic_stat_type {
                                BasicStatType::Strength => &mut game_client.basic_stats.strength,
                                BasicStatType::Dexterity => &mut game_client.basic_stats.dexterity,
                                BasicStatType::Intelligence => {
                                    &mut game_client.basic_stats.intelligence
                                }
                                BasicStatType::Concentration => {
                                    &mut game_client.basic_stats.concentration
                                }
                                BasicStatType::Charm => &mut game_client.basic_stats.charm,
                                BasicStatType::Sense => &mut game_client.basic_stats.sense,
                            };

                            game_client.stat_points.points -= cost;
                            *value += 1;

                            game_client
                                .game_client
                                .server_message_tx
                                .send(ServerMessage::UpdateBasicStat {
                                    basic_stat_type,
                                    value: *value,
                                })
                                .ok();
                        }
                    }
                }
                ClientMessage::PickupItemDrop { target_entity_id } => {
                    if let Some((target_entity, _, _)) = client_entity_list
                        .get_zone(game_client.position.zone_id)
                        .and_then(|zone| zone.get_entity(target_entity_id))
                    {
                        entity_commands.insert(NextCommand::with_pickup_item_drop(*target_entity));
                    } else {
                        entity_commands.insert(NextCommand::with_stop(true));
                    }
                }
                ClientMessage::Logout | ClientMessage::ReturnToCharacterSelect => {
                    if let ClientMessage::ReturnToCharacterSelect = message {
                        // Send ReturnToCharacterSelect via world_client
                        world_client_query.for_each(|world_client| {
                            if world_client.login_token == game_client.game_client.login_token {
                                world_client
                                    .server_message_tx
                                    .send(ServerMessage::ReturnToCharacterSelect)
                                    .ok();
                            }
                        });
                    }

                    game_client
                        .game_client
                        .server_message_tx
                        .send(ServerMessage::LogoutSuccess)
                        .ok();

                    client_entity_leave_zone(
                        &mut commands,
                        &mut client_entity_list,
                        game_client.entity,
                        game_client.client_entity,
                        game_client.client_entity_sector,
                        game_client.position,
                    );
                }
                ClientMessage::ReviveCurrentZone => {
                    if game_client.dead.is_some() {
                        events.revive_events.send(ReviveEvent {
                            entity: game_client.entity,
                            position: RevivePosition::CurrentZone,
                        });
                    }
                }
                ClientMessage::ReviveSaveZone => {
                    if game_client.dead.is_some() {
                        events.revive_events.send(ReviveEvent {
                            entity: game_client.entity,
                            position: RevivePosition::SaveZone,
                        });
                    }
                }
                ClientMessage::SetReviveSaveZone => {
                    if let Some(zone_data) = game_data.zones.get_zone(game_client.position.zone_id)
                    {
                        let revive_position = zone_data
                            .get_closest_revive_position(zone_data.start_position)
                            .unwrap_or(zone_data.start_position);
                        game_client.character_info.revive_zone_id = game_client.position.zone_id;
                        game_client.character_info.revive_position = revive_position;
                    }
                }
                ClientMessage::QuestDelete { slot, quest_id } => {
                    if let Some(quest_slot) = game_client.quest_state.get_quest_slot_mut(slot) {
                        if let Some(quest) = quest_slot {
                            if quest.quest_id == quest_id {
                                *quest_slot = None;
                                game_client
                                    .game_client
                                    .server_message_tx
                                    .send(ServerMessage::QuestDeleteResult {
                                        success: true,
                                        slot,
                                        quest_id,
                                    })
                                    .ok();
                            }
                        }
                    }
                }
                ClientMessage::QuestTrigger { trigger } => {
                    events.quest_trigger_events.send(QuestTriggerEvent {
                        trigger_entity: game_client.entity,
                        trigger_hash: trigger,
                    });
                }
                ClientMessage::PersonalStoreListItems { store_entity_id } => {
                    if let Some((store_entity, _, _)) = client_entity_list
                        .get_zone(game_client.position.zone_id)
                        .and_then(|zone| zone.get_entity(store_entity_id))
                    {
                        events
                            .personal_store_events
                            .send(PersonalStoreEvent::ListItems {
                                store_entity: *store_entity,
                                list_entity: game_client.entity,
                            });
                    }
                }
                ClientMessage::PersonalStoreBuyItem {
                    store_entity_id,
                    store_slot_index,
                    buy_item,
                } => {
                    if let Some((store_entity, _, _)) = client_entity_list
                        .get_zone(game_client.position.zone_id)
                        .and_then(|zone| zone.get_entity(store_entity_id))
                    {
                        events
                            .personal_store_events
                            .send(PersonalStoreEvent::BuyItem {
                                store_entity: *store_entity,
                                buyer_entity: game_client.entity,
                                store_slot_index,
                                buy_item,
                            });
                    }
                }
                ClientMessage::UseItem {
                    item_slot,
                    target_entity_id,
                } => {
                    let target_entity = target_entity_id
                        .and_then(|target_entity_id| {
                            client_entity_list
                                .get_zone(game_client.position.zone_id)
                                .and_then(|zone| zone.get_entity(target_entity_id))
                        })
                        .map(|(target_entity, _, _)| *target_entity);

                    events.use_item_events.send(UseItemEvent::from_inventory(
                        game_client.entity,
                        item_slot,
                        target_entity,
                    ));
                }
                ClientMessage::LevelUpSkill { skill_slot } => {
                    skill_list_try_level_up_skill(
                        &game_data,
                        &mut SkillListBundle {
                            skill_list: &mut game_client.skill_list,
                            skill_points: Some(&mut game_client.skill_points),
                            game_client: Some(game_client.game_client),
                            ability_values: game_client.ability_values,
                            level: game_client.level,
                            move_speed: Some(game_client.move_speed),
                            team: Some(game_client.team),
                            character_info: Some(&game_client.character_info),
                            experience_points: None,
                            inventory: Some(&game_client.inventory),
                            stamina: None,
                            stat_points: None,
                            union_membership: None,
                            health_points: None,
                            mana_points: None,
                        },
                        skill_slot,
                    )
                    .ok();
                }
                ClientMessage::CastSkillSelf { skill_slot } => {
                    if let Some(skill) = game_client.skill_list.get_skill(skill_slot) {
                        entity_commands
                            .insert(NextCommand::with_cast_skill_target_self(skill, None));
                    }
                }
                ClientMessage::CastSkillTargetEntity {
                    skill_slot,
                    target_entity_id,
                } => {
                    if let Some(skill) = game_client.skill_list.get_skill(skill_slot) {
                        if let Some((target_entity, _, _)) = client_entity_list
                            .get_zone(game_client.position.zone_id)
                            .and_then(|zone| zone.get_entity(target_entity_id))
                        {
                            entity_commands.insert(NextCommand::with_cast_skill_target_entity(
                                skill,
                                *target_entity,
                                None,
                            ));
                        }
                    }
                }
                ClientMessage::CastSkillTargetPosition {
                    skill_slot,
                    position,
                } => {
                    if let Some(skill) = game_client.skill_list.get_skill(skill_slot) {
                        entity_commands.insert(NextCommand::with_cast_skill_target_position(
                            skill, position,
                        ));
                    }
                }
                ClientMessage::NpcStoreTransaction {
                    npc_entity_id,
                    buy_items,
                    sell_items,
                } => {
                    if let Some((npc_entity, _, _)) = client_entity_list
                        .get_zone(game_client.position.zone_id)
                        .and_then(|zone| zone.get_entity(npc_entity_id))
                    {
                        events.npc_store_events.send(NpcStoreEvent {
                            store_entity: *npc_entity,
                            transaction_entity: game_client.entity,
                            buy_items,
                            sell_items,
                        });
                    }
                }
                ClientMessage::SitToggle => {
                    if matches!(game_client.command.command, CommandData::Sit) {
                        entity_commands.insert(NextCommand::with_standing());
                    } else {
                        entity_commands.insert(NextCommand::with_sitting());
                    }
                }
                ClientMessage::RunToggle => {
                    if match *game_client.move_mode {
                        MoveMode::Walk => {
                            *game_client.move_mode = MoveMode::Run;
                            true
                        }
                        MoveMode::Run => {
                            *game_client.move_mode = MoveMode::Walk;
                            true
                        }
                        MoveMode::Drive => false,
                    } {
                        server_messages.send_entity_message(
                            game_client.client_entity,
                            ServerMessage::MoveToggle {
                                entity_id: game_client.client_entity.id,
                                move_mode: *game_client.move_mode,
                                run_speed: None,
                            },
                        );
                    }
                }
                ClientMessage::DriveToggle => {
                    if match *game_client.move_mode {
                        MoveMode::Walk | MoveMode::Run => {
                            // TODO: Check if we have a valid cart equipped....

                            // Starting driving decreases vehicle engine life
                            events.item_life_events.send(
                                ItemLifeEvent::DecreaseVehicleEngineLife {
                                    entity: game_client.entity,
                                    amount: None,
                                },
                            );

                            // Start driving
                            *game_client.move_mode = MoveMode::Drive;
                            commands
                                .entity(game_client.entity)
                                .insert(DrivingTime::default());

                            true
                        }
                        MoveMode::Drive => {
                            *game_client.move_mode = MoveMode::Run;
                            commands.entity(game_client.entity).remove::<DrivingTime>();
                            true
                        }
                    } {
                        server_messages.send_entity_message(
                            game_client.client_entity,
                            ServerMessage::MoveToggle {
                                entity_id: game_client.client_entity.id,
                                move_mode: *game_client.move_mode,
                                run_speed: None,
                            },
                        );
                    }
                }
                ClientMessage::DropMoney { quantity } => {
                    let mut money = Money(quantity as i64);
                    if money > game_client.inventory.money {
                        money = game_client.inventory.money;
                        game_client.inventory.money = Money(0)
                    } else {
                        game_client.inventory.money = game_client.inventory.money - money;
                    }

                    if money > Money(0) {
                        ItemDropBundle::spawn(
                            &mut commands,
                            &mut client_entity_list,
                            DroppedItem::Money(money),
                            game_client.position,
                            None,
                            None,
                            &time,
                        );

                        game_client
                            .game_client
                            .server_message_tx
                            .send(ServerMessage::UpdateMoney {
                                money: game_client.inventory.money,
                            })
                            .ok();
                    }
                }
                ClientMessage::DropItem {
                    item_slot,
                    quantity,
                } => {
                    if let Some(inventory_slot) = game_client.inventory.get_item_slot_mut(item_slot)
                    {
                        let quantity = u32::min(
                            quantity as u32,
                            inventory_slot
                                .as_ref()
                                .map(|item| item.get_quantity())
                                .unwrap_or(0),
                        );
                        let item = inventory_slot.try_take_quantity(quantity);

                        if let Some(item) = item {
                            ItemDropBundle::spawn(
                                &mut commands,
                                &mut client_entity_list,
                                DroppedItem::Item(item),
                                game_client.position,
                                None,
                                None,
                                &time,
                            );

                            game_client
                                .game_client
                                .server_message_tx
                                .send(ServerMessage::UpdateInventory {
                                    items: vec![(item_slot, inventory_slot.clone())],
                                    money: None,
                                })
                                .ok();
                        }
                    }
                }
                ClientMessage::UseEmote { motion_id, is_stop } => {
                    entity_commands.insert(NextCommand::with_emote(motion_id, is_stop));
                }
                ClientMessage::WarpGateRequest { warp_gate_id } => {
                    if let Some(warp_gate) = game_data.warp_gates.get_warp_gate(warp_gate_id) {
                        if let Some(zone) = game_data.zones.get_zone(warp_gate.target_zone) {
                            if let Some(event_position) =
                                zone.event_positions.get(&warp_gate.target_event_object)
                            {
                                client_entity_teleport_zone(
                                    &mut commands,
                                    &mut client_entity_list,
                                    game_client.entity,
                                    game_client.client_entity,
                                    game_client.client_entity_sector,
                                    game_client.position,
                                    Position::new(*event_position, warp_gate.target_zone),
                                    Some(game_client.game_client),
                                );
                            }
                        }
                    }
                }
                ClientMessage::PartyCreate { invited_entity_id }
                | ClientMessage::PartyInvite { invited_entity_id } => {
                    if let Some(&(invited_entity, _, _)) = client_entity_list
                        .get_zone(game_client.position.zone_id)
                        .and_then(|zone| zone.get_entity(invited_entity_id))
                    {
                        events.party_events.send(PartyEvent::Invite {
                            owner_entity: game_client.entity,
                            invited_entity,
                        });
                    }
                }
                ClientMessage::PartyLeave => {
                    events.party_events.send(PartyEvent::Leave {
                        leaver_entity: game_client.entity,
                    });
                }
                ClientMessage::PartyChangeOwner {
                    new_owner_entity_id,
                } => {
                    if let Some(&(new_owner_entity, _, _)) = client_entity_list
                        .get_zone(game_client.position.zone_id)
                        .and_then(|zone| zone.get_entity(new_owner_entity_id))
                    {
                        events.party_events.send(PartyEvent::ChangeOwner {
                            owner_entity: game_client.entity,
                            new_owner_entity,
                        });
                    }
                }
                ClientMessage::PartyKick { character_id } => {
                    events.party_events.send(PartyEvent::Kick {
                        owner_entity: game_client.entity,
                        kick_character_id: character_id,
                    });
                }
                ClientMessage::PartyAcceptCreateInvite { owner_entity_id }
                | ClientMessage::PartyAcceptJoinInvite { owner_entity_id } => {
                    if let Some(&(owner_entity, _, _)) = client_entity_list
                        .get_zone(game_client.position.zone_id)
                        .and_then(|zone| zone.get_entity(owner_entity_id))
                    {
                        events.party_events.send(PartyEvent::AcceptInvite {
                            owner_entity,
                            invited_entity: game_client.entity,
                        });
                    }
                }
                ClientMessage::PartyRejectInvite {
                    reason,
                    owner_entity_id,
                } => {
                    if let Some(&(owner_entity, _, _)) = client_entity_list
                        .get_zone(game_client.position.zone_id)
                        .and_then(|zone| zone.get_entity(owner_entity_id))
                    {
                        events.party_events.send(PartyEvent::RejectInvite {
                            reason,
                            owner_entity,
                            invited_entity: game_client.entity,
                        });
                    }
                }
                ClientMessage::PartyUpdateRules {
                    item_sharing,
                    xp_sharing,
                } => {
                    events.party_events.send(PartyEvent::UpdateRules {
                        owner_entity: game_client.entity,
                        item_sharing,
                        xp_sharing,
                    });
                }
                ClientMessage::MoveCollision { position } => {
                    // TODO: Sanity check position
                    entity_commands
                        .insert(NextCommand::with_move(position, None, None))
                        .insert(Position::new(position, game_client.position.zone_id));
                }
                ClientMessage::CraftInsertGem {
                    equipment_index,
                    item_slot,
                } => {
                    if game_client
                        .inventory
                        .get_item(item_slot)
                        .and_then(|item| {
                            if !matches!(item.get_item_type(), ItemType::Gem) {
                                None
                            } else {
                                game_data.items.get_base_item(item.get_item_reference())
                            }
                        })
                        .map_or(false, |item_data| item_data.class == ItemClass::Jewel)
                    {
                        if let Some(equipment_item) =
                            game_client.equipment.get_equipment_item(equipment_index)
                        {
                            if !equipment_item.has_socket {
                                game_client
                                    .game_client
                                    .server_message_tx
                                    .send(ServerMessage::CraftInsertGemError {
                                        error: CraftInsertGemError::NoSocket,
                                    })
                                    .ok();
                            } else if equipment_item.gem > 300 {
                                game_client
                                    .game_client
                                    .server_message_tx
                                    .send(ServerMessage::CraftInsertGemError {
                                        error: CraftInsertGemError::SocketFull,
                                    })
                                    .ok();
                            } else {
                                let equipment_item = game_client
                                    .equipment
                                    .get_equipment_slot_mut(equipment_index)
                                    .as_mut()
                                    .unwrap();

                                if let Some(gem_item) = game_client
                                    .inventory
                                    .get_item_slot_mut(item_slot)
                                    .unwrap()
                                    .try_take_quantity(1)
                                {
                                    equipment_item.gem = gem_item.get_item_number() as u16;

                                    game_client
                                        .game_client
                                        .server_message_tx
                                        .send(ServerMessage::CraftInsertGem {
                                            update_items: vec![
                                                (
                                                    item_slot,
                                                    game_client
                                                        .inventory
                                                        .get_item(item_slot)
                                                        .cloned(),
                                                ),
                                                (
                                                    ItemSlot::Equipment(equipment_index),
                                                    game_client
                                                        .equipment
                                                        .get_equipment_item(equipment_index)
                                                        .cloned()
                                                        .map(Item::Equipment),
                                                ),
                                            ],
                                        })
                                        .ok();

                                    server_messages.send_entity_message(
                                        game_client.client_entity,
                                        ServerMessage::UpdateEquipment {
                                            entity_id: game_client.client_entity.id,
                                            equipment_index,
                                            item: game_client
                                                .equipment
                                                .get_equipment_item(equipment_index)
                                                .cloned(),
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
                ClientMessage::BankOpen => {
                    events.bank_events.send(BankEvent::Open {
                        entity: game_client.entity,
                    });
                }
                ClientMessage::BankDepositItem {
                    item_slot,
                    item,
                    is_premium,
                } => {
                    events.bank_events.send(BankEvent::DepositItem {
                        entity: game_client.entity,
                        item_slot,
                        item,
                        is_premium,
                    });
                }
                ClientMessage::BankWithdrawItem {
                    bank_slot,
                    item,
                    is_premium,
                } => {
                    events.bank_events.send(BankEvent::WithdrawItem {
                        entity: game_client.entity,
                        bank_slot,
                        item,
                        is_premium,
                    });
                }
                ClientMessage::RepairItemUsingNpc {
                    npc_entity_id,
                    item_slot,
                } => {
                    if client_entity_list
                        .get_zone(game_client.position.zone_id)
                        .and_then(|zone| zone.get_entity(npc_entity_id))
                        .map(|(_, _, npc_position)| npc_position.xy())
                        .map_or(false, |npc_position| {
                            game_client.position.position.xy().distance(npc_position) <= 6000.0
                        })
                    {
                        if let Some(Item::Equipment(equipment_item)) =
                            game_client.inventory.get_item(item_slot)
                        {
                            let cost = game_data
                                .ability_value_calculator
                                .calculate_repair_from_npc_price(equipment_item);
                            if game_client.inventory.try_take_money(cost).is_ok() {
                                if let Some(Item::Equipment(equipment_item)) =
                                    game_client.inventory.get_item_mut(item_slot)
                                {
                                    equipment_item.life = 1000;
                                }

                                game_client
                                    .game_client
                                    .server_message_tx
                                    .send(ServerMessage::RepairedItemUsingNpc {
                                        item_slot,
                                        item: game_client
                                            .inventory
                                            .get_item(item_slot)
                                            .unwrap()
                                            .clone(),
                                        updated_money: game_client.inventory.money,
                                    })
                                    .ok();
                            }
                        }
                    }
                }
                ClientMessage::ClanCreate {
                    name,
                    description,
                    mark,
                } => {
                    events.clan_events.send(ClanEvent::Create {
                        creator: game_client.entity,
                        name,
                        description,
                        mark,
                    });
                }
                _ => warn!("[GS] Received unimplemented client message {:?}", message),
            }
        }
    }
}
