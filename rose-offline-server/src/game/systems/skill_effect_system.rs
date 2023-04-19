use bevy::{
    ecs::{
        prelude::{Commands, Entity, EventReader, EventWriter, Local, Query, Res, ResMut},
        query::WorldQuery,
        system::SystemParam,
    },
    math::Vec3Swizzles,
    time::Time,
};
use log::warn;
use rand::Rng;
use std::marker::PhantomData;

use rose_data::{
    AbilityType, SkillData, SkillTargetFilter, SkillType, StatusEffectClearedByType,
    StatusEffectType,
};
use rose_game_common::data::Damage;

use crate::game::{
    bundles::{ability_values_add_value, ability_values_get_value, MonsterBundle},
    components::{
        AbilityValues, ClanMembership, ClientEntity, ClientEntityType, Dead, ExperiencePoints,
        GameClient, HealthPoints, Inventory, Level, ManaPoints, MoveSpeed, PartyMembership,
        Position, SpawnOrigin, Stamina, StatusEffects, Team,
    },
    events::{DamageEvent, ItemLifeEvent, SkillEvent, SkillEventTarget},
    messages::server::{
        ApplySkillEffect, CancelCastingSkillReason, ServerMessage, UseInventoryItem,
    },
    resources::{ClientEntityList, ServerMessages},
    GameData,
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
    damage_events: EventWriter<'w, DamageEvent>,
    item_life_events: EventWriter<'w, ItemLifeEvent>,

    #[system_param(ignore)]
    _secret: PhantomData<&'s ()>,
}

#[derive(SystemParam)]
pub struct SkillSystemResources<'w, 's> {
    game_data: Res<'w, GameData>,
    time: Res<'w, Time>,

    #[system_param(ignore)]
    _secret: PhantomData<&'s ()>,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct SkillCasterQuery<'w> {
    entity: Entity,
    client_entity: &'w ClientEntity,
    position: &'w Position,
    ability_values: &'w AbilityValues,
    experience_points: Option<&'w mut ExperiencePoints>,
    level: &'w Level,
    team: &'w Team,
    game_client: Option<&'w GameClient>,
    inventory: Option<&'w mut Inventory>,
    party_membership: Option<&'w PartyMembership>,
    clan_membership: Option<&'w ClanMembership>,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct SkillTargetQuery<'w> {
    entity: Entity,
    client_entity: &'w ClientEntity,
    position: &'w Position,
    ability_values: &'w AbilityValues,
    status_effects: &'w mut StatusEffects,
    team: &'w Team,
    health_points: &'w mut HealthPoints,
    mana_points: Option<&'w mut ManaPoints>,
    level: &'w Level,
    move_speed: &'w MoveSpeed,
    stamina: Option<&'w mut Stamina>,
    dead: Option<&'w Dead>,
    party_membership: Option<&'w PartyMembership>,
    clan_membership: Option<&'w ClanMembership>,
}

fn check_skill_target_filter(
    skill_caster: &SkillCasterQueryItem,
    skill_target: &mut SkillTargetQueryItem,
    skill_data: &SkillData,
) -> bool {
    let target_is_alive = skill_target.health_points.hp > 0 && skill_target.dead.is_none();

    match skill_data.target_filter {
        SkillTargetFilter::OnlySelf => {
            target_is_alive && skill_caster.entity == skill_target.entity
        }
        SkillTargetFilter::Group => {
            let caster_party = skill_caster
                .party_membership
                .and_then(|party_membership| party_membership.party());
            let target_party = skill_target
                .party_membership
                .and_then(|party_membership| party_membership.party());
            target_is_alive && caster_party.is_some() && caster_party == target_party
        }
        SkillTargetFilter::Guild => {
            let caster_party = skill_caster
                .clan_membership
                .and_then(|clan_membership| clan_membership.clan());
            let target_party = skill_target
                .clan_membership
                .and_then(|clan_membership| clan_membership.clan());
            target_is_alive && caster_party.is_some() && caster_party == target_party
        }
        SkillTargetFilter::Allied => {
            target_is_alive && skill_caster.team.id == skill_target.team.id
        }
        SkillTargetFilter::Monster => {
            target_is_alive
                && matches!(
                    skill_target.client_entity.entity_type,
                    ClientEntityType::Monster
                )
        }
        SkillTargetFilter::Enemy => target_is_alive && skill_caster.team.id != skill_target.team.id,
        SkillTargetFilter::EnemyCharacter => {
            target_is_alive
                && skill_caster.team.id != skill_target.team.id
                && matches!(
                    skill_target.client_entity.entity_type,
                    ClientEntityType::Character
                )
        }
        SkillTargetFilter::Character => {
            target_is_alive
                && matches!(
                    skill_target.client_entity.entity_type,
                    ClientEntityType::Character
                )
        }
        SkillTargetFilter::CharacterOrMonster => {
            target_is_alive
                && matches!(
                    skill_target.client_entity.entity_type,
                    ClientEntityType::Character | ClientEntityType::Monster
                )
        }
        SkillTargetFilter::DeadAlliedCharacter => {
            !target_is_alive
                && skill_caster.team.id == skill_target.team.id
                && matches!(
                    skill_target.client_entity.entity_type,
                    ClientEntityType::Character
                )
        }
        SkillTargetFilter::EnemyMonster => {
            target_is_alive
                && skill_caster.team.id != skill_target.team.id
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
    skill_caster: &SkillCasterQueryItem,
    skill_target: &mut SkillTargetQueryItem,
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

    let mut effect_success = [false, false];
    for (effect_index, status_effect_data) in skill_data
        .status_effects
        .iter()
        .enumerate()
        .filter_map(|(index, id)| {
            id.and_then(|id| {
                skill_system_resources
                    .game_data
                    .status_effects
                    .get_status_effect(id)
            })
            .map(|id| (index, id))
        })
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
        } else if let Some(skill_add_ability) = skill_data.add_ability[effect_index].as_ref() {
            let ability_value = ability_values_get_value(
                skill_add_ability.ability_type,
                Some(skill_target.ability_values),
                Some(skill_target.level),
                Some(skill_target.move_speed),
                Some(skill_target.team),
                None,
                None,
                None,
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
                skill_system_resources.time.last_update().unwrap()
                    + skill_data.status_effect_duration,
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

    for (effect_index, add_ability) in
        skill_data
            .add_ability
            .iter()
            .enumerate()
            .filter_map(|(index, add_ability)| {
                add_ability.as_ref().map(|add_ability| (index, add_ability))
            })
    {
        match add_ability.ability_type {
            AbilityType::Health => {
                skill_target.health_points.hp = i32::min(
                    skill_target.ability_values.get_max_health(),
                    skill_target.health_points.hp
                        + skill_system_resources
                            .game_data
                            .ability_value_calculator
                            .calculate_skill_adjust_value(
                                add_ability,
                                skill_caster.ability_values.get_intelligence(),
                                skill_target.health_points.hp,
                            ),
                );
                effect_success[effect_index] = true;
            }
            AbilityType::Mana => {
                if let Some(target_mana_points) = skill_target.mana_points.as_mut() {
                    target_mana_points.mp = i32::min(
                        skill_target.ability_values.get_max_mana(),
                        target_mana_points.mp + add_ability.value,
                    );
                }
                effect_success[effect_index] = true;
            }
            AbilityType::Stamina | AbilityType::Money => {
                warn!(
                    "Unimplemented skill status effect add ability_type {:?}, value {}",
                    add_ability.ability_type, add_ability.value
                )
            }
            _ => {}
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
    skill_caster: &SkillCasterQueryItem,
    skill_target: &SkillEventTarget,
    skill_data: &SkillData,
    skill_target_query: &mut Query<SkillTargetQuery>,
) -> Result<(), SkillCastError> {
    if skill_data.scope > 0 {
        // Apply in AOE around target position
        let client_entity_zone = client_entity_list
            .get_zone(skill_caster.position.zone_id)
            .ok_or(SkillCastError::InvalidTarget)?;

        let skill_position = match *skill_target {
            SkillEventTarget::Entity(target_entity) => {
                if let Ok(skill_target) = skill_target_query.get_mut(target_entity) {
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
            if let Ok(mut skill_target) = skill_target_query.get_mut(target_entity) {
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
        if let Ok(mut skill_target) = skill_target_query.get_mut(target_entity) {
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
    skill_caster: &SkillCasterQueryItem,
    skill_target: &mut SkillTargetQueryItem,
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

    skill_system_parameters
        .damage_events
        .send(DamageEvent::with_skill(
            skill_caster.entity,
            skill_target.entity,
            damage,
            skill_data.id,
            skill_caster.ability_values.get_intelligence(),
        ));

    Ok(damage)
}

fn apply_skill_damage(
    skill_system_parameters: &mut SkillSystemParameters,
    skill_system_resources: &SkillSystemResources,
    client_entity_list: &ClientEntityList,
    skill_caster: &SkillCasterQueryItem,
    skill_target: &SkillEventTarget,
    skill_data: &SkillData,
    skill_target_query: &mut Query<SkillTargetQuery>,
) -> Result<(), SkillCastError> {
    let result = if skill_data.scope > 0 {
        // Apply in AOE around target position
        let client_entity_zone = client_entity_list
            .get_zone(skill_caster.position.zone_id)
            .ok_or(SkillCastError::InvalidTarget)?;

        let skill_position = match *skill_target {
            SkillEventTarget::Entity(target_entity) => {
                if let Ok(skill_target) = skill_target_query.get_mut(target_entity) {
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
            if let Ok(mut skill_target) = skill_target_query.get_mut(target_entity) {
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
        if let Ok(mut skill_target) = skill_target_query.get_mut(target_entity) {
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
    };

    if result.is_ok() && skill_data.damage_type != 3 {
        skill_system_parameters
            .item_life_events
            .send(ItemLifeEvent::DecreaseWeaponLife(skill_caster.entity));
    }

    result
}

pub fn skill_effect_system(
    mut commands: Commands,
    mut skill_system_parameters: SkillSystemParameters,
    skill_system_resources: SkillSystemResources,
    mut skill_caster_query: Query<SkillCasterQuery>,
    mut skill_target_query: Query<SkillTargetQuery>,
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
        if pending_skill_events[i].when > skill_system_resources.time.last_update().unwrap() {
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

        if let Ok(mut skill_caster) = skill_caster_query.get_mut(caster_entity) {
            let mut consumed_item = None;
            let mut result = Ok(());

            // If the skill is to use an item, try take it from inventory now
            if let Some((item_slot, item)) = use_item {
                if let Some(caster_inventory) = skill_caster.inventory.as_mut() {
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

            // Check and subtract skill usage requirements
            if matches!(skill_data.skill_type, SkillType::SummonPet) {
                let summon_point_requirement = skill_data
                    .summon_npc_id
                    .and_then(|npc_id| skill_system_resources.game_data.npcs.get_npc(npc_id))
                    .map_or(0, |npc_data| npc_data.summon_point_requirement);
                if summon_point_requirement > 0 {
                    // TODO: Check summon_point_requirement and update summon points
                }
            }

            for &(use_ability_type, mut use_ability_value) in skill_data.use_ability.iter() {
                if use_ability_type == AbilityType::Mana {
                    let use_mana_rate =
                        (100 - skill_caster.ability_values.get_save_mana()) as f32 / 100.0;
                    use_ability_value = (use_ability_value as f32 * use_mana_rate) as i32;
                }

                // We use the skill_target_query to access the other required components
                let mut skill_caster2 = skill_target_query.get_mut(caster_entity).unwrap();
                let ability_value = ability_values_get_value(
                    use_ability_type,
                    Some(skill_caster.ability_values),
                    Some(skill_caster.level),
                    None,
                    None,
                    None,
                    skill_caster.experience_points.as_deref(),
                    skill_caster.inventory.as_deref(),
                    None,
                    skill_caster2.stamina.as_deref(),
                    None,
                    None,
                    Some(skill_caster2.health_points.as_ref()),
                    skill_caster2.mana_points.as_deref(),
                    // TODO: Fuel
                );

                if let Some(ability_value) = ability_value {
                    if ability_value < use_ability_value {
                        // Not enough use ability, cancel the skill
                        result = Err(SkillCastError::NotEnoughUseAbility);
                    } else {
                        ability_values_add_value(
                            use_ability_type,
                            -use_ability_value,
                            Some(skill_caster.ability_values),
                            None,
                            skill_caster.experience_points.as_mut(),
                            Some(&mut skill_caster2.health_points),
                            skill_caster.inventory.as_mut(), // For money
                            skill_caster2.mana_points.as_mut(),
                            None,
                            skill_caster2.stamina.as_mut(),
                            None,
                            None,
                            None,
                            // TODO: Fuel
                        );
                    }
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
                            if let Ok(mut skill_target_data) =
                                skill_target_query.get_mut(target_entity)
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
                            if let Some(entity) = MonsterBundle::spawn(
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
                            ) {
                                // Apply status effect to decrease summon's life over time
                                if let Some(status_effect_data) = skill_system_resources
                                    .game_data
                                    .status_effects
                                    .get_decrease_summon_life_status_effect()
                                {
                                    let mut status_effects = StatusEffects::new();
                                    status_effects.apply_summon_decrease_life_status_effect(
                                        status_effect_data,
                                    );
                                    commands.entity(entity).insert(status_effects);
                                }

                                // TODO: Increase summon count point thing
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
                            (skill_caster.inventory, skill_caster.game_client)
                        {
                            match caster_inventory.get_item(item_slot) {
                                None => {
                                    // When the item has been fully consumed we send UpdateInventory packet
                                    caster_game_client
                                        .server_message_tx
                                        .send(ServerMessage::UpdateInventory {
                                            items: vec![(item_slot, None)],
                                            money: None,
                                        })
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
                        skill_caster.client_entity,
                        ServerMessage::FinishCastingSkill(skill_caster.client_entity.id, skill_id),
                    )
                }
                Err(error) => {
                    // Return unused item to inventory
                    if let Some((item_slot, item)) = consumed_item {
                        skill_caster
                            .inventory
                            .unwrap()
                            .try_stack_with_item(item_slot, item)
                            .expect("Unexpected error returning unconsumed item to inventory");
                    }

                    skill_system_parameters.server_messages.send_entity_message(
                        skill_caster.client_entity,
                        ServerMessage::CancelCastingSkill(
                            skill_caster.client_entity.id,
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
