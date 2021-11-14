use std::time::Instant;

pub struct SpawnExpireTime {
    pub when: Instant,
}

impl SpawnExpireTime {
    pub fn new(when: Instant) -> Self {
        Self { when }
    }
}
