use std::time::Duration;

use legion::{system, systems::CommandBuffer, world::SubWorld, Entity, EntityStore, Query};

use crate::{
    game::{
        components::{
            ClientEntity, DamageSource, DamageSources, HealthPoints, MonsterSpawn,
            MonsterSpawnPoint, PendingDamage, Position,
        },
        messages::server::{self, DamageEntity, ServerMessage},
        resources::{DeltaTime, GameData, ServerMessages},
    },
    protocol::Client,
};

#[system(for_each)]
#[read_component(ClientEntity)]
#[read_component(Position)]
#[read_component(MonsterSpawn)]
#[write_component(HealthPoints)]
#[write_component(DamageSources)]
#[write_component(MonsterSpawnPoint)]
pub fn apply_damage(
    world: &mut SubWorld,
    cmd: &mut CommandBuffer,
    entity: &Entity,
    pending_damage: &PendingDamage,
    #[resource] server_messages: &mut ServerMessages,
    #[resource] delta_time: &DeltaTime,
) {
    let mut attacker_entity_id = None;
    if let Ok(entry) = world.entry_mut(pending_damage.attacker) {
        if let Ok(attacker_client_entity) = entry.get_component::<ClientEntity>() {
            attacker_entity_id = Some(attacker_client_entity.id.0);
        }
    }

    let mut notify_spawn_point_entity = None;

    if let Ok(mut entry) = world.entry_mut(pending_damage.defender) {
        if pending_damage.damage.apply_hit_stun {
            // TODO: Apply hit stun by setting next command to HitStun ?
        }

        let mut defender_entity_id = None;
        let mut defender_zone = None;
        if let (Ok(defender_client_entity), Ok(defender_position)) = (
            entry.get_component::<ClientEntity>(),
            entry.get_component::<Position>(),
        ) {
            defender_entity_id = Some(defender_client_entity.id.0);
            defender_zone = Some(defender_position.zone);
        }

        let mut is_killed = false;
        if let Ok(health_points) = entry.get_component_mut::<HealthPoints>() {
            if health_points.hp > 0 {
                health_points.hp = health_points
                    .hp
                    .saturating_sub(pending_damage.damage.amount as u32);
                is_killed = health_points.hp == 0;

                // Send damage packet
                if let (Some(attacker_entity_id), Some(defender_entity_id), Some(defender_zone)) =
                    (attacker_entity_id, defender_entity_id, defender_zone)
                {
                    server_messages.send_zone_message(
                        defender_zone,
                        ServerMessage::DamageEntity(DamageEntity {
                            attacker_entity_id,
                            defender_entity_id,
                            damage: pending_damage.damage,
                            is_killed,
                        }),
                    );
                }
            }
        }

        if is_killed {
            // Notify spawn point that the monster died
            if let Ok(monster_spawn) = entry.get_component::<MonsterSpawn>() {
                notify_spawn_point_entity = Some(monster_spawn.spawn_point_entity);
            }

            // Destroy the entity
            cmd.remove(pending_damage.defender);
        }

        if let Ok(damage_sources) = entry.get_component_mut::<DamageSources>() {
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
    }

    if let Some(spawn_point_entity) = notify_spawn_point_entity {
        if let Ok(mut spawn_point_entry) = world.entry_mut(spawn_point_entity) {
            if let Ok(spawn_point) = spawn_point_entry.get_component_mut::<MonsterSpawnPoint>() {
                spawn_point.num_alive_monsters = spawn_point.num_alive_monsters.saturating_sub(1);
            }
        }
    }

    cmd.remove(*entity);
}
