use std::time::Duration;

use legion::Entity;

use crate::data::Damage;

pub struct NpcAi {
    pub ai_index: usize,
    pub idle_duration: Duration,
    pub has_run_created_trigger: bool,
    pub pending_damage: Vec<(Entity, Damage)>,
}

impl NpcAi {
    pub fn new(ai_index: usize) -> Self {
        Self {
            ai_index,
            idle_duration: Duration::default(),
            has_run_created_trigger: false,
            pending_damage: Vec::new(),
        }
    }
}
