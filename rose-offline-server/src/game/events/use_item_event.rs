use bevy::ecs::prelude::Entity;

use rose_data::Item;

use crate::game::components::ItemSlot;

pub enum UseItemEvent {
    Inventory {
        entity: Entity,
        item_slot: ItemSlot,
        target_entity: Option<Entity>,
    },
    Item {
        entity: Entity,
        item: Item,
    },
}

impl UseItemEvent {
    pub fn from_inventory(
        entity: Entity,
        item_slot: ItemSlot,
        target_entity: Option<Entity>,
    ) -> Self {
        Self::Inventory {
            entity,
            item_slot,
            target_entity,
        }
    }

    pub fn from_item(entity: Entity, item: Item) -> Self {
        Self::Item { entity, item }
    }
}
