use bevy::{ecs::prelude::Entity, prelude::Event};

use rose_data::SkillId;
use rose_game_common::data::Damage;

#[derive(Event)]
pub enum DamageEvent {
    Attack {
        attacker: Entity,
        defender: Entity,
        damage: Damage,
    },
    Immediate {
        attacker: Entity,
        defender: Entity,
        damage: Damage,
    },
    Skill {
        attacker: Entity,
        defender: Entity,
        damage: Damage,
        skill_id: SkillId,
        attacker_intelligence: i32,
    },
    // For aggressive events which do no damage, such as applying a debuff
    Tagged {
        attacker: Entity,
        defender: Entity,
    },
}
