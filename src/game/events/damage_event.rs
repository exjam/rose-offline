use crate::data::{Damage, SkillId};
use bevy_ecs::prelude::Entity;

pub struct DamageEventAttack {
    pub attacker: Entity,
    pub defender: Entity,
    pub damage: Damage,
}

pub struct DamageEventSkill {
    pub attacker: Entity,
    pub defender: Entity,
    pub damage: Damage,
    pub skill_id: SkillId,
    pub attacker_intelligence: i32,
}

// For aggressive events which do no damage, such as applying a debuff
pub struct DamageEventTagged {
    pub attacker: Entity,
    pub defender: Entity,
}

pub enum DamageEvent {
    Attack(DamageEventAttack),
    Skill(DamageEventSkill),
    Tagged(DamageEventTagged),
}

impl DamageEvent {
    pub fn with_attack(attacker: Entity, defender: Entity, damage: Damage) -> Self {
        Self::Attack(DamageEventAttack {
            attacker,
            defender,
            damage,
        })
    }

    pub fn with_skill(
        attacker: Entity,
        defender: Entity,
        damage: Damage,
        skill_id: SkillId,
        attacker_intelligence: i32,
    ) -> Self {
        Self::Skill(DamageEventSkill {
            attacker,
            defender,
            damage,
            skill_id,
            attacker_intelligence,
        })
    }

    pub fn with_tagged(attacker: Entity, defender: Entity) -> Self {
        Self::Tagged(DamageEventTagged { attacker, defender })
    }
}
