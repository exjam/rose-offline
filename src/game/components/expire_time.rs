use std::time::Instant;

pub struct ExpireTime {
    pub when: Instant,
}

impl ExpireTime {
    pub fn new(when: Instant) -> Self {
        Self { when }
    }
}
