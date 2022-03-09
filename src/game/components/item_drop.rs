use bevy_ecs::prelude::Component;

use rose_data::Item;

use crate::game::components::Money;

#[derive(Clone)]
pub enum DroppedItem {
    Item(Item),
    Money(Money),
}

#[derive(Component, Clone)]
pub struct ItemDrop {
    pub item: Option<DroppedItem>,
}

impl ItemDrop {
    pub fn with_dropped_item(item: DroppedItem) -> Self {
        Self { item: Some(item) }
    }
}
