use legion::{component, system, systems::CommandBuffer, world::SubWorld, Entity, Query};
use log::warn;
use nalgebra::Point3;

use crate::{
    data::{account::AccountStorage, character::CharacterStorage, item::Item},
    game::{
        bundles::{
            client_entity_join_zone, client_entity_leave_zone, client_entity_teleport_zone,
            create_character_entity,
        },
        components::{
            AbilityValues, BasicStatType, BasicStats, CharacterInfo, ClientEntity,
            ClientEntityType, ClientEntityVisibility, Command, Equipment, EquipmentIndex,
            EquipmentItemDatabase, ExperiencePoints, GameClient, HealthPoints, Hotbar, Inventory,
            ItemSlot, Level, ManaPoints, MoveMode, MoveSpeed, NextCommand, Position, QuestState,
            SkillList, StatPoints, Team, WorldClient,
        },
        messages::{
            client::{
                ChangeEquipment, ClientMessage, ConnectionRequestError, GameConnectionResponse,
                JoinZoneResponse, LogoutRequest, PersonalStoreBuyItem, QuestDelete,
                ReviveRequestType, SetHotbarSlot, SetHotbarSlotError,
            },
            server::{
                self, LogoutReply, QuestDeleteResult, ServerMessage, UpdateBasicStat,
                UpdateInventory,
            },
        },
        resources::{
            ClientEntityList, GameData, LoginTokens, PendingChatCommandList,
            PendingPersonalStoreEvent, PendingPersonalStoreEventList, PendingQuestTrigger,
            PendingQuestTriggerList, PendingUseItem, PendingUseItemList, PersonalStoreEventBuyItem,
            PersonalStoreEventListItems, ServerMessages, WorldTime,
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

                                // If the character was saved as dead, we must respawn them!
                                let (health_points, mana_points, position) =
                                    if character.health_points.hp == 0 {
                                        (
                                            HealthPoints::new(ability_values.max_health as u32),
                                            ManaPoints::new(ability_values.max_mana as u32),
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
                                    .get_equipped_weapon_item_data(
                                        &character.equipment,
                                        EquipmentIndex::WeaponRight,
                                    )
                                    .map(|item_data| item_data.motion_type)
                                    .unwrap_or(0)
                                    as usize;

                                let motion_data = game_data.motions.get_character_action_motions(
                                    weapon_motion_type,
                                    character.info.gender as usize,
                                );

                                let move_mode = MoveMode::Run;
                                let move_speed = MoveSpeed::new(ability_values.run_speed);

                                create_character_entity(
                                    cmd,
                                    entity,
                                    ability_values,
                                    character.basic_stats.clone(),
                                    Command::default(),
                                    character.equipment.clone(),
                                    character.experience_points.clone(),
                                    health_points,
                                    character.hotbar.clone(),
                                    character.info.clone(),
                                    character.inventory.clone(),
                                    character.level.clone(),
                                    mana_points,
                                    motion_data,
                                    move_mode,
                                    move_speed,
                                    NextCommand::default(),
                                    position.clone(),
                                    character.quest_state.clone(),
                                    character.skill_list.clone(),
                                    character.skill_points,
                                    character.stamina,
                                    character.stat_points,
                                    Team::default_character(),
                                    character.union_membership.clone(),
                                );

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
                                    stamina: character.stamina,
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
                if let Ok(entity_id) = client_entity_join_zone(
                    cmd,
                    client_entity_list,
                    entity,
                    ClientEntityType::Character,
                    position,
                ) {
                    cmd.add_component(*entity, ClientEntityVisibility::new());

                    message
                        .response_tx
                        .send(JoinZoneResponse {
                            entity_id,
                            level: level.clone(),
                            experience_points: experience_points.clone(),
                            team: team.clone(),
                            health_points: *health_points,
                            mana_points: *mana_points,
                            world_time: world_time.now,
                        })
                        .ok();
                }
            }
            _ => warn!("Received unexpected client message {:?}", message),
        }
    }
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
        &mut QuestState,
    )>,
    world_client_query: &mut Query<&WorldClient>,
    #[resource] client_entity_list: &mut ClientEntityList,
    #[resource] pending_chat_command_list: &mut PendingChatCommandList,
    #[resource] pending_quest_trigger_list: &mut PendingQuestTriggerList,
    #[resource] pending_store_event_list: &mut PendingPersonalStoreEventList,
    #[resource] pending_use_item_list: &mut PendingUseItemList,
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
            quest_state,
        )| {
            if let Ok(message) = client.client_message_rx.try_recv() {
                match message {
                    ClientMessage::Chat(text) => {
                        if text.chars().next().map_or(false, |c| c == '/') {
                            pending_chat_command_list.push((*entity, text));
                        } else {
                            server_messages.send_entity_message(
                                client_entity,
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
                            if let Some((target_entity, _, _)) = client_entity_list
                                .get_zone(position.zone_id)
                                .and_then(|zone| zone.get_entity(target_entity_id))
                            {
                                move_target_entity = Some(*target_entity);
                            }
                        }

                        let destination = Point3::new(message.x, message.y, message.z as f32);
                        cmd.add_component(
                            *entity,
                            NextCommand::with_move(destination, move_target_entity),
                        );
                    }
                    ClientMessage::Attack(message) => {
                        if let Some((target_entity, _, _)) = client_entity_list
                            .get_zone(position.zone_id)
                            .and_then(|zone| zone.get_entity(message.target_entity_id))
                        {
                            cmd.add_component(*entity, NextCommand::with_attack(*target_entity));
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
                                        client_entity,
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
                                            client_entity,
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
                        if let Some((target_entity, _, _)) = client_entity_list
                            .get_zone(position.zone_id)
                            .and_then(|zone| zone.get_entity(message.target_entity_id))
                        {
                            cmd.add_component(
                                *entity,
                                NextCommand::with_pickup_dropped_item(*target_entity),
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
                                        game_data.zones.get_zone(position.zone_id)
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

                                    Position::new(revive_position, position.zone_id)
                                }
                                ReviveRequestType::SavePosition => Position::new(
                                    character_info.revive_position,
                                    character_info.revive_zone_id,
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
                    ClientMessage::QuestDelete(QuestDelete { slot, quest_id }) => {
                        if let Some(quest_slot) = quest_state.get_quest_slot_mut(slot) {
                            if let Some(quest) = quest_slot {
                                if quest.quest_id == quest_id {
                                    *quest_slot = None;
                                    client
                                        .server_message_tx
                                        .send(ServerMessage::QuestDeleteResult(QuestDeleteResult {
                                            success: true,
                                            slot,
                                            quest_id,
                                        }))
                                        .ok();
                                }
                            }
                        }
                    }
                    ClientMessage::QuestTrigger(trigger_hash) => {
                        pending_quest_trigger_list.push(PendingQuestTrigger {
                            trigger_entity: *entity,
                            trigger_hash,
                        });
                    }
                    ClientMessage::PersonalStoreListItems(store_entity_id) => {
                        if let Some((store_entity, _, _)) = client_entity_list
                            .get_zone(position.zone_id)
                            .and_then(|zone| zone.get_entity(store_entity_id))
                        {
                            pending_store_event_list.push(PendingPersonalStoreEvent::ListItems(
                                PersonalStoreEventListItems {
                                    store_entity: *store_entity,
                                    list_entity: *entity,
                                },
                            ));
                        }
                    }
                    ClientMessage::PersonalStoreBuyItem(PersonalStoreBuyItem {
                        store_entity_id,
                        store_slot_index,
                        buy_item,
                    }) => {
                        if let Some((store_entity, _, _)) = client_entity_list
                            .get_zone(position.zone_id)
                            .and_then(|zone| zone.get_entity(store_entity_id))
                        {
                            pending_store_event_list.push(PendingPersonalStoreEvent::BuyItem(
                                PersonalStoreEventBuyItem {
                                    store_entity: *store_entity,
                                    buyer_entity: *entity,
                                    store_slot_index,
                                    buy_item,
                                },
                            ));
                        }
                    }
                    ClientMessage::UseItem(item_slot, target_entity_id) => {
                        let target_entity = target_entity_id
                            .and_then(|target_entity_id| {
                                client_entity_list
                                    .get_zone(position.zone_id)
                                    .and_then(|zone| zone.get_entity(target_entity_id))
                            })
                            .map(|(target_entity, _, _)| *target_entity);

                        pending_use_item_list.push(PendingUseItem::new(
                            *entity,
                            item_slot,
                            target_entity,
                        ));
                    }
                    ClientMessage::CastSkillSelf(skill_slot) => {
                        if let Some(skill) = skill_list.get_skill(skill_slot) {
                            cmd.add_component(
                                *entity,
                                NextCommand::with_cast_skill_target_self(skill),
                            );
                        }
                    }
                    ClientMessage::CastSkillTargetEntity(skill_slot, target_entity_id) => {
                        if let Some(skill) = skill_list.get_skill(skill_slot) {
                            if let Some((target_entity, _, _)) = client_entity_list
                                .get_zone(position.zone_id)
                                .and_then(|zone| zone.get_entity(target_entity_id))
                            {
                                cmd.add_component(
                                    *entity,
                                    NextCommand::with_cast_skill_target_entity(
                                        skill,
                                        *target_entity,
                                    ),
                                );
                            }
                        }
                    }
                    ClientMessage::CastSkillTargetPosition(skill_slot, position) => {
                        if let Some(skill) = skill_list.get_skill(skill_slot) {
                            cmd.add_component(
                                *entity,
                                NextCommand::with_cast_skill_target_position(skill, position),
                            );
                        }
                    }
                    _ => warn!("Received unimplemented client message {:?}", message),
                }
            }
        },
    );
}
