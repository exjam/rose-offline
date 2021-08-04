use std::{
    ops::{Range, RangeInclusive},
    time::Duration,
};

use bevy_ecs::prelude::{Commands, Entity, EventWriter, Query, Res, ResMut};
use nalgebra::{Point3, Vector3};
use rand::{prelude::ThreadRng, Rng};

use crate::{
    data::{
        formats::{
            AipAbilityType, AipAction, AipCondition, AipConditionFindNearbyEntities, AipDamageType,
            AipDistanceOrigin, AipEvent, AipMoveMode, AipMoveOrigin, AipOperatorType, AipTrigger,
        },
        Damage,
    },
    game::{
        bundles::client_entity_leave_zone,
        components::{
            AbilityValues, ClientEntity, ClientEntityType, Command, CommandData, CommandDie,
            DamageSources, ExpireTime, GameClient, HealthPoints, Level, MonsterSpawnPoint,
            MoveMode, NextCommand, Npc, NpcAi, Owner, Position, SpawnOrigin, Team,
        },
        events::RewardXpEvent,
        messages::server::ServerMessage,
        resources::{ClientEntityList, ServerTime, WorldRates},
        GameData,
    },
};

const DAMAGE_REWARD_EXPIRE_TIME: Duration = Duration::from_secs(5 * 60);
const DROPPED_ITEM_EXPIRE_TIME: Duration = Duration::from_secs(60);
const DROP_ITEM_RADIUS: i32 = 200;

struct AiSourceEntity<'a> {
    entity: Entity,
    position: &'a Position,
    level: &'a Level,
    team: &'a Team,
    ability_values: &'a AbilityValues,
    health_points: &'a HealthPoints,
    target: Option<&'a Entity>,
    owner: Option<&'a Owner>,
    spawn_origin: Option<&'a SpawnOrigin>,
}

struct AiAttackerEntity<'a> {
    entity: &'a Entity,
    _position: &'a Position,
    level: &'a Level,
    _team: &'a Team,
    ability_values: &'a AbilityValues,
    health_points: &'a HealthPoints,
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

#[allow(dead_code)]
struct AiParameters<'a, 'b> {
    source: &'a AiSourceEntity<'b>,
    attacker: Option<&'a AiAttackerEntity<'b>>,
    find_char: Option<(Entity, Point3<f32>)>,
    near_char: Option<(Entity, Point3<f32>)>,
    damage_received: Option<Damage>,
    is_dead: bool,
}

struct AiWorld<'world, 'world1, 'world2, 'world3, 'a, 'b, 'c> {
    commands: &'a mut Commands<'world>,
    client_entity_list: &'b ClientEntityList,
    nearby_query: &'c Query<'world1, (&'world2 Level, &'world3 Team)>,
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
    count_operator_type: Option<AipOperatorType>,
    count: i32,
) -> Result<(), AiConditionResult> {
    let mut find_char = None;
    let mut near_char_distance = None;
    let mut find_count = 0;

    let zone_entities = ai_world
        .client_entity_list
        .get_zone(ai_parameters.source.position.zone_id)
        .ok_or(AiConditionResult::Failed)?;

    for (entity, position) in zone_entities
        .iter_entities_within_distance(ai_parameters.source.position.position.xy(), distance as f32)
    {
        // Ignore self entity
        if entity == ai_parameters.source.entity {
            continue;
        }

        // Check level and team requirements
        if !ai_world
            .nearby_query
            .get(entity)
            .map_or(false, |(level, team)| {
                let level_diff = ai_parameters.source.level.level as i32 - level.level as i32;

                is_allied == (team.id == ai_parameters.source.team.id)
                    && level_diff_range.contains(&level_diff)
            })
        {
            continue;
        }

        // Update near char for nearest found character
        let distance_squared =
            (ai_parameters.source.position.position - position).magnitude_squared();
        if near_char_distance.map_or(true, |x| distance_squared < x) {
            ai_parameters.near_char = Some((entity, position));
            near_char_distance = Some(distance_squared);
        }

        // Continue until we have satisfy count
        find_count += 1;
        if count_operator_type.is_none() && find_count >= count {
            find_char = Some((entity, position));
            break;
        }
    }

    if let Some(operator) = count_operator_type {
        if compare_aip_value(operator, find_count, count) {
            find_char = ai_parameters.near_char;
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
        AipDamageType::Received => compare_aip_value(
            operator,
            ai_parameters
                .damage_received
                .map_or(0, |damage| damage.amount as i32),
            value,
        ),
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
        AipDistanceOrigin::Spawn => match ai_parameters.source.spawn_origin {
            Some(SpawnOrigin::MonsterSpawnPoint(_, spawn_position)) => Some(
                (ai_parameters.source.position.position.xy() - spawn_position.xy())
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

fn ai_condition_health_percent(
    _ai_world: &mut AiWorld,
    ai_parameters: &mut AiParameters,
    operator: AipOperatorType,
    value: i32,
) -> bool {
    let current = ai_parameters.source.health_points.hp as i32;
    let max = ai_parameters.source.ability_values.get_max_health();

    compare_aip_value(operator, (100 * current) / max, value)
}

fn ai_condition_has_owner(_ai_world: &mut AiWorld, ai_parameters: &mut AiParameters) -> bool {
    ai_parameters.source.owner.is_some()
}

fn ai_condition_is_attacker_current_target(
    _ai_world: &mut AiWorld,
    ai_parameters: &mut AiParameters,
) -> bool {
    if let Some(attacker) = ai_parameters.attacker {
        if let Some(target) = ai_parameters.source.target {
            return attacker.entity == target;
        }
    }

    false
}

fn ai_condition_no_target_and_compare_attacker_ability_value(
    _ai_world: &mut AiWorld,
    ai_parameters: &mut AiParameters,
    operator: AipOperatorType,
    ability: AipAbilityType,
    value: i32,
) -> bool {
    if ai_parameters.source.target.is_some() {
        return false;
    }

    if let Some(attacker) = ai_parameters.attacker {
        let ability_value = match ability {
            AipAbilityType::Level => attacker.level.level as i32,
            AipAbilityType::Attack => attacker.ability_values.get_attack_power(),
            AipAbilityType::Defence => attacker.ability_values.get_defence(),
            AipAbilityType::Resistance => attacker.ability_values.get_resistance(),
            AipAbilityType::HealthPoints => attacker.health_points.hp as i32,
            AipAbilityType::Charm => attacker.ability_values.get_charm(),
        };

        compare_aip_value(operator, ability_value, value)
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

fn ai_condition_source_ability_value(
    _ai_world: &mut AiWorld,
    ai_parameters: &mut AiParameters,
    operator: AipOperatorType,
    ability: AipAbilityType,
    value: i32,
) -> bool {
    let ability_value = match ability {
        AipAbilityType::Level => ai_parameters.source.level.level as i32,
        AipAbilityType::Attack => ai_parameters.source.ability_values.get_attack_power(),
        AipAbilityType::Defence => ai_parameters.source.ability_values.get_defence(),
        AipAbilityType::Resistance => ai_parameters.source.ability_values.get_resistance(),
        AipAbilityType::HealthPoints => ai_parameters.source.health_points.hp as i32,
        AipAbilityType::Charm => ai_parameters.source.ability_values.get_charm(),
    };

    compare_aip_value(operator, ability_value, value)
}

fn npc_ai_check_conditions(
    ai_program_event: &AipEvent,
    ai_world: &mut AiWorld,
    ai_parameters: &mut AiParameters,
) -> bool {
    for condition in ai_program_event.conditions.iter() {
        let result = match condition {
            AipCondition::CompareAttackerAndTargetAbilityValue(_, _) => false,
            AipCondition::FindNearbyEntities(AipConditionFindNearbyEntities {
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
            AipCondition::HasOwner => ai_condition_has_owner(ai_world, ai_parameters),
            AipCondition::HasStatusEffect(_, _, _) => false,
            &AipCondition::HealthPercent(operator, value) => {
                ai_condition_health_percent(ai_world, ai_parameters, operator, value)
            }
            AipCondition::IsAttackerClanMaster => false,
            AipCondition::IsAttackerCurrentTarget => {
                ai_condition_is_attacker_current_target(ai_world, ai_parameters)
            }
            AipCondition::IsDaytime(_) => false,
            AipCondition::IsTargetClanMaster => false,
            AipCondition::MonthDay(_) => false,
            &AipCondition::NoTargetAndCompareAttackerAbilityValue(operator, ability, value) => {
                ai_condition_no_target_and_compare_attacker_ability_value(
                    ai_world,
                    ai_parameters,
                    operator,
                    ability,
                    value,
                )
            }
            AipCondition::OwnerHasTarget => false,
            AipCondition::Random(operator, range, value) => {
                ai_condition_random(ai_world, ai_parameters, *operator, range.clone(), *value)
            }
            AipCondition::SelectLocalNpc(_) => false,
            &AipCondition::SelfAbilityValue(operator, ability, value) => {
                ai_condition_source_ability_value(ai_world, ai_parameters, operator, ability, value)
            }
            AipCondition::ServerChannelNumber(_) => false,
            AipCondition::TargetAbilityValue(_, _, _) => false,
            AipCondition::Variable(_, _, _, _) => false,
            AipCondition::WeekDay(_) => false,
            AipCondition::WorldTime(_) => false,
            AipCondition::ZoneTime(_) => false,
        };

        if !result {
            return false;
        }
    }

    true
}

fn ai_action_stop(ai_world: &mut AiWorld, ai_parameters: &mut AiParameters) {
    ai_world
        .commands
        .entity(ai_parameters.source.entity)
        .insert(NextCommand::with_stop());
}

fn ai_action_move_random_distance(
    ai_world: &mut AiWorld,
    ai_parameters: &mut AiParameters,
    move_origin: AipMoveOrigin,
    move_mode: AipMoveMode,
    distance: i32,
) {
    let dx = ai_world.rng.gen_range(-distance..distance);
    let dy = ai_world.rng.gen_range(-distance..distance);
    let move_origin = match move_origin {
        AipMoveOrigin::CurrentPosition => Some(ai_parameters.source.position.position),
        AipMoveOrigin::Spawn => ai_parameters
            .source
            .spawn_origin
            .map(|SpawnOrigin::MonsterSpawnPoint(_, spawn_position)| *spawn_position),
        AipMoveOrigin::FindChar => ai_parameters.find_char.map(|(_, position)| position),
    };

    if let Some(move_origin) = move_origin {
        let move_mode = match move_mode {
            AipMoveMode::Run => MoveMode::Run,
            AipMoveMode::Walk => MoveMode::Walk,
        };
        let destination = move_origin + Vector3::new(dx as f32, dy as f32, 0.0);
        ai_world
            .commands
            .entity(ai_parameters.source.entity)
            .insert(NextCommand::with_move(destination, None, Some(move_mode)));
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
            AipAction::Emote(_) => {}
            AipAction::Say(_) => {}
            AipAction::AttackNearbyEntityByStat(_, _, _) => {}
            AipAction::SpecialAttack => {}
            AipAction::MoveDistanceFromTarget(_, _) => {}
            AipAction::TransformNpc(_) => {}
            AipAction::SpawnNpc(_, _, _, _) => {}
            AipAction::NearbyAlliesAttackTarget(_, _, _) => {}
            AipAction::AttackNearChar => {
                if let Some((near_char, _)) = &ai_parameters.near_char {
                    ai_world
                        .commands
                        .entity(ai_parameters.source.entity)
                        .insert(NextCommand::with_attack(*near_char));
                }
            }
            AipAction::AttackFindChar => {
                if let Some((find_char, _)) = &ai_parameters.find_char {
                    ai_world
                        .commands
                        .entity(ai_parameters.source.entity)
                        .insert(NextCommand::with_attack(*find_char));
                }
            }
            AipAction::NearbyAlliesSameNpcAttackTarget(_) => {}
            AipAction::AttackAttacker => {
                if let Some(attacker) = ai_parameters.attacker {
                    ai_world
                        .commands
                        .entity(ai_parameters.source.entity)
                        .insert(NextCommand::with_attack(*attacker.entity));
                }
            }
            AipAction::RunAway(_) => {}
            AipAction::DropRandomItem(_) => {}
            AipAction::KillSelf => {
                ai_world
                    .commands
                    .entity(ai_parameters.source.entity)
                    .insert(HealthPoints::new(0))
                    .insert(Command::with_die(None, None, None));
            }
            AipAction::UseSkill(_, _, _) => {}
            AipAction::SetVariable(_, _, _, _) => {}
            AipAction::Message(_, _) => {}
            AipAction::MoveNearOwner => {}
            AipAction::DoQuestTrigger(_) => {}
            AipAction::AttackOwnerTarget => {}
            AipAction::SetPvpFlag(_, _) => {}
            AipAction::SetMonsterSpawnState(_, _) => {}
            AipAction::GiveItemToOwner(_, _) => {}
        }
    }
}

fn npc_ai_run_trigger(
    ai_trigger: &AipTrigger,
    commands: &mut Commands,
    source: &AiSourceEntity,
    attacker: Option<AiAttackerEntity>,
    damage: Option<Damage>,
    is_dead: bool,
    client_entity_list: &ClientEntityList,
    nearby_query: &Query<(&Level, &Team)>,
) {
    let mut ai_world = AiWorld {
        commands,
        client_entity_list,
        nearby_query,
        rng: rand::thread_rng(),
    };
    let mut ai_parameters = AiParameters {
        source,
        attacker: attacker.as_ref(),
        find_char: None,
        near_char: None,
        damage_received: damage,
        is_dead,
    };

    // Do actions for only the first event with valid conditions
    for ai_program_event in ai_trigger.events.iter() {
        if npc_ai_check_conditions(ai_program_event, &mut ai_world, &mut ai_parameters) {
            npc_ai_do_actions(ai_program_event, &mut ai_world, &mut ai_parameters);
            break;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn npc_ai_system(
    mut commands: Commands,
    mut npc_query: Query<(
        Entity,
        &Npc,
        &mut NpcAi,
        &ClientEntity,
        &Command,
        &Position,
        &Level,
        &Team,
        &HealthPoints,
        &AbilityValues,
        Option<&Owner>,
        Option<&SpawnOrigin>,
        Option<&DamageSources>,
    )>,
    mut spawn_point_query: Query<&mut MonsterSpawnPoint>,
    nearby_query: Query<(&Level, &Team)>,
    attacker_query: Query<(&Position, &Level, &Team, &AbilityValues, &HealthPoints)>,
    level_query: Query<&Level>,
    killer_query: Query<(&Level, &AbilityValues, Option<&GameClient>)>,
    server_time: Res<ServerTime>,
    game_data: Res<GameData>,
    world_rates: Res<WorldRates>,
    mut client_entity_list: ResMut<ClientEntityList>,
    mut reward_xp_events: EventWriter<RewardXpEvent>,
) {
    npc_query.for_each_mut(
        |(
            entity,
            npc,
            mut npc_ai,
            client_entity,
            command,
            position,
            level,
            team,
            health_points,
            ability_values,
            owner,
            spawn_origin,
            damage_sources,
        )| {
            if !npc_ai.has_run_created_trigger {
                if let Some(ai_program) = game_data.ai.get_ai(npc_ai.ai_index) {
                    if let Some(trigger_on_created) = ai_program.trigger_on_created.as_ref() {
                        npc_ai_run_trigger(
                            trigger_on_created,
                            &mut commands,
                            &AiSourceEntity {
                                entity,
                                position,
                                level,
                                ability_values,
                                target: None,
                                team,
                                health_points,
                                owner,
                                spawn_origin,
                            },
                            None,
                            None,
                            false,
                            &client_entity_list,
                            &nearby_query,
                        );
                    }
                }

                (*npc_ai).has_run_created_trigger = true;
            }

            if let Some(ai_program) = game_data.ai.get_ai(npc_ai.ai_index) {
                if let Some(trigger_on_damaged) = ai_program.trigger_on_damaged.as_ref() {
                    let mut rng = rand::thread_rng();
                    for (attacker, damage) in npc_ai.pending_damage.iter() {
                        if command.get_target().is_some()
                            && ai_program.damage_trigger_new_target_chance < rng.gen_range(0..100)
                        {
                            continue;
                        }

                        if let Ok((
                            attacker_position,
                            attacker_level,
                            attacker_team,
                            attacker_ability_values,
                            attacker_health_points,
                        )) = attacker_query.get(*attacker)
                        {
                            npc_ai_run_trigger(
                                trigger_on_damaged,
                                &mut commands,
                                &AiSourceEntity {
                                    entity,
                                    position,
                                    level,
                                    ability_values,
                                    target: None,
                                    team,
                                    health_points,
                                    owner,
                                    spawn_origin,
                                },
                                Some(AiAttackerEntity {
                                    entity: attacker,
                                    _position: attacker_position,
                                    level: attacker_level,
                                    _team: attacker_team,
                                    ability_values: attacker_ability_values,
                                    health_points: attacker_health_points,
                                }),
                                Some(*damage),
                                false,
                                &client_entity_list,
                                &nearby_query,
                            );
                        }
                    }
                }
            }
            npc_ai.pending_damage.clear();

            match command.command {
                CommandData::Stop => {
                    if let Some(ai_program) = game_data.ai.get_ai(npc_ai.ai_index) {
                        if let Some(trigger_on_idle) = ai_program.trigger_on_idle.as_ref() {
                            npc_ai.idle_duration += server_time.delta;

                            if npc_ai.idle_duration > ai_program.idle_trigger_interval {
                                npc_ai_run_trigger(
                                    trigger_on_idle,
                                    &mut commands,
                                    &AiSourceEntity {
                                        entity,
                                        position,
                                        level,
                                        ability_values,
                                        target: None,
                                        team,
                                        health_points,
                                        owner,
                                        spawn_origin,
                                    },
                                    None,
                                    None,
                                    false,
                                    &client_entity_list,
                                    &nearby_query,
                                );
                                npc_ai.idle_duration -= ai_program.idle_trigger_interval;
                            }
                        }
                    }
                }
                CommandData::Die(CommandDie {
                    killer: killer_entity,
                }) => {
                    if !npc_ai.has_run_dead_ai {
                        npc_ai.has_run_dead_ai = true;

                        if let Some(&SpawnOrigin::MonsterSpawnPoint(spawn_point_entity, _)) =
                            spawn_origin
                        {
                            if let Ok(mut spawn_point) =
                                spawn_point_query.get_mut(spawn_point_entity)
                            {
                                let mut spawn_point = &mut *spawn_point;
                                spawn_point.num_alive_monsters =
                                    spawn_point.num_alive_monsters.saturating_sub(1);
                            }
                        }

                        if let Some(trigger_on_dead) = game_data
                            .ai
                            .get_ai(npc_ai.ai_index)
                            .and_then(|ai_program| ai_program.trigger_on_dead.as_ref())
                        {
                            npc_ai_run_trigger(
                                trigger_on_dead,
                                &mut commands,
                                &AiSourceEntity {
                                    entity,
                                    position,
                                    level,
                                    ability_values,
                                    target: None,
                                    health_points,
                                    team,
                                    owner,
                                    spawn_origin,
                                },
                                None, // TODO: Pass in killer entity
                                None, // TODO: Pass in killer damage
                                true,
                                &client_entity_list,
                                &nearby_query,
                            );
                        }

                        if let Some(damage_sources) = damage_sources {
                            if let Some(npc_data) = game_data.npcs.get_npc(npc.id) {
                                // Reward XP to all attackers
                                for damage_source in damage_sources.damage_sources.iter() {
                                    let time_since_damage =
                                        server_time.now - damage_source.last_damage_time;
                                    if time_since_damage > DAMAGE_REWARD_EXPIRE_TIME {
                                        // Damage expired, ignore.
                                        continue;
                                    }

                                    if let Ok(damage_source_level) =
                                        level_query.get(damage_source.entity)
                                    {
                                        let reward_xp =
                                            game_data.ability_value_calculator.calculate_give_xp(
                                                damage_source_level.level as i32,
                                                damage_source.total_damage as i32,
                                                level.level as i32,
                                                ability_values.get_max_health(),
                                                npc_data.reward_xp as i32,
                                                world_rates.xp_rate,
                                            );
                                        if reward_xp > 0 {
                                            let stamina = game_data
                                                .ability_value_calculator
                                                .calculate_give_stamina(
                                                    reward_xp,
                                                    level.level as i32,
                                                    world_rates.xp_rate,
                                                );

                                            reward_xp_events.send(RewardXpEvent::new(
                                                damage_source.entity,
                                                reward_xp as u64,
                                                stamina as u32,
                                                Some(entity),
                                            ));
                                        }
                                    }
                                }

                                if let Some(killer_entity) = killer_entity {
                                    if let Ok((
                                        killer_level,
                                        killer_ability_values,
                                        killer_game_client,
                                    )) = killer_query.get(killer_entity)
                                    {
                                        // Inform client to execute npc dead event
                                        if !npc_data.death_quest_trigger_name.is_empty() {
                                            if let Some(killer_game_client) = killer_game_client {
                                                // TODO: Send NPC death trigger to whole party
                                                /*
                                                if npc_data.npc_quest_type != 0 {
                                                }
                                                */

                                                // Send to only client
                                                killer_game_client
                                                    .server_message_tx
                                                    .send(ServerMessage::RunNpcDeathTrigger(npc.id))
                                                    .ok();
                                            }
                                        }

                                        // Drop item owned by killer
                                        let level_difference =
                                            killer_level.level as i32 - level.level as i32;
                                        if let Some(drop_item) = game_data.drop_table.get_drop(
                                            world_rates.drop_rate,
                                            world_rates.drop_money_rate,
                                            npc.id,
                                            position.zone_id,
                                            level_difference,
                                            killer_ability_values.get_drop_rate(),
                                            killer_ability_values.get_charm(),
                                        ) {
                                            let mut rng = rand::thread_rng();
                                            let client_entity_zone = client_entity_list
                                                .get_zone_mut(position.zone_id)
                                                .unwrap();
                                            let drop_position = Point3::new(
                                                position.position.x
                                                    + rng.gen_range(
                                                        -DROP_ITEM_RADIUS..=DROP_ITEM_RADIUS,
                                                    )
                                                        as f32,
                                                position.position.y
                                                    + rng.gen_range(
                                                        -DROP_ITEM_RADIUS..=DROP_ITEM_RADIUS,
                                                    )
                                                        as f32,
                                                position.position.z,
                                            );
                                            let mut drop_entity_commands = commands.spawn();

                                            let drop_entity = drop_entity_commands.id();
                                            drop_entity_commands
                                                .insert(Some(drop_item))
                                                .insert(Position::new(
                                                    drop_position,
                                                    position.zone_id,
                                                ))
                                                .insert(Owner::new(killer_entity))
                                                .insert(ExpireTime::new(
                                                    server_time.now + DROPPED_ITEM_EXPIRE_TIME,
                                                ))
                                                .insert(
                                                    client_entity_zone
                                                        .join_zone(
                                                            ClientEntityType::DroppedItem,
                                                            drop_entity,
                                                            drop_position,
                                                        )
                                                        .unwrap(),
                                                );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Once the death animation has completed, we can remove this entity
                    let command_complete = command
                        .required_duration
                        .map_or(false, |required_duration| {
                            command.duration >= required_duration
                        });
                    if command_complete {
                        client_entity_leave_zone(
                            &mut commands,
                            &mut client_entity_list,
                            entity,
                            client_entity,
                            position,
                        );
                        commands.entity(entity).despawn();
                    }
                }
                CommandData::Move(_) => {}
                CommandData::Attack(_) => {}
                CommandData::CastSkill(_) => {}
                CommandData::PickupDroppedItem(_) => {}
                CommandData::PersonalStore => {}
            }
        },
    );
}
