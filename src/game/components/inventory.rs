use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::ops::{Add, Sub};

use crate::game::data::items::*;

const INVENTORY_PAGE_SIZE: usize = 5 * 6;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InventoryPage<T> {
    pub slots: [Option<T>; INVENTORY_PAGE_SIZE],
}

impl<T> Default for InventoryPage<T> {
    fn default() -> Self {
        Self {
            slots: Default::default(),
        }
    }
}

impl<T> InventoryPage<T> {
    pub fn try_add_item(&mut self, item: T) -> Result<usize, T> {
        if let Some((index, slot)) = self.slots.iter_mut().enumerate().find(|x| x.1.is_none()) {
            *slot = Some(item);
            Ok(index)
        } else {
            Err(item)
        }
    }
}

impl InventoryPage<StackableItem> {
    pub fn try_add_stackable(
        &mut self,
        item: StackableItem,
    ) -> (Vec<usize>, Option<StackableItem>) {
        let mut slots_updated: Vec<usize> = Vec::new();

        // First try to combine with other stacks of same item
        let mut remaining = Some(item);
        for (index, slot) in self.slots.iter_mut().enumerate() {
            if remaining.is_none() {
                break;
            }

            if slot.is_some() {
                let combine_result = slot
                    .as_mut()
                    .unwrap()
                    .try_combine(remaining.as_ref().unwrap());
                if combine_result.is_ok() {
                    slots_updated.push(index);
                    remaining = combine_result.unwrap();
                }
            }
        }

        // If there is any remaining, then find an empty slot to add it
        if let Some(item) = remaining {
            match self.try_add_item(item) {
                Ok(slot) => {
                    slots_updated.push(slot);
                    remaining = None;
                }
                Err(item) => remaining = Some(item),
            }
        }

        (slots_updated, remaining)
    }
}

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

impl Into<u32> for Money {
    fn into(self) -> u32 {
        if self.0 > (u32::MAX as i64) {
            u32::MAX
        } else if let Ok(result) = u32::try_from(self.0) {
            result
        } else {
            0
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum InventoryPageType {
    Money,
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
            ItemType::Money => InventoryPageType::Money,
        }
    }
}

pub struct ItemSlot(InventoryPageType, usize);

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Inventory {
    pub money: Money,
    pub equipment: InventoryPage<EquipmentItem>,
    pub consumables: InventoryPage<StackableItem>,
    pub materials: InventoryPage<StackableItem>,
    pub vehicles: InventoryPage<EquipmentItem>,
}

#[derive(Default)]
pub struct InventoryUpdateResult {
    pub updated_slots: Vec<ItemSlot>,
    pub updated_money: bool,
    pub remaining_items: Vec<Item>,
}

impl Inventory {
    pub fn add_money(&mut self, money: Money) -> Option<Money> {
        let before = self.money;
        self.money = self.money + money;

        let remaining = money - (self.money - before);
        if remaining > Money(0) {
            Some(remaining)
        } else {
            None
        }
    }

    pub fn add_item(&mut self, item: Item) -> InventoryUpdateResult {
        let mut updated_slots = Vec::new();
        let mut updated_money = false;
        let mut remaining_items = Vec::new();

        match item {
            Item::Equipment(equipment) => {
                let page_type = InventoryPageType::from_item_type(equipment.item_type);
                let page = match page_type {
                    InventoryPageType::Equipment => &mut self.equipment,
                    InventoryPageType::Vehicles => &mut self.vehicles,
                    _ => panic!("Unexpected inventory page for stackable"),
                };

                match page.try_add_item(equipment) {
                    Ok(slot) => updated_slots.push(ItemSlot(page_type, slot)),
                    Err(remaining) => remaining_items.push(Item::Equipment(remaining)),
                }
            }
            Item::Stackable(stackable) => {
                let page_type = InventoryPageType::from_item_type(stackable.item_type);
                let page = match page_type {
                    InventoryPageType::Consumables => &mut self.consumables,
                    InventoryPageType::Materials => &mut self.materials,
                    _ => panic!("Unexpected inventory page for stackable"),
                };

                let (updated, remaining) = page.try_add_stackable(stackable);
                updated_slots = updated.iter().map(|x| ItemSlot(page_type, *x)).collect();

                if let Some(item) = remaining {
                    remaining_items.push(Item::Stackable(item));
                }
            }
            Item::Money(money) => {
                if let Some(remaining) = self.add_money(Money(money.quantity as i64)) {
                    remaining_items.push(Item::Money(MoneyItem {
                        quantity: remaining.into(),
                    }));
                }
                updated_money = true;
            }
        }

        InventoryUpdateResult {
            updated_slots,
            updated_money,
            remaining_items,
        }
    }

    pub fn add_items(&mut self, items: Vec<Item>) -> InventoryUpdateResult {
        let mut result = InventoryUpdateResult::default();
        for item in items {
            let InventoryUpdateResult {
                updated_slots,
                updated_money,
                remaining_items,
            } = self.add_item(item);

            result.updated_slots.extend(updated_slots);
            result.updated_money = result.updated_money || updated_money;
            result.remaining_items.extend(remaining_items);
        }

        result
    }
}
