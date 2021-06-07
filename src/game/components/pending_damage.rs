use crate::data::Damage;
use legion::Entity;

pub struct PendingDamage {
    pub attacker: Entity,
    pub defender: Entity,
    pub damage: Damage,
}
