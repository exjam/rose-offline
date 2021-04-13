use crossbeam_channel::Receiver;
use legion::world::SubWorld;
use legion::*;
use std::{sync::atomic::AtomicU32, sync::atomic::AtomicU8, time::Duration};

use super::{messages::control::ControlMessage, resources::ClientEntityIdList};
use super::resources::{ControlChannel, LoginTokens, ServerList};
use super::systems::*;

pub struct Game {
    tick_rate_hz: u64,
    control_rx: Receiver<ControlMessage>,
}

impl Game {
    pub fn new(control_rx: Receiver<ControlMessage>) -> Game {
        Game {
            tick_rate_hz: 30,
            control_rx,
        }
    }

    pub fn run(&mut self) {
        let mut world = World::default();

        let mut resources = Resources::default();
        resources.insert(ControlChannel {
            control_rx: self.control_rx.clone(),
        });
        resources.insert(ServerList {
            world_servers: Vec::new(),
        });
        resources.insert(LoginTokens { tokens: Vec::new() });
        resources.insert(ClientEntityIdList::new());

        let mut schedule = Schedule::builder()
            .add_system(control_server_system())
            .add_system(login_server_authentication_system())
            .add_system(login_server_system())
            .add_system(world_server_authentication_system())
            .add_system(world_server_system())
            .add_system(game_server_authentication_system())
            .add_system(game_server_system())
            .build();

        let min_tick_duration = Duration::from_millis(1000 / self.tick_rate_hz);

        loop {
            schedule.execute(&mut world, &mut resources);
            std::thread::sleep(min_tick_duration); // TODO: This should account for duration of execution
        }
    }
}
