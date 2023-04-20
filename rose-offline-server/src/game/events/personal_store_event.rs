use bevy::ecs::prelude::Entity;

use rose_data::Item;

pub enum PersonalStoreEvent {
    ListItems {
        store_entity: Entity,
        list_entity: Entity,
    },
    BuyItem {
        store_entity: Entity,
        buyer_entity: Entity,
        store_slot_index: usize,
        buy_item: Item,
    },
}
