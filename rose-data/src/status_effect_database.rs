use arrayvec::ArrayVec;
use enum_map::Enum;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, num::NonZeroU16, str::FromStr, sync::Arc};

use crate::{EffectFileId, StringDatabase};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct StatusEffectId(NonZeroU16);

id_wrapper_impl!(StatusEffectId, NonZeroU16, u16);

#[derive(Copy, Clone, Debug, Enum, Serialize, Deserialize)]
pub enum StatusEffectType {
    IncreaseHp,
    IncreaseMp,
    Poisoned,
    IncreaseMaxHp,
    IncreaseMaxMp,
    IncreaseMoveSpeed,
    DecreaseMoveSpeed,
    IncreaseAttackSpeed,
    DecreaseAttackSpeed,
    IncreaseAttackPower,
    DecreaseAttackPower,
    IncreaseDefence,
    DecreaseDefence,
    IncreaseResistance,
    DecreaseResistance,
    IncreaseHit,
    DecreaseHit,
    IncreaseCritical,
    DecreaseCritical,
    IncreaseAvoid,
    DecreaseAvoid,
    Dumb,
    Sleep,
    Fainting,
    Disguise,
    Transparent,
    ShieldDamage,
    AdditionalDamageRate,
    DecreaseLifeTime,
    ClearGood,
    ClearBad,
    ClearAll,
    ClearInvisible,
    Taunt,
    Revive,
}

#[allow(dead_code)]
impl StatusEffectType {
    pub fn is_bad(&self) -> bool {
        matches!(
            *self,
            StatusEffectType::Poisoned
                | StatusEffectType::DecreaseMoveSpeed
                | StatusEffectType::DecreaseAttackSpeed
                | StatusEffectType::DecreaseAttackPower
                | StatusEffectType::DecreaseDefence
                | StatusEffectType::DecreaseResistance
                | StatusEffectType::DecreaseHit
                | StatusEffectType::DecreaseCritical
                | StatusEffectType::DecreaseAvoid
                | StatusEffectType::Dumb
                | StatusEffectType::Sleep
                | StatusEffectType::Fainting
        )
    }

    pub fn is_good(&self) -> bool {
        matches!(
            *self,
            StatusEffectType::IncreaseMaxHp
                | StatusEffectType::IncreaseMaxMp
                | StatusEffectType::IncreaseMoveSpeed
                | StatusEffectType::IncreaseAttackSpeed
                | StatusEffectType::IncreaseAttackPower
                | StatusEffectType::IncreaseDefence
                | StatusEffectType::IncreaseResistance
                | StatusEffectType::IncreaseHit
                | StatusEffectType::IncreaseCritical
                | StatusEffectType::IncreaseAvoid
                | StatusEffectType::Disguise
                | StatusEffectType::Transparent
                | StatusEffectType::ShieldDamage
                | StatusEffectType::AdditionalDamageRate
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StatusEffectClearedByType {
    // This status effect is cleared by ClearGood
    ClearGood,

    // This status effect is cleared by ClearBad
    ClearBad,

    // Cannot be cleared
    ClearNone,
}

#[derive(Debug)]
pub struct StatusEffectData {
    pub id: StatusEffectId,
    pub name: &'static str,
    pub description: &'static str,
    pub start_message: &'static str,
    pub end_message: &'static str,
    pub status_effect_type: StatusEffectType,
    pub can_be_reapplied: bool,
    pub cleared_by_type: StatusEffectClearedByType,
    pub apply_status_effects: ArrayVec<(StatusEffectId, i32), 2>,
    pub apply_per_second_value: i32,
    pub effect_file_id: Option<EffectFileId>,
    pub icon_id: u32,
}

pub struct StatusEffectDatabase {
    _string_database: Arc<StringDatabase>,
    status_effects: HashMap<u16, StatusEffectData>,
    decrease_summon_life_status_effect_id: StatusEffectId,
}

impl StatusEffectDatabase {
    pub fn new(
        string_database: Arc<StringDatabase>,
        status_effects: HashMap<u16, StatusEffectData>,
        decrease_summon_life_status_effect_id: StatusEffectId,
    ) -> Self {
        Self {
            _string_database: string_database,
            status_effects,
            decrease_summon_life_status_effect_id,
        }
    }

    pub fn get_status_effect(&self, id: StatusEffectId) -> Option<&StatusEffectData> {
        self.status_effects.get(&id.get())
    }

    pub fn get_decrease_summon_life_status_effect(&self) -> Option<&StatusEffectData> {
        self.get_status_effect(self.decrease_summon_life_status_effect_id)
    }
}
