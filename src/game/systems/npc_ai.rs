use legion::{systems::CommandBuffer, world::SubWorld, Entity, Query};
use nalgebra::{Point3, Vector3};
use rand::Rng;

use crate::{
    data::formats::{
        AipAction, AipCondition, AipEvent, AipMoveOrigin, AipOperatorType, AipTrigger,
    },
    game::{
        self,
        components::{
            Command, CommandData, MonsterSpawnPoint, NextCommand, Npc, NpcAi, Position, SpawnOrigin,
        },
        resources::DeltaTime,
        GameData,
    },
};

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

fn npc_ai_check_conditions(ai_program_event: &AipEvent) -> bool {
    for condition in ai_program_event.conditions.iter() {
        let result = match condition {
            AipCondition::Random(operator, range, value) => compare_aip_value(
                *operator,
                rand::thread_rng().gen_range(range.clone()),
                *value,
            ),
            _ => false,
        };

        if !result {
            return false;
        }
    }

    true
}

fn npc_ai_do_actions(
    ai_program_event: &AipEvent,
    cmd: &mut CommandBuffer,
    entity: &Entity,
    position: &Position,
    spawn_origin: Option<&SpawnOrigin>,
) {
    for action in ai_program_event.actions.iter() {
        match *action {
            AipAction::Stop => cmd.add_component(*entity, NextCommand::with_stop()),
            AipAction::MoveRandomDistance(move_origin, _move_mode, distance) => {
                let dx = rand::thread_rng().gen_range(-distance..distance);
                let dy = rand::thread_rng().gen_range(-distance..distance);
                let move_origin = match move_origin {
                    AipMoveOrigin::CurrentPosition => position.position,
                    AipMoveOrigin::Spawn => match spawn_origin {
                        Some(&SpawnOrigin::MonsterSpawnPoint(_, spawn_position)) => spawn_position,
                        None => position.position,
                    },
                };
                let destination = move_origin + Vector3::new(dx as f32, dy as f32, 0.0);
                cmd.add_component(*entity, NextCommand::with_move(destination, None))
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
    spawn_origin: Option<&SpawnOrigin>,
) {
    // Do actions for only the first event with valid conditions
    for ai_program_event in ai_trigger.events.iter() {
        if npc_ai_check_conditions(ai_program_event) {
            npc_ai_do_actions(ai_program_event, cmd, entity, position, spawn_origin);
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
        Option<&SpawnOrigin>,
        Option<&mut NpcAi>,
    )>,
    spawn_point_query: &mut Query<&mut MonsterSpawnPoint>,
    #[resource] delta_time: &DeltaTime,
    #[resource] game_data: &GameData,
) {
    let mut spawn_point_deaths: Vec<(Entity, u32)> = Vec::new();

    npc_query.for_each_mut(
        world,
        |(entity, _npc, command, position, spawn_origin, npc_ai)| {
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
                                        spawn_origin,
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
                CommandData::Move(_) => {}
                CommandData::Attack(_) => {}
            }
        },
    );

    for (entity, deaths) in spawn_point_deaths {
        if let Ok(spawn_point) = spawn_point_query.get_mut(world, entity) {
            spawn_point.num_alive_monsters = spawn_point.num_alive_monsters.saturating_sub(deaths);
        }
    }
}
