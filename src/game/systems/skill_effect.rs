use std::marker::PhantomData;

use bevy_ecs::{
    prelude::{Commands, Entity, EventReader, EventWriter, Local, Mut, Query, Res, ResMut},
    system::SystemParam,
};
use log::warn;
use rand::Rng;

use crate::{
    data::{
        AbilityType, Damage, SkillData, SkillTargetFilter, SkillType, StatusEffectClearedByType,
        StatusEffectType,
    },
    game::{
        bundles::{ability_values_get_value, MonsterBundle},
        components::{
            AbilityValues, BasicStats, CharacterInfo, ClientEntity, ClientEntityType, Equipment,
            GameClient, HealthPoints, Inventory, Level, MoveSpeed, Npc, Position, SkillList,
            SpawnOrigin, StatusEffects, Team,
        },
        events::{DamageEvent, SkillEvent, SkillEventTarget},
        messages::server::{
            ApplySkillEffect, CancelCastingSkillReason, ServerMessage, UseInventoryItem,
        },
        resources::{ClientEntityList, ServerMessages, ServerTime},
        GameData,
    },
};

#[allow(dead_code)]
enum SkillCastError {
    InvalidSkill,
    InvalidTarget,
    NotEnoughUseAbility,
}

#[derive(SystemParam)]
pub struct SkillSystemParameters<'w, 's> {
    server_messages: ResMut<'w, ServerMessages>,
    damage_events: EventWriter<'w, 's, DamageEvent>,
}

#[derive(SystemParam)]
pub struct SkillSystemResources<'w, 's> {
    game_data: Res<'w, GameData>,
    server_time: Res<'w, ServerTime>,

    #[system_param(ignore)]
    _secret: PhantomData<&'s ()>,
}

struct SkillCaster<'a> {
    entity: Entity,
    client_entity: &'a ClientEntity,
    position: &'a Position,
    ability_values: &'a AbilityValues,
    level: &'a Level,
    team: &'a Team,
}

#[allow(dead_code)]
struct SkillTargetData<'a> {
    entity: Entity,
    client_entity: &'a ClientEntity,
    position: &'a Position,
    ability_values: &'a AbilityValues,
    status_effects: Mut<'a, StatusEffects>,
    team: &'a Team,
    health_points: &'a HealthPoints,
    level: &'a Level,
    move_speed: &'a MoveSpeed,

    // To update character ability_values
    character_info: Option<&'a CharacterInfo>,
    equipment: Option<&'a Equipment>,
    basic_stats: Option<&'a BasicStats>,
    skill_list: Option<&'a SkillList>,

    // To update NPC ability_values
    npc: Option<&'a Npc>,
}

#[derive(SystemParam)]
pub struct SkillTargetQuery<'w, 's> {
    query: Query<
        'w,
        's,
        (
            &'static ClientEntity,
            &'static Position,
            &'static AbilityValues,
            &'static mut StatusEffects,
            &'static Team,
            &'static HealthPoints,
            &'static Level,
            &'static MoveSpeed,
            Option<&'static CharacterInfo>,
            Option<&'static Equipment>,
            Option<&'static BasicStats>,
            Option<&'static SkillList>,
            Option<&'static Npc>,
        ),
    >,
}

impl<'w, 's> SkillTargetQuery<'w, 's> {
    fn get_skill_target_data(&mut self, entity: Entity) -> Option<SkillTargetData> {
        let (
            client_entity,
            position,
            ability_values,
            status_effects,
            team,
            health_points,
            level,
            move_speed,
            character_info,
            equipment,
            basic_stats,
            skill_list,
            npc,
        ) = self.query.get_mut(entity).ok()?;

        Some(SkillTargetData {
            entity,
            client_entity,
            position,
            ability_values,
            status_effects,
            team,
            health_points,
            level,
            move_speed,
            character_info,
            equipment,
            basic_stats,
            skill_list,
            npc,
        })
    }
}

fn check_skill_target_filter(
    skill_caster: &SkillCaster,
    skill_target: &mut SkillTargetData,
    skill_data: &SkillData,
) -> bool {
    match skill_data.target_filter {
        SkillTargetFilter::OnlySelf => skill_caster.entity == skill_target.entity,
        SkillTargetFilter::Group => true, // TODO: Implement SkillTargetFilter::Group
        SkillTargetFilter::Guild => true, // TODO: Implement SkillTargetFilter::Guild
        SkillTargetFilter::Allied => skill_caster.team.id == skill_target.team.id,
        SkillTargetFilter::Monster => matches!(
            skill_target.client_entity.entity_type,
            ClientEntityType::Monster
        ),
        SkillTargetFilter::Enemy => skill_caster.team.id != skill_target.team.id,
        SkillTargetFilter::EnemyCharacter => {
            skill_caster.team.id != skill_target.team.id
                && matches!(
                    skill_target.client_entity.entity_type,
                    ClientEntityType::Character
                )
        }
        SkillTargetFilter::Character => matches!(
            skill_target.client_entity.entity_type,
            ClientEntityType::Character
        ),
        SkillTargetFilter::CharacterOrMonster => matches!(
            skill_target.client_entity.entity_type,
            ClientEntityType::Character | ClientEntityType::Monster
        ),
        SkillTargetFilter::DeadAlliedCharacter => {
            skill_caster.team.id == skill_target.team.id
                && skill_target.health_points.hp == 0
                && matches!(
                    skill_target.client_entity.entity_type,
                    ClientEntityType::Character
                )
        }
        SkillTargetFilter::EnemyMonster => {
            skill_caster.team.id != skill_target.team.id
                && matches!(
                    skill_target.client_entity.entity_type,
                    ClientEntityType::Monster
                )
        }
    }
}

fn apply_skill_status_effects_to_entity(
    skill_system_parameters: &mut SkillSystemParameters,
    skill_system_resources: &SkillSystemResources,
    skill_caster: &SkillCaster,
    skill_target: &mut SkillTargetData,
    skill_data: &SkillData,
) -> Result<(), SkillCastError> {
    if !check_skill_target_filter(skill_caster, skill_target, skill_data) {
        return Err(SkillCastError::InvalidTarget);
    }

    if skill_data.harm != 0 {
        skill_system_parameters
            .damage_events
            .send(DamageEvent::with_tagged(
                skill_caster.entity,
                skill_target.entity,
            ));
    }

    for add_ability in skill_data.add_ability.iter() {
        match add_ability.ability_type {
            AbilityType::Stamina | AbilityType::Money | AbilityType::Health | AbilityType::Mana => {
                warn!(
                    "Unimplemented skill status effect add ability_type {:?}, value {}",
                    add_ability.ability_type, add_ability.value
                )
            }
            _ => {}
        }
    }

    let mut effect_success = [false, false];
    for (effect_index, status_effect_id) in skill_data
        .status_effects
        .iter()
        .enumerate()
        .filter_map(|(index, id)| id.map(|id| (index, id)))
    {
        if let Some(status_effect_data) = skill_system_resources
            .game_data
            .status_effects
            .get_status_effect(status_effect_id)
        {
            if skill_data.success_ratio > 0 {
                match status_effect_data.cleared_by_type {
                    StatusEffectClearedByType::ClearGood => {
                        if skill_data.success_ratio
                            < skill_target.level.level as i32 - skill_caster.level.level as i32
                                + rand::thread_rng().gen_range(1..=100)
                        {
                            continue;
                        }
                    }
                    _ => {
                        if skill_data.success_ratio as f32
                            * (skill_caster.level.level as i32 * 2
                                + skill_caster.ability_values.get_intelligence()
                                + 20) as f32
                            / (skill_target.ability_values.get_resistance() as f32 * 0.6
                                + 5.0
                                + skill_target.ability_values.get_avoid() as f32)
                            <= rand::thread_rng().gen_range(1..=100) as f32
                        {
                            continue;
                        }
                    }
                }
            }

            let adjust_value = if matches!(
                status_effect_data.status_effect_type,
                StatusEffectType::AdditionalDamageRate
            ) {
                skill_data.power as i32
            } else if let Some(skill_add_ability) = skill_data.add_ability.get(effect_index) {
                let ability_value = ability_values_get_value(
                    skill_add_ability.ability_type,
                    skill_target.ability_values,
                    skill_target.level,
                    skill_target.move_speed,
                    skill_target.team,
                    skill_target.character_info,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap_or(0);
                skill_system_resources
                    .game_data
                    .ability_value_calculator
                    .calculate_skill_adjust_value(
                        skill_add_ability,
                        skill_caster.ability_values.get_intelligence(),
                        ability_value,
                    )
            } else {
                0
            };

            if skill_target
                .status_effects
                .can_apply(status_effect_data, adjust_value)
            {
                skill_target.status_effects.apply_status_effect(
                    status_effect_data,
                    skill_system_resources.server_time.now + skill_data.status_effect_duration,
                    adjust_value,
                );

                match status_effect_data.status_effect_type {
                    StatusEffectType::Fainting | StatusEffectType::Sleep => {
                        // TODO: Set current + next command to stop
                    }
                    StatusEffectType::Taunt => {
                        // TODO: Set current + next command to attack spell cast entity
                    }
                    _ => {}
                }

                effect_success[effect_index] = true;
            }
        }
    }

    if effect_success.iter().any(|x| *x) {
        skill_system_parameters.server_messages.send_entity_message(
            skill_target.client_entity,
            ServerMessage::ApplySkillEffect(ApplySkillEffect {
                entity_id: skill_target.client_entity.id,
                caster_entity_id: skill_caster.client_entity.id,
                caster_intelligence: skill_caster.ability_values.get_intelligence(),
                skill_id: skill_data.id,
                effect_success,
            }),
        );
    }

    Ok(())
}

fn apply_skill_status_effects(
    skill_system_parameters: &mut SkillSystemParameters,
    skill_system_resources: &SkillSystemResources,
    client_entity_list: &ClientEntityList,
    skill_caster: &SkillCaster,
    skill_target: &SkillEventTarget,
    skill_data: &SkillData,
    skill_target_query: &mut SkillTargetQuery,
) -> Result<(), SkillCastError> {
    if skill_data.scope > 0 {
        // Apply in AOE around target position
        let client_entity_zone = client_entity_list
            .get_zone(skill_caster.position.zone_id)
            .ok_or(SkillCastError::InvalidTarget)?;

        let skill_position = match *skill_target {
            SkillEventTarget::Entity(target_entity) => {
                if let Some(skill_target) = skill_target_query.get_skill_target_data(target_entity)
                {
                    Some(skill_target.position.position.xy())
                } else {
                    None
                }
            }
            SkillEventTarget::Position(position) => Some(position),
        }
        .ok_or(SkillCastError::InvalidTarget)?;

        for (target_entity, _) in client_entity_zone
            .iter_entities_within_distance(skill_position, skill_data.scope as f32)
        {
            if let Some(mut skill_target) = skill_target_query.get_skill_target_data(target_entity)
            {
                apply_skill_status_effects_to_entity(
                    skill_system_parameters,
                    skill_system_resources,
                    skill_caster,
                    &mut skill_target,
                    skill_data,
                )
                .ok();
            }
        }

        Ok(())
    } else if let SkillEventTarget::Entity(target_entity) = *skill_target {
        if let Some(mut skill_target) = skill_target_query.get_skill_target_data(target_entity) {
            apply_skill_status_effects_to_entity(
                skill_system_parameters,
                skill_system_resources,
                skill_caster,
                &mut skill_target,
                skill_data,
            )
            .ok();
            Ok(())
        } else {
            Err(SkillCastError::InvalidTarget)
        }
    } else {
        Err(SkillCastError::InvalidTarget)
    }
}

fn apply_skill_damage_to_entity(
    skill_system_parameters: &mut SkillSystemParameters,
    skill_system_resources: &SkillSystemResources,
    skill_caster: &SkillCaster,
    skill_target: &mut SkillTargetData,
    skill_data: &SkillData,
) -> Result<Damage, SkillCastError> {
    if !check_skill_target_filter(skill_caster, skill_target, skill_data) {
        return Err(SkillCastError::InvalidTarget);
    }

    // TODO: Get hit count from skill action motion
    let damage = skill_system_resources
        .game_data
        .ability_value_calculator
        .calculate_skill_damage(
            skill_caster.ability_values,
            skill_target.ability_values,
            skill_data,
            1,
        );

    if matches!(skill_data.skill_type, SkillType::FireBullet) {
        skill_system_parameters
            .damage_events
            .send(DamageEvent::with_attack(
                skill_caster.entity,
                skill_target.entity,
                damage,
            ));
    } else {
        skill_system_parameters
            .damage_events
            .send(DamageEvent::with_skill(
                skill_caster.entity,
                skill_target.entity,
                damage,
                skill_data.id,
                skill_caster.ability_values.get_intelligence(),
            ));
    }

    Ok(damage)
}

fn apply_skill_damage(
    skill_system_parameters: &mut SkillSystemParameters,
    skill_system_resources: &SkillSystemResources,
    client_entity_list: &ClientEntityList,
    skill_caster: &SkillCaster,
    skill_target: &SkillEventTarget,
    skill_data: &SkillData,
    skill_target_query: &mut SkillTargetQuery,
) -> Result<(), SkillCastError> {
    if skill_data.scope > 0 {
        // Apply in AOE around target position
        let client_entity_zone = client_entity_list
            .get_zone(skill_caster.position.zone_id)
            .ok_or(SkillCastError::InvalidTarget)?;

        let skill_position = match *skill_target {
            SkillEventTarget::Entity(target_entity) => {
                if let Some(skill_target) = skill_target_query.get_skill_target_data(target_entity)
                {
                    Some(skill_target.position.position.xy())
                } else {
                    None
                }
            }
            SkillEventTarget::Position(position) => Some(position),
        }
        .ok_or(SkillCastError::InvalidTarget)?;

        for (target_entity, _) in client_entity_zone
            .iter_entities_within_distance(skill_position, skill_data.scope as f32)
        {
            if let Some(mut skill_target) = skill_target_query.get_skill_target_data(target_entity)
            {
                apply_skill_damage_to_entity(
                    skill_system_parameters,
                    skill_system_resources,
                    skill_caster,
                    &mut skill_target,
                    skill_data,
                )
                .ok();
            }
        }

        Ok(())
    } else if let SkillEventTarget::Entity(target_entity) = *skill_target {
        // Apply directly to entity
        if let Some(mut skill_target) = skill_target_query.get_skill_target_data(target_entity) {
            apply_skill_damage_to_entity(
                skill_system_parameters,
                skill_system_resources,
                skill_caster,
                &mut skill_target,
                skill_data,
            )
            .ok();
            Ok(())
        } else {
            Err(SkillCastError::InvalidTarget)
        }
    } else {
        Err(SkillCastError::InvalidTarget)
    }
}

pub fn skill_effect_system(
    mut skill_system_parameters: SkillSystemParameters,
    skill_system_resources: SkillSystemResources,
    mut skill_target_query: SkillTargetQuery,

    mut commands: Commands,
    mut caster_query: Query<(
        &ClientEntity,
        &Position,
        &AbilityValues,
        &Team,
        &Level,
        Option<&GameClient>,
        Option<&mut Inventory>,
    )>,
    mut client_entity_list: ResMut<ClientEntityList>,
    mut skill_events: EventReader<SkillEvent>,
    mut pending_skill_events: Local<Vec<SkillEvent>>,
) {
    // Read events into pending_skill_events for executing at specific time
    for skill_event in skill_events.iter() {
        pending_skill_events.push(skill_event.clone());
    }

    // TODO: drain_filter pls
    let mut i = 0;
    while i != pending_skill_events.len() {
        if pending_skill_events[i].when > skill_system_resources.server_time.now {
            i += 1;
            continue;
        }

        let SkillEvent {
            skill_id,
            caster_entity,
            skill_target,
            use_item,
            ..
        } = pending_skill_events.remove(i);

        let skill_data = skill_system_resources.game_data.skills.get_skill(skill_id);
        if skill_data.is_none() {
            continue;
        }
        let skill_data = skill_data.unwrap();

        if let Ok((
            caster_client_entity,
            caster_position,
            caster_ability_values,
            caster_team,
            caster_level,
            caster_game_client,
            mut caster_inventory,
        )) = caster_query.get_mut(caster_entity)
        {
            let skill_caster = SkillCaster {
                entity: caster_entity,
                client_entity: caster_client_entity,
                level: caster_level,
                position: caster_position,
                ability_values: caster_ability_values,
                team: caster_team,
            };

            let mut consumed_item = None;
            let mut result = Ok(());

            // If the skill is to use an item, try take it from inventory now
            if let Some((item_slot, item)) = use_item {
                if let Some(caster_inventory) = caster_inventory.as_mut() {
                    if let Some(inventory_item) = caster_inventory.get_item(item_slot) {
                        if item.is_same_item(inventory_item) {
                            if let Some(item) = caster_inventory.try_take_quantity(item_slot, 1) {
                                consumed_item = Some((item_slot, item));
                            }
                        }
                    }
                }

                if consumed_item.is_none() {
                    // Failed to take item from inventory, cancel the skill
                    result = Err(SkillCastError::NotEnoughUseAbility);
                }
            }

            if result.is_ok() {
                result = match skill_data.skill_type {
                    SkillType::Immediate
                    | SkillType::EnforceWeapon
                    | SkillType::EnforceBullet
                    | SkillType::FireBullet
                    | SkillType::AreaTarget
                    | SkillType::SelfDamage => {
                        match apply_skill_damage(
                            &mut skill_system_parameters,
                            &skill_system_resources,
                            &client_entity_list,
                            &skill_caster,
                            &skill_target,
                            skill_data,
                            &mut skill_target_query,
                        ) {
                            Ok(_) => apply_skill_status_effects(
                                &mut skill_system_parameters,
                                &skill_system_resources,
                                &client_entity_list,
                                &skill_caster,
                                &skill_target,
                                skill_data,
                                &mut skill_target_query,
                            ),
                            Err(err) => Err(err),
                        }
                    }
                    SkillType::SelfBoundDuration
                    | SkillType::SelfStateDuration
                    | SkillType::TargetBoundDuration
                    | SkillType::TargetStateDuration
                    | SkillType::SelfBound
                    | SkillType::TargetBound => apply_skill_status_effects(
                        &mut skill_system_parameters,
                        &skill_system_resources,
                        &client_entity_list,
                        &skill_caster,
                        &skill_target,
                        skill_data,
                        &mut skill_target_query,
                    ),
                    SkillType::SelfAndTarget => {
                        // Only applies status effect if damage > 0
                        if let SkillEventTarget::Entity(target_entity) = skill_target {
                            if let Some(mut skill_target_data) =
                                skill_target_query.get_skill_target_data(target_entity)
                            {
                                match apply_skill_damage_to_entity(
                                    &mut skill_system_parameters,
                                    &skill_system_resources,
                                    &skill_caster,
                                    &mut skill_target_data,
                                    skill_data,
                                ) {
                                    Ok(damage) if damage.amount > 0 => apply_skill_status_effects(
                                        &mut skill_system_parameters,
                                        &skill_system_resources,
                                        &client_entity_list,
                                        &skill_caster,
                                        &skill_target,
                                        skill_data,
                                        &mut skill_target_query,
                                    ),
                                    Ok(_) => Ok(()),
                                    Err(err) => Err(err),
                                }
                            } else {
                                Err(SkillCastError::InvalidTarget)
                            }
                        } else {
                            Err(SkillCastError::InvalidTarget)
                        }
                    }
                    SkillType::SummonPet => {
                        if let Some(npc_id) = skill_data.summon_npc_id {
                            if MonsterBundle::spawn(
                                &mut commands,
                                &mut client_entity_list,
                                &skill_system_resources.game_data,
                                npc_id,
                                skill_caster.position.zone_id,
                                SpawnOrigin::Summoned(
                                    skill_caster.entity,
                                    skill_caster.position.position,
                                ),
                                150,
                                skill_caster.team.clone(),
                                Some((skill_caster.entity, skill_caster.level)),
                                Some(skill_data.level as i32),
                            )
                            .is_some()
                            {
                                // TODO: Increase summon count point thing
                                // TODO: Apply status effect to decrease life over time
                                Ok(())
                            } else {
                                Err(SkillCastError::InvalidSkill)
                            }
                        } else {
                            Err(SkillCastError::InvalidSkill)
                        }
                    }
                    SkillType::BasicAction
                    | SkillType::CreateWindow
                    | SkillType::Passive
                    | SkillType::Emote
                    | SkillType::Warp => Ok(()),
                    SkillType::Resurrection => {
                        warn!("Unimplemented skill type used {:?}", skill_data);
                        Ok(())
                    }
                };
            }

            match result {
                Ok(_) => {
                    // Send message notifying client of consumption of item
                    if let Some((item_slot, _)) = consumed_item {
                        if let (Some(caster_inventory), Some(caster_game_client)) =
                            (caster_inventory, caster_game_client)
                        {
                            match caster_inventory.get_item(item_slot) {
                                None => {
                                    // When the item has been fully consumed we send UpdateInventory packet
                                    caster_game_client
                                        .server_message_tx
                                        .send(ServerMessage::UpdateInventory(
                                            vec![(item_slot, None)],
                                            None,
                                        ))
                                        .ok();
                                }
                                Some(item) => {
                                    // When there is still remaining quantity we send UseItem packet
                                    caster_game_client
                                        .server_message_tx
                                        .send(ServerMessage::UseInventoryItem(UseInventoryItem {
                                            entity_id: skill_caster.client_entity.id,
                                            item: item.get_item_reference(),
                                            inventory_slot: item_slot,
                                        }))
                                        .ok();
                                }
                            }
                        }
                    }

                    skill_system_parameters.server_messages.send_entity_message(
                        caster_client_entity,
                        ServerMessage::FinishCastingSkill(caster_client_entity.id, skill_id),
                    )
                }
                Err(error) => {
                    // Return unused item to inventory
                    if let Some((item_slot, item)) = consumed_item {
                        caster_inventory
                            .unwrap()
                            .try_stack_with_item(item_slot, item)
                            .expect("Unexpected error returning unconsumed item to inventory");
                    }

                    skill_system_parameters.server_messages.send_entity_message(
                        caster_client_entity,
                        ServerMessage::CancelCastingSkill(
                            caster_client_entity.id,
                            match error {
                                SkillCastError::NotEnoughUseAbility => {
                                    CancelCastingSkillReason::NeedAbility
                                }
                                _ => CancelCastingSkillReason::NeedTarget,
                            },
                        ),
                    )
                }
            }
        }
    }
}
