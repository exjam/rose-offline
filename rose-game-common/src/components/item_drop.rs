use bevy::ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use rose_data::Item;

use crate::components::Money;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
