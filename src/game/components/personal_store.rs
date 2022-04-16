use bevy::ecs::prelude::Component;

use rose_data::Item;

use crate::game::components::{ItemSlot, Money};

pub const PERSONAL_STORE_ITEM_SLOTS: usize = 30;

#[derive(Clone, Component)]
pub struct PersonalStore {
    pub title: String,
    pub skin: i32,
    pub buy_items: [Option<(Item, Money)>; PERSONAL_STORE_ITEM_SLOTS],
    pub sell_items: [Option<(ItemSlot, Money)>; PERSONAL_STORE_ITEM_SLOTS],
}

pub enum PersonalStoreError {
    Full,
}

impl PersonalStore {
    pub fn new(title: String, skin: i32) -> Self {
        Self {
            title,
            skin,
            buy_items: Default::default(),
            sell_items: Default::default(),
        }
    }

    pub fn add_sell_item(
        &mut self,
        item: ItemSlot,
        price: Money,
    ) -> Result<(), PersonalStoreError> {
        for slot in self.sell_items.iter_mut() {
            if slot.is_none() {
                *slot = Some((item, price));
                return Ok(());
            }
        }

        Err(PersonalStoreError::Full)
    }
}
