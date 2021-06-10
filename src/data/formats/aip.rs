use std::{collections::HashMap, ops::Range, time::Duration};

use super::reader::{FileReader, ReadError};

pub enum AipAbilityType {
    Level,
    Attack,
    Defence,
    Resistance,
    HealthPoints,
    Charm,
}

fn decode_ability_type(value: u32) -> Option<AipAbilityType> {
    match value {
        0 => Some(AipAbilityType::Level),
        1 => Some(AipAbilityType::Attack),
        2 => Some(AipAbilityType::Defence),
        3 => Some(AipAbilityType::Resistance),
        4 => Some(AipAbilityType::HealthPoints),
        5 => Some(AipAbilityType::Charm),
        _ => None,
    }
}

#[derive(Copy, Clone)]
pub enum AipOperatorType {
    Equals,
    GreaterThan,
    GreaterThanEqual,
    LessThan,
    LessThanEqual,
    NotEqual,
}

#[derive(Clone, Copy)]
pub enum AipHaveStatusTarget {
    This,
    Target,
}

#[derive(Clone, Copy)]
pub enum AipHaveStatusType {
    Good,
    Bad,
    Any,
}

pub struct AipConditionCountNearbyEntities {
    pub distance: u32,
    pub is_allied: bool,
    pub level_diff_range: Range<i32>,
    pub count: u32,
}

pub struct AipConditionTargetNearbyEntity {
    pub distance: u32,
    pub is_allied: bool,
    pub level_diff_range: Range<i32>,
}

pub enum AipCondition {
    DamageReceived(AipOperatorType, u32),
    DamageGiven(AipOperatorType, u32),
    CountNearbyEntities(AipConditionCountNearbyEntities),
    DistanceFromSpawn(AipOperatorType, u32),
    DistanceFromTarget(AipOperatorType, u32),
    TargetAbilityValue(AipOperatorType, AipAbilityType, u32),
    HealthPercent(AipOperatorType, u32),
    Random(AipOperatorType, Range<i32>, i32),
    TargetNearbyEntity(AipConditionTargetNearbyEntity),
    IsAttackerCurrentTarget,
    CompareAttackerAndTargetAbilityValue(AipOperatorType, AipAbilityType),
    NoTargetAndCompareAttackerAbilityValue(AipOperatorType, AipAbilityType, i32),
    IsDaytime(bool),
    HasStatusEffect(AipHaveStatusTarget, AipHaveStatusType, bool),
}

#[derive(Clone, Copy)]
pub enum AipMoveMode {
    Walk,
    Run,
}

#[derive(Clone, Copy)]
pub enum AipAttackNearbyStat {
    Lowest,
    Highest,
}

#[derive(Clone, Copy)]
pub enum AipMoveOrigin {
    Spawn,
    CurrentPosition,
}

pub enum AipAction {
    Stop,
    Emote(u8),
    Say(usize),
    MoveRandomDistance(AipMoveOrigin, AipMoveMode, i32),
    AttackNearbyEntityByStat(i32, AipAbilityType, AipAttackNearbyStat),
    SpecialAttack,
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
                    println!("AIP condition opcode: {:#X}", opcode);

                    match opcode ^ 0x04000000 {
                        2 => {
                            let damage = reader.read_u32()?;
                            let is_give_damage = reader.read_u8()? != 0;
                            reader.skip(3); // padding

                            if is_give_damage {
                                conditions.push(AipCondition::DamageGiven(
                                    AipOperatorType::GreaterThanEqual,
                                    damage,
                                ));
                            } else {
                                conditions.push(AipCondition::DamageReceived(
                                    AipOperatorType::GreaterThanEqual,
                                    damage,
                                ));
                            }
                        }
                        3 => {
                            let distance = reader.read_u32()?;
                            let is_allied = reader.read_u8()? > 0;
                            reader.skip(1); // padding
                            let min_level_diff = reader.read_i16()? as i32;
                            let max_level_diff = reader.read_i16()? as i32;
                            let count = reader.read_u16()? as u32;

                            conditions.push(AipCondition::CountNearbyEntities(
                                AipConditionCountNearbyEntities {
                                    distance,
                                    is_allied,
                                    level_diff_range: min_level_diff..max_level_diff,
                                    count,
                                },
                            ))
                        }
                        4 => {
                            let distance = reader.read_u32()?;
                            conditions.push(AipCondition::DistanceFromSpawn(
                                AipOperatorType::GreaterThanEqual,
                                distance * 100,
                            ));
                        }
                        5 => {
                            let distance = reader.read_u32()?;
                            let is_lte = reader.read_u8()? != 0;
                            reader.skip(3); // padding

                            if is_lte {
                                conditions.push(AipCondition::DistanceFromTarget(
                                    AipOperatorType::LessThanEqual,
                                    distance * 100,
                                ));
                            } else {
                                conditions.push(AipCondition::DistanceFromTarget(
                                    AipOperatorType::GreaterThanEqual,
                                    distance * 100,
                                ));
                            }
                        }
                        6 => {
                            let ability_type = reader.read_u32()?;
                            let value = reader.read_u32()?;
                            let is_lte = reader.read_u8()? != 0;
                            reader.skip(3); // padding

                            if let Some(ability_type) = decode_ability_type(ability_type) {
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
                        }
                        7 => {
                            let value = reader.read_u32()?;
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
                            let distance = reader.read_u32()?;
                            let min_level_diff = reader.read_i16()? as i32;
                            let max_level_diff = reader.read_i16()? as i32;
                            let is_allied = reader.read_u8()? != 0;
                            reader.skip(3); // padding

                            conditions.push(AipCondition::TargetNearbyEntity(
                                AipConditionTargetNearbyEntity {
                                    distance: distance * 100,
                                    is_allied,
                                    level_diff_range: min_level_diff..max_level_diff,
                                },
                            ))
                        }
                        10 => {
                            conditions.push(AipCondition::IsAttackerCurrentTarget);
                        }
                        11 => {
                            let ability_type = reader.read_u8()? as u32;
                            let is_lt = reader.read_u8()? != 0;
                            reader.skip(2); // padding

                            if let Some(ability_type) = decode_ability_type(ability_type) {
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
                        }
                        12 => {
                            let ability_type = reader.read_u8()? as u32;
                            reader.skip(3); // padding
                            let value = reader.read_i32()?;
                            let is_lte = reader.read_u8()? != 0;
                            reader.skip(3); // padding

                            if let Some(ability_type) = decode_ability_type(ability_type) {
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
                        // 18 and 27 have distance *= 100
                        _ => {
                            println!("Unimplemented AIP condition opcode: {:X}", opcode);
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
                    println!("AIP action opcode: {:#X}", opcode);
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
                                AipMoveOrigin::CurrentPosition,
                                move_mode,
                                200,
                            ));
                        }
                        7 => {
                            let distance = reader.read_i32()?;
                            let ability_type = reader.read_u8()? as u32;
                            let stat_amount = if reader.read_u8()? != 0 {
                                AipAttackNearbyStat::Lowest
                            } else {
                                AipAttackNearbyStat::Highest
                            };
                            reader.skip(2); // padding

                            if let Some(ability_type) = decode_ability_type(ability_type) {
                                actions.push(AipAction::AttackNearbyEntityByStat(
                                    distance * 100,
                                    ability_type,
                                    stat_amount,
                                ));
                            }
                        }
                        8 => {
                            actions.push(AipAction::SpecialAttack);
                        }
                        _ => {
                            println!("Unimplemented AIP action opcode: {:#X}", opcode);
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
