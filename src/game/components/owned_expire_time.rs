use std::time::Instant;

pub struct OwnedExpireTime {
    pub when: Instant,
}

impl OwnedExpireTime {
    pub fn new(when: Instant) -> Self {
        Self { when }
    }
}
