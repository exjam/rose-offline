use arrayvec::ArrayVec;
use enum_map::Enum;
use std::{collections::HashMap, num::NonZeroU16, str::FromStr};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct StatusEffectId(NonZeroU16);

id_wrapper_impl!(StatusEffectId, NonZeroU16, u16);

#[derive(Copy, Clone, Debug, Enum)]
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

#[derive(Debug)]
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
    pub name: String,
    pub status_effect_type: StatusEffectType,
    pub can_be_reapplied: bool,
    pub cleared_by_type: StatusEffectClearedByType,
    pub apply_status_effects: ArrayVec<(StatusEffectId, i32), 2>,
    pub apply_per_second_value: i32,
}

pub struct StatusEffectDatabase {
    status_effects: HashMap<u16, StatusEffectData>,
    decrease_summon_life_status_effect_id: StatusEffectId,
}

impl StatusEffectDatabase {
    pub fn new(
        status_effects: HashMap<u16, StatusEffectData>,
        decrease_summon_life_status_effect_id: StatusEffectId,
    ) -> Self {
        Self {
            status_effects,
            decrease_summon_life_status_effect_id,
        }
    }

    pub fn get_status_effect(&self, id: StatusEffectId) -> Option<&StatusEffectData> {
        self.status_effects.get(&(id.get() as u16))
    }

    pub fn get_decrease_summon_life_status_effect(&self) -> Option<&StatusEffectData> {
        self.get_status_effect(self.decrease_summon_life_status_effect_id)
    }
}
