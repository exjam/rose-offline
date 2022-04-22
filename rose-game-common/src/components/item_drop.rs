use bevy::ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use rose_data::Item;

use crate::components::Money;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DroppedItem {
    Item(Item),
    Money(Money),
}

impl<T: Into<Item>> From<T> for DroppedItem {
    fn from(item: T) -> Self {
        DroppedItem::Item(item.into())
    }
}

impl From<Money> for DroppedItem {
    fn from(money: Money) -> Self {
        DroppedItem::Money(money)
    }
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
