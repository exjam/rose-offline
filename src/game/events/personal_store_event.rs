use bevy_ecs::prelude::Entity;

use rose_data::Item;

pub struct PersonalStoreEventListItems {
    pub store_entity: Entity,
    pub list_entity: Entity,
}

pub struct PersonalStoreEventBuyItem {
    pub store_entity: Entity,
    pub buyer_entity: Entity,
    pub store_slot_index: usize,
    pub buy_item: Item,
}

pub enum PersonalStoreEvent {
    ListItems(PersonalStoreEventListItems),
    BuyItem(PersonalStoreEventBuyItem),
}
