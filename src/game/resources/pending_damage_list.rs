use crate::data::Damage;
use bevy_ecs::prelude::Entity;

pub struct PendingDamage {
    pub attacker: Entity,
    pub defender: Entity,
    pub damage: Damage,
}

pub type PendingDamageList = Vec<PendingDamage>;
