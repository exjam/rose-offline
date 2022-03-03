use bevy_ecs::prelude::{Commands, Entity, EventWriter, Query, Res, ResMut, Without};
use log::warn;
use nalgebra::Point3;

use crate::{
    data::{
        account::AccountStorage,
        character::CharacterStorage,
        item::{Item, ItemSlotBehaviour, ItemType, StackError, StackableSlotBehaviour},
        VehicleItemPart,
    },
    game::{
        bundles::{
            client_entity_join_zone, client_entity_leave_zone, client_entity_teleport_zone,
            skill_list_try_level_up_skill, CharacterBundle, ItemDropBundle,
        },
        components::{
            AbilityValues, BasicStatType, BasicStats, CharacterInfo, ClientEntity,
            ClientEntitySector, ClientEntityType, ClientEntityVisibility, Command, CommandData,
            CommandSit, DroppedItem, Equipment, EquipmentIndex, EquipmentItemDatabase,
            ExperiencePoints, GameClient, HealthPoints, Hotbar, Inventory, ItemSlot, Level,
            ManaPoints, Money, MotionData, MoveMode, MoveSpeed, NextCommand, Party, PartyMember,
            PartyMembership, PassiveRecoveryTime, Position, QuestState, SkillList, SkillPoints,
            StatPoints, StatusEffects, StatusEffectsRegen, Team, VehiclePartIndex, WorldClient,
        },
        events::{
            ChatCommandEvent, NpcStoreEvent, PartyEvent, PartyEventChangeOwner, PartyEventInvite,
            PartyEventKick, PartyEventLeave, PartyMemberReconnect, PersonalStoreEvent,
            PersonalStoreEventBuyItem, PersonalStoreEventListItems, QuestTriggerEvent,
            UseItemEvent,
        },
        messages::{
            client::{
                ChangeEquipment, ClientMessage, ConnectionRequestError, GameConnectionResponse,
                JoinZoneResponse, LogoutRequest, NpcStoreTransaction, PartyReply, PartyRequest,
                PersonalStoreBuyItem, QuestDelete, ReviveRequestType, SetHotbarSlot,
                SetHotbarSlotError,
            },
            server::{self, LogoutReply, QuestDeleteResult, ServerMessage, UpdateBasicStat},
        },
        resources::{
            ClientEntityList, GameData, LoginTokens, ServerMessages, ServerTime, WorldTime,
        },
    },
};

fn handle_game_connection_request(
    commands: &mut Commands,
    game_data: &GameData,
    login_tokens: &mut LoginTokens,
    party_query: &mut Query<(Entity, &mut Party)>,
    party_events: &mut EventWriter<PartyEvent>,
    entity: Entity,
    game_client: &mut GameClient,
    token_id: u32,
    password_md5: &str,
) -> Result<GameConnectionResponse, ConnectionRequestError> {
    // Verify token
    let login_token = login_tokens
        .get_token_mut(token_id)
        .ok_or(ConnectionRequestError::InvalidToken)?;
    if login_token.world_client.is_none() || login_token.game_client.is_some() {
        return Err(ConnectionRequestError::InvalidToken);
    }

    // Verify account password
    let _ = AccountStorage::try_load(&login_token.username, password_md5)
        .map_err(|_| ConnectionRequestError::InvalidPassword)?;

    // Try load character
    let character = CharacterStorage::try_load(&login_token.selected_character)
        .map_err(|_| ConnectionRequestError::Failed)?;

    // Update token
    login_token.game_client = Some(entity);
    game_client.login_token = login_token.token;

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
            HealthPoints::new(ability_values.get_max_health()),
            ManaPoints::new(ability_values.get_max_mana()),
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

    // See if we are in a party as an offline member
    let mut party_membership = PartyMembership::default();
    for (party_entity, mut party) in party_query.iter_mut() {
        for party_member in party.members.iter_mut() {
            if let PartyMember::Offline(party_member_character_id, party_member_name) = party_member
            {
                if *party_member_character_id == character.info.unique_id
                    && party_member_name == &character.info.name
                {
                    *party_member = PartyMember::Online(entity);
                    party_membership = PartyMembership::new(party_entity);
                    party_events.send(PartyEvent::MemberReconnect(PartyMemberReconnect {
                        party_entity,
                        reconnect_entity: entity,
                        character_id: character.info.unique_id,
                        name: character.info.name.clone(),
                    }));
                    break;
                }
            }
        }
    }

    let weapon_motion_type = game_data
        .items
        .get_equipped_weapon_item_data(&character.equipment, EquipmentIndex::WeaponRight)
        .map(|item_data| item_data.motion_type)
        .unwrap_or(0) as usize;

    let motion_data = MotionData::from_character(
        game_data.motions.as_ref(),
        weapon_motion_type,
        character.info.gender as usize,
    );

    let move_mode = MoveMode::Run;
    let move_speed = MoveSpeed::new(ability_values.get_run_speed());

    commands.entity(entity).insert_bundle(CharacterBundle {
        ability_values,
        basic_stats: character.basic_stats.clone(),
        command: Command::default(),
        equipment: character.equipment.clone(),
        experience_points: character.experience_points.clone(),
        health_points,
        hotbar: character.hotbar.clone(),
        info: character.info.clone(),
        inventory: character.inventory.clone(),
        level: character.level.clone(),
        mana_points,
        motion_data,
        move_mode,
        move_speed,
        next_command: NextCommand::default(),
        party_membership,
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
    });

    Ok(GameConnectionResponse {
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
    })
}

pub fn game_server_authentication_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut GameClient), Without<CharacterInfo>>,
    mut party_query: Query<(Entity, &mut Party)>,
    mut login_tokens: ResMut<LoginTokens>,
    game_data: Res<GameData>,
    mut party_events: EventWriter<PartyEvent>,
) {
    query.for_each_mut(|(entity, mut game_client)| {
        if let Ok(message) = game_client.client_message_rx.try_recv() {
            match message {
                ClientMessage::GameConnectionRequest(message) => {
                    message
                        .response_tx
                        .send(handle_game_connection_request(
                            &mut commands,
                            game_data.as_ref(),
                            login_tokens.as_mut(),
                            &mut party_query,
                            &mut party_events,
                            entity,
                            game_client.as_mut(),
                            message.login_token,
                            &message.password_md5,
                        ))
                        .ok();
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
            &Level,
            &ExperiencePoints,
            &Team,
            &HealthPoints,
            &ManaPoints,
            &Position,
        ),
        Without<ClientEntity>,
    >,
    mut client_entity_list: ResMut<ClientEntityList>,
    world_time: Res<WorldTime>,
) {
    query.for_each(
        |(
            entity,
            game_client,
            level,
            experience_points,
            team,
            health_points,
            mana_points,
            position,
        )| {
            if let Ok(message) = game_client.client_message_rx.try_recv() {
                match message {
                    ClientMessage::JoinZoneRequest(message) => {
                        if let Ok(entity_id) = client_entity_join_zone(
                            &mut commands,
                            &mut client_entity_list,
                            entity,
                            ClientEntityType::Character,
                            position,
                        ) {
                            commands
                                .entity(entity)
                                .insert(ClientEntityVisibility::new())
                                .insert(PassiveRecoveryTime::default());

                            message
                                .response_tx
                                .send(JoinZoneResponse {
                                    entity_id,
                                    level: level.clone(),
                                    experience_points: experience_points.clone(),
                                    team: team.clone(),
                                    health_points: *health_points,
                                    mana_points: *mana_points,
                                    world_ticks: world_time.ticks,
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

enum UnequipError {
    NoItem,
    InventoryFull,
}

fn unequip_to_inventory(
    equipment: &mut Equipment,
    inventory: &mut Inventory,
    equipment_index: EquipmentIndex,
) -> Result<Vec<(ItemSlot, Option<Item>)>, UnequipError> {
    let equipment_slot = equipment.get_equipment_slot_mut(equipment_index);
    let equipment_item = equipment_slot.take().ok_or(UnequipError::NoItem)?;

    match inventory.try_add_equipment_item(equipment_item) {
        Ok((item_slot, item)) => Ok(vec![
            (item_slot, Some(item.clone())),
            (ItemSlot::Equipment(equipment_index), None),
        ]),
        Err(equipment_item) => {
            // Failed to add to inventory, return item to equipment
            *equipment_slot = Some(equipment_item);
            Err(UnequipError::InventoryFull)
        }
    }
}

fn unequip_vehicle_to_inventory(
    equipment: &mut Equipment,
    inventory: &mut Inventory,
    vehicle_part_index: VehiclePartIndex,
) -> Result<Vec<(ItemSlot, Option<Item>)>, UnequipError> {
    let vehicle_slot = equipment.get_vehicle_slot_mut(vehicle_part_index);
    let vehicle_item = vehicle_slot.take().ok_or(UnequipError::NoItem)?;

    match inventory.try_add_equipment_item(vehicle_item) {
        Ok((item_slot, item)) => Ok(vec![
            (item_slot, Some(item.clone())),
            (ItemSlot::Vehicle(vehicle_part_index), None),
        ]),
        Err(vehicle_item) => {
            // Failed to add to inventory, return item to equipment
            *vehicle_slot = Some(vehicle_item);
            Err(UnequipError::InventoryFull)
        }
    }
}

enum EquipItemError {
    ItemBroken,
    InvalidEquipmentIndex,
    InvalidItem,
    InvalidItemData,
    CannotUnequipOffhand,
    InventoryFull,
}

fn equip_from_inventory(
    game_data: &GameData,
    equipment: &mut Equipment,
    inventory: &mut Inventory,
    equipment_index: EquipmentIndex,
    item_slot: ItemSlot,
) -> Result<Vec<(ItemSlot, Option<Item>)>, EquipItemError> {
    // TODO: Cannot change equipment whilst casting spell
    // TODO: Cannot change equipment whilst stunned

    let equipment_item = inventory
        .get_equipment_item(item_slot)
        .ok_or(EquipItemError::InvalidItem)?;

    let item_data = game_data
        .items
        .get_base_item(equipment_item.item)
        .ok_or(EquipItemError::InvalidItemData)?;

    if equipment_item.life == 0 {
        return Err(EquipItemError::ItemBroken);
    }

    let correct_equipment_index = match equipment_item.item.item_type {
        ItemType::Face => matches!(equipment_index, EquipmentIndex::Face),
        ItemType::Head => matches!(equipment_index, EquipmentIndex::Head),
        ItemType::Body => matches!(equipment_index, EquipmentIndex::Body),
        ItemType::Hands => matches!(equipment_index, EquipmentIndex::Hands),
        ItemType::Feet => matches!(equipment_index, EquipmentIndex::Feet),
        ItemType::Back => matches!(equipment_index, EquipmentIndex::Back),
        ItemType::Jewellery => matches!(
            equipment_index,
            EquipmentIndex::Necklace | EquipmentIndex::Ring | EquipmentIndex::Earring
        ),
        ItemType::Weapon => matches!(equipment_index, EquipmentIndex::WeaponRight),
        ItemType::SubWeapon => matches!(equipment_index, EquipmentIndex::WeaponLeft),
        _ => false,
    };
    if !correct_equipment_index {
        return Err(EquipItemError::InvalidEquipmentIndex);
    }

    // TODO: Check equipment ability requirements

    let mut updated_inventory_items = Vec::new();

    // If we are equipping a two handed weapon, we must unequip offhand first
    if item_data.class.is_two_handed_weapon() {
        let equipment_slot = equipment.get_equipment_slot_mut(EquipmentIndex::WeaponLeft);
        if equipment_slot.is_some() {
            let item = equipment_slot.take();
            if let Some(item) = item {
                match inventory.try_add_equipment_item(item) {
                    Ok((inventory_slot, item)) => {
                        updated_inventory_items
                            .push((ItemSlot::Equipment(EquipmentIndex::WeaponLeft), None));
                        updated_inventory_items.push((inventory_slot, Some(item.clone())));
                    }
                    Err(item) => {
                        // Failed to add to inventory, return item to equipment
                        *equipment_slot = Some(item);
                        return Err(EquipItemError::CannotUnequipOffhand);
                    }
                }
            }
        }
    }

    // Equip item from inventory
    let inventory_slot = inventory.get_item_slot_mut(item_slot).unwrap();
    let equipment_slot = equipment.get_equipment_slot_mut(equipment_index);
    let equipment_item = match inventory_slot.take() {
        Some(Item::Equipment(equipment_item)) => equipment_item,
        _ => unreachable!(),
    };
    *inventory_slot = equipment_slot.take().map(Item::Equipment);
    *equipment_slot = Some(equipment_item);

    updated_inventory_items.push((
        ItemSlot::Equipment(equipment_index),
        equipment_slot.clone().map(Item::Equipment),
    ));
    updated_inventory_items.push((item_slot, inventory_slot.clone()));

    Ok(updated_inventory_items)
}

fn equip_vehicle_from_inventory(
    game_data: &GameData,
    equipment: &mut Equipment,
    inventory: &mut Inventory,
    vehicle_part_index: VehiclePartIndex,
    item_slot: ItemSlot,
) -> Result<Vec<(ItemSlot, Option<Item>)>, EquipItemError> {
    // TODO: Cannot change equipment whilst casting spell
    // TODO: Cannot change equipment whilst stunned

    let equipment_item = inventory
        .get_equipment_item(item_slot)
        .ok_or(EquipItemError::InvalidItem)?;

    let item_data = game_data
        .items
        .get_vehicle_item(equipment_item.item.item_number)
        .ok_or(EquipItemError::InvalidItemData)?;

    if match item_data.vehicle_part {
        VehicleItemPart::Body => vehicle_part_index != VehiclePartIndex::Body,
        VehicleItemPart::Engine => vehicle_part_index != VehiclePartIndex::Engine,
        VehicleItemPart::Leg => vehicle_part_index != VehiclePartIndex::Leg,
        VehicleItemPart::Ability => vehicle_part_index != VehiclePartIndex::Ability,
        VehicleItemPart::Arms => vehicle_part_index != VehiclePartIndex::Arms,
    } {
        return Err(EquipItemError::InvalidEquipmentIndex);
    }

    if vehicle_part_index != VehiclePartIndex::Engine && equipment_item.life == 0 {
        return Err(EquipItemError::ItemBroken);
    }

    // TODO: Check equipment ability requirements

    let mut updated_inventory_items = Vec::new();

    // Equip item from inventory
    let inventory_slot = inventory.get_item_slot_mut(item_slot).unwrap();
    let vehicle_slot = equipment.get_vehicle_slot_mut(vehicle_part_index);
    let equipment_item = match inventory_slot.take() {
        Some(Item::Equipment(equipment_item)) => equipment_item,
        _ => unreachable!(),
    };
    *inventory_slot = vehicle_slot.take().map(Item::Equipment);
    *vehicle_slot = Some(equipment_item);

    updated_inventory_items.push((
        ItemSlot::Vehicle(vehicle_part_index),
        vehicle_slot.clone().map(Item::Equipment),
    ));
    updated_inventory_items.push((item_slot, inventory_slot.clone()));

    Ok(updated_inventory_items)
}

pub fn game_server_main_system(
    mut commands: Commands,
    mut game_client_query: Query<(
        Entity,
        &GameClient,
        &ClientEntity,
        &ClientEntitySector,
        &Position,
        &AbilityValues,
        &Command,
        (
            &mut BasicStats,
            &mut CharacterInfo,
            &mut StatPoints,
            &mut SkillPoints,
            &mut SkillList,
            &mut Hotbar,
            &mut Equipment,
            &mut Inventory,
            &mut QuestState,
            &mut MoveMode,
        ),
    )>,
    world_client_query: Query<&WorldClient>,
    mut client_entity_list: ResMut<ClientEntityList>,
    mut chat_command_events: EventWriter<ChatCommandEvent>,
    mut npc_store_events: EventWriter<NpcStoreEvent>,
    mut party_events: EventWriter<PartyEvent>,
    mut personal_store_events: EventWriter<PersonalStoreEvent>,
    mut quest_trigger_events: EventWriter<QuestTriggerEvent>,
    mut use_item_events: EventWriter<UseItemEvent>,
    mut server_messages: ResMut<ServerMessages>,
    game_data: Res<GameData>,
    server_time: Res<ServerTime>,
) {
    game_client_query.for_each_mut(
        |(
            entity,
            client,
            client_entity,
            client_entity_sector,
            position,
            ability_values,
            command,
            (
                mut basic_stats,
                mut character_info,
                mut stat_points,
                mut skill_points,
                mut skill_list,
                mut hotbar,
                mut equipment,
                mut inventory,
                mut quest_state,
                mut move_mode,
            ),
        )| {
            let mut entity_commands = commands.entity(entity);

            if let Ok(message) = client.client_message_rx.try_recv() {
                match message {
                    ClientMessage::Chat(text) => {
                        if text.chars().next().map_or(false, |c| c == '/') {
                            chat_command_events.send(ChatCommandEvent::new(entity, text));
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
                        entity_commands.insert(NextCommand::with_move(
                            destination,
                            move_target_entity,
                            None,
                        ));
                    }
                    ClientMessage::Attack(message) => {
                        if let Some((target_entity, _, _)) = client_entity_list
                            .get_zone(position.zone_id)
                            .and_then(|zone| zone.get_entity(message.target_entity_id))
                        {
                            entity_commands.insert(NextCommand::with_attack(*target_entity));
                        } else {
                            entity_commands.insert(NextCommand::with_stop(true));
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
                        let updated_inventory_items = if let Some(item_slot) = item_slot {
                            equip_from_inventory(
                                &game_data,
                                &mut equipment,
                                &mut inventory,
                                equipment_index,
                                item_slot,
                            )
                            .ok()
                        } else {
                            unequip_to_inventory(&mut equipment, &mut inventory, equipment_index)
                                .ok()
                        };

                        if let Some(updated_inventory_items) = updated_inventory_items {
                            client
                                .server_message_tx
                                .send(ServerMessage::UpdateInventory(
                                    updated_inventory_items,
                                    None,
                                ))
                                .ok();

                            server_messages.send_entity_message(
                                client_entity,
                                ServerMessage::UpdateEquipment(server::UpdateEquipment {
                                    entity_id: client_entity.id,
                                    equipment_index,
                                    item: equipment.get_equipment_item(equipment_index).cloned(),
                                }),
                            );
                        }
                    }
                    ClientMessage::ChangeVehiclePart(vehicle_part_index, item_slot) => {
                        let updated_inventory_items = if let Some(item_slot) = item_slot {
                            equip_vehicle_from_inventory(
                                &game_data,
                                &mut equipment,
                                &mut inventory,
                                vehicle_part_index,
                                item_slot,
                            )
                            .ok()
                        } else {
                            unequip_vehicle_to_inventory(
                                &mut equipment,
                                &mut inventory,
                                vehicle_part_index,
                            )
                            .ok()
                        };

                        if let Some(updated_inventory_items) = updated_inventory_items {
                            client
                                .server_message_tx
                                .send(ServerMessage::UpdateInventory(
                                    updated_inventory_items,
                                    None,
                                ))
                                .ok();

                            server_messages.send_entity_message(
                                client_entity,
                                ServerMessage::UpdateVehiclePart(server::UpdateVehiclePart {
                                    entity_id: client_entity.id,
                                    vehicle_part_index,
                                    item: equipment.get_vehicle_item(vehicle_part_index).cloned(),
                                }),
                            );
                        }
                    }
                    ClientMessage::ChangeAmmo(ammo_index, item_slot) => {
                        if let Some(item_slot) = item_slot {
                            // Try equip ammo from inventory
                            if let Some(inventory_slot) = inventory.get_item_slot_mut(item_slot) {
                                let ammo_slot = equipment.get_ammo_slot_mut(ammo_index);

                                if let Some(Item::Stackable(ammo_item)) = inventory_slot {
                                    // TODO: Verify bullet type
                                    match ammo_slot.can_stack_with(ammo_item) {
                                        Ok(_) => {
                                            // Can fully stack into ammo slot
                                            ammo_slot.try_stack_with(ammo_item.clone()).unwrap();
                                            *inventory_slot = None;
                                        }
                                        Err(StackError::PartialStack(partial_stack_quantity)) => {
                                            // Can partially stack
                                            ammo_slot
                                                .try_stack_with(
                                                    ammo_item
                                                        .try_take_subquantity(
                                                            partial_stack_quantity,
                                                        )
                                                        .unwrap(),
                                                )
                                                .unwrap();
                                        }
                                        Err(_) => {
                                            // Can't stack, must swap
                                            let previous = ammo_slot.take();
                                            *ammo_slot = Some(ammo_item.clone());
                                            *inventory_slot = previous.map(Item::Stackable);
                                        }
                                    }

                                    client
                                        .server_message_tx
                                        .send(ServerMessage::UpdateInventory(
                                            vec![
                                                (
                                                    ItemSlot::Ammo(ammo_index),
                                                    ammo_slot.clone().map(Item::Stackable),
                                                ),
                                                (item_slot, inventory_slot.clone()),
                                            ],
                                            None,
                                        ))
                                        .ok();

                                    server_messages.send_entity_message(
                                        client_entity,
                                        ServerMessage::UpdateAmmo(
                                            client_entity.id,
                                            ammo_index,
                                            ammo_slot.clone(),
                                        ),
                                    );
                                }
                            }
                        } else {
                            // Try unequip to inventory
                            let ammo_slot = equipment.get_ammo_slot_mut(ammo_index);
                            let item = ammo_slot.take();
                            if let Some(item) = item {
                                match inventory.try_add_stackable_item(item) {
                                    Ok((inventory_slot, item)) => {
                                        *ammo_slot = None;

                                        client
                                            .server_message_tx
                                            .send(ServerMessage::UpdateInventory(
                                                vec![
                                                    (ItemSlot::Ammo(ammo_index), None),
                                                    (inventory_slot, Some(item.clone())),
                                                ],
                                                None,
                                            ))
                                            .ok();

                                        server_messages.send_entity_message(
                                            client_entity,
                                            ServerMessage::UpdateAmmo(
                                                client_entity.id,
                                                ammo_index,
                                                None,
                                            ),
                                        );
                                    }
                                    Err(item) => {
                                        *ammo_slot = Some(item);
                                    }
                                }
                            }
                        }
                    }
                    ClientMessage::IncreaseBasicStat(basic_stat_type) => {
                        if let Some(cost) = game_data
                            .ability_value_calculator
                            .calculate_basic_stat_increase_cost(&basic_stats, basic_stat_type)
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
                    ClientMessage::PickupItemDrop(message) => {
                        if let Some((target_entity, _, _)) = client_entity_list
                            .get_zone(position.zone_id)
                            .and_then(|zone| zone.get_entity(message.target_entity_id))
                        {
                            entity_commands
                                .insert(NextCommand::with_pickup_item_drop(*target_entity));
                        } else {
                            entity_commands.insert(NextCommand::with_stop(true));
                        }
                    }
                    ClientMessage::LogoutRequest(request) => {
                        if let LogoutRequest::ReturnToCharacterSelect = request {
                            // Send ReturnToCharacterSelect via world_client
                            world_client_query.for_each(|world_client| {
                                if world_client.login_token == client.login_token {
                                    world_client
                                        .server_message_tx
                                        .send(ServerMessage::ReturnToCharacterSelect)
                                        .ok();
                                }
                            });
                        }

                        client
                            .server_message_tx
                            .send(ServerMessage::LogoutReply(LogoutReply { result: Ok(()) }))
                            .ok();

                        client_entity_leave_zone(
                            &mut commands,
                            &mut client_entity_list,
                            entity,
                            client_entity,
                            client_entity_sector,
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

                            entity_commands
                                .insert(HealthPoints::new(ability_values.get_max_health()))
                                .insert(ManaPoints::new(ability_values.get_max_mana()));
                            client_entity_teleport_zone(
                                &mut commands,
                                &mut client_entity_list,
                                entity,
                                client_entity,
                                client_entity_sector,
                                position,
                                new_position,
                                Some(client),
                            );
                        }
                    }
                    ClientMessage::SetReviveZone => {
                        if let Some(zone_data) = game_data.zones.get_zone(position.zone_id) {
                            let revive_position = zone_data
                                .get_closest_revive_position(zone_data.start_position)
                                .unwrap_or(zone_data.start_position);
                            character_info.revive_zone_id = position.zone_id;
                            character_info.revive_position = revive_position;
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
                        quest_trigger_events.send(QuestTriggerEvent {
                            trigger_entity: entity,
                            trigger_hash,
                        });
                    }
                    ClientMessage::PersonalStoreListItems(store_entity_id) => {
                        if let Some((store_entity, _, _)) = client_entity_list
                            .get_zone(position.zone_id)
                            .and_then(|zone| zone.get_entity(store_entity_id))
                        {
                            personal_store_events.send(PersonalStoreEvent::ListItems(
                                PersonalStoreEventListItems {
                                    store_entity: *store_entity,
                                    list_entity: entity,
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
                            personal_store_events.send(PersonalStoreEvent::BuyItem(
                                PersonalStoreEventBuyItem {
                                    store_entity: *store_entity,
                                    buyer_entity: entity,
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

                        use_item_events.send(UseItemEvent::new(entity, item_slot, target_entity));
                    }
                    ClientMessage::LevelUpSkill(skill_slot) => {
                        skill_list_try_level_up_skill(
                            &game_data.skills,
                            skill_slot,
                            &mut skill_list,
                            Some(&mut skill_points),
                            Some(client),
                        )
                        .ok();
                    }
                    ClientMessage::CastSkillSelf(skill_slot) => {
                        if let Some(skill) = skill_list.get_skill(skill_slot) {
                            entity_commands
                                .insert(NextCommand::with_cast_skill_target_self(skill, None));
                        }
                    }
                    ClientMessage::CastSkillTargetEntity(skill_slot, target_entity_id) => {
                        if let Some(skill) = skill_list.get_skill(skill_slot) {
                            if let Some((target_entity, _, _)) = client_entity_list
                                .get_zone(position.zone_id)
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
                    ClientMessage::CastSkillTargetPosition(skill_slot, position) => {
                        if let Some(skill) = skill_list.get_skill(skill_slot) {
                            entity_commands.insert(NextCommand::with_cast_skill_target_position(
                                skill, position,
                            ));
                        }
                    }
                    ClientMessage::NpcStoreTransaction(NpcStoreTransaction {
                        npc_entity_id,
                        buy_items,
                        sell_items,
                    }) => {
                        if let Some((npc_entity, _, _)) = client_entity_list
                            .get_zone(position.zone_id)
                            .and_then(|zone| zone.get_entity(npc_entity_id))
                        {
                            npc_store_events.send(NpcStoreEvent {
                                store_entity: *npc_entity,
                                transaction_entity: entity,
                                buy_items,
                                sell_items,
                            });
                        }
                    }
                    ClientMessage::SitToggle => {
                        if matches!(command.command, CommandData::Sit(CommandSit::Sit)) {
                            entity_commands.insert(NextCommand::with_standing());
                        } else {
                            entity_commands.insert(NextCommand::with_sitting());
                        }
                    }
                    ClientMessage::RunToggle => {
                        if match *move_mode {
                            MoveMode::Walk => {
                                *move_mode = MoveMode::Run;
                                true
                            }
                            MoveMode::Run => {
                                *move_mode = MoveMode::Walk;
                                true
                            }
                            MoveMode::Drive => false,
                        } {
                            server_messages.send_entity_message(
                                client_entity,
                                ServerMessage::MoveToggle(server::MoveToggle {
                                    entity_id: client_entity.id,
                                    move_mode: *move_mode,
                                    run_speed: None,
                                }),
                            );
                        }
                    }
                    ClientMessage::DriveToggle => {
                        if match *move_mode {
                            MoveMode::Walk | MoveMode::Run => {
                                *move_mode = MoveMode::Drive;
                                true
                            }
                            MoveMode::Drive => {
                                *move_mode = MoveMode::Run;
                                true
                            }
                        } {
                            server_messages.send_entity_message(
                                client_entity,
                                ServerMessage::MoveToggle(server::MoveToggle {
                                    entity_id: client_entity.id,
                                    move_mode: *move_mode,
                                    run_speed: None,
                                }),
                            );
                        }
                    }
                    ClientMessage::DropMoney(quantity) => {
                        let mut money = Money(quantity as i64);
                        if money > inventory.money {
                            money = inventory.money;
                            inventory.money = Money(0)
                        } else {
                            inventory.money = inventory.money - money;
                        }

                        if money > Money(0) {
                            ItemDropBundle::spawn(
                                &mut commands,
                                &mut client_entity_list,
                                DroppedItem::Money(money),
                                position,
                                None,
                                &server_time,
                            );

                            client
                                .server_message_tx
                                .send(ServerMessage::UpdateMoney(inventory.money))
                                .ok();
                        }
                    }
                    ClientMessage::DropItem(item_slot, quantity) => {
                        if let Some(inventory_slot) = inventory.get_item_slot_mut(item_slot) {
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
                                    position,
                                    None,
                                    &server_time,
                                );

                                client
                                    .server_message_tx
                                    .send(ServerMessage::UpdateInventory(
                                        vec![(item_slot, inventory_slot.clone())],
                                        None,
                                    ))
                                    .ok();
                            }
                        }
                    }
                    ClientMessage::UseEmote(motion_id, is_stop) => {
                        entity_commands.insert(NextCommand::with_emote(motion_id, is_stop));
                    }
                    ClientMessage::WarpGateRequest(warp_gate_id) => {
                        if let Some(warp_gate) = game_data.warp_gates.get_warp_gate(warp_gate_id) {
                            if let Some(zone) = game_data.zones.get_zone(warp_gate.target_zone) {
                                if let Some(event_position) =
                                    zone.event_positions.get(&warp_gate.target_event_object)
                                {
                                    client_entity_teleport_zone(
                                        &mut commands,
                                        &mut client_entity_list,
                                        entity,
                                        client_entity,
                                        client_entity_sector,
                                        position,
                                        Position::new(*event_position, warp_gate.target_zone),
                                        Some(client),
                                    );
                                }
                            }
                        }
                    }
                    ClientMessage::PartyRequest(request) => match request {
                        PartyRequest::Create(invited_entity_id)
                        | PartyRequest::Invite(invited_entity_id) => {
                            if let Some(&(invited_entity, _, _)) = client_entity_list
                                .get_zone(position.zone_id)
                                .and_then(|zone| zone.get_entity(invited_entity_id))
                            {
                                party_events.send(PartyEvent::Invite(PartyEventInvite {
                                    owner_entity: entity,
                                    invited_entity,
                                }));
                            }
                        }
                        PartyRequest::Leave => {
                            party_events.send(PartyEvent::Leave(PartyEventLeave {
                                leaver_entity: entity,
                            }));
                        }
                        PartyRequest::ChangeOwner(new_owner_entity_id) => {
                            if let Some(&(new_owner_entity, _, _)) = client_entity_list
                                .get_zone(position.zone_id)
                                .and_then(|zone| zone.get_entity(new_owner_entity_id))
                            {
                                party_events.send(PartyEvent::ChangeOwner(PartyEventChangeOwner {
                                    owner_entity: entity,
                                    new_owner_entity,
                                }));
                            }
                        }
                        PartyRequest::Kick(character_id) => {
                            party_events.send(PartyEvent::Kick(PartyEventKick {
                                owner_entity: entity,
                                kick_character_id: character_id,
                            }));
                        }
                    },
                    ClientMessage::PartyReply(reply) => match reply {
                        PartyReply::Accept(owner_entity_id) => {
                            if let Some(&(owner_entity, _, _)) = client_entity_list
                                .get_zone(position.zone_id)
                                .and_then(|zone| zone.get_entity(owner_entity_id))
                            {
                                party_events.send(PartyEvent::AcceptInvite(PartyEventInvite {
                                    owner_entity,
                                    invited_entity: entity,
                                }));
                            }
                        }
                        PartyReply::Busy(owner_entity_id) | PartyReply::Reject(owner_entity_id) => {
                            if let Some(&(owner_entity, _, _)) = client_entity_list
                                .get_zone(position.zone_id)
                                .and_then(|zone| zone.get_entity(owner_entity_id))
                            {
                                party_events.send(PartyEvent::RejectInvite(PartyEventInvite {
                                    owner_entity,
                                    invited_entity: entity,
                                }));
                            }
                        }
                    },
                    _ => warn!("Received unimplemented client message {:?}", message),
                }
            }
        },
    );
}
