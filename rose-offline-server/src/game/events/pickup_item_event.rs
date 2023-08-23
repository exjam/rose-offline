use bevy::prelude::{Entity, Event};

#[derive(Event)]
pub struct PickupItemEvent {
    pub pickup_entity: Entity,
    pub item_entity: Entity,
}
