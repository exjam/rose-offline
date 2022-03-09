use bevy_ecs::prelude::{Res, ResMut};

use rose_data::{WorldTicks, WORLD_TICK_DURATION};

use crate::game::resources::{ServerTime, WorldTime};

pub fn world_time_system(server_time: Res<ServerTime>, mut world_time: ResMut<WorldTime>) {
    world_time.time_since_last_tick += server_time.delta;

    if world_time.time_since_last_tick > WORLD_TICK_DURATION {
        world_time.ticks = world_time.ticks + WorldTicks(1);
        world_time.time_since_last_tick -= WORLD_TICK_DURATION;
    }
}
