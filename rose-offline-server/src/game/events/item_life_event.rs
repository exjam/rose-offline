use bevy::prelude::{Entity, Event};

use rose_game_common::data::Damage;

#[derive(Event)]
pub enum ItemLifeEvent {
    DecreaseWeaponLife { entity: Entity },
    DecreaseArmourLife { entity: Entity, damage: Damage },
    DecreaseVehicleEngineLife { entity: Entity, amount: Option<u16> },
}
