use legion::{system, systems::CommandBuffer, world::SubWorld, Query};

use crate::game::{
    components::{
        ClientEntity, Command, DamageSource, DamageSources, HealthPoints, NpcAi, Position,
    },
    messages::server::{DamageEntity, ServerMessage},
    resources::{DeltaTime, PendingDamageList, ServerMessages},
};

#[allow(clippy::type_complexity)]
#[system]
pub fn apply_damage(
    world: &mut SubWorld,
    cmd: &mut CommandBuffer,
    attacker_query: &mut Query<&ClientEntity>,
    defender_query: &mut Query<(
        &ClientEntity,
        &Position,
        &mut HealthPoints,
        Option<&mut DamageSources>,
        Option<&mut NpcAi>,
    )>,
    #[resource] pending_damage_list: &mut PendingDamageList,
    #[resource] server_messages: &mut ServerMessages,
    #[resource] delta_time: &DeltaTime,
) {
    for pending_damage in pending_damage_list.iter() {
        let attacker_entity_id = attacker_query
            .get(world, pending_damage.attacker)
            .map(|client_entity| Some(client_entity.id))
            .unwrap_or(None);

        if let Ok((client_entity, position, health_points, damage_sources, npc_ai)) =
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
                        defender_entity_id: client_entity.id,
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
                    // If we have a full list of damage sources, remove the oldest
                    if damage_sources.damage_sources.len() == damage_sources.max_damage_sources {
                        let mut oldest_time = delta_time.now;
                        let mut oldest_index = None;

                        for i in 0..damage_sources.damage_sources.len() {
                            let damage_source = &damage_sources.damage_sources[i];
                            if damage_source.last_damage_time < oldest_time {
                                oldest_time = damage_source.last_damage_time;
                                oldest_index = Some(i);
                            }
                        }

                        damage_sources.damage_sources.remove(
                            oldest_index.unwrap_or(damage_sources.damage_sources.len() - 1),
                        );
                    }

                    damage_sources.damage_sources.push(DamageSource {
                        entity: pending_damage.attacker,
                        total_damage: pending_damage.damage.amount as usize,
                        first_damage_time: delta_time.now,
                        last_damage_time: delta_time.now,
                    });
                }
            }

            if let Some(npc_ai) = npc_ai {
                npc_ai
                    .pending_damage
                    .push((pending_damage.attacker, pending_damage.damage));
            }

            if health_points.hp == 0 {
                cmd.add_component(
                    pending_damage.defender,
                    Command::with_die(Some(pending_damage.attacker)),
                );
            }
        }
    }

    pending_damage_list.clear();
}
