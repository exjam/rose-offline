use std::time::Instant;

use bevy_ecs::prelude::Entity;
use nalgebra::Point2;

use crate::data::SkillId;

pub enum PendingSkillEffectTarget {
    Entity(Entity),
    Position(Point2<f32>),
}

pub struct PendingSkillEffect {
    pub caster_entity: Entity,
    pub when: Instant,
    pub skill_id: SkillId,
    pub skill_target: PendingSkillEffectTarget,
}

pub type PendingSkillEffectList = Vec<PendingSkillEffect>;
