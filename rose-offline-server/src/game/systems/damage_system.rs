use std::time::Duration;

use bevy::{
    ecs::prelude::{Commands, EventReader, Query, Res, ResMut},
    prelude::EventWriter,
    time::Time,
};
use rose_game_common::data::Damage;

use crate::game::{
    components::{
        ClientEntity, ClientEntityType, Command, DamageSource, DamageSources, Dead, HealthPoints,
        MotionData, NpcAi,
    },
    events::{DamageEvent, ItemLifeEvent},
    messages::server::ServerMessage,
    resources::ServerMessages,
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
    mut item_life_events: EventWriter<ItemLifeEvent>,
    mut server_messages: ResMut<ServerMessages>,
    time: Res<Time>,
) {
    for damage_event in damage_events.iter() {
        let (attacker_entity, defender_entity, damage, from_skill) = match *damage_event {
            DamageEvent::Attack {
                attacker: attacker_entity,
                defender: defender_entity,
                damage,
            } => (attacker_entity, defender_entity, damage, None),
            DamageEvent::Immediate {
                attacker: attacker_entity,
                defender: defender_entity,
                damage,
            } => (attacker_entity, defender_entity, damage, None),
            DamageEvent::Skill {
                attacker: attacker_entity,
                defender: defender_entity,
                damage,
                skill_id,
                attacker_intelligence,
            } => (
                attacker_entity,
                defender_entity,
                damage,
                Some((skill_id, attacker_intelligence)),
            ),
            DamageEvent::Tagged {
                attacker: attacker_entity,
                defender: defender_entity,
            } => (
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

            if !matches!(damage_event, DamageEvent::Tagged { .. }) {
                if let Some(attacker_entity_id) = attacker_entity_id {
                    server_messages.send_entity_message(
                        client_entity,
                        ServerMessage::DamageEntity {
                            attacker_entity_id,
                            defender_entity_id: client_entity.id,
                            damage,
                            is_killed: health_points.hp == 0,
                            is_immediate: matches!(damage_event, DamageEvent::Immediate { .. }),
                            from_skill,
                        },
                    );
                }

                if matches!(client_entity.entity_type, ClientEntityType::Character) {
                    item_life_events.send(ItemLifeEvent::DecreaseArmourLife {
                        entity: defender_entity,
                        damage,
                    });
                }
            }

            if let Some(mut damage_sources) = damage_sources {
                if let Some(source) = damage_sources
                    .damage_sources
                    .iter_mut()
                    .find(|source| source.entity == attacker_entity)
                {
                    source.last_damage_time = time.last_update().unwrap();
                    source.total_damage += damage.amount as usize;
                } else {
                    // If we have a full list of damage sources, remove the oldest
                    if damage_sources.damage_sources.len() == damage_sources.max_damage_sources {
                        let mut oldest_time = time.last_update().unwrap();
                        let mut oldest_index = None;

                        for i in 0..damage_sources.damage_sources.len() {
                            let damage_source = &damage_sources.damage_sources[i];
                            if damage_source.last_damage_time < oldest_time {
                                oldest_time = damage_source.last_damage_time;
                                oldest_index = Some(i);
                            }
                        }

                        if damage_sources.damage_sources.is_empty() {
                            println!("how cunt, how?");
                        }

                        let default_oldest = damage_sources.damage_sources.len() - 1;
                        damage_sources
                            .damage_sources
                            .swap_remove(oldest_index.unwrap_or(default_oldest));
                    }

                    damage_sources.damage_sources.push(DamageSource {
                        entity: attacker_entity,
                        total_damage: damage.amount as usize,
                        first_damage_time: time.last_update().unwrap(),
                        last_damage_time: time.last_update().unwrap(),
                    });
                }
            }

            if let Some(mut npc_ai) = npc_ai {
                npc_ai.pending_damage.push((attacker_entity, damage));
            }

            if health_points.hp == 0 {
                commands.entity(defender_entity).insert((
                    Dead,
                    Command::with_die(
                        Some(attacker_entity),
                        Some(damage),
                        motion_data
                            .and_then(|motion_data| motion_data.get_die())
                            .map(|die_motion| die_motion.duration)
                            .or_else(|| Some(Duration::from_secs(1))),
                    ),
                ));
            }
        }
    }
}
