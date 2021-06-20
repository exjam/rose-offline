use std::ops::{Range, RangeInclusive};

use legion::{systems::CommandBuffer, world::SubWorld, Entity, Query};
use nalgebra::{Point3, Vector3};
use rand::{prelude::ThreadRng, Rng};

use crate::{
    data::formats::{
        AipAction, AipCondition, AipConditionCountNearbyEntities, AipDamageType, AipDistanceOrigin,
        AipEvent, AipMoveMode, AipMoveOrigin, AipOperatorType, AipTrigger,
    },
    game::{
        components::{
            AbilityValues, Command, CommandData, Level, MonsterSpawnPoint, NextCommand, Npc, NpcAi,
            Position, SpawnOrigin, Team,
        },
        resources::{ClientEntityList, DeltaTime},
        GameData,
    },
};

struct AiSourceEntity {
    entity: Entity,
    position: Position,
    level: Level,
    team: Team,
    spawn_origin: Option<SpawnOrigin>,
}

struct AiAttackerEntity {
    entity: Entity,
    position: Position,
    level: Level,
    team: Team,
    ability_values: AbilityValues,
    // TODO: Missing data on if clan master
}

fn compare_aip_value(operator: AipOperatorType, value1: i32, value2: i32) -> bool {
    match operator {
        AipOperatorType::Equals => value1 == value2,
        AipOperatorType::GreaterThan => value1 > value2,
        AipOperatorType::GreaterThanEqual => value1 >= value2,
        AipOperatorType::LessThan => value1 < value2,
        AipOperatorType::LessThanEqual => value1 <= value2,
        AipOperatorType::NotEqual => value1 != value2,
    }
}

struct AiParameters {
    ai_entity: AiSourceEntity,
    attacker: Option<AiAttackerEntity>,
    find_char: Option<(Entity, Point3<f32>)>,
    near_char: Option<(Entity, Point3<f32>)>,
    damage_received: i32,
    is_dead: bool,
}

struct AiWorld<'a, 'b, 'c, 'd> {
    cmd: &'a mut CommandBuffer,
    client_entity_list: &'a ClientEntityList,
    nearby_query_world: &'a mut SubWorld<'b>,
    nearby_query: &'a mut Query<(&'c Level, &'d Team)>,
    rng: ThreadRng,
}

enum AiConditionResult {
    Failed,
}

fn ai_condition_count_nearby_entities(
    ai_world: &mut AiWorld,
    ai_parameters: &mut AiParameters,
    distance: i32,
    is_allied: bool,
    level_diff_range: RangeInclusive<i32>,
    count_operator_type: AipOperatorType,
    count: i32,
) -> Result<(), AiConditionResult> {
    let mut find_char = None;
    let mut near_char_distance = None;
    let mut find_count = 0;

    let zone_entities = ai_world
        .client_entity_list
        .get_zone(ai_parameters.ai_entity.position.zone as usize)
        .ok_or(AiConditionResult::Failed)?;

    for (entity, position) in zone_entities.iter_entities_within_distance(
        ai_parameters.ai_entity.position.position.xy(),
        distance as f32,
    ) {
        // Ignore self entity
        if entity == ai_parameters.ai_entity.entity {
            continue;
        }

        // Check level and team requirements
        if !ai_world
            .nearby_query
            .get(ai_world.nearby_query_world, entity)
            .map_or(false, |(level, team)| {
                let level_diff = ai_parameters.ai_entity.level.level as i32 - level.level as i32;

                is_allied == (team.id == ai_parameters.ai_entity.team.id)
                    && level_diff_range.contains(&level_diff)
            })
        {
            continue;
        }

        // Update near char for nearest found character
        let distance_squared =
            (ai_parameters.ai_entity.position.position - position).magnitude_squared();
        if near_char_distance.map_or(true, |x| distance_squared < x) {
            ai_parameters.near_char = Some((entity, position));
            near_char_distance = Some(distance_squared);
        }

        // Continue until we have satisfy count
        find_count += 1;
        if compare_aip_value(count_operator_type, find_count, count) {
            find_char = Some((entity, position));
            break;
        }
    }

    if find_char.is_some() {
        ai_parameters.find_char = find_char;
        Ok(())
    } else {
        Err(AiConditionResult::Failed)
    }
}

fn ai_condition_damage(
    _ai_world: &mut AiWorld,
    ai_parameters: &mut AiParameters,
    damage_type: AipDamageType,
    operator: AipOperatorType,
    value: i32,
) -> bool {
    match damage_type {
        AipDamageType::Given => false,
        AipDamageType::Received => {
            compare_aip_value(operator, ai_parameters.damage_received, value)
        }
    }
}

fn ai_condition_distance(
    _ai_world: &mut AiWorld,
    ai_parameters: &mut AiParameters,
    origin: AipDistanceOrigin,
    operator: AipOperatorType,
    value: i32,
) -> bool {
    let distance_squared = match origin {
        AipDistanceOrigin::Spawn => match ai_parameters.ai_entity.spawn_origin {
            Some(SpawnOrigin::MonsterSpawnPoint(_, spawn_position)) => Some(
                (ai_parameters.ai_entity.position.position.xy() - spawn_position.xy())
                    .magnitude_squared() as i32,
            ),
            _ => None,
        },
        AipDistanceOrigin::Owner => {
            // TODO: Distance to owner
            None
        }
        AipDistanceOrigin::Target => {
            // TODO: Distance to target
            None
        }
    };

    if let Some(distance_squared) = distance_squared {
        compare_aip_value(operator, distance_squared, value * value)
    } else {
        false
    }
}

fn ai_condition_random(
    ai_world: &mut AiWorld,
    _ai_parameters: &mut AiParameters,
    operator: AipOperatorType,
    range: Range<i32>,
    value: i32,
) -> bool {
    compare_aip_value(operator, ai_world.rng.gen_range(range), value)
}

fn npc_ai_check_conditions(
    ai_program_event: &AipEvent,
    ai_world: &mut AiWorld,
    ai_parameters: &mut AiParameters,
) -> bool {
    for condition in ai_program_event.conditions.iter() {
        let result = match condition {
            AipCondition::CountNearbyEntities(AipConditionCountNearbyEntities {
                distance,
                is_allied,
                level_diff_range,
                count_operator_type,
                count,
            }) => ai_condition_count_nearby_entities(
                ai_world,
                ai_parameters,
                *distance,
                *is_allied,
                level_diff_range.clone(),
                *count_operator_type,
                *count,
            )
            .is_ok(),
            &AipCondition::Damage(damage_type, operator, value) => {
                ai_condition_damage(ai_world, ai_parameters, damage_type, operator, value)
            }
            &AipCondition::Distance(origin, operator, value) => {
                ai_condition_distance(ai_world, ai_parameters, origin, operator, value)
            }
            AipCondition::Random(operator, range, value) => {
                ai_condition_random(ai_world, ai_parameters, *operator, range.clone(), *value)
            }
            _ => false,
        };

        if !result {
            return false;
        }
    }

    true
}

fn ai_action_stop(ai_world: &mut AiWorld, ai_parameters: &mut AiParameters) {
    ai_world
        .cmd
        .add_component(ai_parameters.ai_entity.entity, NextCommand::with_stop());
}

fn ai_action_move_random_distance(
    ai_world: &mut AiWorld,
    ai_parameters: &mut AiParameters,
    move_origin: AipMoveOrigin,
    _move_mode: AipMoveMode,
    distance: i32,
) {
    let dx = ai_world.rng.gen_range(-distance..distance);
    let dy = ai_world.rng.gen_range(-distance..distance);
    let move_origin = match move_origin {
        AipMoveOrigin::CurrentPosition => Some(ai_parameters.ai_entity.position.position),
        AipMoveOrigin::Spawn => ai_parameters
            .ai_entity
            .spawn_origin
            .map(|SpawnOrigin::MonsterSpawnPoint(_, spawn_position)| spawn_position),
        AipMoveOrigin::FindChar => ai_parameters.find_char.map(|(_, position)| position),
    };

    // TODO: Handle move_mode to do walk or run

    if let Some(move_origin) = move_origin {
        let destination = move_origin + Vector3::new(dx as f32, dy as f32, 0.0);
        ai_world.cmd.add_component(
            ai_parameters.ai_entity.entity,
            NextCommand::with_move(destination, None),
        )
    }
}

fn npc_ai_do_actions(
    ai_program_event: &AipEvent,
    ai_world: &mut AiWorld,
    ai_parameters: &mut AiParameters,
) {
    for action in ai_program_event.actions.iter() {
        match *action {
            AipAction::Stop => ai_action_stop(ai_world, ai_parameters),
            AipAction::MoveRandomDistance(move_origin, move_mode, distance) => {
                ai_action_move_random_distance(
                    ai_world,
                    ai_parameters,
                    move_origin,
                    move_mode,
                    distance,
                )
            }
            _ => {}
        }
    }
}

fn npc_ai_run_trigger(
    ai_trigger: &AipTrigger,
    cmd: &mut CommandBuffer,
    entity: &Entity,
    position: &Position,
    level: &Level,
    team: &Team,
    spawn_origin: Option<&SpawnOrigin>,
    client_entity_list: &ClientEntityList,
    nearby_query: &mut Query<(&Level, &Team)>,
    nearby_query_world: &mut SubWorld,
) {
    let mut ai_world = AiWorld {
        cmd,
        client_entity_list,
        nearby_query_world,
        nearby_query,
        rng: rand::thread_rng(),
    };
    let mut ai_parameters = AiParameters {
        ai_entity: AiSourceEntity {
            entity: *entity,
            position: position.clone(),
            level: level.clone(),
            team: team.clone(),
            spawn_origin: spawn_origin.cloned(),
        },
        attacker: None,
        find_char: None,
        near_char: None,
        damage_received: 0,
        is_dead: false,
    };

    // Do actions for only the first event with valid conditions
    for ai_program_event in ai_trigger.events.iter() {
        if npc_ai_check_conditions(ai_program_event, &mut ai_world, &mut ai_parameters) {
            npc_ai_do_actions(ai_program_event, &mut ai_world, &mut ai_parameters);
            break;
        }
    }
}

#[legion::system]
pub fn npc_ai(
    world: &mut SubWorld,
    cmd: &mut CommandBuffer,
    npc_query: &mut Query<(
        Entity,
        &Npc,
        &Command,
        &Position,
        &Level,
        &Team,
        Option<&SpawnOrigin>,
        Option<&mut NpcAi>,
    )>,
    spawn_point_query: &mut Query<&mut MonsterSpawnPoint>,
    nearby_query: &mut Query<(&Level, &Team)>,
    #[resource] client_entity_list: &ClientEntityList,
    #[resource] delta_time: &DeltaTime,
    #[resource] game_data: &GameData,
) {
    let (mut spawn_point_world, mut npc_world) = world.split_for_query(&spawn_point_query);
    let (mut nearby_query_world, mut npc_world) = npc_world.split_for_query(&nearby_query);

    npc_query.for_each_mut(
        &mut npc_world,
        |(entity, _npc, command, position, level, team, spawn_origin, npc_ai)| {
            match command.command {
                CommandData::Stop => {
                    if let Some(npc_ai) = npc_ai {
                        if let Some(ai_program) = game_data.ai.get_ai(npc_ai.ai_index) {
                            if let Some(trigger_on_idle) = ai_program.trigger_on_idle.as_ref() {
                                npc_ai.idle_duration += delta_time.delta;

                                if npc_ai.idle_duration > ai_program.idle_trigger_interval {
                                    npc_ai_run_trigger(
                                        trigger_on_idle,
                                        cmd,
                                        entity,
                                        position,
                                        level,
                                        team,
                                        spawn_origin,
                                        client_entity_list,
                                        nearby_query,
                                        &mut nearby_query_world,
                                    );
                                    npc_ai.idle_duration -= ai_program.idle_trigger_interval;
                                }
                            }
                        }
                    }
                }
                CommandData::Die => {
                    if let Some(&SpawnOrigin::MonsterSpawnPoint(spawn_point_entity, _)) =
                        spawn_origin
                    {
                        if let Ok(spawn_point) =
                            spawn_point_query.get_mut(&mut spawn_point_world, spawn_point_entity)
                        {
                            spawn_point.num_alive_monsters =
                                spawn_point.num_alive_monsters.saturating_sub(1);
                        }
                    }

                    // TODO: Call on death AI
                    // TODO: Maybe drop item
                    cmd.remove(*entity);
                }
                CommandData::Move(_) => {}
                CommandData::Attack(_) => {}
            }
        },
    );
}
