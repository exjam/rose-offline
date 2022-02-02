use std::{collections::HashMap, num::NonZeroU16, str::FromStr};

use arrayvec::ArrayVec;
use enum_map::Enum;
use num_derive::FromPrimitive;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct StatusEffectId(NonZeroU16);

id_wrapper_impl!(StatusEffectId, NonZeroU16, u16);

#[derive(Copy, Clone, Debug, Enum, FromPrimitive)]
pub enum StatusEffectType {
    IncreaseHp = 1,
    IncreaseMp = 2,
    Poisoned = 3,
    IncreaseMaxHp = 4,
    IncreaseMaxMp = 5,
    IncreaseMoveSpeed = 6,
    DecreaseMoveSpeed = 7,
    IncreaseAttackSpeed = 8,
    DecreaseAttackSpeed = 9,
    IncreaseAttackPower = 10,
    DecreaseAttackPower = 11,
    IncreaseDefence = 12,
    DecreaseDefence = 13,
    IncreaseResistance = 14,
    DecreaseResistance = 15,
    IncreaseHit = 16,
    DecreaseHit = 17,
    IncreaseCritical = 18,
    DecreaseCritical = 19,
    IncreaseAvoid = 20,
    DecreaseAvoid = 21,
    Dumb = 22,
    Sleep = 23,
    Fainting = 24,
    Disguise = 25,
    Transparent = 26,
    ShieldDamage = 27,
    AdditionalDamageRate = 28,
    DecreaseLifeTime = 29,
    ClearGood = 30,
    ClearBad = 31,
    ClearAll = 32,
    ClearInvisible = 33,
    Taunt = 34,
    Revive = 35,
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

#[derive(Debug, FromPrimitive)]
pub enum StatusEffectClearedByType {
    // This status effect is cleared by ClearGood
    ClearGood = 0,

    // This status effect is cleared by ClearBad
    ClearBad = 1,

    // Cannot be cleared
    ClearNone = 2,
}

#[derive(Debug)]
pub struct StatusEffectData {
    pub id: StatusEffectId,
    pub name: String,
    pub status_effect_type: StatusEffectType,
    pub can_be_reapplied: bool,
    pub cleared_by_type: StatusEffectClearedByType,
    pub apply_status_effects: ArrayVec<(StatusEffectId, i32), 2>,
}

pub struct StatusEffectDatabase {
    status_effects: HashMap<u16, StatusEffectData>,
}

impl StatusEffectDatabase {
    pub fn new(status_effects: HashMap<u16, StatusEffectData>) -> Self {
        Self { status_effects }
    }

    pub fn get_status_effect(&self, id: StatusEffectId) -> Option<&StatusEffectData> {
        self.status_effects.get(&(id.get() as u16))
    }
}
