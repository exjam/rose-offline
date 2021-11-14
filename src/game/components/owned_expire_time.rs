use std::time::Instant;

#[derive(Clone)]
pub struct OwnedExpireTime {
    pub when: Instant,
}

impl OwnedExpireTime {
    pub fn new(when: Instant) -> Self {
        Self { when }
    }
}