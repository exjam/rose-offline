use std::time::Duration;

use bevy::{
    ecs::prelude::{Query, Res},
    prelude::{Entity, EventWriter},
    time::Time,
};

use crate::game::{components::DrivingTime, events::ItemLifeEvent};

const ENGINE_USE_INTERVAL: Duration = Duration::from_secs(10);

pub fn driving_time_system(
    mut query: Query<(Entity, &mut DrivingTime)>,
    time: Res<Time>,
    mut item_life_events: EventWriter<ItemLifeEvent>,
) {
    for (entity, mut driving_time) in query.iter_mut() {
        driving_time.time += time.delta();

        if driving_time.time > ENGINE_USE_INTERVAL {
            driving_time.time -= ENGINE_USE_INTERVAL;

            item_life_events.send(ItemLifeEvent::DecreaseVehicleEngineLife { entity });
        }
    }
}
