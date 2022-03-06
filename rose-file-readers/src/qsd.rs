use anyhow::anyhow;
use log::warn;
use std::{
    collections::HashMap,
    num::{NonZeroU8, NonZeroUsize},
    ops::RangeInclusive,
    time::Duration,
};

use crate::{reader::RoseFileReader, types::Vec2, RoseFile};

#[derive(Copy, Clone, Debug)]
pub enum QsdVariableType {
    Variable,
    Switch,
    Timer,
    Episode,
    Job,
    Planet,
    Union,
}

fn decode_variable_type(value: u16) -> Result<QsdVariableType, anyhow::Error> {
    match value {
        0x0000 => Ok(QsdVariableType::Variable),
        0x0100 => Ok(QsdVariableType::Switch),
        0x0200 => Ok(QsdVariableType::Timer),
        0x0300 => Ok(QsdVariableType::Episode),
        0x0400 => Ok(QsdVariableType::Job),
        0x0500 => Ok(QsdVariableType::Planet),
        0x0600 => Ok(QsdVariableType::Union),
        invalid => Err(anyhow!("Invalid QsdVariableType {}", invalid)),
    }
}

#[derive(Copy, Clone, Debug)]
pub enum QsdConditionOperator {
    Equals,
    GreaterThan,
    GreaterThanEqual,
    LessThan,
    LessThanEqual,
    NotEqual,
}

fn decode_condition_operator(value: u8) -> Result<QsdConditionOperator, anyhow::Error> {
    match value {
        0 => Ok(QsdConditionOperator::Equals),
        1 => Ok(QsdConditionOperator::GreaterThan),
        2 => Ok(QsdConditionOperator::GreaterThanEqual),
        3 => Ok(QsdConditionOperator::LessThan),
        4 => Ok(QsdConditionOperator::LessThanEqual),
        10 => Ok(QsdConditionOperator::NotEqual),
        invalid => Err(anyhow!("Invalid QsdConditionOperator {}", invalid)),
    }
}

pub type QsdAbilityType = NonZeroUsize;
pub type QsdClanLevel = i32;
pub type QsdClanPosition = usize;
pub type QsdClanPoints = i32;
pub type QsdDistance = i32;
pub type QsdEquipmentIndex = NonZeroUsize;
pub type QsdQuestId = usize;
pub type QsdSkillId = usize;
pub type QsdQuestSwitchId = usize;
pub type QsdQuestSwitchGroupId = usize;
pub type QsdEventId = usize;
pub type QsdNpcId = usize;
pub type QsdZoneId = usize;
pub type QsdVariableId = usize;
pub type QsdMoney = i32;
pub type QsdTeamNumber = usize;
pub type QsdServerChannelId = usize;
pub type QsdEquationId = usize;
pub type QsdStringId = usize;
pub type QsdItemBase1000 = NonZeroUsize;

#[derive(Copy, Clone, Debug)]
pub enum QsdObjectType {
    Npc,
    Event,
    Owner,
}

#[derive(Debug)]
pub struct QsdConditionQuestVariable {
    pub variable_type: QsdVariableType,
    pub variable_id: usize,
    pub operator: QsdConditionOperator,
    pub value: i32,
}

#[derive(Debug)]
pub struct QsdConditionQuestItem {
    pub item: Option<QsdItemBase1000>,
    pub equipment_index: Option<QsdEquipmentIndex>,
    pub required_count: u32,
    pub operator: QsdConditionOperator,
}

#[derive(Debug)]
pub struct QsdConditionCheckParty {
    pub is_leader: bool,
    pub level_operator: QsdConditionOperator,
    pub level: i32,
}

#[derive(Debug)]
pub struct QsdConditionObjectVariable {
    pub object_type: QsdObjectType,
    pub variable_id: usize,
    pub operator: QsdConditionOperator,
    pub value: i32,
}

#[derive(Debug)]
pub struct QsdConditionSelectEventObject {
    pub zone: QsdZoneId,
    pub chunk: Vec2<usize>,
    pub event_id: QsdEventId,
}

#[derive(Debug)]
pub struct QsdConditionWeekDayTime {
    pub week_day: u8,
    pub day_minutes_range: RangeInclusive<i32>,
}

#[derive(Debug)]
pub struct QsdConditionMonthDayTime {
    pub month_day: Option<NonZeroU8>,
    pub day_minutes_range: RangeInclusive<i32>,
}

#[derive(Debug)]
pub enum QsdCondition {
    SelectQuest(QsdQuestId),
    QuestVariable(Vec<QsdConditionQuestVariable>),
    AbilityValue(Vec<(QsdAbilityType, QsdConditionOperator, i32)>),
    QuestItems(Vec<QsdConditionQuestItem>),
    Party(QsdConditionCheckParty),
    Position(QsdZoneId, Vec2<f32>, QsdDistance),
    WorldTime(RangeInclusive<u32>),
    HasSkill(RangeInclusive<QsdSkillId>, bool),
    RandomPercent(RangeInclusive<u8>),
    ObjectVariable(QsdConditionObjectVariable),
    SelectEventObject(QsdConditionSelectEventObject),
    SelectNpc(QsdNpcId),
    QuestSwitch(QsdQuestSwitchId, bool),
    PartyMemberCount(RangeInclusive<usize>),
    ObjectZoneTime(QsdObjectType, RangeInclusive<u32>),
    CompareNpcVariables(
        (QsdNpcId, QsdVariableId),
        QsdConditionOperator,
        (QsdNpcId, QsdVariableId),
    ),
    MonthDayTime(QsdConditionMonthDayTime),
    WeekDayTime(QsdConditionWeekDayTime),
    TeamNumber(RangeInclusive<QsdTeamNumber>),
    ObjectDistance(QsdObjectType, QsdDistance),
    ServerChannelNumber(RangeInclusive<QsdServerChannelId>),
    InClan(bool),
    ClanPosition(QsdConditionOperator, QsdClanPosition),
    ClanPointContribution(QsdConditionOperator, QsdClanPoints),
    ClanLevel(QsdConditionOperator, QsdClanLevel),
    ClanPoints(QsdConditionOperator, QsdClanPoints),
    ClanMoney(QsdConditionOperator, QsdMoney),
    ClanMemberCount(QsdConditionOperator, usize),
    HasClanSkill(RangeInclusive<QsdSkillId>, bool),
}

#[derive(Copy, Clone, Debug)]
pub enum QsdRewardOperator {
    Set,
    Add,
    Subtract,
    Zero,
    One,
}

fn decode_reward_operator(value: u8) -> Result<QsdRewardOperator, anyhow::Error> {
    match value {
        5 => Ok(QsdRewardOperator::Set),
        6 => Ok(QsdRewardOperator::Add),
        7 => Ok(QsdRewardOperator::Subtract),
        8 => Ok(QsdRewardOperator::Zero),
        9 => Ok(QsdRewardOperator::One),
        invalid => Err(anyhow!("Invalid QsdRewardOperator {}", invalid)),
    }
}

#[derive(Copy, Clone, Debug)]
pub enum QsdRewardTarget {
    Player,
    Party,
}

#[derive(Debug)]
pub enum QsdRewardQuestAction {
    RemoveSelected,
    Add(QsdQuestId),
    ChangeSelectedIdKeepData(QsdQuestId),
    ChangeSelectedIdResetData(QsdQuestId),
    Select(QsdQuestId),
}

#[derive(Debug)]
pub struct QsdRewardQuestVariable {
    pub variable_type: QsdVariableType,
    pub variable_id: usize,
    pub operator: QsdRewardOperator,
    pub value: i32,
}

#[derive(Debug)]
pub struct QsdRewardCalculatedItem {
    pub equation: usize,
    pub value: i32,
    pub item: QsdItemBase1000,
    pub gem: Option<QsdItemBase1000>,
}

pub type QsdHealthPercent = u8;
pub type QsdManaPercent = u8;

#[derive(Copy, Clone, Debug)]
pub enum QsdRewardSpawnMonsterLocation {
    Owner,
    Npc,
    Event,
    Position(QsdZoneId, Vec2<f32>),
}

#[derive(Debug)]
pub struct QsdRewardSpawnMonster {
    pub npc: QsdNpcId,
    pub count: usize,
    pub location: QsdRewardSpawnMonsterLocation,
    pub distance: QsdDistance,
    pub team_number: QsdTeamNumber,
}

#[derive(Debug)]
pub struct QsdRewardObjectVariable {
    pub object_type: QsdObjectType,
    pub variable_id: usize,
    pub operator: QsdRewardOperator,
    pub value: i32,
}

#[derive(Copy, Clone, Debug)]
pub enum QsdRewardNpcMessageType {
    Chat,
    Shout,
    Announce,
}

#[derive(Copy, Clone, Debug)]
pub enum QsdRewardSetTeamNumberSource {
    Unique,
    Clan,
    Party,
}

#[derive(Copy, Clone, Debug)]
pub enum QsdRewardMonsterSpawnState {
    Disabled,
    Enabled,
    Toggle,
}

#[derive(Debug)]
pub enum QsdReward {
    Quest(QsdRewardQuestAction),
    AddItem(QsdRewardTarget, QsdItemBase1000, usize),
    RemoveItem(QsdRewardTarget, QsdItemBase1000, usize),
    QuestVariable(Vec<QsdRewardQuestVariable>),
    AbilityValue(Vec<(QsdAbilityType, QsdRewardOperator, i32)>),
    CalculatedExperiencePoints(QsdRewardTarget, QsdEquationId, i32),
    CalculatedMoney(QsdRewardTarget, QsdEquationId, i32),
    CalculatedItem(QsdRewardTarget, QsdRewardCalculatedItem),
    SetHealthManaPercent(QsdRewardTarget, QsdHealthPercent, QsdManaPercent),
    Teleport(QsdRewardTarget, QsdZoneId, Vec2<f32>),
    SpawnMonster(QsdRewardSpawnMonster),
    Trigger(String),
    ResetBasicStats,
    ObjectVariable(QsdRewardObjectVariable),
    NpcMessage(QsdRewardNpcMessageType, QsdStringId),
    TriggerAfterDelayForObject(QsdObjectType, Duration, String),
    AddSkill(QsdSkillId),
    RemoveSkill(QsdSkillId),
    SetQuestSwitch(QsdQuestSwitchId, bool),
    ClearSwitchGroup(QsdQuestSwitchGroupId),
    ClearAllSwitches,
    FormatAnnounceMessage(QsdStringId, Vec<(QsdNpcId, QsdVariableId)>),
    TriggerForZoneTeam(QsdZoneId, QsdTeamNumber, String),
    SetTeamNumber(QsdRewardSetTeamNumberSource),
    SetRevivePosition(Vec2<f32>),
    SetMonsterSpawnState(QsdZoneId, QsdRewardMonsterSpawnState),
    ClanLevel(QsdRewardOperator, QsdClanLevel),
    ClanMoney(QsdRewardOperator, QsdMoney),
    ClanPoints(QsdRewardOperator, QsdClanPoints),
    AddClanSkill(QsdSkillId),
    RemoveClanSkill(QsdSkillId),
    ClanPointContribution(QsdRewardOperator, QsdClanPoints),
    TeleportNearbyClanMembers(QsdDistance, QsdZoneId, Vec2<f32>),
    CallLuaFunction(String),
    ResetSkills,
}

pub struct QsdTrigger {
    pub name: String,
    pub conditions: Vec<QsdCondition>,
    pub rewards: Vec<QsdReward>,
    pub next_trigger_name: Option<String>,
}

pub struct QsdFile {
    pub triggers: HashMap<String, QsdTrigger>,
}

impl RoseFile for QsdFile {
    type ReadOptions = ();

    fn read(mut reader: RoseFileReader, _: &Self::ReadOptions) -> Result<Self, anyhow::Error> {
        let _file_version = reader.read_u32()?;
        let group_count = reader.read_u32()?;
        let _filename = reader.read_u16_length_string()?;
        let mut triggers = HashMap::new();

        for _ in 0..group_count {
            let trigger_count = reader.read_u32()?;
            let _group_name = reader.read_u16_length_string()?;
            let mut previous_trigger_name = None;

            for _ in 0..trigger_count {
                let check_next = reader.read_u8()? != 0;
                let condition_count = reader.read_u32()?;
                let reward_count = reader.read_u32()?;
                let trigger_name = reader.read_u16_length_string()?;
                let mut conditions = Vec::new();
                let mut rewards = Vec::new();

                for _ in 0..condition_count {
                    let start_position = reader.position();
                    let size_bytes = reader.read_u32()? as u64;
                    let opcode = reader.read_u32()? & 0x0ffff;

                    match opcode {
                        0 => {
                            let quest_id = reader.read_u32()? as QsdQuestId;

                            conditions.push(QsdCondition::SelectQuest(quest_id));
                        }
                        1 | 2 => {
                            let data_count = reader.read_u32()?;
                            let mut variables = Vec::new();
                            for _ in 0..data_count {
                                let variable_id = reader.read_u16()? as usize;
                                let variable_type = decode_variable_type(reader.read_u16()?)?;
                                let value = reader.read_i16()? as i32;
                                let operator = decode_condition_operator(reader.read_u8()?)?;
                                reader.skip(1); // padding
                                variables.push(QsdConditionQuestVariable {
                                    variable_type,
                                    variable_id,
                                    operator,
                                    value,
                                });
                            }

                            conditions.push(QsdCondition::QuestVariable(variables));
                        }
                        3 => {
                            let data_count = reader.read_u32()?;
                            let mut variables = Vec::new();
                            for _ in 0..data_count {
                                let ability_type = QsdAbilityType::new(reader.read_u32()? as usize)
                                    .ok_or_else(|| {
                                        anyhow!(
                                            "Invalid QsdCondition::AbilityValue ability_type: 0"
                                        )
                                    })?;
                                let value = reader.read_i32()?;
                                let operator = decode_condition_operator(reader.read_u8()?)?;
                                reader.skip(3); // padding
                                variables.push((ability_type, operator, value));
                            }

                            conditions.push(QsdCondition::AbilityValue(variables));
                        }
                        4 => {
                            let data_count = reader.read_u32()?;
                            let mut items = Vec::new();
                            for _ in 0..data_count {
                                let item = QsdItemBase1000::new(reader.read_u32()? as usize);
                                let equipment_index =
                                    QsdEquipmentIndex::new(reader.read_u32()? as usize);
                                let required_count = reader.read_u32()?;
                                let operator = decode_condition_operator(reader.read_u8()?)?;
                                reader.skip(3); // padding
                                items.push(QsdConditionQuestItem {
                                    item,
                                    equipment_index,
                                    required_count,
                                    operator,
                                });
                            }

                            conditions.push(QsdCondition::QuestItems(items));
                        }
                        5 => {
                            let is_leader = reader.read_u8()? != 0;
                            reader.skip(3);
                            let level = reader.read_i32()?;
                            let level_operator = if reader.read_u8()? != 0 {
                                QsdConditionOperator::LessThan
                            } else {
                                QsdConditionOperator::GreaterThanEqual
                            };
                            reader.skip(3);

                            conditions.push(QsdCondition::Party(QsdConditionCheckParty {
                                is_leader,
                                level_operator,
                                level,
                            }));
                        }
                        6 => {
                            let zone = reader.read_u32()? as QsdZoneId;
                            let x = reader.read_u32()? as f32;
                            let y = reader.read_u32()? as f32;
                            let _z = reader.read_u32()?;
                            let distance = reader.read_u32()? as QsdDistance;

                            conditions.push(QsdCondition::Position(zone, Vec2 { x, y }, distance));
                        }
                        7 => {
                            let start_time = reader.read_u32()?;
                            let end_time = reader.read_u32()?;

                            conditions.push(QsdCondition::WorldTime(start_time..=end_time));
                        }
                        8 => {
                            let value = reader.read_i32()?;
                            let operator = decode_condition_operator(reader.read_u8()?)?;
                            reader.skip(3); // padding

                            conditions.push(QsdCondition::QuestVariable(vec![
                                QsdConditionQuestVariable {
                                    variable_type: QsdVariableType::Timer,
                                    variable_id: 0,
                                    operator,
                                    value,
                                },
                            ]));
                        }
                        9 => {
                            let start_skill_id = reader.read_u32()? as QsdSkillId;
                            let end_skill_id = reader.read_u32()? as QsdSkillId;
                            let have = reader.read_u8()? != 0;
                            reader.skip(3); // padding
                            conditions
                                .push(QsdCondition::HasSkill(start_skill_id..=end_skill_id, have));
                        }
                        10 => {
                            let start_percent = reader.read_u8()?;
                            let end_percent = reader.read_u8()?;
                            reader.skip(2); // padding

                            conditions
                                .push(QsdCondition::RandomPercent(start_percent..=end_percent));
                        }
                        11 => {
                            let object_type = if reader.read_u8()? == 0 {
                                QsdObjectType::Npc
                            } else {
                                QsdObjectType::Event
                            };
                            reader.skip(1); // padding
                            let variable_id = reader.read_u16()? as usize;
                            let value = reader.read_i32()?;
                            let operator = decode_condition_operator(reader.read_u8()?)?;
                            reader.skip(3); // padding

                            conditions.push(QsdCondition::ObjectVariable(
                                QsdConditionObjectVariable {
                                    object_type,
                                    variable_id,
                                    operator,
                                    value,
                                },
                            ));
                        }
                        12 => {
                            let zone = reader.read_u32()? as QsdZoneId;
                            let chunk_x = reader.read_u32()? as usize;
                            let chunk_y = reader.read_u32()? as usize;
                            let event_id = reader.read_u32()? as QsdEventId;

                            conditions.push(QsdCondition::SelectEventObject(
                                QsdConditionSelectEventObject {
                                    zone,
                                    chunk: Vec2 {
                                        x: chunk_x,
                                        y: chunk_y,
                                    },
                                    event_id,
                                },
                            ));
                        }
                        13 => {
                            let npc_id = reader.read_u32()? as QsdNpcId;

                            conditions.push(QsdCondition::SelectNpc(npc_id));
                        }
                        14 => {
                            let quest_id = reader.read_u16()? as QsdQuestId;
                            let value = reader.read_u8()? != 0;
                            reader.skip(1); // padding

                            conditions.push(QsdCondition::QuestSwitch(quest_id, value));
                        }
                        15 => {
                            let start_count = reader.read_u16()? as usize;
                            let end_count = reader.read_u16()? as usize;

                            conditions
                                .push(QsdCondition::PartyMemberCount(start_count..=end_count));
                        }
                        16 => {
                            let object_type = match reader.read_u8()? {
                                0 => QsdObjectType::Npc,
                                1 => QsdObjectType::Event,
                                2 => QsdObjectType::Owner,
                                invalid => {
                                    return Err(anyhow!(
                                        "Invalid QsdCondition::ObjectZoneTime object_type {}",
                                        invalid
                                    ))
                                }
                            };
                            reader.skip(3); // padding
                            let start_time = reader.read_u32()?;
                            let end_time = reader.read_u32()?;

                            conditions.push(QsdCondition::ObjectZoneTime(
                                object_type,
                                start_time..=end_time,
                            ));
                        }
                        17 => {
                            let npc_id_1 = reader.read_u32()? as QsdNpcId;
                            let variable_id_1 = reader.read_u16()? as QsdVariableId;
                            reader.skip(2); // padding
                            let npc_id_2 = reader.read_u32()? as QsdNpcId;
                            let variable_id_2 = reader.read_u16()? as QsdVariableId;
                            reader.skip(2); // padding
                            let operator = decode_condition_operator(reader.read_u8()?)?;
                            reader.skip(3); // padding

                            conditions.push(QsdCondition::CompareNpcVariables(
                                (npc_id_1, variable_id_1),
                                operator,
                                (npc_id_2, variable_id_2),
                            ));
                        }
                        18 => {
                            let day = reader.read_u8()?;
                            let hour_min = reader.read_u8()?;
                            let minute_min = reader.read_u8()?;
                            let hour_max = reader.read_u8()?;
                            let minute_max = reader.read_u8()?;
                            reader.skip(3); // padding

                            conditions.push(QsdCondition::MonthDayTime(QsdConditionMonthDayTime {
                                month_day: NonZeroU8::new(day),
                                day_minutes_range: (hour_min as i32 * 60 + minute_min as i32)
                                    ..=(hour_max as i32 * 60 + minute_max as i32),
                            }));
                        }
                        19 => {
                            let day = reader.read_u8()?;
                            let hour_min = reader.read_u8()?;
                            let minute_min = reader.read_u8()?;
                            let hour_max = reader.read_u8()?;
                            let minute_max = reader.read_u8()?;
                            reader.skip(3); // padding

                            conditions.push(QsdCondition::WeekDayTime(QsdConditionWeekDayTime {
                                week_day: day,
                                day_minutes_range: (hour_min as i32 * 60 + minute_min as i32)
                                    ..=(hour_max as i32 * 60 + minute_max as i32),
                            }));
                        }
                        20 => {
                            let start = reader.read_u32()? as QsdTeamNumber;
                            let end = reader.read_u32()? as QsdTeamNumber;

                            conditions.push(QsdCondition::TeamNumber(start..=end));
                        }
                        21 => {
                            let object_type = match reader.read_u8()? {
                                0 => QsdObjectType::Npc,
                                1 => QsdObjectType::Event,
                                invalid => {
                                    return Err(anyhow!(
                                        "Invalid QsdCondition::ObjectDistance object_type {}",
                                        invalid
                                    ))
                                }
                            };
                            reader.skip(3); // padding
                            let distance = reader.read_u32()? as QsdDistance;

                            conditions.push(QsdCondition::ObjectDistance(object_type, distance));
                        }
                        22 => {
                            let start = reader.read_u16()? as QsdServerChannelId;
                            let end = reader.read_u16()? as QsdServerChannelId;

                            conditions.push(QsdCondition::ServerChannelNumber(start..=end));
                        }
                        23 => {
                            let in_clan = reader.read_u8()? != 0;
                            reader.skip(3); // padding

                            conditions.push(QsdCondition::InClan(in_clan));
                        }
                        24 => {
                            let value = reader.read_u16()? as QsdClanPosition;
                            let operator = decode_condition_operator(reader.read_u8()?)?;
                            reader.skip(1); // padding

                            conditions.push(QsdCondition::ClanPosition(operator, value));
                        }
                        25 => {
                            let value = reader.read_u16()? as QsdClanPoints;
                            let operator = decode_condition_operator(reader.read_u8()?)?;
                            reader.skip(1); // padding

                            conditions.push(QsdCondition::ClanPointContribution(operator, value));
                        }
                        26 => {
                            let value = reader.read_u16()? as QsdClanLevel;
                            let operator = decode_condition_operator(reader.read_u8()?)?;
                            reader.skip(1); // padding

                            conditions.push(QsdCondition::ClanLevel(operator, value));
                        }
                        27 => {
                            let value = reader.read_u16()? as QsdClanPoints;
                            let operator = decode_condition_operator(reader.read_u8()?)?;
                            reader.skip(1); // padding

                            conditions.push(QsdCondition::ClanPoints(operator, value));
                        }
                        28 => {
                            let value = reader.read_i32()? as QsdMoney;
                            let operator = decode_condition_operator(reader.read_u8()?)?;
                            reader.skip(3); // padding

                            conditions.push(QsdCondition::ClanMoney(operator, value));
                        }
                        29 => {
                            let value = reader.read_u16()? as usize;
                            let operator = decode_condition_operator(reader.read_u8()?)?;
                            reader.skip(1); // padding

                            conditions.push(QsdCondition::ClanMemberCount(operator, value));
                        }
                        30 => {
                            let start = reader.read_u16()? as QsdSkillId;
                            let end = reader.read_u16()? as QsdSkillId;
                            let have = reader.read_u8()? != 0;
                            reader.skip(3); // padding

                            conditions.push(QsdCondition::HasClanSkill(start..=end, have));
                        }
                        _ => {
                            warn!("Unimplemented QSD condition opcode: {:X}", opcode);
                            reader.skip(size_bytes - 8);
                        }
                    }
                    assert_eq!(
                        reader.position(),
                        start_position + size_bytes,
                        "Unexpected number of bytes read for opcode {:X}",
                        opcode
                    );
                }
                let conditions = conditions;

                for _ in 0..reward_count {
                    let start_position = reader.position();
                    let size_bytes = reader.read_u32()? as u64;
                    let opcode = reader.read_u32()? & 0x0ffff;

                    match opcode {
                        0 => {
                            let quest_id = reader.read_u32()? as QsdQuestId;
                            let action = match reader.read_u8()? {
                                0 => QsdRewardQuestAction::RemoveSelected,
                                1 => QsdRewardQuestAction::Add(quest_id),
                                2 => QsdRewardQuestAction::ChangeSelectedIdKeepData(quest_id),
                                3 => QsdRewardQuestAction::ChangeSelectedIdResetData(quest_id),
                                4 => QsdRewardQuestAction::Select(quest_id),
                                invalid => {
                                    return Err(anyhow!(
                                        "Invalid QsdReward::Quest action {}",
                                        invalid
                                    ))
                                }
                            };
                            reader.skip(3); // padding

                            rewards.push(QsdReward::Quest(action));
                        }
                        1 => {
                            let item_base1000 =
                                QsdItemBase1000::new(reader.read_u32()? as usize)
                                    .ok_or_else(|| anyhow!("Invalid QsdReward::Item item: 0"))?;
                            let add_or_remove = reader.read_u8()? != 0;
                            reader.skip(1); // padding
                            let count = reader.read_u16()? as usize;
                            let target = if reader.read_u8()? == 0 {
                                QsdRewardTarget::Player
                            } else {
                                QsdRewardTarget::Party
                            };
                            reader.skip(3); // padding

                            if add_or_remove {
                                rewards.push(QsdReward::AddItem(target, item_base1000, count));
                            } else {
                                rewards.push(QsdReward::RemoveItem(target, item_base1000, count));
                            }
                        }
                        2 | 4 => {
                            let data_count = reader.read_u32()?;
                            let mut variables = Vec::new();
                            for _ in 0..data_count {
                                let variable_id = reader.read_u16()? as usize;
                                let variable_type = decode_variable_type(reader.read_u16()?)?;
                                let value = reader.read_i16()? as i32;
                                let operator = decode_reward_operator(reader.read_u8()?)?;
                                reader.skip(1); // padding
                                variables.push(QsdRewardQuestVariable {
                                    variable_type,
                                    variable_id,
                                    operator,
                                    value,
                                });
                            }

                            rewards.push(QsdReward::QuestVariable(variables));
                        }
                        3 => {
                            let data_count = reader.read_u32()?;
                            let mut variables = Vec::new();
                            for _ in 0..data_count {
                                let ability_type = QsdAbilityType::new(reader.read_u32()? as usize)
                                    .ok_or_else(|| {
                                        anyhow!("Invalid QsdReward::AbilityValue ability_type: 0")
                                    })?;
                                let value = reader.read_i32()?;
                                let operator = decode_reward_operator(reader.read_u8()?)?;
                                reader.skip(3); // padding
                                variables.push((ability_type, operator, value));
                            }

                            rewards.push(QsdReward::AbilityValue(variables));
                        }
                        5 => {
                            let reward_type = reader.read_u8()?;
                            let equation = reader.read_u8()? as QsdEquationId;
                            reader.skip(2);
                            let value = reader.read_i32()?;
                            let item = reader.read_u32()? as usize;
                            let target = if reader.read_u8()? == 0 {
                                QsdRewardTarget::Player
                            } else {
                                QsdRewardTarget::Party
                            };
                            reader.skip(1);
                            let gem = QsdItemBase1000::new(reader.read_u16()? as u32 as usize);

                            match reward_type {
                                0 => {
                                    rewards.push(QsdReward::CalculatedExperiencePoints(
                                        target, equation, value,
                                    ));
                                }
                                1 => {
                                    rewards
                                        .push(QsdReward::CalculatedMoney(target, equation, value));
                                }
                                2 => {
                                    rewards.push(QsdReward::CalculatedItem(
                                        target,
                                        QsdRewardCalculatedItem {
                                            equation,
                                            value,
                                            item: QsdItemBase1000::new(item).ok_or_else(|| {
                                                anyhow!(
                                                    "Invalid QsdReward::CalculatedReward item: 0"
                                                )
                                            })?,
                                            gem,
                                        },
                                    ));
                                }
                                invalid => {
                                    return Err(anyhow!(
                                        "Invalid QsdReward::CalculatedReward reward_type {}",
                                        invalid
                                    ))
                                }
                            }
                        }
                        6 => {
                            let health_percent = reader.read_i32()? as u8;
                            let mana_percent = reader.read_i32()? as u8;
                            let target = if reader.read_u8()? == 0 {
                                QsdRewardTarget::Player
                            } else {
                                QsdRewardTarget::Party
                            };
                            reader.skip(3);

                            rewards.push(QsdReward::SetHealthManaPercent(
                                target,
                                health_percent,
                                mana_percent,
                            ));
                        }
                        7 => {
                            let zone = reader.read_i32()? as QsdZoneId;
                            let x = reader.read_i32()? as f32;
                            let y = reader.read_i32()? as f32;
                            let target = if reader.read_u8()? == 0 {
                                QsdRewardTarget::Player
                            } else {
                                QsdRewardTarget::Party
                            };
                            reader.skip(3);

                            rewards.push(QsdReward::Teleport(target, zone, Vec2 { x, y }));
                        }
                        8 => {
                            let npc = reader.read_u32()? as QsdNpcId;
                            let count = reader.read_u32()? as usize;
                            let location_type = reader.read_u8()?;
                            reader.skip(3);
                            let zone = reader.read_u32()? as QsdZoneId;
                            let x = reader.read_i32()? as f32;
                            let y = reader.read_i32()? as f32;
                            let distance = reader.read_i32()? as QsdDistance;
                            let team_number = reader.read_u32()? as QsdTeamNumber;

                            let location = match location_type {
                                0 => QsdRewardSpawnMonsterLocation::Owner,
                                1 => QsdRewardSpawnMonsterLocation::Npc,
                                2 => QsdRewardSpawnMonsterLocation::Event,
                                3 => QsdRewardSpawnMonsterLocation::Position(zone, Vec2 { x, y }),
                                invalid => {
                                    return Err(anyhow!(
                                        "Invalid QsdReward::SpawnMonster location {}",
                                        invalid
                                    ))
                                }
                            };

                            rewards.push(QsdReward::SpawnMonster(QsdRewardSpawnMonster {
                                npc,
                                count,
                                location,
                                distance,
                                team_number,
                            }));
                        }
                        9 => {
                            let trigger = reader.read_u16_length_string()?;
                            reader.set_position(start_position + size_bytes); // padding

                            rewards.push(QsdReward::Trigger(trigger.to_string()));
                        }
                        10 => {
                            rewards.push(QsdReward::ResetBasicStats);
                        }
                        11 => {
                            let object_type = if reader.read_u8()? == 0 {
                                QsdObjectType::Npc
                            } else {
                                QsdObjectType::Event
                            };
                            reader.skip(1); // padding
                            let variable_id = reader.read_u16()? as usize;
                            let value = reader.read_i32()?;
                            let operator = decode_reward_operator(reader.read_u8()?)?;
                            reader.skip(3); // padding

                            rewards.push(QsdReward::ObjectVariable(QsdRewardObjectVariable {
                                object_type,
                                variable_id,
                                operator,
                                value,
                            }));
                        }
                        12 => {
                            let message_type = match reader.read_u8()? {
                                0 => QsdRewardNpcMessageType::Chat,
                                1 => QsdRewardNpcMessageType::Shout,
                                2 => QsdRewardNpcMessageType::Announce,
                                invalid => {
                                    return Err(anyhow!(
                                        "Invalid QsdReward::NpcMessage message_type {}",
                                        invalid
                                    ))
                                }
                            };
                            reader.skip(3); // padding
                            let string_id = reader.read_u32()? as QsdStringId;

                            rewards.push(QsdReward::NpcMessage(message_type, string_id));
                        }
                        13 => {
                            let object_type = if reader.read_u8()? == 0 {
                                QsdObjectType::Npc
                            } else {
                                QsdObjectType::Event
                            };
                            reader.skip(3); // padding
                            let delay = Duration::from_secs(reader.read_u32()? as u64);
                            let trigger = reader.read_u16_length_string()?;
                            reader.set_position(start_position + size_bytes); // padding

                            rewards.push(QsdReward::TriggerAfterDelayForObject(
                                object_type,
                                delay,
                                trigger.to_string(),
                            ));
                        }
                        14 => {
                            let add_or_remove = reader.read_u8()? != 0;
                            reader.skip(3); // padding
                            let skill_id = reader.read_u32()? as QsdSkillId;

                            if add_or_remove {
                                rewards.push(QsdReward::AddSkill(skill_id));
                            } else {
                                rewards.push(QsdReward::RemoveSkill(skill_id));
                            }
                        }
                        15 => {
                            let switch_id = reader.read_u16()? as QsdQuestSwitchId;
                            let value = reader.read_u8()? != 0;
                            reader.skip(1); // padding

                            rewards.push(QsdReward::SetQuestSwitch(switch_id, value));
                        }
                        16 => {
                            let switch_group_id = reader.read_u16()? as QsdQuestSwitchGroupId;
                            reader.skip(2); // padding

                            rewards.push(QsdReward::ClearSwitchGroup(switch_group_id));
                        }
                        17 => rewards.push(QsdReward::ClearAllSwitches),
                        18 => {
                            let string_id = reader.read_u32()? as QsdStringId;
                            let data_count = reader.read_u16()?;
                            let mut variables = Vec::new();
                            for _ in 0..data_count {
                                let npc_id = reader.read_u16()? as QsdNpcId;
                                let variable_id = reader.read_u16()? as QsdVariableId;
                                variables.push((npc_id, variable_id));
                            }
                            reader.skip(2); // padding

                            rewards.push(QsdReward::FormatAnnounceMessage(string_id, variables));
                        }
                        19 => {
                            let zone = reader.read_u16()? as QsdZoneId;
                            let team_number = reader.read_u16()? as QsdTeamNumber;
                            let trigger = reader.read_u16_length_string()?;
                            reader.set_position(start_position + size_bytes); // padding

                            rewards.push(QsdReward::TriggerForZoneTeam(
                                zone,
                                team_number,
                                trigger.to_string(),
                            ));
                        }
                        20 => {
                            let source = match reader.read_u8()? {
                                0 => QsdRewardSetTeamNumberSource::Unique,
                                1 => QsdRewardSetTeamNumberSource::Clan,
                                2 => QsdRewardSetTeamNumberSource::Party,
                                invalid => {
                                    return Err(anyhow!(
                                        "Invalid QsdReward::SetTeamNumber source {}",
                                        invalid
                                    ))
                                }
                            };
                            reader.skip(3); // padding

                            rewards.push(QsdReward::SetTeamNumber(source));
                        }
                        21 => {
                            let x = reader.read_i32()? as f32;
                            let y = reader.read_i32()? as f32;

                            rewards.push(QsdReward::SetRevivePosition(Vec2 { x, y }));
                        }
                        22 => {
                            let zone = reader.read_u16()? as QsdZoneId;
                            let state = match reader.read_u8()? {
                                0 => QsdRewardMonsterSpawnState::Disabled,
                                1 => QsdRewardMonsterSpawnState::Enabled,
                                2 => QsdRewardMonsterSpawnState::Toggle,
                                invalid => {
                                    return Err(anyhow!(
                                        "Invalid QsdReward::SetMonsterSpawnState state {}",
                                        invalid
                                    ))
                                }
                            };
                            reader.skip(1); // padding

                            rewards.push(QsdReward::SetMonsterSpawnState(zone, state));
                        }
                        23 => {
                            rewards.push(QsdReward::ClanLevel(QsdRewardOperator::Add, 1));
                        }
                        24 => {
                            let value = reader.read_i32()?;
                            let operator = decode_reward_operator(reader.read_u8()?)?;
                            reader.skip(3); // padding
                            rewards.push(QsdReward::ClanMoney(operator, value));
                        }
                        25 => {
                            let value = reader.read_i32()?;
                            let operator = decode_reward_operator(reader.read_u8()?)?;
                            reader.skip(3); // padding
                            rewards.push(QsdReward::ClanPoints(operator, value));
                        }
                        26 => {
                            let skill_id = reader.read_u16()? as QsdSkillId;
                            let add_or_remove = reader.read_u8()? != 0;
                            reader.skip(1); // padding

                            if add_or_remove {
                                rewards.push(QsdReward::AddClanSkill(skill_id));
                            } else {
                                rewards.push(QsdReward::RemoveClanSkill(skill_id));
                            }
                        }
                        27 => {
                            let value = reader.read_i32()?;
                            let operator = decode_reward_operator(reader.read_u8()?)?;
                            reader.skip(3); // padding
                            rewards.push(QsdReward::ClanPointContribution(operator, value));
                        }
                        28 => {
                            let distance = reader.read_i32()? as QsdDistance;
                            let zone = reader.read_u16()? as QsdZoneId;
                            reader.skip(2); // padding
                            let x = reader.read_i32()? as f32;
                            let y = reader.read_i32()? as f32;

                            rewards.push(QsdReward::TeleportNearbyClanMembers(
                                distance,
                                zone,
                                Vec2 { x, y },
                            ));
                        }
                        29 => {
                            let function = reader.read_u16_length_string()?;
                            reader.set_position(start_position + size_bytes); // padding

                            rewards.push(QsdReward::CallLuaFunction(function.to_string()));
                        }
                        30 => {
                            rewards.push(QsdReward::ResetSkills);
                        }
                        _ => {
                            warn!("Unimplemented QSD action opcode: {:X}", opcode);
                            reader.skip(size_bytes - 8);
                        }
                    }
                    assert_eq!(
                        reader.position(),
                        start_position + size_bytes,
                        "Unexpected number of bytes read for opcode {:X}",
                        opcode
                    );
                }
                let rewards = rewards;

                triggers.insert(
                    trigger_name.to_string(),
                    QsdTrigger {
                        name: trigger_name.to_string(),
                        conditions,
                        rewards,
                        next_trigger_name: None,
                    },
                );

                if let Some(previous_trigger_name) = previous_trigger_name {
                    triggers
                        .get_mut(&previous_trigger_name)
                        .unwrap()
                        .next_trigger_name = Some(trigger_name.to_string());
                }

                if check_next {
                    previous_trigger_name = Some(trigger_name.to_string());
                } else {
                    previous_trigger_name = None;
                }
            }
        }

        Ok(QsdFile { triggers })
    }
}
