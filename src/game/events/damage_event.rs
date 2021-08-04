use crate::data::Damage;
use bevy_ecs::prelude::Entity;

pub struct DamageEvent {
    pub attacker: Entity,
    pub defender: Entity,
    pub damage: Damage,
}

impl DamageEvent {
    pub fn new(attacker: Entity, defender: Entity, damage: Damage) -> Self {
        Self {
            attacker,
            defender,
            damage,
        }
    }
}
