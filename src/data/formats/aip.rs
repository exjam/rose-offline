use log::warn;
use std::{
    num::NonZeroU8,
    ops::{Range, RangeInclusive},
    time::Duration,
};

use crate::data::{
    formats::reader::{FileReader, ReadError},
    ItemReference,
};

#[derive(Copy, Clone, Debug)]
pub enum AipAbilityType {
    Level,
    Attack,
    Defence,
    Resistance,
    HealthPoints,
    Charm,
}

fn decode_ability_type(value: u8) -> Result<AipAbilityType, AipReadError> {
    match value {
        0 => Ok(AipAbilityType::Level),
        1 => Ok(AipAbilityType::Attack),
        2 => Ok(AipAbilityType::Defence),
        3 => Ok(AipAbilityType::Resistance),
        4 => Ok(AipAbilityType::HealthPoints),
        5 => Ok(AipAbilityType::Charm),
        _ => Err(AipReadError::InvalidValue),
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AipOperatorType {
    Equals,
    GreaterThan,
    GreaterThanEqual,
    LessThan,
    LessThanEqual,
    NotEqual,
}

fn decode_operator_type(value: u8) -> Result<AipOperatorType, AipReadError> {
    match value {
        0 => Ok(AipOperatorType::Equals),
        1 => Ok(AipOperatorType::GreaterThan),
        2 => Ok(AipOperatorType::GreaterThanEqual),
        3 => Ok(AipOperatorType::LessThan),
        4 => Ok(AipOperatorType::LessThanEqual),
        10 => Ok(AipOperatorType::NotEqual),
        _ => Err(AipReadError::InvalidValue),
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AipResultOperator {
    Set,
    Add,
    Subtract,
}

fn decode_result_operator_type(value: u8) -> Result<AipResultOperator, AipReadError> {
    match value {
        5 => Ok(AipResultOperator::Set),
        6 => Ok(AipResultOperator::Add),
        7 => Ok(AipResultOperator::Subtract),
        _ => Err(AipReadError::InvalidValue),
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AipHaveStatusTarget {
    This,
    Target,
}

#[derive(Clone, Copy, Debug)]
pub enum AipHaveStatusType {
    Good,
    Bad,
    Any,
}

#[derive(Debug)]
pub struct AipConditionFindNearbyEntities {
    pub distance: AipDistance,
    pub is_allied: bool,
    pub level_diff_range: RangeInclusive<i32>,
    pub count_operator_type: Option<AipOperatorType>,
    pub count: i32,
}

#[derive(Debug)]
pub struct AipConditionMonthDayTime {
    pub month_day: Option<NonZeroU8>,
    pub day_minutes_range: RangeInclusive<i32>,
}

#[derive(Debug)]
pub struct AipConditionWeekDayTime {
    pub week_day: u8,
    pub day_minutes_range: RangeInclusive<i32>,
}

#[derive(Clone, Copy, Debug)]
pub enum AipMoveMode {
    Walk,
    Run,
}

#[derive(Clone, Copy, Debug)]
pub enum AipAttackNearbyStat {
    Lowest,
    Highest,
}

#[derive(Clone, Copy, Debug)]
pub enum AipMoveOrigin {
    Spawn,
    CurrentPosition,
    FindChar,
}

#[derive(Clone, Copy, Debug)]
pub enum AipSpawnNpcOrigin {
    CurrentPosition,
    AttackerPosition,
    TargetPosition,
}

#[derive(Clone, Copy, Debug)]
pub enum AipSkillTarget {
    FindChar,
    Target,
    This,
    NearChar,
}

#[derive(Clone, Copy, Debug)]
pub enum AipVariableType {
    LocalNpcObject,
    Ai,
    World,
    Economy,
}

#[derive(Clone, Copy, Debug)]
pub enum AipMessageType {
    Say,
    Shout,
    Announce,
}

#[derive(Clone, Copy, Debug)]
pub enum AipMonsterSpawnState {
    Disabled,
    Enabled,
    Toggle,
}

#[derive(Clone, Copy, Debug)]
pub enum AipDamageType {
    Received,
    Given,
}

#[derive(Clone, Copy, Debug)]
pub enum AipDistanceOrigin {
    Spawn,
    Owner,
    Target,
}

pub type AipDistance = i32;
pub type AipNpcId = i32;
pub type AipSkillId = i32;
pub type AipMotionId = i32;
pub type AipZoneId = usize;
pub type AipIsSpawnOwner = bool;

#[derive(Debug)]
pub enum AipCondition {
    CompareAttackerAndTargetAbilityValue(AipOperatorType, AipAbilityType),
    FindNearbyEntities(AipConditionFindNearbyEntities),
    Damage(AipDamageType, AipOperatorType, i32),
    Distance(AipDistanceOrigin, AipOperatorType, AipDistance),
    HasOwner,
    HasStatusEffect(AipHaveStatusTarget, AipHaveStatusType, bool),
    HealthPercent(AipOperatorType, i32),
    IsAttackerClanMaster,
    IsAttackerCurrentTarget,
    IsDaytime(bool),
    IsTargetClanMaster,
    MonthDay(AipConditionMonthDayTime),
    NoTargetAndCompareAttackerAbilityValue(AipOperatorType, AipAbilityType, i32),
    OwnerHasTarget,
    Random(AipOperatorType, Range<i32>, i32),
    SelectLocalNpc(AipNpcId),
    SelfAbilityValue(AipOperatorType, AipAbilityType, i32),
    ServerChannelNumber(RangeInclusive<u16>),
    TargetAbilityValue(AipOperatorType, AipAbilityType, i32),
    Variable(AipVariableType, usize, AipOperatorType, i32),
    WeekDay(AipConditionWeekDayTime),
    WorldTime(RangeInclusive<u32>),
    ZoneTime(RangeInclusive<u32>),
}

#[derive(Debug)]
pub enum AipAction {
    Stop,
    Emote(u8),
    Say(usize),
    MoveRandomDistance(AipMoveOrigin, AipMoveMode, AipDistance),
    AttackNearbyEntityByStat(AipDistance, AipAbilityType, AipAttackNearbyStat),
    SpecialAttack,
    MoveDistanceFromTarget(AipMoveMode, AipDistance),
    TransformNpc(AipNpcId),
    SpawnNpc(AipNpcId, AipDistance, AipSpawnNpcOrigin, AipIsSpawnOwner),
    NearbyAlliesAttackTarget(usize, AipDistance, Option<AipNpcId>),
    AttackNearChar,
    AttackFindChar,
    NearbyAlliesSameNpcAttackTarget(AipDistance), // Nearby allies with same npc id attack target
    AttackAttacker,
    RunAway(AipDistance),
    DropRandomItem(Vec<ItemReference>),
    KillSelf,
    UseSkill(AipSkillTarget, AipSkillId, AipMotionId),
    SetVariable(AipVariableType, usize, AipResultOperator, i32),
    Message(AipMessageType, usize),
    MoveNearOwner,
    DoQuestTrigger(String),
    AttackOwnerTarget,
    SetPvpFlag(Option<AipZoneId>, bool),
    SetMonsterSpawnState(Option<AipZoneId>, AipMonsterSpawnState),
    GiveItemToOwner(ItemReference, usize),
}

pub struct AipEvent {
    pub name: String,
    pub conditions: Vec<AipCondition>,
    pub actions: Vec<AipAction>,
}

pub struct AipTrigger {
    pub name: String,
    pub events: Vec<AipEvent>,
}

pub struct AipFile {
    pub idle_trigger_interval: Duration,
    pub damage_trigger_new_target_chance: u32,
    pub trigger_on_created: Option<AipTrigger>,
    pub trigger_on_idle: Option<AipTrigger>,
    pub trigger_on_attack_move: Option<AipTrigger>,
    pub trigger_on_damaged: Option<AipTrigger>,
    pub trigger_on_kill: Option<AipTrigger>,
    pub trigger_on_dead: Option<AipTrigger>,
}

#[derive(Debug)]
pub enum AipReadError {
    UnexpectedEof,
    InvalidValue,
}

impl From<ReadError> for AipReadError {
    fn from(err: ReadError) -> Self {
        match err {
            ReadError::UnexpectedEof => AipReadError::UnexpectedEof,
        }
    }
}

#[allow(dead_code)]
impl AipFile {
    pub fn read(mut reader: FileReader) -> Result<Self, AipReadError> {
        let num_triggers = reader.read_u32()?;
        let idle_trigger_interval = Duration::from_secs(reader.read_u32()? as u64);
        let damage_trigger_new_target_chance = reader.read_u32()?;
        let _title = reader.read_u32_length_string()?;
        let mut triggers = Vec::new();

        for _ in 0..num_triggers {
            let trigger_name = reader.read_fixed_length_string(32)?;
            let num_events = reader.read_u32()?;
            let mut events = Vec::new();

            for _ in 0..num_events {
                let event_name = reader.read_fixed_length_string(32)?;
                let num_conditions = reader.read_u32()?;
                let mut conditions = Vec::new();

                for _ in 0..num_conditions {
                    let condition_start_position = reader.position();
                    let size_bytes = reader.read_u32()? as u64;
                    let opcode = reader.read_u32()?;

                    match opcode ^ 0x04000000 {
                        2 => {
                            let damage = reader.read_i32()?;
                            let damage_type = if reader.read_u8()? != 0 {
                                AipDamageType::Given
                            } else {
                                AipDamageType::Received
                            };
                            reader.skip(3); // padding

                            conditions.push(AipCondition::Damage(
                                damage_type,
                                AipOperatorType::GreaterThanEqual,
                                damage,
                            ));
                        }
                        3 => {
                            let distance = reader.read_i32()?;
                            let is_allied = reader.read_u8()? > 0;
                            reader.skip(1); // padding
                            let min_level_diff = reader.read_i16()? as i32;
                            let max_level_diff = reader.read_i16()? as i32;
                            let count = reader.read_u16()? as i32;

                            conditions.push(AipCondition::FindNearbyEntities(
                                AipConditionFindNearbyEntities {
                                    distance: distance * 100,
                                    is_allied,
                                    level_diff_range: min_level_diff..=max_level_diff,
                                    count_operator_type: None,
                                    count,
                                },
                            ))
                        }
                        4 => {
                            let distance = reader.read_i32()?;
                            conditions.push(AipCondition::Distance(
                                AipDistanceOrigin::Spawn,
                                AipOperatorType::GreaterThanEqual,
                                distance * 100,
                            ));
                        }
                        5 => {
                            let distance = reader.read_i32()?;
                            let is_lte = reader.read_u8()? != 0;
                            reader.skip(3); // padding

                            if is_lte {
                                conditions.push(AipCondition::Distance(
                                    AipDistanceOrigin::Target,
                                    AipOperatorType::LessThanEqual,
                                    distance * 100,
                                ));
                            } else {
                                conditions.push(AipCondition::Distance(
                                    AipDistanceOrigin::Target,
                                    AipOperatorType::GreaterThanEqual,
                                    distance * 100,
                                ));
                            }
                        }
                        6 => {
                            let ability_type = decode_ability_type(reader.read_u32()? as u8)?;
                            let value = reader.read_i32()?;
                            let is_lte = reader.read_u8()? != 0;
                            reader.skip(3); // padding

                            if is_lte {
                                conditions.push(AipCondition::TargetAbilityValue(
                                    AipOperatorType::LessThanEqual,
                                    ability_type,
                                    value,
                                ));
                            } else {
                                conditions.push(AipCondition::TargetAbilityValue(
                                    AipOperatorType::GreaterThanEqual,
                                    ability_type,
                                    value,
                                ));
                            }
                        }
                        7 => {
                            let value = reader.read_i32()?;
                            let is_lte = reader.read_u8()? != 0;
                            reader.skip(3); // padding

                            if is_lte {
                                conditions.push(AipCondition::HealthPercent(
                                    AipOperatorType::LessThanEqual,
                                    value,
                                ));
                            } else {
                                conditions.push(AipCondition::HealthPercent(
                                    AipOperatorType::GreaterThanEqual,
                                    value,
                                ));
                            }
                        }
                        8 => {
                            let value = reader.read_u8()? as i32;
                            reader.skip(3); // padding

                            conditions.push(AipCondition::Random(
                                AipOperatorType::LessThan,
                                0..100,
                                value,
                            ));
                        }
                        9 => {
                            let distance = reader.read_i32()?;
                            let min_level_diff = reader.read_i16()? as i32;
                            let max_level_diff = reader.read_i16()? as i32;
                            let is_allied = reader.read_u8()? != 0;
                            reader.skip(3); // padding

                            conditions.push(AipCondition::FindNearbyEntities(
                                AipConditionFindNearbyEntities {
                                    distance: distance * 100,
                                    is_allied,
                                    level_diff_range: min_level_diff..=max_level_diff,
                                    count_operator_type: None,
                                    count: 1,
                                },
                            ))
                        }
                        10 => {
                            conditions.push(AipCondition::IsAttackerCurrentTarget);
                        }
                        11 => {
                            let ability_type = decode_ability_type(reader.read_u8()?)?;
                            let is_lt = reader.read_u8()? != 0;
                            reader.skip(2); // padding

                            if is_lt {
                                conditions.push(
                                    AipCondition::CompareAttackerAndTargetAbilityValue(
                                        AipOperatorType::LessThan,
                                        ability_type,
                                    ),
                                );
                            } else {
                                conditions.push(
                                    AipCondition::CompareAttackerAndTargetAbilityValue(
                                        AipOperatorType::GreaterThan,
                                        ability_type,
                                    ),
                                );
                            }
                        }
                        12 => {
                            let ability_type = decode_ability_type(reader.read_u8()?)?;
                            reader.skip(3); // padding
                            let value = reader.read_i32()?;
                            let is_lte = reader.read_u8()? != 0;
                            reader.skip(3); // padding

                            if is_lte {
                                conditions.push(
                                    AipCondition::NoTargetAndCompareAttackerAbilityValue(
                                        AipOperatorType::LessThanEqual,
                                        ability_type,
                                        value,
                                    ),
                                );
                            } else {
                                conditions.push(
                                    AipCondition::NoTargetAndCompareAttackerAbilityValue(
                                        AipOperatorType::GreaterThanEqual,
                                        ability_type,
                                        value,
                                    ),
                                );
                            }
                        }
                        13 => {
                            let is_night = reader.read_u8()? != 0;
                            reader.skip(3); // padding
                            conditions.push(AipCondition::IsDaytime(!is_night));
                        }
                        14 => {
                            let target = if reader.read_u8()? != 0 {
                                AipHaveStatusTarget::Target
                            } else {
                                AipHaveStatusTarget::This
                            };
                            let status_type = match reader.read_u8()? {
                                0 => AipHaveStatusType::Good,
                                1 => AipHaveStatusType::Bad,
                                _ => AipHaveStatusType::Any,
                            };
                            let have = reader.read_u8()? != 0;
                            reader.skip(1); // padding
                            conditions.push(AipCondition::HasStatusEffect(
                                target,
                                status_type,
                                have,
                            ));
                        }
                        15 => {
                            let variable = reader.read_u16()? as usize;
                            reader.skip(2); // padding
                            let value = reader.read_i32()?;
                            let opcode = decode_operator_type(reader.read_u8()?)?;
                            reader.skip(3); // padding
                            conditions.push(AipCondition::Variable(
                                AipVariableType::LocalNpcObject,
                                variable,
                                opcode,
                                value,
                            ));
                        }
                        16 => {
                            let variable = reader.read_u16()? as usize;
                            reader.skip(2); // padding
                            let value = reader.read_i32()?;
                            let opcode = decode_operator_type(reader.read_u8()?)?;
                            reader.skip(3); // padding
                            conditions.push(AipCondition::Variable(
                                AipVariableType::World,
                                variable,
                                opcode,
                                value,
                            ));
                        }
                        17 => {
                            let variable = reader.read_u16()? as usize;
                            reader.skip(2); // padding
                            let value = reader.read_i32()?;
                            let opcode = decode_operator_type(reader.read_u8()?)?;
                            reader.skip(3); // padding
                            conditions.push(AipCondition::Variable(
                                AipVariableType::Economy,
                                variable,
                                opcode,
                                value,
                            ));
                        }
                        18 => {
                            let npc_id = reader.read_u32()? as AipNpcId;
                            conditions.push(AipCondition::SelectLocalNpc(npc_id));
                        }
                        19 => {
                            let distance = reader.read_i32()?;
                            let opcode = decode_operator_type(reader.read_u8()?)?;
                            reader.skip(3); // padding
                            conditions.push(AipCondition::Distance(
                                AipDistanceOrigin::Owner,
                                opcode,
                                distance,
                            ));
                        }
                        20 => {
                            let start_time = reader.read_u32()?;
                            let end_time = reader.read_u32()?;
                            conditions.push(AipCondition::ZoneTime(start_time..=end_time));
                        }
                        21 => {
                            let ability_type = decode_ability_type(reader.read_u8()?)?;
                            reader.skip(3); // padding
                            let value = reader.read_i32()?;
                            let opcode = decode_operator_type(reader.read_u8()?)?;
                            reader.skip(3); // padding
                            conditions.push(AipCondition::SelfAbilityValue(
                                opcode,
                                ability_type,
                                value,
                            ));
                        }
                        22 => conditions.push(AipCondition::HasOwner),
                        23 => conditions.push(AipCondition::OwnerHasTarget),
                        24 => {
                            let start_time = reader.read_u32()?;
                            let end_time = reader.read_u32()?;
                            conditions.push(AipCondition::WorldTime(start_time..=end_time));
                        }
                        25 => {
                            let day = reader.read_u8()?;
                            let hour_min = reader.read_u8()?;
                            let minute_min = reader.read_u8()?;
                            let hour_max = reader.read_u8()?;
                            let minute_max = reader.read_u8()?;
                            reader.skip(3); // padding

                            conditions.push(AipCondition::MonthDay(AipConditionMonthDayTime {
                                month_day: NonZeroU8::new(day),
                                day_minutes_range: (hour_min as i32 * 60 + minute_min as i32)
                                    ..=(hour_max as i32 * 60 + minute_max as i32),
                            }));
                        }
                        26 => {
                            let day = reader.read_u8()?;
                            let hour_min = reader.read_u8()?;
                            let minute_min = reader.read_u8()?;
                            let hour_max = reader.read_u8()?;
                            let minute_max = reader.read_u8()?;
                            reader.skip(3); // padding

                            conditions.push(AipCondition::WeekDay(AipConditionWeekDayTime {
                                week_day: day,
                                day_minutes_range: (hour_min as i32 * 60 + minute_min as i32)
                                    ..=(hour_max as i32 * 60 + minute_max as i32),
                            }));
                        }
                        27 => {
                            let channel_no_min = reader.read_u16()?;
                            let channel_no_max = reader.read_u16()?;
                            conditions.push(AipCondition::ServerChannelNumber(
                                channel_no_min..=channel_no_max,
                            ));
                        }
                        28 => {
                            let distance = reader.read_i32()?;
                            let is_allied = reader.read_u8()? > 0;
                            reader.skip(1); // padding
                            let min_level_diff = reader.read_i16()? as i32;
                            let max_level_diff = reader.read_i16()? as i32;
                            let count = reader.read_u16()? as i32;
                            let count_operator_type = decode_operator_type(reader.read_u8()?)?;
                            reader.skip(3); // padding

                            conditions.push(AipCondition::FindNearbyEntities(
                                AipConditionFindNearbyEntities {
                                    distance: distance * 100,
                                    is_allied,
                                    level_diff_range: min_level_diff..=max_level_diff,
                                    count_operator_type: Some(count_operator_type),
                                    count,
                                },
                            ))
                        }
                        29 => {
                            let variable = reader.read_u16()? as usize;
                            reader.skip(2); // padding
                            let value = reader.read_i32()?;
                            let opcode = decode_operator_type(reader.read_u8()?)?;
                            reader.skip(3); // padding
                            conditions.push(AipCondition::Variable(
                                AipVariableType::Ai,
                                variable,
                                opcode,
                                value,
                            ));
                        }
                        30 => {
                            let target = reader.read_u8()?;
                            reader.skip(3); // padding

                            if target == 0 {
                                conditions.push(AipCondition::IsAttackerClanMaster);
                            } else if target == 1 {
                                conditions.push(AipCondition::IsTargetClanMaster);
                            } else {
                                return Err(AipReadError::InvalidValue);
                            }
                        }
                        _ => {
                            warn!("Unimplemented AIP condition opcode: {:X}", opcode);
                            reader.skip(size_bytes - 8);
                        }
                    }
                    assert_eq!(
                        reader.position(),
                        condition_start_position + size_bytes,
                        "Unexpected number of bytes read for opcode {:X}",
                        opcode
                    );
                }

                let num_actions = reader.read_u32()?;
                let mut actions = Vec::new();

                for _ in 0..num_actions {
                    let action_start_position = reader.position();
                    let size_bytes = reader.read_u32()? as u64;
                    let opcode = reader.read_u32()?;

                    match opcode ^ 0x0B000000 {
                        1 => {
                            actions.push(AipAction::Stop);
                        }
                        2 => {
                            let emote = reader.read_u8()?;
                            reader.skip(3); // padding
                            actions.push(AipAction::Emote(emote));
                        }
                        3 => {
                            let string_id = reader.read_u32()? as usize;
                            actions.push(AipAction::Say(string_id));
                        }
                        4 => {
                            let distance = reader.read_i32()?;
                            let move_mode = if reader.read_u8()? != 0 {
                                AipMoveMode::Run
                            } else {
                                AipMoveMode::Walk
                            };
                            reader.skip(3); // padding

                            actions.push(AipAction::MoveRandomDistance(
                                AipMoveOrigin::CurrentPosition,
                                move_mode,
                                distance * 100,
                            ));
                        }
                        5 => {
                            let distance = reader.read_i32()?;
                            let move_mode = if reader.read_u8()? != 0 {
                                AipMoveMode::Run
                            } else {
                                AipMoveMode::Walk
                            };
                            reader.skip(3); // padding

                            actions.push(AipAction::MoveRandomDistance(
                                AipMoveOrigin::Spawn,
                                move_mode,
                                distance * 100,
                            ));
                        }
                        6 => {
                            let move_mode = if reader.read_u8()? != 0 {
                                AipMoveMode::Run
                            } else {
                                AipMoveMode::Walk
                            };
                            reader.skip(3); // padding

                            actions.push(AipAction::MoveRandomDistance(
                                AipMoveOrigin::FindChar,
                                move_mode,
                                200,
                            ));
                        }
                        7 => {
                            let distance = reader.read_i32()?;
                            let ability_type = decode_ability_type(reader.read_u8()?)?;
                            let stat_amount = if reader.read_u8()? != 0 {
                                AipAttackNearbyStat::Lowest
                            } else {
                                AipAttackNearbyStat::Highest
                            };
                            reader.skip(2); // padding

                            actions.push(AipAction::AttackNearbyEntityByStat(
                                distance * 100,
                                ability_type,
                                stat_amount,
                            ));
                        }
                        8 => {
                            actions.push(AipAction::SpecialAttack);
                        }
                        9 => {
                            let distance = reader.read_i32()?;
                            let move_mode = if reader.read_u8()? != 0 {
                                AipMoveMode::Run
                            } else {
                                AipMoveMode::Walk
                            };
                            reader.skip(3); // padding

                            actions
                                .push(AipAction::MoveDistanceFromTarget(move_mode, distance * 100));
                        }
                        10 => {
                            let npc_id = reader.read_u16()? as i32;
                            reader.skip(2); // padding

                            actions.push(AipAction::TransformNpc(npc_id));
                        }
                        11 => {
                            let npc_id = reader.read_u16()? as i32;
                            reader.skip(2); // padding

                            actions.push(AipAction::SpawnNpc(
                                npc_id,
                                150,
                                AipSpawnNpcOrigin::CurrentPosition,
                                false,
                            ));
                        }
                        12 => {
                            let distance = reader.read_i32()?;
                            let count = reader.read_i32()? as usize;
                            actions
                                .push(AipAction::NearbyAlliesAttackTarget(count, distance, None));
                        }
                        13 => actions.push(AipAction::AttackNearChar),
                        14 => actions.push(AipAction::AttackFindChar),
                        15 => {
                            let distance = reader.read_i32()?;
                            actions.push(AipAction::NearbyAlliesSameNpcAttackTarget(distance));
                        }
                        16 => actions.push(AipAction::AttackAttacker),
                        17 => {
                            let distance = reader.read_i32()?;
                            actions.push(AipAction::RunAway(distance));
                        }
                        18 => {
                            let mut items = Vec::new();
                            for _ in 0..5 {
                                let value = reader.read_u16()? as u32;
                                if let Ok(item) = ItemReference::from_base1000(value) {
                                    items.push(item);
                                }
                            }
                            reader.skip(2); // padding

                            actions.push(AipAction::DropRandomItem(items));
                        }
                        19 => {
                            let npc_id = reader.read_u16()? as AipNpcId;
                            let count = reader.read_u16()? as usize;
                            let distance = reader.read_i32()?;

                            actions.push(AipAction::NearbyAlliesAttackTarget(
                                count,
                                distance,
                                Some(npc_id),
                            ));
                        }
                        20 => actions.push(AipAction::AttackNearChar),
                        21 => {
                            let npc_id = reader.read_u16()? as i32;
                            let position = match reader.read_u8()? {
                                0 => AipSpawnNpcOrigin::CurrentPosition,
                                1 => AipSpawnNpcOrigin::AttackerPosition,
                                2 => AipSpawnNpcOrigin::TargetPosition,
                                _ => return Err(AipReadError::InvalidValue),
                            };
                            reader.skip(1); // padding
                            let distance = reader.read_i32()?;

                            actions.push(AipAction::SpawnNpc(npc_id, distance, position, false));
                        }
                        22 => { /* no-op */ }
                        23 => { /* no-op */ }
                        24 => actions.push(AipAction::KillSelf),
                        25 => {
                            let target = match reader.read_u8()? {
                                0 => AipSkillTarget::FindChar,
                                1 => AipSkillTarget::Target,
                                2 => AipSkillTarget::This,
                                3 => AipSkillTarget::NearChar,
                                _ => return Err(AipReadError::InvalidValue),
                            };
                            reader.skip(1); // padding
                            let skill_id = reader.read_u16()? as i32;
                            let motion_id = reader.read_u16()? as i32;
                            reader.skip(2); // padding

                            actions.push(AipAction::UseSkill(target, skill_id, motion_id));
                        }
                        26 => {
                            let variable = reader.read_u16()? as usize;
                            reader.skip(2); // padding
                            let value = reader.read_i32()?;
                            let operator = decode_result_operator_type(reader.read_u8()?)?;
                            reader.skip(3); // padding

                            actions.push(AipAction::SetVariable(
                                AipVariableType::LocalNpcObject,
                                variable,
                                operator,
                                value,
                            ));
                        }
                        27 => {
                            let variable = reader.read_u16()? as usize;
                            reader.skip(2); // padding
                            let value = reader.read_i32()?;
                            let operator = decode_result_operator_type(reader.read_u8()?)?;
                            reader.skip(3); // padding

                            actions.push(AipAction::SetVariable(
                                AipVariableType::World,
                                variable,
                                operator,
                                value,
                            ));
                        }
                        28 => {
                            let variable = reader.read_u16()? as usize;
                            reader.skip(2); // padding
                            let value = reader.read_i32()?;
                            let operator = decode_result_operator_type(reader.read_u8()?)?;
                            reader.skip(3); // padding

                            actions.push(AipAction::SetVariable(
                                AipVariableType::Economy,
                                variable,
                                operator,
                                value,
                            ));
                        }
                        29 => {
                            let message_type = match reader.read_u8()? {
                                0 => AipMessageType::Say,
                                1 => AipMessageType::Shout,
                                2 => AipMessageType::Announce,
                                _ => return Err(AipReadError::InvalidValue),
                            };
                            reader.skip(3); // padding
                            let string_id = reader.read_u32()? as usize;

                            actions.push(AipAction::Message(message_type, string_id));
                        }
                        30 => actions.push(AipAction::MoveNearOwner),
                        31 => {
                            let quest_trigger = reader.read_u16_length_string()?;
                            reader.set_position(action_start_position + size_bytes); // padding

                            actions.push(AipAction::DoQuestTrigger(quest_trigger.to_string()));
                        }
                        32 => actions.push(AipAction::AttackOwnerTarget),
                        33 => {
                            let zone_id = reader.read_u16()? as AipZoneId;
                            let value = reader.read_u8()? != 0;
                            reader.skip(1); // padding

                            actions.push(AipAction::SetPvpFlag(
                                if zone_id == 0 { None } else { Some(zone_id) },
                                value,
                            ));
                        }
                        34 => {
                            let zone_id = reader.read_u16()? as AipZoneId;
                            let value = match reader.read_u8()? {
                                0 => AipMonsterSpawnState::Disabled,
                                1 => AipMonsterSpawnState::Enabled,
                                2 => AipMonsterSpawnState::Toggle,
                                _ => return Err(AipReadError::InvalidValue),
                            };
                            reader.skip(1); // padding

                            actions.push(AipAction::SetMonsterSpawnState(
                                if zone_id == 0 { None } else { Some(zone_id) },
                                value,
                            ));
                        }
                        35 => {
                            let item = ItemReference::from_base1000(reader.read_u16()? as u32)
                                .map_err(|_| AipReadError::InvalidValue)?;
                            let count = reader.read_u16()? as usize;

                            actions.push(AipAction::GiveItemToOwner(item, count));
                        }
                        36 => {
                            let variable = reader.read_u16()? as usize;
                            reader.skip(2); // padding
                            let value = reader.read_i32()?;
                            let operator = decode_result_operator_type(reader.read_u8()?)?;
                            reader.skip(3); // padding

                            actions.push(AipAction::SetVariable(
                                AipVariableType::Ai,
                                variable,
                                operator,
                                value,
                            ));
                        }
                        37 => {
                            let npc_id = reader.read_u16()? as i32;
                            let is_owner = reader.read_u8()? != 0;
                            reader.skip(1); // padding

                            actions.push(AipAction::SpawnNpc(
                                npc_id,
                                150,
                                AipSpawnNpcOrigin::CurrentPosition,
                                is_owner,
                            ));
                        }
                        38 => {
                            let npc_id = reader.read_u16()? as i32;
                            let position = match reader.read_u8()? {
                                0 => AipSpawnNpcOrigin::CurrentPosition,
                                1 => AipSpawnNpcOrigin::AttackerPosition,
                                2 => AipSpawnNpcOrigin::TargetPosition,
                                _ => return Err(AipReadError::InvalidValue),
                            };
                            reader.skip(1); // padding
                            let distance = reader.read_i32()?;
                            let is_owner = reader.read_u8()? != 0;
                            reader.skip(3); // padding

                            actions.push(AipAction::SpawnNpc(npc_id, distance, position, is_owner));
                        }
                        _ => {
                            warn!("Unimplemented AIP action opcode: {:#X}", opcode);
                            reader.skip(size_bytes - 8);
                        }
                    }
                    assert_eq!(
                        reader.position(),
                        action_start_position + size_bytes,
                        "Unexpected number of bytes read for opcode {:X}",
                        opcode
                    );
                }

                events.push(AipEvent {
                    name: event_name.to_string(),
                    conditions,
                    actions,
                });
            }

            triggers.push(Some(AipTrigger {
                name: trigger_name.to_string(),
                events,
            }));
        }

        Ok(Self {
            idle_trigger_interval,
            damage_trigger_new_target_chance,
            trigger_on_created: triggers.get_mut(0).and_then(|x| x.take()),
            trigger_on_idle: triggers.get_mut(1).and_then(|x| x.take()),
            trigger_on_attack_move: triggers.get_mut(2).and_then(|x| x.take()),
            trigger_on_damaged: triggers.get_mut(3).and_then(|x| x.take()),
            trigger_on_kill: triggers.get_mut(4).and_then(|x| x.take()),
            trigger_on_dead: triggers.get_mut(5).and_then(|x| x.take()),
        })
    }
}
