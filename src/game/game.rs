use crossbeam_channel::Receiver;
use legion::*;
use nalgebra::Point3;
use std::time::Duration;

use super::systems::*;
use super::{
    components::MonsterSpawnPoint,
    resources::{
        ClientEntityList, ControlChannel, DeltaTime, LoginTokens, ServerList, ServerMessages,
    },
};
use super::{
    components::{Npc, Position, Zone},
    messages::control::ControlMessage,
};
use crate::game::data::ZONE_LIST;

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
        let mut client_entity_list = ClientEntityList::new();
        let mut world = World::default();
        for zone_info in ZONE_LIST.zones.iter().filter_map(|x| x.as_ref()) {
            let x_offset = (64.0 / 2.0) * (zone_info.grid_size * zone_info.grid_per_patch * 16.0)
                + (zone_info.grid_size * zone_info.grid_per_patch * 16.0) / 2.0;
            let y_offset = (64.0 / 2.0) * (zone_info.grid_size * zone_info.grid_per_patch * 16.0)
                + (zone_info.grid_size * zone_info.grid_per_patch * 16.0) / 2.0;
            world.push((Zone { id: zone_info.id },));
            let client_entity_zone = client_entity_list
                .get_zone_mut(zone_info.id as usize)
                .unwrap();

            for npc in &zone_info.npcs {
                let position = Point3::new(
                    npc.object.position.x + x_offset,
                    npc.object.position.y + y_offset,
                    npc.object.position.z,
                );
                let entity = world.push((Npc::from(npc), Position::new(position, zone_info.id)));
                world
                    .entry(entity)
                    .unwrap()
                    .add_component(client_entity_zone.allocate(entity, position).unwrap());
            }

            for spawn in &zone_info.monster_spawns {
                world.push((
                    MonsterSpawnPoint::from(spawn),
                    Position::new(
                        Point3::new(
                            spawn.object.position.x + x_offset,
                            spawn.object.position.y + y_offset,
                            spawn.object.position.z,
                        ),
                        zone_info.id,
                    ),
                ));
            }
            /*
            TODO:
                zone.event_objects: Vec<ifo::EventObject>,
                zone.event_positions: Vec<zon::EventPosition>,
            */
        }

        let mut resources = Resources::default();
        resources.insert(ControlChannel::new(self.control_rx.clone()));
        resources.insert(ServerList::new());
        resources.insert(LoginTokens::new());
        resources.insert(ServerMessages::new());
        resources.insert(client_entity_list);

        let mut schedule = Schedule::builder()
            .add_system(control_server_system())
            .add_system(login_server_authentication_system())
            .add_system(login_server_system())
            .add_system(world_server_authentication_system())
            .add_system(world_server_system())
            .add_system(game_server_authentication_system())
            .add_system(game_server_join_system())
            .add_system(game_server_main_system())
            .add_system(game_server_disconnect_handler_system())
            .flush()
            .add_system(update_position_system())
            .flush()
            .add_system(client_entity_visibility_system())
            .add_system(server_messages_sender_system())
            .build();

        let min_tick_duration = Duration::from_millis(1000 / self.tick_rate_hz);
        let mut last_tick = std::time::Instant::now();

        loop {
            let current_tick = std::time::Instant::now();
            resources.insert(DeltaTime {
                delta: current_tick - last_tick,
            });
            schedule.execute(&mut world, &mut resources);
            last_tick = current_tick;
            std::thread::sleep(min_tick_duration); // TODO: This should account for duration of execution
        }
    }
}
