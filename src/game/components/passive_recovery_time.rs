use std::time::Duration;

pub struct PassiveRecoveryTime {
    pub time: Duration,
}

impl PassiveRecoveryTime {
    pub fn default() -> Self {
        Self {
            time: Duration::from_secs(0),
        }
    }
}
