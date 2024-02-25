use anyhow::anyhow;
use log::warn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    num::{NonZeroU8, NonZeroUsize},
    ops::RangeInclusive,
    time::Duration,
};

use crate::{reader::RoseFileReader, RoseFile};

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

#[derive(Copy, Clone, Debug, JsonSchema, Serialize, Deserialize)]
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

fn encode_variable_type(value: QsdVariableType) -> u16 {
    match value {
        QsdVariableType::Variable => 0x0000,
        QsdVariableType::Switch => 0x0100,
        QsdVariableType::Timer => 0x0200,
        QsdVariableType::Episode => 0x0300,
        QsdVariableType::Job => 0x0400,
        QsdVariableType::Planet => 0x0500,
        QsdVariableType::Union => 0x0600,
    }
}

#[derive(Copy, Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub enum QsdConditionOperator {
    #[serde(rename = "==")]
    Equals,
    #[serde(rename = ">")]
    GreaterThan,
    #[serde(rename = ">=")]
    GreaterThanEqual,
    #[serde(rename = "<")]
    LessThan,
    #[serde(rename = "<=")]
    LessThanEqual,
    #[serde(rename = "!=")]
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

fn encode_condition_operator(value: QsdConditionOperator) -> u8 {
    match value {
        QsdConditionOperator::Equals => 0,
        QsdConditionOperator::GreaterThan => 1,
        QsdConditionOperator::GreaterThanEqual => 2,
        QsdConditionOperator::LessThan => 3,
        QsdConditionOperator::LessThanEqual => 4,
        QsdConditionOperator::NotEqual => 10,
    }
}

#[derive(Copy, Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub enum QsdObjectType {
    SelectedNpc,
    SelectedEvent,
    QuestOwner,
}

#[derive(Debug, Copy, Clone, JsonSchema, Serialize, Deserialize)]
#[serde(rename = "Item")]
pub struct QsdItem {
    #[serde(rename = "id")]
    pub item_number: usize,
    #[serde(rename = "type")]
    pub item_type: usize,
}

impl QsdItem {
    pub fn to_sn(&self) -> usize {
        if self.item_number > 999 {
            self.item_number + self.item_type * 1000000
        } else {
            self.item_number + self.item_type * 1000
        }
    }

    pub fn from_sn(sn: usize) -> Option<QsdItem> {
        let (item_number, item_type) = if sn < 1000000 {
            (sn % 1000, sn / 1000)
        } else {
            (sn % 1000000, sn / 1000000)
        };

        if item_number == 0 || item_type == 0 {
            None
        } else {
            Some(QsdItem {
                item_number,
                item_type,
            })
        }
    }
}

#[derive(Copy, Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub enum QsdRewardOperator {
    #[serde(rename = "=")]
    Set,
    #[serde(rename = "+")]
    Add,
    #[serde(rename = "-")]
    Subtract,
    #[serde(rename = "0")]
    Zero,
    #[serde(rename = "1")]
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

fn encode_reward_operator(value: QsdRewardOperator) -> u8 {
    match value {
        QsdRewardOperator::Set => 5,
        QsdRewardOperator::Add => 6,
        QsdRewardOperator::Subtract => 7,
        QsdRewardOperator::Zero => 8,
        QsdRewardOperator::One => 9,
    }
}

#[derive(Copy, Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub enum QsdSpawnMonsterLocation {
    QuestOwner,
    SelectedNpc,
    SelectedEvent,
    Position { zone: QsdZoneId, x: f32, y: f32 },
}

#[derive(Copy, Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub enum QsdNpcMessageType {
    Chat,
    Shout,
    Announce,
}

#[derive(Copy, Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub enum QsdTeamNumberSource {
    Unique,
    Clan,
    Party,
}

fn get_true() -> bool {
    true
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_true(value: &bool) -> bool {
    *value
}

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "condition")]
pub enum QsdCondition {
    SelectQuest {
        id: QsdQuestId,
    },
    QuestVariable {
        variable_type: QsdVariableType,
        variable_id: usize,
        operator: QsdConditionOperator,
        value: i32,
    },
    AbilityValue {
        ability_type: QsdAbilityType,
        operator: QsdConditionOperator,
        value: i32,
    },
    QuestItem {
        item: Option<QsdItem>,
        #[serde(
            default = "Option::default",
            skip_serializing_if = "Option::<QsdEquipmentIndex>::is_none"
        )]
        equipment_index: Option<QsdEquipmentIndex>,
        required_count: u32,
        operator: QsdConditionOperator,
    },
    Party {
        is_leader: bool,
        level_operator: QsdConditionOperator,
        level: i32,
    },
    Position {
        zone: QsdZoneId,
        x: f32,
        y: f32,
        distance: QsdDistance,
    },
    WorldTime {
        range: RangeInclusive<u32>,
    },
    HasSkill {
        id: QsdSkillId,
        #[serde(default = "get_true", skip_serializing_if = "is_true")]
        has_skill: bool,
    },
    HasSkillInRange {
        range: RangeInclusive<QsdSkillId>,
        #[serde(default = "get_true", skip_serializing_if = "is_true")]
        has_skill: bool,
    },
    RandomPercent {
        range: RangeInclusive<u8>,
    },
    ObjectVariable {
        object: QsdObjectType,
        variable_id: usize,
        operator: QsdConditionOperator,
        value: i32,
    },
    SelectEventObject {
        zone: QsdZoneId,
        chunk_x: usize,
        chunk_y: usize,
        event_id: QsdEventId,
    },
    SelectNpc {
        id: QsdNpcId,
    },
    QuestSwitch {
        id: QsdQuestSwitchId,
        value: bool,
    },
    PartyMemberCount {
        range: RangeInclusive<usize>,
    },
    ObjectZoneTime {
        object: QsdObjectType,
        time_range: RangeInclusive<u32>,
    },
    CompareNpcVariables {
        npc_id_1: QsdNpcId,
        variable_id_1: QsdVariableId,
        operator: QsdConditionOperator,
        npc_id_2: QsdNpcId,
        variable_id_2: QsdVariableId,
    },
    MonthDayTime {
        month_day: Option<NonZeroU8>,
        day_minutes_range: RangeInclusive<i32>,
    },
    WeekDayTime {
        week_day: u8,
        day_minutes_range: RangeInclusive<i32>,
    },
    TeamNumber {
        range: RangeInclusive<QsdTeamNumber>,
    },
    ObjectDistance {
        object: QsdObjectType,
        distance: QsdDistance,
    },
    ServerChannelNumber {
        range: RangeInclusive<QsdServerChannelId>,
    },
    HasClan {
        has_clan: bool,
    },
    ClanPosition {
        operator: QsdConditionOperator,
        value: QsdClanPosition,
    },
    ClanPointContribution {
        operator: QsdConditionOperator,
        value: QsdClanPoints,
    },
    ClanLevel {
        operator: QsdConditionOperator,
        value: QsdClanLevel,
    },
    ClanPoints {
        operator: QsdConditionOperator,
        value: QsdClanPoints,
    },
    ClanMoney {
        operator: QsdConditionOperator,
        value: QsdMoney,
    },
    ClanMemberCount {
        operator: QsdConditionOperator,
        value: usize,
    },
    HasClanSkill {
        id: QsdSkillId,
        #[serde(default = "get_true", skip_serializing_if = "is_true")]
        has_skill: bool,
    },
    HasClanSkillInRange {
        range: RangeInclusive<QsdSkillId>,
        #[serde(default = "get_true", skip_serializing_if = "is_true")]
        has_skill: bool,
    },
}

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "reward")]
pub enum QsdReward {
    RemoveSelectedQuest,
    AddQuest {
        id: QsdQuestId,
    },
    ChangeSelectedQuest {
        id: QsdQuestId,
        keep_data: bool,
    },
    SelectQuest {
        id: QsdQuestId,
    },
    AddItem {
        item: QsdItem,
        quantity: usize,
    },
    RemoveItem {
        item: QsdItem,
        quantity: usize,
    },
    QuestVariable {
        variable_type: QsdVariableType,
        variable_id: usize,
        operator: QsdRewardOperator,
        value: i32,
    },
    AbilityValue {
        ability_type: QsdAbilityType,
        operator: QsdRewardOperator,
        value: i32,
    },
    CalculatedExperiencePoints {
        equation: QsdEquationId,
        value: i32,
    },
    CalculatedMoney {
        equation: QsdEquationId,
        value: i32,
    },
    CalculatedItem {
        equation: usize,
        value: i32,
        item: QsdItem,
        #[serde(
            default = "Option::default",
            skip_serializing_if = "Option::<NonZeroUsize>::is_none"
        )]
        gem: Option<NonZeroUsize>,
    },
    SetHealthManaPercent {
        health_percent: u8,
        mana_percent: u8,
    },
    Teleport {
        zone: QsdZoneId,
        x: u32,
        y: u32,
    },
    SpawnMonster {
        npc: QsdNpcId,
        count: usize,
        location: QsdSpawnMonsterLocation,
        distance: QsdDistance,
        team_number: QsdTeamNumber,
    },
    Trigger {
        name: String,
    },
    ResetBasicStats,
    ObjectVariable {
        object: QsdObjectType,
        variable_id: usize,
        operator: QsdRewardOperator,
        value: i32,
    },
    NpcMessage {
        message_type: QsdNpcMessageType,
        string_id: QsdStringId,
    },
    TriggerAfterDelay {
        object: QsdObjectType,
        delay: Duration,
        trigger: String,
    },
    AddSkill {
        id: QsdSkillId,
    },
    RemoveSkill {
        id: QsdSkillId,
    },
    SetQuestSwitch {
        id: QsdQuestSwitchId,
        value: bool,
    },
    ClearSwitchGroup {
        group: QsdQuestSwitchGroupId,
    },
    ClearAllSwitches,
    FormatAnnounceMessage {
        string_id: QsdStringId,
        variables: Vec<(QsdNpcId, QsdVariableId)>,
    },
    TriggerForZoneTeam {
        zone: QsdZoneId,
        team_number: QsdTeamNumber,
        trigger: String,
    },
    SetTeamNumber {
        source: QsdTeamNumberSource,
    },
    SetRevivePosition {
        x: f32,
        y: f32,
    },
    EnableMonsterSpawns {
        zone: QsdZoneId,
    },
    DisableMonsterSpawns {
        zone: QsdZoneId,
    },
    ToggleMonsterSpawns {
        zone: QsdZoneId,
    },
    ClanLevelIncrease,
    ClanMoney {
        operator: QsdRewardOperator,
        value: QsdMoney,
    },
    ClanPoints {
        operator: QsdRewardOperator,
        value: QsdClanPoints,
    },
    AddClanSkill {
        id: QsdSkillId,
    },
    RemoveClanSkill {
        id: QsdSkillId,
    },
    ClanPointContribution {
        operator: QsdRewardOperator,
        value: QsdClanPoints,
    },
    TeleportNearbyClanMembers {
        distance: QsdDistance,
        zone: QsdZoneId,
        x: f32,
        y: f32,
    },
    CallLuaFunction {
        name: String,
    },
    ResetSkills,
}

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
pub struct QsdTrigger {
    pub name: String,
    pub conditions: Vec<QsdCondition>,
    pub rewards: Vec<QsdReward>,
    pub next_trigger_name: Option<String>,
}

#[derive(Debug, Default, JsonSchema, Serialize, Deserialize)]
pub struct QsdFile {
    pub triggers: HashMap<String, QsdTrigger>,
}

#[derive(Copy, Clone, Debug)]
pub enum QsdGameVersion {
    Irose,
    Narose667,
}

#[derive(Copy, Clone, Debug)]
pub struct QsdReadOptions {
    pub game_version: QsdGameVersion,
}

impl Default for QsdReadOptions {
    fn default() -> Self {
        Self {
            game_version: QsdGameVersion::Irose,
        }
    }
}

impl RoseFile for QsdFile {
    type ReadOptions = QsdReadOptions;
    type WriteOptions = ();

    fn read(reader: RoseFileReader, options: &Self::ReadOptions) -> Result<Self, anyhow::Error> {
        match options.game_version {
            QsdGameVersion::Irose => Self::read_irose(reader),
            QsdGameVersion::Narose667 => Self::read_narose667(reader),
        }
    }
}

impl QsdFile {
    fn read_narose667(_reader: RoseFileReader) -> Result<Self, anyhow::Error> {
        // TODO: QsdFile::read_narose667
        Ok(QsdFile::default())
    }

    fn read_irose(mut reader: RoseFileReader) -> Result<Self, anyhow::Error> {
        let _file_version = reader.read_u32()?;
        let group_count = reader.read_u32()?;
        let _filename = reader.read_u16_length_string()?;
        let mut triggers = HashMap::new();

        for _ in 0..group_count {
            let trigger_count = reader.read_u32()?;
            let _group_name = reader.read_u16_length_string()?;
            let mut previous_trigger_name = None;

            for _ in 0..trigger_count {
                let (trigger_name, rewards, conditions, check_next) = read_trigger(&mut reader)?;

                {
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
        }

        Ok(QsdFile { triggers })
    }
}

pub mod editor_friendly {
    use crate::{
        writer::RoseFileWriter, QsdCondition, QsdGameVersion, QsdObjectType, QsdReadOptions,
        QsdReward, RoseFile, RoseFileReader,
    };
    use bytes::Buf;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use super::{
        encode_condition_operator, encode_reward_operator, encode_variable_type, is_false,
    };

    #[derive(Debug, JsonSchema, Serialize, Deserialize)]
    pub struct QsdTrigger {
        pub name: String,
        pub conditions: Vec<QsdCondition>,
        pub rewards: Vec<QsdReward>,
        #[serde(default = "bool::default", skip_serializing_if = "is_false")]
        pub check_next: bool,
    }

    #[derive(Debug, Default, JsonSchema, Serialize, Deserialize)]
    pub struct QsdGroup {
        pub name: String,
        pub triggers: Vec<QsdTrigger>,
    }

    #[derive(Debug, Default, JsonSchema, Serialize, Deserialize)]
    pub struct QsdFile {
        pub name: String,
        pub groups: Vec<QsdGroup>,
    }

    impl RoseFile for QsdFile {
        type ReadOptions = QsdReadOptions;
        type WriteOptions = ();

        fn read(
            reader: RoseFileReader,
            options: &Self::ReadOptions,
        ) -> Result<Self, anyhow::Error> {
            match options.game_version {
                QsdGameVersion::Irose => Self::read_irose(reader),
                QsdGameVersion::Narose667 => Self::read_narose667(reader),
            }
        }

        fn write(
            &self,
            writer: &mut RoseFileWriter,
            _options: &Self::WriteOptions,
        ) -> Result<(), anyhow::Error> {
            self.write_irose(writer)
        }
    }

    impl QsdFile {
        fn read_narose667(_reader: RoseFileReader) -> Result<Self, anyhow::Error> {
            // TODO: QsdFile::read_narose667
            Ok(QsdFile::default())
        }

        fn read_irose(mut reader: RoseFileReader) -> Result<Self, anyhow::Error> {
            let _file_version = reader.read_u32()?;
            let group_count = reader.read_u32()?;
            let filename = reader.read_u16_length_string()?;
            let mut groups = Vec::new();

            for _ in 0..group_count {
                let trigger_count = reader.read_u32()?;
                let group_name = reader.read_u16_length_string()?;
                let mut triggers = Vec::with_capacity(trigger_count as usize);

                for _ in 0..trigger_count {
                    let (trigger_name, rewards, conditions, check_next) =
                        crate::qsd::read_trigger(&mut reader)?;

                    triggers.push(QsdTrigger {
                        name: trigger_name.to_string(),
                        conditions,
                        rewards,
                        check_next,
                    });
                }

                groups.push(QsdGroup {
                    name: group_name.to_string(),
                    triggers,
                });
            }

            Ok(QsdFile {
                name: filename.to_string(),
                groups,
            })
        }

        fn write_irose(&self, writer: &mut RoseFileWriter) -> Result<(), anyhow::Error> {
            writer.write_u32(0); // unused: file version
            writer.write_u32(self.groups.len() as u32);
            writer.write_u16_length_string(&self.name);

            for group in self.groups.iter() {
                writer.write_u32(group.triggers.len() as u32);
                writer.write_u16_length_string(&group.name);

                for trigger in group.triggers.iter() {
                    writer.write_u8(u8::from(trigger.check_next));
                    writer.write_u32(trigger.conditions.len() as u32);
                    writer.write_u32(trigger.rewards.len() as u32);
                    writer.write_u16_length_string(&trigger.name);

                    for condition in trigger.conditions.iter() {
                        let pos_before = writer.buffer.len();
                        match condition {
                            &QsdCondition::SelectQuest { id } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(0);

                                writer.write_u32(id as u32);
                            }
                            &QsdCondition::QuestVariable {
                                variable_type,
                                variable_id,
                                operator,
                                value,
                            } => {
                                writer.write_u32(8 + 12);
                                writer.write_u32(1);

                                writer.write_u32(1); // data count
                                writer.write_u16(variable_id as u16);
                                writer.write_u16(encode_variable_type(variable_type));
                                writer.write_i16(value as i16);
                                writer.write_u8(encode_condition_operator(operator));
                                writer.write_padding(1);
                            }
                            &QsdCondition::AbilityValue {
                                ability_type,
                                operator,
                                value,
                            } => {
                                writer.write_u32(8 + 16);
                                writer.write_u32(3);

                                writer.write_u32(1); // data count
                                writer.write_u32(ability_type.get() as u32);
                                writer.write_i32(value);
                                writer.write_u8(encode_condition_operator(operator));
                                writer.write_padding(3);
                            }
                            &QsdCondition::QuestItem {
                                item,
                                equipment_index,
                                required_count,
                                operator,
                            } => {
                                writer.write_u32(8 + 20);
                                writer.write_u32(4);

                                writer.write_u32(1); // data count
                                writer.write_u32(item.map_or(0, |item| item.to_sn() as u32));
                                writer.write_u32(equipment_index.map_or(0, |v| v.get() as u32));
                                writer.write_u32(required_count);
                                writer.write_u8(encode_condition_operator(operator));
                                writer.write_padding(3);
                            }
                            &QsdCondition::Party {
                                is_leader,
                                level_operator,
                                level,
                            } => {
                                writer.write_u32(8 + 12);
                                writer.write_u32(5);

                                writer.write_u8(u8::from(is_leader));
                                writer.write_padding(3);
                                writer.write_i32(level);
                                writer.write_u8(encode_condition_operator(level_operator));
                                writer.write_padding(3);
                            }
                            &QsdCondition::Position {
                                zone,
                                x,
                                y,
                                distance,
                            } => {
                                writer.write_u32(8 + 20);
                                writer.write_u32(6);

                                writer.write_u32(zone as u32);
                                writer.write_u32(x as u32);
                                writer.write_u32(y as u32);
                                writer.write_u32(0); // z: unused
                                writer.write_i32(distance);
                            }
                            QsdCondition::WorldTime { range } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(7);

                                writer.write_u32(*range.start());
                                writer.write_u32(*range.end());
                            }
                            &QsdCondition::HasSkill { id, has_skill } => {
                                writer.write_u32(8 + 12);
                                writer.write_u32(9);

                                writer.write_u32(id as u32);
                                writer.write_u32(id as u32);
                                writer.write_u8(u8::from(has_skill));
                                writer.write_padding(3);
                            }
                            &QsdCondition::HasSkillInRange {
                                ref range,
                                has_skill,
                            } => {
                                writer.write_u32(8 + 12);
                                writer.write_u32(9);

                                writer.write_u32(*range.start() as u32);
                                writer.write_u32(*range.end() as u32);
                                writer.write_u8(u8::from(has_skill));
                                writer.write_padding(3);
                            }
                            QsdCondition::RandomPercent { range } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(10);

                                writer.write_u8(*range.start());
                                writer.write_u8(*range.end());
                                writer.write_padding(2);
                            }
                            &QsdCondition::ObjectVariable {
                                object,
                                variable_id,
                                operator,
                                value,
                            } => {
                                writer.write_u32(8 + 12);
                                writer.write_u32(11);

                                writer.write_u8(match object {
                                    QsdObjectType::SelectedNpc => 0,
                                    QsdObjectType::SelectedEvent => 1,
                                    QsdObjectType::QuestOwner => unimplemented!(),
                                });
                                writer.write_padding(1);
                                writer.write_u16(variable_id as u16);
                                writer.write_i32(value);
                                writer.write_u8(encode_condition_operator(operator));
                                writer.write_padding(3);
                            }
                            &QsdCondition::SelectEventObject {
                                zone,
                                chunk_x,
                                chunk_y,
                                event_id,
                            } => {
                                writer.write_u32(8 + 16);
                                writer.write_u32(12);

                                writer.write_u32(zone as u32);
                                writer.write_u32(chunk_x as u32);
                                writer.write_u32(chunk_y as u32);
                                writer.write_u32(event_id as u32);
                            }
                            &QsdCondition::SelectNpc { id } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(13);

                                writer.write_u32(id as u32);
                            }
                            &QsdCondition::QuestSwitch { id, value } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(14);

                                writer.write_u16(id as u16);
                                writer.write_u8(u8::from(value));
                                writer.write_padding(1);
                            }
                            QsdCondition::PartyMemberCount { range } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(15);

                                writer.write_u16(*range.start() as u16);
                                writer.write_u16(*range.end() as u16);
                            }
                            &QsdCondition::ObjectZoneTime {
                                object,
                                ref time_range,
                            } => {
                                writer.write_u32(8 + 12);
                                writer.write_u32(16);

                                writer.write_u8(match object {
                                    QsdObjectType::SelectedNpc => 0,
                                    QsdObjectType::SelectedEvent => 1,
                                    QsdObjectType::QuestOwner => 2,
                                });
                                writer.write_padding(3);
                                writer.write_u32(*time_range.start());
                                writer.write_u32(*time_range.end());
                            }
                            &QsdCondition::CompareNpcVariables {
                                npc_id_1,
                                variable_id_1,
                                operator,
                                npc_id_2,
                                variable_id_2,
                            } => {
                                writer.write_u32(8 + 20);
                                writer.write_u32(17);

                                writer.write_u32(npc_id_1 as u32);
                                writer.write_u16(variable_id_1 as u16);
                                writer.write_padding(2);
                                writer.write_u32(npc_id_2 as u32);
                                writer.write_u16(variable_id_2 as u16);
                                writer.write_padding(2);
                                writer.write_u8(encode_condition_operator(operator));
                                writer.write_padding(3);
                            }
                            &QsdCondition::MonthDayTime {
                                month_day,
                                ref day_minutes_range,
                            } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(18);

                                let hour_min = day_minutes_range.start() / 60;
                                let minute_min = day_minutes_range.start() % 60;
                                let hour_max = day_minutes_range.end() / 60;
                                let minute_max = day_minutes_range.end() % 60;

                                writer.write_u8(month_day.map_or(0, |day| day.get()));
                                writer.write_u8(hour_min as u8);
                                writer.write_u8(minute_min as u8);
                                writer.write_u8(hour_max as u8);
                                writer.write_u8(minute_max as u8);
                                writer.write_padding(3);
                            }
                            &QsdCondition::WeekDayTime {
                                week_day,
                                ref day_minutes_range,
                            } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(19);

                                let hour_min = day_minutes_range.start() / 60;
                                let minute_min = day_minutes_range.start() % 60;
                                let hour_max = day_minutes_range.end() / 60;
                                let minute_max = day_minutes_range.end() % 60;

                                writer.write_u8(week_day);
                                writer.write_u8(hour_min as u8);
                                writer.write_u8(minute_min as u8);
                                writer.write_u8(hour_max as u8);
                                writer.write_u8(minute_max as u8);
                                writer.write_padding(3);
                            }
                            QsdCondition::TeamNumber { range } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(20);

                                writer.write_u32(*range.start() as u32);
                                writer.write_u32(*range.end() as u32);
                            }
                            &QsdCondition::ObjectDistance { object, distance } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(21);

                                writer.write_u8(match object {
                                    QsdObjectType::SelectedNpc => 0,
                                    QsdObjectType::SelectedEvent => 1,
                                    QsdObjectType::QuestOwner => unimplemented!(),
                                });
                                writer.write_padding(3);
                                writer.write_i32(distance);
                            }
                            QsdCondition::ServerChannelNumber { range } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(22);

                                writer.write_u16(*range.start() as u16);
                                writer.write_u16(*range.end() as u16);
                            }
                            &QsdCondition::HasClan { has_clan } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(23);

                                writer.write_u8(u8::from(has_clan));
                                writer.write_padding(3);
                            }
                            &QsdCondition::ClanPosition { operator, value } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(24);

                                writer.write_u16(value as u16);
                                writer.write_u8(encode_condition_operator(operator));
                                writer.write_padding(1);
                            }
                            &QsdCondition::ClanPointContribution { operator, value } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(25);

                                writer.write_u16(value as u16);
                                writer.write_u8(encode_condition_operator(operator));
                                writer.write_padding(1);
                            }
                            &QsdCondition::ClanLevel { operator, value } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(26);

                                writer.write_u16(value as u16);
                                writer.write_u8(encode_condition_operator(operator));
                                writer.write_padding(1);
                            }
                            &QsdCondition::ClanPoints { operator, value } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(27);

                                writer.write_u16(value as u16);
                                writer.write_u8(encode_condition_operator(operator));
                                writer.write_padding(1);
                            }
                            &QsdCondition::ClanMoney { operator, value } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(28);

                                writer.write_i32(value);
                                writer.write_u8(encode_condition_operator(operator));
                                writer.write_padding(3);
                            }
                            &QsdCondition::ClanMemberCount { operator, value } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(29);

                                writer.write_u16(value as u16);
                                writer.write_u8(encode_condition_operator(operator));
                                writer.write_padding(1);
                            }
                            &QsdCondition::HasClanSkill { id, has_skill } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(30);

                                writer.write_u16(id as u16);
                                writer.write_u16(id as u16);
                                writer.write_u8(u8::from(has_skill));
                                writer.write_padding(3);
                            }
                            &QsdCondition::HasClanSkillInRange {
                                ref range,
                                has_skill,
                            } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(30);

                                writer.write_u16(*range.start() as u16);
                                writer.write_u16(*range.end() as u16);
                                writer.write_u8(u8::from(has_skill));
                                writer.write_padding(3);
                            }
                        }
                        let pos_after = writer.buffer.len();
                        let written_size =
                            (&writer.buffer[pos_before..pos_before + 4]).get_u32_le();
                        if (pos_after - pos_before) as u32 != written_size {
                            panic!("pos_after - pos_before != written_size, {} != {} for condition {:?}", pos_after - pos_before, written_size, condition);
                        }
                    }

                    for reward in trigger.rewards.iter() {
                        let pos_before = writer.buffer.len();
                        match reward {
                            QsdReward::RemoveSelectedQuest => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(0);

                                writer.write_u32(0);
                                writer.write_u8(0);
                                writer.write_padding(3);
                            }
                            &QsdReward::AddQuest { id } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(0);

                                writer.write_u32(id as u32);
                                writer.write_u8(1);
                                writer.write_padding(3);
                            }
                            &QsdReward::ChangeSelectedQuest { id, keep_data } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(0);

                                writer.write_u32(id as u32);
                                writer.write_u8(if keep_data { 2 } else { 3 });
                                writer.write_padding(3);
                            }
                            &QsdReward::SelectQuest { id } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(0);

                                writer.write_u32(id as u32);
                                writer.write_u8(4);
                                writer.write_padding(3);
                            }
                            &QsdReward::AddItem { item, quantity } => {
                                writer.write_u32(8 + 12);
                                writer.write_u32(1);

                                writer.write_u32(item.to_sn() as u32);
                                writer.write_u8(1);
                                writer.write_padding(1);
                                writer.write_u16(quantity as u16);
                                writer.write_u8(0);
                                writer.write_padding(3);
                            }
                            &QsdReward::RemoveItem { item, quantity } => {
                                writer.write_u32(8 + 12);
                                writer.write_u32(1);

                                writer.write_u32(item.to_sn() as u32);
                                writer.write_u8(0);
                                writer.write_padding(1);
                                writer.write_u16(quantity as u16);
                                writer.write_u8(0);
                                writer.write_padding(3);
                            }
                            &QsdReward::QuestVariable {
                                variable_type,
                                variable_id,
                                operator,
                                value,
                            } => {
                                writer.write_u32(8 + 16);
                                writer.write_u32(4);

                                writer.write_u32(1);
                                writer.write_u16(variable_id as u16);
                                writer.write_u16(encode_variable_type(variable_type));
                                writer.write_i16(value as i16);
                                writer.write_u8(encode_reward_operator(operator));
                                writer.write_padding(1);
                            }
                            &QsdReward::AbilityValue {
                                ability_type,
                                operator,
                                value,
                            } => {
                                writer.write_u32(8 + 16);
                                writer.write_u32(3);

                                writer.write_u32(1);
                                writer.write_u32(ability_type.get() as u32);
                                writer.write_i32(value);
                                writer.write_u8(encode_reward_operator(operator));
                                writer.write_padding(3);
                            }
                            &QsdReward::CalculatedExperiencePoints { equation, value } => {
                                writer.write_u32(8 + 16);
                                writer.write_u32(5);

                                writer.write_u8(0);
                                writer.write_u8(equation as u8);
                                writer.write_padding(2);
                                writer.write_i32(value);
                                writer.write_u32(0);
                                writer.write_u8(0);
                                writer.write_padding(1);
                                writer.write_u16(0);
                            }
                            &QsdReward::CalculatedMoney { equation, value } => {
                                writer.write_u32(8 + 16);
                                writer.write_u32(5);

                                writer.write_u8(1);
                                writer.write_u8(equation as u8);
                                writer.write_padding(2);
                                writer.write_i32(value);
                                writer.write_u32(0);
                                writer.write_u8(0);
                                writer.write_padding(1);
                                writer.write_u16(0);
                            }
                            &QsdReward::CalculatedItem {
                                equation,
                                value,
                                item,
                                gem,
                            } => {
                                writer.write_u32(8 + 16);
                                writer.write_u32(5);

                                writer.write_u8(2);
                                writer.write_u8(equation as u8);
                                writer.write_padding(2);
                                writer.write_i32(value);
                                writer.write_u32(item.to_sn() as u32);
                                writer.write_u8(0);
                                writer.write_padding(1);
                                writer.write_u16(gem.map_or(0, |gem| gem.get() as u16));
                            }
                            &QsdReward::SetHealthManaPercent {
                                health_percent,
                                mana_percent,
                            } => {
                                writer.write_u32(8 + 12);
                                writer.write_u32(6);

                                writer.write_u32(health_percent as u32);
                                writer.write_u32(mana_percent as u32);
                                writer.write_u8(0);
                                writer.write_padding(3);
                            }
                            &QsdReward::Teleport { zone, x, y } => {
                                writer.write_u32(8 + 16);
                                writer.write_u32(7);

                                writer.write_u32(zone as u32);
                                writer.write_u32(x);
                                writer.write_u32(y);
                                writer.write_u8(0);
                                writer.write_padding(3);
                            }
                            &QsdReward::SpawnMonster {
                                npc,
                                count,
                                location,
                                distance,
                                team_number,
                            } => {
                                writer.write_u32(8 + 32);
                                writer.write_u32(8);

                                let mut x_ = 0;
                                let mut y_ = 0;
                                let mut zone_ = 0;
                                writer.write_u32(npc as u32);
                                writer.write_u32(count as u32);
                                writer.write_u8(match location {
                                    crate::QsdSpawnMonsterLocation::QuestOwner => 0,
                                    crate::QsdSpawnMonsterLocation::SelectedNpc => 1,
                                    crate::QsdSpawnMonsterLocation::SelectedEvent => 2,
                                    crate::QsdSpawnMonsterLocation::Position { zone, x, y } => {
                                        x_ = x as i32;
                                        y_ = y as i32;
                                        zone_ = zone as u32;
                                        3
                                    }
                                });
                                writer.write_padding(3);
                                writer.write_u32(zone_);
                                writer.write_i32(x_);
                                writer.write_i32(y_);
                                writer.write_i32(distance);
                                writer.write_u32(team_number as u32);
                            }
                            QsdReward::Trigger { name } => {
                                let size = 8 + 2 + name.len();
                                let padding = 4 - (size % 4);

                                writer.write_u32((size + padding) as u32);
                                writer.write_u32(9);

                                writer.write_u16_length_string(name);
                                writer.write_padding(padding as u64);
                            }
                            &QsdReward::ResetBasicStats => {
                                writer.write_u32(8);
                                writer.write_u32(10);
                            }
                            &QsdReward::ObjectVariable {
                                object,
                                variable_id,
                                operator,
                                value,
                            } => {
                                writer.write_u32(8 + 12);
                                writer.write_u32(11);

                                writer.write_u8(match object {
                                    QsdObjectType::SelectedNpc => 0,
                                    QsdObjectType::SelectedEvent => 1,
                                    QsdObjectType::QuestOwner => unimplemented!(),
                                });
                                writer.write_padding(1);
                                writer.write_u16(variable_id as u16);
                                writer.write_i32(value);
                                writer.write_u8(encode_reward_operator(operator));
                                writer.write_padding(3);
                            }
                            &QsdReward::NpcMessage {
                                message_type,
                                string_id,
                            } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(12);

                                writer.write_u8(match message_type {
                                    crate::QsdNpcMessageType::Chat => 0,
                                    crate::QsdNpcMessageType::Shout => 1,
                                    crate::QsdNpcMessageType::Announce => 2,
                                });
                                writer.write_padding(3);
                                writer.write_u32(string_id as u32);
                            }
                            &QsdReward::TriggerAfterDelay {
                                object,
                                delay,
                                ref trigger,
                            } => {
                                let size = 8 + 8 + 2 + trigger.len();
                                let padding = 4 - (size % 4);

                                writer.write_u32((size + padding) as u32);
                                writer.write_u32(13);

                                writer.write_u8(match object {
                                    QsdObjectType::SelectedNpc => 0,
                                    QsdObjectType::SelectedEvent => 1,
                                    QsdObjectType::QuestOwner => unimplemented!(),
                                }); // Object: Selected Event
                                writer.write_padding(3);
                                writer.write_u32(delay.as_secs() as u32);
                                writer.write_u16_length_string(trigger);
                                writer.write_padding(padding as u64);
                            }
                            &QsdReward::AddSkill { id } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(14);

                                writer.write_u8(1);
                                writer.write_padding(3);
                                writer.write_u32(id as u32);
                            }
                            &QsdReward::RemoveSkill { id } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(14);

                                writer.write_u8(0);
                                writer.write_padding(3);
                                writer.write_u32(id as u32);
                            }
                            &QsdReward::SetQuestSwitch { id, value } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(15);

                                writer.write_u16(id as u16);
                                writer.write_u8(u8::from(value));
                                writer.write_padding(1);
                            }
                            &QsdReward::ClearSwitchGroup { group } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(16);

                                writer.write_u16(group as u16);
                                writer.write_padding(2);
                            }
                            &QsdReward::ClearAllSwitches => {
                                writer.write_u32(8);
                                writer.write_u32(17);
                            }
                            &QsdReward::FormatAnnounceMessage {
                                string_id,
                                ref variables,
                            } => {
                                writer.write_u32(8 + 8 + variables.len() as u32 * 4);
                                writer.write_u32(18);

                                writer.write_u32(string_id as u32);
                                writer.write_u16(variables.len() as u16);
                                for &(npc_id, variable_id) in variables.iter() {
                                    writer.write_u16(npc_id as u16);
                                    writer.write_u16(variable_id as u16);
                                }
                                writer.write_padding(2);
                            }
                            &QsdReward::TriggerForZoneTeam {
                                zone,
                                team_number,
                                ref trigger,
                            } => {
                                let size = 8 + 4 + 2 + trigger.len();
                                let padding = 4 - (size % 4);

                                writer.write_u32((size + padding) as u32);
                                writer.write_u32(19);

                                writer.write_u16(zone as u16);
                                writer.write_u16(team_number as u16);
                                writer.write_u16_length_string(trigger);
                                writer.write_padding(padding as u64);
                            }
                            &QsdReward::SetTeamNumber { source } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(20);

                                writer.write_u8(match source {
                                    crate::QsdTeamNumberSource::Unique => 0,
                                    crate::QsdTeamNumberSource::Clan => 1,
                                    crate::QsdTeamNumberSource::Party => 2,
                                });
                                writer.write_padding(3);
                            }
                            &QsdReward::SetRevivePosition { x, y } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(21);

                                writer.write_i32(x as i32);
                                writer.write_i32(y as i32);
                            }
                            &QsdReward::EnableMonsterSpawns { zone } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(22);

                                writer.write_u16(zone as u16);
                                writer.write_u8(1); // Enable
                                writer.write_padding(1);
                            }
                            &QsdReward::DisableMonsterSpawns { zone } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(22);

                                writer.write_u16(zone as u16);
                                writer.write_u8(0); // Disable
                                writer.write_padding(1);
                            }
                            &QsdReward::ToggleMonsterSpawns { zone } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(22);

                                writer.write_u16(zone as u16);
                                writer.write_u8(2); // Toggle
                                writer.write_padding(1);
                            }
                            &QsdReward::ClanLevelIncrease => {
                                writer.write_u32(8);
                                writer.write_u32(23);
                            }
                            &QsdReward::ClanMoney { operator, value } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(24);

                                writer.write_i32(value);
                                writer.write_u8(encode_reward_operator(operator));
                                writer.write_padding(3);
                            }
                            &QsdReward::ClanPoints { operator, value } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(25);

                                writer.write_i32(value);
                                writer.write_u8(encode_reward_operator(operator));
                                writer.write_padding(3);
                            }
                            &QsdReward::AddClanSkill { id } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(26);

                                writer.write_u16(id as u16);
                                writer.write_u8(1);
                                writer.write_padding(1);
                            }
                            &QsdReward::RemoveClanSkill { id } => {
                                writer.write_u32(8 + 4);
                                writer.write_u32(26);

                                writer.write_u16(id as u16);
                                writer.write_u8(0);
                                writer.write_padding(1);
                            }
                            &QsdReward::ClanPointContribution { operator, value } => {
                                writer.write_u32(8 + 8);
                                writer.write_u32(27);

                                writer.write_i32(value);
                                writer.write_u8(encode_reward_operator(operator));
                                writer.write_padding(3);
                            }
                            &QsdReward::TeleportNearbyClanMembers {
                                distance,
                                zone,
                                x,
                                y,
                            } => {
                                writer.write_u32(8 + 16);
                                writer.write_u32(28);

                                writer.write_i32(distance);
                                writer.write_u16(zone as u16);
                                writer.write_padding(2);
                                writer.write_i32(x as i32);
                                writer.write_i32(y as i32);
                            }
                            QsdReward::CallLuaFunction { name } => {
                                let size = 8 + 2 + name.len();
                                let padding = 4 - (size % 4);

                                writer.write_u32((size + padding) as u32);
                                writer.write_u32(29);

                                writer.write_u16_length_string(name);
                                writer.write_padding(padding as u64);
                            }
                            &QsdReward::ResetSkills => {
                                writer.write_u32(8);
                                writer.write_u32(30);
                            }
                        }
                        let pos_after = writer.buffer.len();
                        let written_size =
                            (&writer.buffer[pos_before..pos_before + 4]).get_u32_le();
                        if (pos_after - pos_before) as u32 != written_size {
                            panic!(
                                "pos_after - pos_before != written_size, {} != {} for reward {:?}",
                                pos_after - pos_before,
                                written_size,
                                reward
                            );
                        }
                    }
                }
            }
            Ok(())
        }
    }
}

fn read_trigger(
    reader: &mut RoseFileReader,
) -> Result<(String, Vec<QsdReward>, Vec<QsdCondition>, bool), anyhow::Error> {
    let check_next = reader.read_u8()? != 0;
    let condition_count = reader.read_u32()?;
    let reward_count = reader.read_u32()?;
    let trigger_name = reader.read_u16_length_string()?.to_string();
    let mut conditions = Vec::new();
    let mut rewards = Vec::new();
    for _ in 0..condition_count {
        let start_position = reader.position();
        let size_bytes = reader.read_u32()? as u64;
        let opcode = reader.read_u32()? & 0x0ffff;

        match opcode {
            0 => {
                let quest_id = reader.read_u32()? as QsdQuestId;

                conditions.push(QsdCondition::SelectQuest { id: quest_id });
            }
            1 | 2 => {
                let data_count = reader.read_u32()?;
                for _ in 0..data_count {
                    let variable_id = reader.read_u16()? as usize;
                    let variable_type = decode_variable_type(reader.read_u16()?)?;
                    let value = reader.read_i16()? as i32;
                    let operator = decode_condition_operator(reader.read_u8()?)?;
                    reader.skip(1); // padding
                    conditions.push(QsdCondition::QuestVariable {
                        variable_type,
                        variable_id,
                        operator,
                        value,
                    });
                }
            }
            3 => {
                let data_count = reader.read_u32()?;
                for _ in 0..data_count {
                    let ability_type = QsdAbilityType::new(reader.read_u32()? as usize)
                        .ok_or_else(|| {
                            anyhow!("Invalid QsdCondition::AbilityValue ability_type: 0")
                        })?;
                    let value = reader.read_i32()?;
                    let operator = decode_condition_operator(reader.read_u8()?)?;
                    reader.skip(3); // padding
                    conditions.push(QsdCondition::AbilityValue {
                        ability_type,
                        operator,
                        value,
                    });
                }
            }
            4 => {
                let data_count = reader.read_u32()?;
                for _ in 0..data_count {
                    let item = QsdItem::from_sn(reader.read_u32()? as usize);
                    let equipment_index = QsdEquipmentIndex::new(reader.read_u32()? as usize);
                    let required_count = reader.read_u32()?;
                    let operator = decode_condition_operator(reader.read_u8()?)?;
                    reader.skip(3); // padding
                    conditions.push(QsdCondition::QuestItem {
                        item,
                        equipment_index,
                        required_count,
                        operator,
                    });
                }
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

                conditions.push(QsdCondition::Party {
                    is_leader,
                    level_operator,
                    level,
                });
            }
            6 => {
                let zone = reader.read_u32()? as QsdZoneId;
                let x = reader.read_u32()? as f32;
                let y = reader.read_u32()? as f32;
                let _z = reader.read_u32()?;
                let distance = reader.read_u32()? as QsdDistance;

                conditions.push(QsdCondition::Position {
                    zone,
                    x,
                    y,
                    distance,
                });
            }
            7 => {
                let start_time = reader.read_u32()?;
                let end_time = reader.read_u32()?;

                conditions.push(QsdCondition::WorldTime {
                    range: start_time..=end_time,
                });
            }
            8 => {
                let value = reader.read_i32()?;
                let operator = decode_condition_operator(reader.read_u8()?)?;
                reader.skip(3); // padding

                conditions.push(QsdCondition::QuestVariable {
                    variable_type: QsdVariableType::Timer,
                    variable_id: 0,
                    operator,
                    value,
                });
            }
            9 => {
                let start_skill_id = reader.read_u32()? as QsdSkillId;
                let end_skill_id = reader.read_u32()? as QsdSkillId;
                let has_skill = reader.read_u8()? != 0;
                reader.skip(3); // padding

                if start_skill_id == end_skill_id {
                    conditions.push(QsdCondition::HasSkill {
                        id: start_skill_id,
                        has_skill,
                    });
                } else {
                    conditions.push(QsdCondition::HasSkillInRange {
                        range: start_skill_id..=end_skill_id,
                        has_skill,
                    });
                }
            }
            10 => {
                let start_percent = reader.read_u8()?;
                let end_percent = reader.read_u8()?;
                reader.skip(2); // padding

                conditions.push(QsdCondition::RandomPercent {
                    range: start_percent..=end_percent,
                });
            }
            11 => {
                let object = match reader.read_u8()? {
                    0 => QsdObjectType::SelectedNpc,
                    1 => QsdObjectType::SelectedEvent,
                    invalid => {
                        return Err(anyhow!(
                            "Invalid QsdCondition::ObjectVariable object {}",
                            invalid
                        ))
                    }
                };
                reader.skip(1); // padding
                let variable_id = reader.read_u16()? as usize;
                let value = reader.read_i32()?;
                let operator = decode_condition_operator(reader.read_u8()?)?;
                reader.skip(3); // padding

                conditions.push(QsdCondition::ObjectVariable {
                    object,
                    variable_id,
                    operator,
                    value,
                });
            }
            12 => {
                let zone = reader.read_u32()? as QsdZoneId;
                let chunk_x = reader.read_u32()? as usize;
                let chunk_y = reader.read_u32()? as usize;
                let event_id = reader.read_u32()? as QsdEventId;

                conditions.push(QsdCondition::SelectEventObject {
                    zone,
                    chunk_x,
                    chunk_y,
                    event_id,
                });
            }
            13 => {
                let npc_id = reader.read_u32()? as QsdNpcId;

                conditions.push(QsdCondition::SelectNpc { id: npc_id });
            }
            14 => {
                let id = reader.read_u16()? as QsdQuestSwitchId;
                let value = reader.read_u8()? != 0;
                reader.skip(1); // padding

                conditions.push(QsdCondition::QuestSwitch { id, value });
            }
            15 => {
                let start_count = reader.read_u16()? as usize;
                let end_count = reader.read_u16()? as usize;

                conditions.push(QsdCondition::PartyMemberCount {
                    range: start_count..=end_count,
                });
            }
            16 => {
                let object = match reader.read_u8()? {
                    0 => QsdObjectType::SelectedNpc,
                    1 => QsdObjectType::SelectedEvent,
                    2 => QsdObjectType::QuestOwner,
                    invalid => {
                        return Err(anyhow!(
                            "Invalid QsdCondition::ObjectZoneTime object {}",
                            invalid
                        ))
                    }
                };
                reader.skip(3); // padding
                let start_time = reader.read_u32()?;
                let end_time = reader.read_u32()?;

                conditions.push(QsdCondition::ObjectZoneTime {
                    object,
                    time_range: start_time..=end_time,
                });
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

                conditions.push(QsdCondition::CompareNpcVariables {
                    npc_id_1,
                    variable_id_1,
                    operator,
                    npc_id_2,
                    variable_id_2,
                });
            }
            18 => {
                let day = reader.read_u8()?;
                let hour_min = reader.read_u8()?;
                let minute_min = reader.read_u8()?;
                let hour_max = reader.read_u8()?;
                let minute_max = reader.read_u8()?;
                reader.skip(3); // padding

                conditions.push(QsdCondition::MonthDayTime {
                    month_day: NonZeroU8::new(day),
                    day_minutes_range: (hour_min as i32 * 60 + minute_min as i32)
                        ..=(hour_max as i32 * 60 + minute_max as i32),
                });
            }
            19 => {
                let day = reader.read_u8()?;
                let hour_min = reader.read_u8()?;
                let minute_min = reader.read_u8()?;
                let hour_max = reader.read_u8()?;
                let minute_max = reader.read_u8()?;
                reader.skip(3); // padding

                conditions.push(QsdCondition::WeekDayTime {
                    week_day: day,
                    day_minutes_range: (hour_min as i32 * 60 + minute_min as i32)
                        ..=(hour_max as i32 * 60 + minute_max as i32),
                });
            }
            20 => {
                let start = reader.read_u32()? as QsdTeamNumber;
                let end = reader.read_u32()? as QsdTeamNumber;

                conditions.push(QsdCondition::TeamNumber { range: start..=end });
            }
            21 => {
                let object = match reader.read_u8()? {
                    0 => QsdObjectType::SelectedNpc,
                    1 => QsdObjectType::SelectedEvent,
                    invalid => {
                        return Err(anyhow!(
                            "Invalid QsdCondition::ObjectDistance object {}",
                            invalid
                        ))
                    }
                };
                reader.skip(3); // padding
                let distance = reader.read_u32()? as QsdDistance;

                conditions.push(QsdCondition::ObjectDistance { object, distance });
            }
            22 => {
                let start = reader.read_u16()? as QsdServerChannelId;
                let end = reader.read_u16()? as QsdServerChannelId;

                conditions.push(QsdCondition::ServerChannelNumber { range: start..=end });
            }
            23 => {
                let has_clan = reader.read_u8()? != 0;
                reader.skip(3); // padding

                conditions.push(QsdCondition::HasClan { has_clan });
            }
            24 => {
                let value = reader.read_u16()? as QsdClanPosition;
                let operator = decode_condition_operator(reader.read_u8()?)?;
                reader.skip(1); // padding

                conditions.push(QsdCondition::ClanPosition { operator, value });
            }
            25 => {
                let value = reader.read_u16()? as QsdClanPoints;
                let operator = decode_condition_operator(reader.read_u8()?)?;
                reader.skip(1); // padding

                conditions.push(QsdCondition::ClanPointContribution { operator, value });
            }
            26 => {
                let value = reader.read_u16()? as QsdClanLevel;
                let operator = decode_condition_operator(reader.read_u8()?)?;
                reader.skip(1); // padding

                conditions.push(QsdCondition::ClanLevel { operator, value });
            }
            27 => {
                let value = reader.read_u16()? as QsdClanPoints;
                let operator = decode_condition_operator(reader.read_u8()?)?;
                reader.skip(1); // padding

                conditions.push(QsdCondition::ClanPoints { operator, value });
            }
            28 => {
                let value = reader.read_i32()? as QsdMoney;
                let operator = decode_condition_operator(reader.read_u8()?)?;
                reader.skip(3); // padding

                conditions.push(QsdCondition::ClanMoney { operator, value });
            }
            29 => {
                let value = reader.read_u16()? as usize;
                let operator = decode_condition_operator(reader.read_u8()?)?;
                reader.skip(1); // padding

                conditions.push(QsdCondition::ClanMemberCount { operator, value });
            }
            30 => {
                let start = reader.read_u16()? as QsdSkillId;
                let end = reader.read_u16()? as QsdSkillId;
                let has_skill = reader.read_u8()? != 0;
                reader.skip(3); // padding

                if start == end {
                    conditions.push(QsdCondition::HasClanSkill {
                        id: start,
                        has_skill,
                    });
                } else {
                    conditions.push(QsdCondition::HasClanSkillInRange {
                        range: start..=end,
                        has_skill,
                    });
                }
            }
            _ => {
                warn!("Unimplemented QSD condition opcode: {:X}", opcode);
                reader.skip(size_bytes - 8);
            }
        }

        if reader.position() != start_position + size_bytes {
            return Err(anyhow!(
                "Unexpected number of bytes read for condition opcode {:X}",
                opcode
            ));
        }
    }
    let conditions = conditions;
    for _ in 0..reward_count {
        let start_position = reader.position();
        let size_bytes = reader.read_u32()? as u64;
        let opcode = reader.read_u32()? & 0x0ffff;

        match opcode {
            0 => {
                let quest_id = reader.read_u32()? as QsdQuestId;
                let reward = match reader.read_u8()? {
                    0 => QsdReward::RemoveSelectedQuest,
                    1 => QsdReward::AddQuest { id: quest_id },
                    2 => QsdReward::ChangeSelectedQuest {
                        id: quest_id,
                        keep_data: true,
                    },
                    3 => QsdReward::ChangeSelectedQuest {
                        id: quest_id,
                        keep_data: false,
                    },
                    4 => QsdReward::SelectQuest { id: quest_id },
                    invalid => return Err(anyhow!("Invalid QsdReward::Quest action {}", invalid)),
                };
                reader.skip(3); // padding

                rewards.push(reward);
            }
            1 => {
                let item_sn = reader.read_u32()? as usize;
                let item = QsdItem::from_sn(item_sn)
                    .ok_or_else(|| anyhow!("Invalid QsdReward::AddItem item_sn {}", item_sn))?;
                let add_or_remove = reader.read_u8()? != 0;
                reader.skip(1); // padding
                let quantity = reader.read_u16()? as usize;
                let _to_party = reader.read_u8()?; // unused
                reader.skip(3); // padding

                if add_or_remove {
                    rewards.push(QsdReward::AddItem { item, quantity });
                } else {
                    rewards.push(QsdReward::RemoveItem { item, quantity });
                }
            }
            2 | 4 => {
                let data_count = reader.read_u32()?;
                for _ in 0..data_count {
                    let variable_id = reader.read_u16()? as usize;
                    let variable_type = decode_variable_type(reader.read_u16()?)?;
                    let value = reader.read_i16()? as i32;
                    let operator = decode_reward_operator(reader.read_u8()?)?;
                    reader.skip(1); // padding
                    rewards.push(QsdReward::QuestVariable {
                        variable_type,
                        variable_id,
                        operator,
                        value,
                    });
                }
            }
            3 => {
                let data_count = reader.read_u32()?;
                for _ in 0..data_count {
                    let ability_type = QsdAbilityType::new(reader.read_u32()? as usize)
                        .ok_or_else(|| {
                            anyhow!("Invalid QsdReward::AbilityValue ability_type: 0")
                        })?;
                    let value = reader.read_i32()?;
                    let operator = decode_reward_operator(reader.read_u8()?)?;
                    reader.skip(3); // padding
                    rewards.push(QsdReward::AbilityValue {
                        ability_type,
                        operator,
                        value,
                    });
                }
            }
            5 => {
                let reward_type = reader.read_u8()?;
                let equation = reader.read_u8()? as QsdEquationId;
                reader.skip(2);
                let value = reader.read_i32()?;
                let item_sn = reader.read_u32()? as usize;
                let _to_party = reader.read_u8()?; // unused
                reader.skip(1);
                let gem = NonZeroUsize::new(reader.read_u16()? as u32 as usize);

                match reward_type {
                    0 => {
                        rewards.push(QsdReward::CalculatedExperiencePoints { equation, value });
                    }
                    1 => {
                        rewards.push(QsdReward::CalculatedMoney { equation, value });
                    }
                    2 => {
                        rewards.push(QsdReward::CalculatedItem {
                            equation,
                            value,
                            item: QsdItem::from_sn(item_sn).ok_or_else(|| {
                                anyhow!("Invalid QsdReward::CalculatedReward item: 0")
                            })?,
                            gem,
                        });
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
                let _to_party = reader.read_u8()?; // unused
                reader.skip(3);

                rewards.push(QsdReward::SetHealthManaPercent {
                    health_percent,
                    mana_percent,
                });
            }
            7 => {
                let zone = reader.read_u32()? as QsdZoneId;
                let x = reader.read_u32()?;
                let y = reader.read_u32()?;
                let _to_party = reader.read_u8()?; // unused
                reader.skip(3);

                rewards.push(QsdReward::Teleport { zone, x, y });
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
                    0 => QsdSpawnMonsterLocation::QuestOwner,
                    1 => QsdSpawnMonsterLocation::SelectedNpc,
                    2 => QsdSpawnMonsterLocation::SelectedEvent,
                    3 => QsdSpawnMonsterLocation::Position { zone, x, y },
                    invalid => {
                        return Err(anyhow!(
                            "Invalid QsdReward::SpawnMonster location {}",
                            invalid
                        ))
                    }
                };

                rewards.push(QsdReward::SpawnMonster {
                    npc,
                    count,
                    location,
                    distance,
                    team_number,
                });
            }
            9 => {
                let trigger = reader.read_u16_length_string()?;
                reader.set_position(start_position + size_bytes); // padding

                rewards.push(QsdReward::Trigger {
                    name: trigger.to_string(),
                });
            }
            10 => {
                rewards.push(QsdReward::ResetBasicStats);
            }
            11 => {
                let object = match reader.read_u8()? {
                    0 => QsdObjectType::SelectedNpc,
                    1 => QsdObjectType::SelectedEvent,
                    invalid => {
                        return Err(anyhow!(
                            "Invalid QsdReward::ObjectVariable object {}",
                            invalid
                        ))
                    }
                };
                reader.skip(1); // padding
                let variable_id = reader.read_u16()? as usize;
                let value = reader.read_i32()?;
                let operator = decode_reward_operator(reader.read_u8()?)?;
                reader.skip(3); // padding

                rewards.push(QsdReward::ObjectVariable {
                    object,
                    variable_id,
                    operator,
                    value,
                });
            }
            12 => {
                let message_type = match reader.read_u8()? {
                    0 => QsdNpcMessageType::Chat,
                    1 => QsdNpcMessageType::Shout,
                    2 => QsdNpcMessageType::Announce,
                    invalid => {
                        return Err(anyhow!(
                            "Invalid QsdReward::NpcMessage message_type {}",
                            invalid
                        ))
                    }
                };
                reader.skip(3); // padding
                let string_id = reader.read_u32()? as QsdStringId;

                rewards.push(QsdReward::NpcMessage {
                    message_type,
                    string_id,
                });
            }
            13 => {
                let object = match reader.read_u8()? {
                    0 => QsdObjectType::SelectedNpc,
                    1 => QsdObjectType::SelectedEvent,
                    invalid => {
                        return Err(anyhow!(
                            "Invalid QsdReward::TriggerAfterDelay object {}",
                            invalid
                        ))
                    }
                };
                reader.skip(3); // padding
                let delay = Duration::from_secs(reader.read_u32()? as u64);
                let trigger = reader.read_u16_length_string()?;
                reader.set_position(start_position + size_bytes); // padding

                rewards.push(QsdReward::TriggerAfterDelay {
                    object,
                    delay,
                    trigger: trigger.to_string(),
                });
            }
            14 => {
                let add_or_remove = reader.read_u8()? != 0;
                reader.skip(3); // padding
                let id = reader.read_u32()? as QsdSkillId;

                if add_or_remove {
                    rewards.push(QsdReward::AddSkill { id });
                } else {
                    rewards.push(QsdReward::RemoveSkill { id });
                }
            }
            15 => {
                let id = reader.read_u16()? as QsdQuestSwitchId;
                let value = reader.read_u8()? != 0;
                reader.skip(1); // padding

                rewards.push(QsdReward::SetQuestSwitch { id, value });
            }
            16 => {
                let group = reader.read_u16()? as QsdQuestSwitchGroupId;
                reader.skip(2); // padding

                rewards.push(QsdReward::ClearSwitchGroup { group });
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

                rewards.push(QsdReward::FormatAnnounceMessage {
                    string_id,
                    variables,
                });
            }
            19 => {
                let zone = reader.read_u16()? as QsdZoneId;
                let team_number = reader.read_u16()? as QsdTeamNumber;
                let trigger = reader.read_u16_length_string()?.to_string();
                reader.set_position(start_position + size_bytes); // padding

                rewards.push(QsdReward::TriggerForZoneTeam {
                    zone,
                    team_number,
                    trigger,
                });
            }
            20 => {
                let source = match reader.read_u8()? {
                    0 => QsdTeamNumberSource::Unique,
                    1 => QsdTeamNumberSource::Clan,
                    2 => QsdTeamNumberSource::Party,
                    invalid => {
                        return Err(anyhow!(
                            "Invalid QsdReward::SetTeamNumber source {}",
                            invalid
                        ))
                    }
                };
                reader.skip(3); // padding

                rewards.push(QsdReward::SetTeamNumber { source });
            }
            21 => {
                let x = reader.read_i32()? as f32;
                let y = reader.read_i32()? as f32;

                rewards.push(QsdReward::SetRevivePosition { x, y });
            }
            22 => {
                let zone = reader.read_u16()? as QsdZoneId;
                let reward = match reader.read_u8()? {
                    0 => QsdReward::DisableMonsterSpawns { zone },
                    1 => QsdReward::EnableMonsterSpawns { zone },
                    2 => QsdReward::ToggleMonsterSpawns { zone },
                    invalid => {
                        return Err(anyhow!(
                            "Invalid QsdReward SetMonsterSpawnState state {}",
                            invalid
                        ))
                    }
                };
                reader.skip(1); // padding

                rewards.push(reward);
            }
            23 => {
                rewards.push(QsdReward::ClanLevelIncrease);
            }
            24 => {
                let value = reader.read_i32()?;
                let operator = decode_reward_operator(reader.read_u8()?)?;
                reader.skip(3); // padding
                rewards.push(QsdReward::ClanMoney { operator, value });
            }
            25 => {
                let value = reader.read_i32()?;
                let operator = decode_reward_operator(reader.read_u8()?)?;
                reader.skip(3); // padding
                rewards.push(QsdReward::ClanPoints { operator, value });
            }
            26 => {
                let id = reader.read_u16()? as QsdSkillId;
                let add_or_remove = reader.read_u8()? != 0;
                reader.skip(1); // padding

                if add_or_remove {
                    rewards.push(QsdReward::AddClanSkill { id });
                } else {
                    rewards.push(QsdReward::RemoveClanSkill { id });
                }
            }
            27 => {
                let value = reader.read_i32()?;
                let operator = decode_reward_operator(reader.read_u8()?)?;
                reader.skip(3); // padding
                rewards.push(QsdReward::ClanPointContribution { operator, value });
            }
            28 => {
                let distance = reader.read_i32()? as QsdDistance;
                let zone = reader.read_u16()? as QsdZoneId;
                reader.skip(2); // padding
                let x = reader.read_i32()? as f32;
                let y = reader.read_i32()? as f32;

                rewards.push(QsdReward::TeleportNearbyClanMembers {
                    distance,
                    zone,
                    x,
                    y,
                });
            }
            29 => {
                let name = reader.read_u16_length_string()?.to_string();
                reader.set_position(start_position + size_bytes); // padding

                rewards.push(QsdReward::CallLuaFunction { name });
            }
            30 => {
                rewards.push(QsdReward::ResetSkills);
            }
            _ => {
                warn!("Unimplemented QSD action opcode: {:X}", opcode);
                reader.skip(size_bytes - 8);
            }
        }

        if reader.position() != start_position + size_bytes {
            return Err(anyhow!(
                "Unexpected number of bytes read for action opcode {:X}",
                opcode
            ));
        }
    }

    Ok((trigger_name, rewards, conditions, check_next))
}
