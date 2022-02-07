use std::time::Duration;

use bevy_ecs::prelude::{Commands, EventReader, Query, Res, ResMut};

use crate::{
    data::Damage,
    game::{
        components::{
            ClientEntity, Command, DamageSource, DamageSources, HealthPoints, MotionData, NpcAi,
        },
        events::{DamageEvent, DamageEventAttack, DamageEventSkill, DamageEventTagged},
        messages::server::{DamageEntity, ServerMessage},
        resources::{ServerMessages, ServerTime},
    },
};

pub fn damage_system(
    mut commands: Commands,
    attacker_query: Query<&ClientEntity>,
    mut defender_query: Query<(
        &ClientEntity,
        &mut HealthPoints,
        Option<&mut DamageSources>,
        Option<&mut NpcAi>,
        Option<&MotionData>,
    )>,
    mut damage_events: EventReader<DamageEvent>,
    mut server_messages: ResMut<ServerMessages>,
    server_time: Res<ServerTime>,
) {
    for damage_event in damage_events.iter() {
        let (attacker_entity, defender_entity, damage, from_skill) = match *damage_event {
            DamageEvent::Attack(DamageEventAttack {
                attacker: attacker_entity,
                defender: defender_entity,
                damage,
            }) => (attacker_entity, defender_entity, damage, None),
            DamageEvent::Skill(DamageEventSkill {
                attacker: attacker_entity,
                defender: defender_entity,
                damage,
                skill_id,
                attacker_intelligence,
            }) => (
                attacker_entity,
                defender_entity,
                damage,
                Some((skill_id, attacker_intelligence)),
            ),
            DamageEvent::Tagged(DamageEventTagged {
                attacker: attacker_entity,
                defender: defender_entity,
            }) => (
                attacker_entity,
                defender_entity,
                Damage {
                    amount: 0,
                    is_critical: false,
                    apply_hit_stun: false,
                },
                None,
            ),
        };

        let attacker_entity_id = attacker_query
            .get(attacker_entity)
            .map(|client_entity| Some(client_entity.id))
            .unwrap_or(None);

        if let Ok((client_entity, mut health_points, damage_sources, npc_ai, motion_data)) =
            defender_query.get_mut(defender_entity)
        {
            if damage.apply_hit_stun {
                // TODO: Apply hit stun by setting next command to HitStun ?
            }

            if health_points.hp == 0 {
                // Entity already dead, ignore any further damage
                continue;
            }

            health_points.hp = i32::max(health_points.hp - damage.amount as i32, 0);

            if !matches!(damage_event, DamageEvent::Tagged(_)) {
                if let Some(attacker_entity_id) = attacker_entity_id {
                    server_messages.send_entity_message(
                        client_entity,
                        ServerMessage::DamageEntity(DamageEntity {
                            attacker_entity_id,
                            defender_entity_id: client_entity.id,
                            damage,
                            is_killed: health_points.hp == 0,
                            from_skill,
                        }),
                    );
                }
            }

            if let Some(mut damage_sources) = damage_sources {
                if let Some(mut source) = damage_sources
                    .damage_sources
                    .iter_mut()
                    .find(|source| source.entity == attacker_entity)
                {
                    source.last_damage_time = server_time.now;
                    source.total_damage += damage.amount as usize;
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
                        entity: attacker_entity,
                        total_damage: damage.amount as usize,
                        first_damage_time: server_time.now,
                        last_damage_time: server_time.now,
                    });
                }
            }

            if let Some(mut npc_ai) = npc_ai {
                npc_ai.pending_damage.push((attacker_entity, damage));
            }

            if health_points.hp == 0 {
                commands.entity(defender_entity).insert(Command::with_die(
                    Some(attacker_entity),
                    Some(damage),
                    motion_data
                        .and_then(|motion_data| motion_data.get_die())
                        .map(|die_motion| die_motion.duration)
                        .or_else(|| Some(Duration::from_secs(1))),
                ));
            }
        }
    }
}
