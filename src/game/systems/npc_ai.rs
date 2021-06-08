use legion::{systems::CommandBuffer, world::SubWorld, Entity, Query};

use crate::game::components::{Command, CommandData, MonsterSpawn, MonsterSpawnPoint, Npc};

#[legion::system]
pub fn npc_ai(
    world: &mut SubWorld,
    cmd: &mut CommandBuffer,
    npc_query: &mut Query<(Entity, &Npc, &Command, Option<&MonsterSpawn>)>,
    spawn_point_query: &mut Query<&mut MonsterSpawnPoint>,
) {
    let mut spawn_point_deaths: Vec<(Entity, u32)> = Vec::new();

    npc_query.for_each(world, |(entity, _npc, command, monster_spawn)| {
        if let CommandData::Die = command.command {
            if let Some(monster_spawn) = monster_spawn {
                if let Some((_, deaths)) = spawn_point_deaths
                    .iter_mut()
                    .find(|(entity, _)| *entity == monster_spawn.spawn_point_entity)
                {
                    *deaths += 1;
                } else {
                    spawn_point_deaths.push((monster_spawn.spawn_point_entity, 1));
                }
            }

            // TODO: Call on death AI
            // TODO: Maybe drop item
            cmd.remove(*entity);
        }
    });

    for (entity, deaths) in spawn_point_deaths {
        if let Ok(spawn_point) = spawn_point_query.get_mut(world, entity) {
            spawn_point.num_alive_monsters = spawn_point.num_alive_monsters.saturating_sub(deaths);
        }
    }
}
