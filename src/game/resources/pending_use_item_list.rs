use bevy_ecs::prelude::Entity;

use crate::game::components::ItemSlot;

pub struct PendingUseItem {
    pub entity: Entity,
    pub item_slot: ItemSlot,
    pub target_entity: Option<Entity>,
}

impl PendingUseItem {
    pub fn new(entity: Entity, item_slot: ItemSlot, target_entity: Option<Entity>) -> Self {
        Self {
            entity,
            item_slot,
            target_entity,
        }
    }
}

pub type PendingUseItemList = Vec<PendingUseItem>;
