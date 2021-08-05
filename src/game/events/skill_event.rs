use std::time::Instant;

use bevy_ecs::prelude::Entity;
use nalgebra::Point2;

use crate::{
    data::{item::Item, SkillId},
    game::components::ItemSlot,
};

#[derive(Clone)]
pub enum SkillEventTarget {
    Entity(Entity),
    Position(Point2<f32>),
}

#[derive(Clone)]
pub struct SkillEvent {
    pub caster_entity: Entity,
    pub when: Instant,
    pub skill_id: SkillId,
    pub skill_target: SkillEventTarget,
    pub use_item: Option<(ItemSlot, Item)>,
}

impl SkillEvent {
    pub fn new(
        caster_entity: Entity,
        when: Instant,
        skill_id: SkillId,
        skill_target: SkillEventTarget,
        use_item: Option<(ItemSlot, Item)>,
    ) -> Self {
        Self {
            caster_entity,
            when,
            skill_id,
            skill_target,
            use_item,
        }
    }
}
