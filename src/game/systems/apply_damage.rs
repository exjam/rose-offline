use legion::{system, systems::CommandBuffer, world::SubWorld, Query};

use crate::game::{components::{ClientEntity, Command, CommandData, DamageSource, DamageSources, HealthPoints, MonsterSpawn, MonsterSpawnPoint, Position}, messages::server::{DamageEntity, ServerMessage}, resources::{DeltaTime, PendingDamageList, ServerMessages}};

#[allow(clippy::type_complexity)]
#[system]
pub fn apply_damage(
    world: &mut SubWorld,
    cmd: &mut CommandBuffer,
    attacker_query: &mut Query<&ClientEntity>,
    defender_query: &mut Query<(
        &ClientEntity,
        &Position,
        Option<&MonsterSpawn>,
        &mut HealthPoints,
        Option<&mut DamageSources>,
    )>,
    spawn_point_query: &mut Query<&mut MonsterSpawnPoint>,
    #[resource] pending_damage_list: &mut PendingDamageList,
    #[resource] server_messages: &mut ServerMessages,
    #[resource] delta_time: &DeltaTime,
) {
    for pending_damage in pending_damage_list.iter() {
        let attacker_entity_id = attacker_query
            .get(world, pending_damage.attacker)
            .map(|client_entity| Some(client_entity.id.0))
            .unwrap_or(None);

        let mut notify_spawn_point_entity = None;

        if let Ok((client_entity, position, monster_spawn, health_points, damage_sources)) =
            defender_query.get_mut(world, pending_damage.defender)
        {
            if pending_damage.damage.apply_hit_stun {
                // TODO: Apply hit stun by setting next command to HitStun ?
            }

            if health_points.hp == 0 {
                // Entity already dead, ignore any further damage
                continue;
            }

            health_points.hp = health_points
                .hp
                .saturating_sub(pending_damage.damage.amount as u32);

            if let Some(attacker_entity_id) = attacker_entity_id {
                server_messages.send_zone_message(
                    position.zone,
                    ServerMessage::DamageEntity(DamageEntity {
                        attacker_entity_id,
                        defender_entity_id: client_entity.id.0,
                        damage: pending_damage.damage,
                        is_killed: health_points.hp == 0,
                    }),
                );
            }

            if let Some(damage_sources) = damage_sources {
                if let Some(source) = damage_sources
                    .damage_sources
                    .iter_mut()
                    .find(|source| source.entity == pending_damage.attacker)
                {
                    source.last_damage_time = delta_time.now;
                    source.total_damage += pending_damage.damage.amount as usize;
                } else {
                    damage_sources.damage_sources.push(DamageSource {
                        entity: pending_damage.attacker,
                        total_damage: pending_damage.damage.amount as usize,
                        first_damage_time: delta_time.now,
                        last_damage_time: delta_time.now,
                    });
                }
            }

            if health_points.hp == 0 {
                // Notify spawn point that the monster died
                if let Some(monster_spawn) = monster_spawn {
                    notify_spawn_point_entity = Some(monster_spawn.spawn_point_entity);
                }

                // TODO: We should not destroy entity immediately, there is on death AI to run
                //       for monsters and players can be revived etc
                // Destroy the entity
                cmd.remove(pending_damage.defender);
            }
        }

        // Notify spawn point that the monster died
        if let Some(spawn_point) = notify_spawn_point_entity
            .and_then(|entity| spawn_point_query.get_mut(world, entity).ok())
        {
            spawn_point.num_alive_monsters = spawn_point.num_alive_monsters.saturating_sub(1);
        }
    }

    pending_damage_list.clear();
}
