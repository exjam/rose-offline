use legion::{systems::CommandBuffer, world::SubWorld, Entity, Query};

use crate::game::{
    components::{Command, CommandData, MonsterSpawnPoint, Npc, SpawnOrigin},
    GameData,
};

#[legion::system]
pub fn npc_ai(
    world: &mut SubWorld,
    cmd: &mut CommandBuffer,
    npc_query: &mut Query<(Entity, &Npc, &Command, Option<&SpawnOrigin>)>,
    spawn_point_query: &mut Query<&mut MonsterSpawnPoint>,
    #[resource] game_data: &GameData,
) {
    let mut spawn_point_deaths: Vec<(Entity, u32)> = Vec::new();

    npc_query.for_each(world, |(entity, _npc, command, spawn_origin)| {
        match command.command {
            CommandData::Stop => {
                // TODO: Run idle ai
            }
            CommandData::Die => {
                if let Some(&SpawnOrigin::MonsterSpawnPoint(spawn_point_entity, _)) = spawn_origin {
                    if let Some((_, deaths)) = spawn_point_deaths
                        .iter_mut()
                        .find(|(entity, _)| *entity == spawn_point_entity)
                    {
                        *deaths += 1;
                    } else {
                        spawn_point_deaths.push((spawn_point_entity, 1));
                    }
                }

                // TODO: Call on death AI
                // TODO: Maybe drop item
                cmd.remove(*entity);
            }
            CommandData::Move(_) => todo!(),
            CommandData::Attack(_) => todo!(),
        }
    });

    for (entity, deaths) in spawn_point_deaths {
        if let Ok(spawn_point) = spawn_point_query.get_mut(world, entity) {
            spawn_point.num_alive_monsters = spawn_point.num_alive_monsters.saturating_sub(deaths);
        }
    }
}
