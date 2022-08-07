use bevy::ecs::prelude::Component;
use enum_map::Enum;
use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    ops::{Add, Sub},
};

use rose_data::{
    AmmoIndex, EquipmentIndex, EquipmentItem, Item, ItemReference, ItemSlotBehaviour, ItemType,
    StackableItem, VehiclePartIndex,
};

pub const INVENTORY_PAGE_SIZE: usize = 5 * 6;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Money(pub i64);

impl Add for Money {
    type Output = Self;

    fn add(self, rhs: Money) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl Add<u32> for Money {
    type Output = Self;

    fn add(self, rhs: u32) -> Self {
        Self(self.0.saturating_add(rhs as i64))
    }
}

impl Sub for Money {
    type Output = Self;

    fn sub(self, rhs: Money) -> Self {
        Self(std::cmp::max(self.0.saturating_sub(rhs.0), 0))
    }
}

impl Sub<u32> for Money {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self {
        Self(std::cmp::max(self.0.saturating_sub(rhs as i64), 0))
    }
}

impl From<Money> for u32 {
    fn from(value: Money) -> u32 {
        if value.0 > (u32::MAX as i64) {
            u32::MAX
        } else if let Ok(result) = u32::try_from(value.0) {
            result
        } else {
            0
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Enum)]
pub enum InventoryPageType {
    Equipment,
    Consumables,
    Materials,
    Vehicles,
}

impl InventoryPageType {
    pub fn from_item_type(item_type: ItemType) -> InventoryPageType {
        match item_type {
            ItemType::Face
            | ItemType::Head
            | ItemType::Body
            | ItemType::Hands
            | ItemType::Feet
            | ItemType::Back
            | ItemType::Jewellery
            | ItemType::Weapon
            | ItemType::SubWeapon => InventoryPageType::Equipment,
            ItemType::Consumable => InventoryPageType::Consumables,
            ItemType::Gem | ItemType::Material | ItemType::Quest => InventoryPageType::Materials,
            ItemType::Vehicle => InventoryPageType::Vehicles,
        }
    }
}

impl From<ItemType> for InventoryPageType {
    fn from(val: ItemType) -> Self {
        InventoryPageType::from_item_type(val)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InventoryPage {
    pub page_type: InventoryPageType,
    pub slots: [Option<Item>; INVENTORY_PAGE_SIZE],
}

impl InventoryPage {
    pub fn new(page_type: InventoryPageType) -> Self {
        Self {
            page_type,
            slots: Default::default(),
        }
    }

    pub fn try_add_item(&mut self, item: Item) -> Result<(ItemSlot, &Item), Item> {
        match item {
            Item::Equipment(item) => self.try_add_equipment_item(item).map_err(Item::Equipment),
            Item::Stackable(item) => self.try_add_stackable_item(item).map_err(Item::Stackable),
        }
    }

    pub fn try_add_equipment_item(
        &mut self,
        item: EquipmentItem,
    ) -> Result<(ItemSlot, &Item), EquipmentItem> {
        if let Some((index, slot)) = self
            .slots
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_none())
        {
            *slot = Some(Item::Equipment(item));
            Ok((
                ItemSlot::Inventory(self.page_type, index),
                slot.as_ref().unwrap(),
            ))
        } else {
            Err(item)
        }
    }

    pub fn try_add_stackable_item(
        &mut self,
        item: StackableItem,
    ) -> Result<(ItemSlot, &Item), StackableItem> {
        // First try find an existing item slot we can stack with
        let mut index = self
            .slots
            .iter()
            .enumerate()
            .find(|(_, slot)| {
                slot.as_ref()
                    .map(|slot_item| slot_item.can_stack_with(&item).is_ok())
                    .unwrap_or(false)
            })
            .map(|(index, _)| index);

        if index.is_none() {
            // Else, find the first empty slot
            index = self
                .slots
                .iter()
                .enumerate()
                .find(|(_, slot)| slot.is_none())
                .map(|(index, _)| index);
        }

        if let Some(index) = index {
            if self.slots[index].is_none() {
                self.slots[index] = Some(Item::Stackable(item));
            } else {
                self.slots[index]
                    .as_mut()
                    .unwrap()
                    .try_stack_with(item)
                    .expect("how did we get here");
            }

            Ok((
                ItemSlot::Inventory(self.page_type, index),
                self.slots[index].as_ref().unwrap(),
            ))
        } else {
            Err(item)
        }
    }

    pub fn try_take_item(
        &mut self,
        item_reference: ItemReference,
        quantity: u32,
    ) -> Option<(ItemSlot, Item)> {
        for i in 0..self.slots.len() {
            if let Some(slot_item) = &self.slots[i] {
                if slot_item.is_same_item_reference(item_reference) {
                    if let Some(taken_item) = self.slots[i].try_take_quantity(quantity) {
                        return Some((ItemSlot::Inventory(self.page_type, i), taken_item));
                    }
                }
            }
        }

        None
    }

    pub fn find_item(&self, item_reference: ItemReference) -> Option<ItemSlot> {
        for i in 0..self.slots.len() {
            if let Some(slot_item) = &self.slots[i] {
                if slot_item.is_same_item_reference(item_reference) {
                    return Some(ItemSlot::Inventory(self.page_type, i));
                }
            }
        }

        None
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemSlot {
    Equipment(EquipmentIndex),
    Inventory(InventoryPageType, usize),
    Ammo(AmmoIndex),
    Vehicle(VehiclePartIndex),
}

#[derive(Component, Clone, Debug, Deserialize, Serialize)]
pub struct Inventory {
    pub money: Money,
    pub equipment: InventoryPage,
    pub consumables: InventoryPage,
    pub materials: InventoryPage,
    pub vehicles: InventoryPage,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            money: Default::default(),
            equipment: InventoryPage::new(InventoryPageType::Equipment),
            consumables: InventoryPage::new(InventoryPageType::Consumables),
            materials: InventoryPage::new(InventoryPageType::Materials),
            vehicles: InventoryPage::new(InventoryPageType::Vehicles),
        }
    }
}

#[derive(Debug)]
pub enum InventoryError {
    NotEnoughMoney,
}

#[allow(dead_code)]
impl Inventory {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn try_add_money(&mut self, money: Money) -> Result<(), Money> {
        let before = self.money;
        self.money = self.money + money;

        let remaining = money - (self.money - before);
        if remaining > Money(0) {
            self.money = before;
            Err(money)
        } else {
            Ok(())
        }
    }

    pub fn try_take_money(&mut self, money: Money) -> Result<Money, InventoryError> {
        if self.money >= money {
            self.money = self.money - money;
            Ok(money)
        } else {
            Err(InventoryError::NotEnoughMoney)
        }
    }

    fn get_page(&self, page_type: InventoryPageType) -> &InventoryPage {
        match page_type {
            InventoryPageType::Equipment => &self.equipment,
            InventoryPageType::Consumables => &self.consumables,
            InventoryPageType::Materials => &self.materials,
            InventoryPageType::Vehicles => &self.vehicles,
        }
    }

    fn get_page_mut(&mut self, page_type: InventoryPageType) -> &mut InventoryPage {
        match page_type {
            InventoryPageType::Equipment => &mut self.equipment,
            InventoryPageType::Consumables => &mut self.consumables,
            InventoryPageType::Materials => &mut self.materials,
            InventoryPageType::Vehicles => &mut self.vehicles,
        }
    }

    pub fn try_add_item(&mut self, item: Item) -> Result<(ItemSlot, &Item), Item> {
        let page_type = InventoryPageType::from_item_type(item.get_item_type());
        self.get_page_mut(page_type).try_add_item(item)
    }

    pub fn try_add_equipment_item(
        &mut self,
        item: EquipmentItem,
    ) -> Result<(ItemSlot, &Item), EquipmentItem> {
        let page_type = InventoryPageType::from_item_type(item.item.item_type);
        self.get_page_mut(page_type).try_add_equipment_item(item)
    }

    pub fn try_add_stackable_item(
        &mut self,
        item: StackableItem,
    ) -> Result<(ItemSlot, &Item), StackableItem> {
        let page_type = InventoryPageType::from_item_type(item.item.item_type);
        self.get_page_mut(page_type).try_add_stackable_item(item)
    }

    pub fn get_item(&self, slot: ItemSlot) -> Option<&Item> {
        match slot {
            ItemSlot::Inventory(page_type, index) => self
                .get_page(page_type)
                .slots
                .get(index)
                .and_then(|x| x.as_ref()),
            _ => None,
        }
    }

    pub fn get_equipment_item(&self, slot: ItemSlot) -> Option<&EquipmentItem> {
        match slot {
            ItemSlot::Inventory(page_type, index) => self
                .get_page(page_type)
                .slots
                .get(index)
                .and_then(|x| x.as_ref())
                .and_then(|x| x.as_equipment()),
            _ => None,
        }
    }

    pub fn get_item_slot(&self, slot: ItemSlot) -> Option<&Option<Item>> {
        match slot {
            ItemSlot::Inventory(page_type, index) => self.get_page(page_type).slots.get(index),
            _ => None,
        }
    }

    pub fn get_item_slot_mut(&mut self, slot: ItemSlot) -> Option<&mut Option<Item>> {
        match slot {
            ItemSlot::Inventory(page_type, index) => {
                self.get_page_mut(page_type).slots.get_mut(index)
            }
            _ => None,
        }
    }

    pub fn try_stack_with_item(&mut self, slot: ItemSlot, with_item: Item) -> Option<&Item> {
        self.get_item_slot_mut(slot)
            .and_then(|item_slot| item_slot.try_stack_with_item(with_item).ok())
    }

    pub fn try_take_quantity(&mut self, slot: ItemSlot, quantity: u32) -> Option<Item> {
        self.get_item_slot_mut(slot)
            .and_then(|item_slot| item_slot.try_take_quantity(quantity))
    }

    pub fn try_take_item(
        &mut self,
        item_reference: ItemReference,
        quantity: u32,
    ) -> Option<(ItemSlot, Item)> {
        let page_type = InventoryPageType::from_item_type(item_reference.item_type);
        self.get_page_mut(page_type)
            .try_take_item(item_reference, quantity)
    }

    pub fn find_item(&self, item_reference: ItemReference) -> Option<ItemSlot> {
        for page in [
            &self.equipment,
            &self.consumables,
            &self.materials,
            &self.vehicles,
        ]
        .iter()
        {
            if let Some(slot) = page.find_item(item_reference) {
                return Some(slot);
            }
        }

        None
    }

    pub fn has_empty_slot(&self, page_type: InventoryPageType) -> bool {
        self.get_page(page_type)
            .slots
            .iter()
            .any(|slot| slot.is_none())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Option<Item>> {
        self.equipment
            .slots
            .iter()
            .chain(self.consumables.slots.iter())
            .chain(self.materials.slots.iter())
            .chain(self.vehicles.slots.iter())
    }
}
