use bevy_ecs::prelude::{Component, Entity};
use std::time::Duration;

use rose_game_common::data::Damage;

#[derive(Component)]
pub struct NpcAi {
    pub ai_index: usize,
    pub idle_duration: Duration,
    pub has_run_created_trigger: bool,
    pub pending_damage: Vec<(Entity, Damage)>,
    pub has_run_dead_ai: bool,
}

impl NpcAi {
    pub fn new(ai_index: usize) -> Self {
        Self {
            ai_index,
            idle_duration: Duration::default(),
            has_run_created_trigger: false,
            pending_damage: Vec::new(),
            has_run_dead_ai: false,
        }
    }
}
