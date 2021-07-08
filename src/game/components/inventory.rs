use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::ops::{Add, Sub};

use super::EquipmentIndex;
use crate::data::item::{EquipmentItem, Item, ItemType, StackableItem};

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

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
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
            _ => panic!("Unexpected item_type in InventoryPageType"),
        }
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
        if let Some(index) = self
            .slots
            .iter()
            .enumerate()
            .find(|(_, slot)| {
                slot.as_ref()
                    .map(|slot_item| slot_item.can_stack_with(&item).is_ok())
                    .unwrap_or(true)
            })
            .map(|(index, _)| index)
        {
            if self.slots[index].is_none() {
                self.slots[index] = Some(Item::Stackable(item));
            } else {
                self.slots[index]
                    .as_mut()
                    .unwrap()
                    .stack_with(item)
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
}

#[derive(Copy, Clone)]
pub enum ItemSlot {
    Equipped(EquipmentIndex),
    Inventory(InventoryPageType, usize),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
            Err(remaining)
        } else {
            Ok(())
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

    pub fn get_item(&self, slot: ItemSlot) -> Option<Item> {
        match slot {
            ItemSlot::Inventory(page_type, index) => self
                .get_page(page_type)
                .slots
                .get(index)
                .cloned()
                .unwrap_or(None),
            ItemSlot::Equipped(_) => None,
        }
    }

    pub fn get_item_slot(&self, slot: ItemSlot) -> Option<&Option<Item>> {
        match slot {
            ItemSlot::Inventory(page_type, index) => self.get_page(page_type).slots.get(index),
            ItemSlot::Equipped(_) => None,
        }
    }

    pub fn get_item_slot_mut(&mut self, slot: ItemSlot) -> Option<&mut Option<Item>> {
        match slot {
            ItemSlot::Inventory(page_type, index) => {
                self.get_page_mut(page_type).slots.get_mut(index)
            }
            ItemSlot::Equipped(_) => None,
        }
    }
}
