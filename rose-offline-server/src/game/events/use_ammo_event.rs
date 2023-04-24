use bevy::prelude::Entity;

use rose_data::AmmoIndex;

pub struct UseAmmoEvent {
    pub entity: Entity,
    pub ammo_index: AmmoIndex,
    pub quantity: usize,
}
