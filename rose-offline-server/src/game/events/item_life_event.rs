use bevy::prelude::Entity;

use rose_game_common::data::Damage;

pub enum ItemLifeEvent {
    DecreaseWeaponLife { entity: Entity },
    DecreaseArmourLife { entity: Entity, damage: Damage },
    DecreaseVehicleEngineLife { entity: Entity, amount: Option<u16> },
}
