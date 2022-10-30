use bevy::prelude::Entity;

use rose_game_common::data::Damage;

pub enum ItemLifeEvent {
    DecreaseWeaponLife(Entity),
    DecreaseArmourLife(Entity, Damage),
}
