use bevy_ecs::prelude::Entity;

use crate::game::{components::ItemSlot, messages::client::NpcStoreBuyItem};

pub struct NpcStoreEvent {
    pub store_entity: Entity,
    pub transaction_entity: Entity,
    pub buy_items: Vec<NpcStoreBuyItem>,
    pub sell_items: Vec<(ItemSlot, usize)>,
}
