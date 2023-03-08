use bevy::{
    ecs::prelude::{Res, ResMut},
    time::Time,
};

use rose_data::{WorldTicks, WORLD_TICK_DURATION};

use crate::game::resources::WorldTime;

pub fn world_time_system(time: Res<Time>, mut world_time: ResMut<WorldTime>) {
    world_time.time_since_last_tick += time.delta();

    if world_time.time_since_last_tick > WORLD_TICK_DURATION {
        world_time.ticks = world_time.ticks + WorldTicks(1);
        world_time.time_since_last_tick -= WORLD_TICK_DURATION;
    }
}
