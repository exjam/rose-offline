use std::time::Duration;

use bevy_ecs::prelude::{Commands, Query, Res, ResMut};

use crate::game::{
    components::{
        ClientEntity, Command, DamageSource, DamageSources, HealthPoints, MotionData, NpcAi,
        Position,
    },
    messages::server::{DamageEntity, ServerMessage},
    resources::{PendingDamageList, ServerMessages, ServerTime},
};

#[allow(clippy::type_complexity)]
pub fn damage_system(
    mut commands: Commands,
    attacker_query: Query<&ClientEntity>,
    mut defender_query: Query<(
        &ClientEntity,
        &Position,
        &mut HealthPoints,
        Option<&mut DamageSources>,
        Option<&mut NpcAi>,
        Option<&MotionData>,
    )>,
    mut pending_damage_list: ResMut<PendingDamageList>,
    mut server_messages: ResMut<ServerMessages>,
    server_time: Res<ServerTime>,
) {
    for pending_damage in pending_damage_list.iter() {
        let attacker_entity_id = attacker_query
            .get(pending_damage.attacker)
            .map(|client_entity| Some(client_entity.id))
            .unwrap_or(None);

        if let Ok((
            client_entity,
            position,
            mut health_points,
            damage_sources,
            npc_ai,
            motion_data,
        )) = defender_query.get_mut(pending_damage.defender)
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
                    position.zone_id,
                    ServerMessage::DamageEntity(DamageEntity {
                        attacker_entity_id,
                        defender_entity_id: client_entity.id,
                        damage: pending_damage.damage,
                        is_killed: health_points.hp == 0,
                    }),
                );
            }

            if let Some(mut damage_sources) = damage_sources {
                if let Some(mut source) = damage_sources
                    .damage_sources
                    .iter_mut()
                    .find(|source| source.entity == pending_damage.attacker)
                {
                    source.last_damage_time = server_time.now;
                    source.total_damage += pending_damage.damage.amount as usize;
                } else {
                    // If we have a full list of damage sources, remove the oldest
                    if damage_sources.damage_sources.len() == damage_sources.max_damage_sources {
                        let mut oldest_time = server_time.now;
                        let mut oldest_index = None;

                        for i in 0..damage_sources.damage_sources.len() {
                            let damage_source = &damage_sources.damage_sources[i];
                            if damage_source.last_damage_time < oldest_time {
                                oldest_time = damage_source.last_damage_time;
                                oldest_index = Some(i);
                            }
                        }

                        let default_oldest = damage_sources.damage_sources.len() - 1;
                        damage_sources
                            .damage_sources
                            .remove(oldest_index.unwrap_or(default_oldest));
                    }

                    damage_sources.damage_sources.push(DamageSource {
                        entity: pending_damage.attacker,
                        total_damage: pending_damage.damage.amount as usize,
                        first_damage_time: server_time.now,
                        last_damage_time: server_time.now,
                    });
                }
            }

            if let Some(mut npc_ai) = npc_ai {
                npc_ai
                    .pending_damage
                    .push((pending_damage.attacker, pending_damage.damage));
            }

            if health_points.hp == 0 {
                commands
                    .entity(pending_damage.defender)
                    .insert(Command::with_die(
                        Some(pending_damage.attacker),
                        motion_data
                            .and_then(|motion_data| motion_data.get_die())
                            .map(|die_motion| die_motion.duration)
                            .or_else(|| Some(Duration::from_secs(1))),
                    ));
            }
        }
    }

    pending_damage_list.clear();
}
