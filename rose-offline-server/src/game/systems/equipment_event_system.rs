use bevy::{
    ecs::query::WorldQuery,
    prelude::{Entity, EventReader, Query, Res, ResMut},
};

use rose_data::{
    BaseItemData, EquipmentIndex, Item, ItemType, JobId, StackError, StackableSlotBehaviour,
    VehiclePartIndex,
};
use rose_game_common::messages::server::ServerMessage;

use crate::game::{
    bundles::ability_values_get_value,
    components::{
        AbilityValues, CharacterInfo, ClientEntity, Command, Equipment, ExperiencePoints,
        GameClient, HealthPoints, Inventory, ItemSlot, Level, ManaPoints, MoveSpeed, SkillPoints,
        Stamina, StatPoints, Team, UnionMembership,
    },
    events::EquipmentEvent,
    resources::ServerMessages,
    GameData,
};

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct EquipmentEventEntity<'w> {
    entity: Entity,

    ability_values: &'w AbilityValues,
    character_info: &'w CharacterInfo,
    client_entity: &'w ClientEntity,
    command: &'w Command,
    experience_points: &'w ExperiencePoints,
    health_points: &'w HealthPoints,
    level: &'w Level,
    mana_points: &'w ManaPoints,
    move_speed: &'w MoveSpeed,
    skill_points: &'w SkillPoints,
    stamina: &'w Stamina,
    stat_points: &'w StatPoints,
    team: &'w Team,
    union_membership: &'w UnionMembership,

    game_client: Option<&'w GameClient>,

    inventory: &'w mut Inventory,
    equipment: &'w mut Equipment,
}

pub fn equipment_event_system(
    mut equipment_events: EventReader<EquipmentEvent>,
    mut query: Query<EquipmentEventEntity>,
    game_data: Res<GameData>,
    mut server_messages: ResMut<ServerMessages>,
) {
    for event in equipment_events.iter() {
        match *event {
            EquipmentEvent::ChangeEquipment {
                entity,
                equipment_index,
                item_slot,
            } => {
                let Ok(mut entity) = query.get_mut(entity) else {
                    continue;
                };
                if !entity.command.can_equip_items() {
                    continue;
                }

                let updated_inventory_items = if let Some(item_slot) = item_slot {
                    equip_from_inventory(&game_data, &mut entity, equipment_index, item_slot).ok()
                } else {
                    unequip_to_inventory(
                        &mut entity.equipment,
                        &mut entity.inventory,
                        equipment_index,
                    )
                    .ok()
                };

                if let Some(updated_inventory_items) = updated_inventory_items {
                    if let Some(game_client) = entity.game_client {
                        game_client
                            .server_message_tx
                            .send(ServerMessage::UpdateInventory {
                                items: updated_inventory_items,
                                money: None,
                            })
                            .ok();
                    }

                    server_messages.send_entity_message(
                        entity.client_entity,
                        ServerMessage::UpdateEquipment {
                            entity_id: entity.client_entity.id,
                            equipment_index,
                            item: entity
                                .equipment
                                .get_equipment_item(equipment_index)
                                .cloned(),
                        },
                    );
                }
            }
            EquipmentEvent::ChangeAmmo {
                entity,
                ammo_index,
                item_slot,
            } => {
                let Ok(mut entity) = query.get_mut(entity) else {
                    continue;
                };
                if !entity.command.can_equip_ammo() {
                    continue;
                }

                if let Some(item_slot) = item_slot {
                    // Try equip ammo from inventory
                    if let Some(inventory_slot) = entity.inventory.get_item_slot_mut(item_slot) {
                        let ammo_slot = entity.equipment.get_ammo_slot_mut(ammo_index);

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
                                                .try_take_subquantity(partial_stack_quantity)
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

                            if let Some(game_client) = entity.game_client {
                                game_client
                                    .server_message_tx
                                    .send(ServerMessage::UpdateInventory {
                                        items: vec![
                                            (
                                                ItemSlot::Ammo(ammo_index),
                                                ammo_slot.clone().map(Item::Stackable),
                                            ),
                                            (item_slot, inventory_slot.clone()),
                                        ],
                                        money: None,
                                    })
                                    .ok();
                            }

                            server_messages.send_entity_message(
                                entity.client_entity,
                                ServerMessage::UpdateAmmo {
                                    entity_id: entity.client_entity.id,
                                    ammo_index,
                                    item: ammo_slot.clone(),
                                },
                            );
                        }
                    }
                } else {
                    // Try unequip to inventory
                    let ammo_slot = entity.equipment.get_ammo_slot_mut(ammo_index);
                    let item = ammo_slot.take();
                    if let Some(item) = item {
                        match entity.inventory.try_add_stackable_item(item) {
                            Ok((inventory_slot, item)) => {
                                *ammo_slot = None;

                                if let Some(game_client) = entity.game_client {
                                    game_client
                                        .server_message_tx
                                        .send(ServerMessage::UpdateInventory {
                                            items: vec![
                                                (ItemSlot::Ammo(ammo_index), None),
                                                (inventory_slot, Some(item.clone())),
                                            ],
                                            money: None,
                                        })
                                        .ok();
                                }

                                server_messages.send_entity_message(
                                    entity.client_entity,
                                    ServerMessage::UpdateAmmo {
                                        entity_id: entity.client_entity.id,
                                        ammo_index,
                                        item: None,
                                    },
                                );
                            }
                            Err(item) => {
                                *ammo_slot = Some(item);
                            }
                        }
                    }
                }
            }
            EquipmentEvent::ChangeVehiclePart {
                entity,
                vehicle_part_index,
                item_slot,
            } => {
                let Ok(mut entity) = query.get_mut(entity) else {
                    continue;
                };
                if !entity.command.can_equip_items() {
                    continue;
                }

                let updated_inventory_items = if let Some(item_slot) = item_slot {
                    equip_vehicle_from_inventory(
                        &game_data,
                        &mut entity,
                        vehicle_part_index,
                        item_slot,
                    )
                    .ok()
                } else {
                    unequip_vehicle_to_inventory(
                        &mut entity.equipment,
                        &mut entity.inventory,
                        vehicle_part_index,
                    )
                    .ok()
                };

                if let Some(updated_inventory_items) = updated_inventory_items {
                    if let Some(game_client) = entity.game_client {
                        game_client
                            .server_message_tx
                            .send(ServerMessage::UpdateInventory {
                                items: updated_inventory_items,
                                money: None,
                            })
                            .ok();
                    }

                    server_messages.send_entity_message(
                        entity.client_entity,
                        ServerMessage::UpdateVehiclePart {
                            entity_id: entity.client_entity.id,
                            vehicle_part_index,
                            item: entity
                                .equipment
                                .get_vehicle_item(vehicle_part_index)
                                .cloned(),
                        },
                    );
                }
            }
        }
    }
}

enum EquipItemError {
    ItemBroken,
    InvalidEquipmentIndex,
    InvalidItem,
    InvalidItemData,
    FailedRequirements,
    CannotUnequipOffhand,
    InventoryFull,
}

fn equip_from_inventory(
    game_data: &GameData,
    entity: &mut EquipmentEventEntityItem,
    equipment_index: EquipmentIndex,
    item_slot: ItemSlot,
) -> Result<Vec<(ItemSlot, Option<Item>)>, EquipItemError> {
    // TODO: Cannot change equipment whilst casting spell
    // TODO: Cannot change equipment whilst stunned

    let equipment_item = entity
        .inventory
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
        ItemType::Weapon => matches!(equipment_index, EquipmentIndex::Weapon),
        ItemType::SubWeapon => matches!(equipment_index, EquipmentIndex::SubWeapon),
        _ => false,
    };
    if !correct_equipment_index {
        return Err(EquipItemError::InvalidEquipmentIndex);
    }

    if !check_equipment_job_class(game_data, item_data, entity)
        || !check_equipment_union_membership(item_data, entity)
        || !check_equipment_ability_requirement(item_data, entity)
    {
        return Err(EquipItemError::FailedRequirements);
    }

    let mut updated_inventory_items = Vec::new();

    // If we are equipping a two handed weapon, we must unequip offhand first
    if item_data.class.is_two_handed_weapon() {
        let equipment_slot = entity
            .equipment
            .get_equipment_slot_mut(EquipmentIndex::SubWeapon);
        if equipment_slot.is_some() {
            let item = equipment_slot.take();
            if let Some(item) = item {
                match entity.inventory.try_add_equipment_item(item) {
                    Ok((inventory_slot, item)) => {
                        updated_inventory_items
                            .push((ItemSlot::Equipment(EquipmentIndex::SubWeapon), None));
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
    let inventory_slot = entity.inventory.get_item_slot_mut(item_slot).unwrap();
    let equipment_slot = entity.equipment.get_equipment_slot_mut(equipment_index);
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
    entity: &mut EquipmentEventEntityItem,
    vehicle_part_index: VehiclePartIndex,
    item_slot: ItemSlot,
) -> Result<Vec<(ItemSlot, Option<Item>)>, EquipItemError> {
    // TODO: Cannot change equipment whilst casting spell
    // TODO: Cannot change equipment whilst stunned
    // TODO: Do not allow mixing of cart / castle gear parts

    let equipment_item = entity
        .inventory
        .get_equipment_item(item_slot)
        .ok_or(EquipItemError::InvalidItem)?;

    let item_data = game_data
        .items
        .get_vehicle_item(equipment_item.item.item_number)
        .ok_or(EquipItemError::InvalidItemData)?;

    if vehicle_part_index != item_data.vehicle_part {
        return Err(EquipItemError::InvalidEquipmentIndex);
    }

    if vehicle_part_index != VehiclePartIndex::Engine && equipment_item.life == 0 {
        return Err(EquipItemError::ItemBroken);
    }

    if !check_equipment_job_class(game_data, &item_data.item_data, entity)
        || !check_equipment_ability_requirement(&item_data.item_data, entity)
    {
        return Err(EquipItemError::FailedRequirements);
    }

    let mut updated_inventory_items = Vec::new();

    // Equip item from inventory
    let inventory_slot = entity.inventory.get_item_slot_mut(item_slot).unwrap();
    let vehicle_slot = entity.equipment.get_vehicle_slot_mut(vehicle_part_index);
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

fn check_equipment_job_class(
    game_data: &GameData,
    item_data: &BaseItemData,
    entity: &EquipmentEventEntityItem,
) -> bool {
    let Some(equip_job_class_requirement) = item_data.equip_job_class_requirement else {
        return true;
    };

    let Some(job_class) = game_data.job_class.get(equip_job_class_requirement) else {
        return true;
    };

    if job_class.jobs.is_empty() {
        return true;
    }

    job_class
        .jobs
        .contains(&JobId::new(entity.character_info.job))
}

fn check_equipment_union_membership(
    item_data: &BaseItemData,
    entity: &EquipmentEventEntityItem,
) -> bool {
    if item_data.equip_union_requirement.is_empty() {
        return true;
    }

    item_data
        .equip_union_requirement
        .iter()
        .any(|union| entity.union_membership.current_union == Some(*union))
}

fn check_equipment_ability_requirement(
    item_data: &BaseItemData,
    entity: &EquipmentEventEntityItem,
) -> bool {
    if item_data.equip_ability_requirement.is_empty() {
        return true;
    }

    for &(ability_type, require_value) in item_data.equip_ability_requirement.iter() {
        let value = ability_values_get_value(
            ability_type,
            Some(entity.ability_values),
            Some(entity.level),
            Some(entity.move_speed),
            Some(entity.team),
            Some(entity.character_info),
            Some(entity.experience_points),
            Some(&entity.inventory),
            Some(entity.skill_points),
            Some(entity.stamina),
            Some(entity.stat_points),
            Some(entity.union_membership),
            Some(entity.health_points),
            Some(entity.mana_points),
        )
        .unwrap_or(0);

        if value < require_value as i32 {
            return false;
        }
    }

    true
}
