use std::time::Duration;

pub struct NpcAi {
    pub ai_index: usize,
    pub idle_duration: Duration,
}

impl NpcAi {
    pub fn new(ai_index: usize) -> Self {
        Self {
            ai_index,
            idle_duration: Duration::default(),
        }
    }
}
