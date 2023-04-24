use std::time::Instant;

use bevy::{prelude::Component, utils::HashMap};

use rose_data::SkillId;

const MAX_SKILL_COOLDOWN_GROUPS: usize = 16;

#[derive(Default, Component)]
pub struct Cooldowns {
    pub skill: HashMap<SkillId, Instant>,
    pub skill_global: Option<Instant>,
    pub skill_group: [Option<Instant>; MAX_SKILL_COOLDOWN_GROUPS],
}
