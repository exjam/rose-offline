use bevy::ecs::prelude::Entity;

use rose_data::Item;

pub struct RewardItemEvent {
    pub entity: Entity,
    pub item: Item,
    pub drop_on_full_inventory: bool,
}

impl RewardItemEvent {
    pub fn new(entity: Entity, item: Item, drop_on_full_inventory: bool) -> Self {
        Self {
            entity,
            item,
            drop_on_full_inventory,
        }
    }
}
