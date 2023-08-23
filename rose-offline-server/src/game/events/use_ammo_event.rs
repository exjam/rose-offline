use bevy::prelude::{Entity, Event};

use rose_data::AmmoIndex;

#[derive(Event)]
pub struct UseAmmoEvent {
    pub entity: Entity,
    pub ammo_index: AmmoIndex,
    pub quantity: usize,
}
