use crossbeam_channel::Receiver;
use legion::*;
use std::time::Duration;

use super::{
    components::{MonsterSpawnPoint, Npc, NpcStandingDirection, Position, Team, Zone},
    messages::control::ControlMessage,
    resources::{
        ClientEntityList, ControlChannel, DeltaTime, GameData, LoginTokens, ServerList,
        ServerMessages,
    },
    systems::*,
};
use crate::data::{CharacterCreator, ItemDatabase, NpcDatabase, SkillDatabase, ZoneDatabase};

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

    pub fn run(
        &mut self,
        character_creator: Box<dyn CharacterCreator + Send + Sync>,
        item_database: ItemDatabase,
        npc_database: NpcDatabase,
        skill_database: SkillDatabase,
        zone_database: ZoneDatabase,
    ) {
        let game_data = GameData {
            character_creator,
            items: item_database,
            npcs: npc_database,
            skills: skill_database,
            zones: zone_database,
        };
        let mut client_entity_list = ClientEntityList::new(&game_data.zones);
        let mut world = World::default();

        for (zone_id, zone_data) in game_data.zones.iter() {
            let zone_id = *zone_id;
            let client_entity_zone = client_entity_list.get_zone_mut(zone_id as usize).unwrap();
            world.push((Zone { id: zone_id },));

            for npc in &zone_data.npcs {
                let conversation_index = game_data
                    .npcs
                    .get_conversation(&npc.conversation)
                    .map(|x| x.index)
                    .unwrap_or(0);
                let entity = world.push((
                    Npc::new(npc.npc.0 as u32, conversation_index as u16),
                    NpcStandingDirection::new(npc.direction),
                    Position::new(npc.position, zone_id),
                    Team::default_npc(),
                ));
                world
                    .entry(entity)
                    .unwrap()
                    .add_component(client_entity_zone.allocate(entity, npc.position).unwrap());
            }

            for spawn in &zone_data.monster_spawns {
                world.push((
                    MonsterSpawnPoint::from(spawn),
                    Position::new(spawn.position, zone_id),
                ));
            }
        }

        let mut resources = Resources::default();
        resources.insert(ControlChannel::new(self.control_rx.clone()));
        resources.insert(ServerList::new());
        resources.insert(LoginTokens::new());
        resources.insert(ServerMessages::new());
        resources.insert(client_entity_list);
        resources.insert(game_data);

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
            .add_system(monster_spawn_system())
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
