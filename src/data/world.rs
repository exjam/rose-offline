use num_derive::NumOps;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, time::Duration};

pub const WORLD_TICK_DURATION: Duration = Duration::from_secs(10);

#[derive(Copy, Clone, Debug, NumOps, Deserialize, Serialize)]
pub struct WorldTicks(pub u64);

impl From<WorldTicks> for Duration {
    fn from(ticks: WorldTicks) -> Duration {
        Duration::from_millis(ticks.0 * WORLD_TICK_DURATION.as_millis() as u64)
    }
}

impl FromStr for WorldTicks {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u64>().map_err(|_| ())?;
        Ok(WorldTicks(value))
    }
}
