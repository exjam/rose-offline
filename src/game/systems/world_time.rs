use legion::system;

use crate::{
    data::{WorldTicks, WORLD_TICK_DURATION},
    game::resources::{DeltaTime, WorldTime},
};

#[system]
pub fn world_time(#[resource] delta_time: &DeltaTime, #[resource] world_time: &mut WorldTime) {
    world_time.time_since_last_tick += delta_time.delta;

    if world_time.time_since_last_tick > WORLD_TICK_DURATION {
        world_time.now = world_time.now + WorldTicks(1);
        world_time.time_since_last_tick -= WORLD_TICK_DURATION;
    }
}
