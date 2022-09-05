use bevy::ecs::prelude::Component;

use rose_data::{EquipmentItem, Item, StackableItem};

use crate::game::storage::bank::BankStorage;

pub const BANK_MAX_NORMAL_SLOTS: usize = 30 * 3;
pub const BANK_MAX_PREMIUM_SLOTS: usize = 30;

#[derive(Component)]
pub struct Bank {
    pub slots: Vec<Option<Item>>,
    pub sent_to_client: bool,
}

impl Default for Bank {
    fn default() -> Self {
        Self {
            slots: vec![None; BANK_MAX_NORMAL_SLOTS + BANK_MAX_PREMIUM_SLOTS],
            sent_to_client: false,
        }
    }
}

impl From<&Bank> for BankStorage {
    fn from(bank: &Bank) -> Self {
        Self {
            slots: bank.slots.clone(),
        }
    }
}

impl From<BankStorage> for Bank {
    fn from(storage: BankStorage) -> Self {
        Self {
            slots: storage.slots,
            sent_to_client: false,
        }
    }
}

impl Bank {
    pub fn try_add_item(&mut self, item: Item) -> Result<(usize, &Item), Item> {
        match item {
            Item::Equipment(item) => self.try_add_equipment_item(item).map_err(Item::Equipment),
            Item::Stackable(item) => self.try_add_stackable_item(item).map_err(Item::Stackable),
        }
    }

    pub fn try_add_equipment_item(
        &mut self,
        item: EquipmentItem,
    ) -> Result<(usize, &Item), EquipmentItem> {
        let mut index = self
            .slots
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_none())
            .map(|(index, _)| index);

        if index.is_none() && self.slots.len() < BANK_MAX_NORMAL_SLOTS {
            // Add to end
            index = Some(self.slots.len());
            self.slots.push(None);
        }

        if let Some(index) = index {
            self.slots[index] = Some(Item::Equipment(item));
            Ok((index, self.slots[index].as_ref().unwrap()))
        } else {
            Err(item)
        }
    }

    pub fn try_add_stackable_item(
        &mut self,
        item: StackableItem,
    ) -> Result<(usize, &Item), StackableItem> {
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

        if index.is_none() && self.slots.len() < BANK_MAX_NORMAL_SLOTS {
            // Add to end
            index = Some(self.slots.len());
            self.slots.push(None);
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

            Ok((index, self.slots[index].as_ref().unwrap()))
        } else {
            Err(item)
        }
    }
}
