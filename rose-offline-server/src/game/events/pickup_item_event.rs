use bevy::prelude::Entity;

pub struct PickupItemEvent {
    pub pickup_entity: Entity,
    pub item_entity: Entity,
}
