use std::time::Duration;

use enum_map::EnumMap;
use log::warn;

use crate::data::{StatusEffectData, StatusEffectType};

#[derive(Clone)]
pub struct ActiveStatusEffect {
    pub value: i32,
    pub time_remaining: Duration,
}

#[derive(Clone)]
pub struct StatusEffects {
    pub active: EnumMap<StatusEffectType, Option<ActiveStatusEffect>>,
}

impl StatusEffects {
    pub fn new() -> Self {
        Self {
            active: Default::default(),
        }
    }

    pub fn can_apply(&self, status_effect_data: &StatusEffectData, value: i32) -> bool {
        match &self.active[status_effect_data.status_effect_type] {
            Some(status_effect) => {
                !status_effect_data.can_be_reapplied || value > status_effect.value
            }
            None => true,
        }
    }

    pub fn apply_status_effect(
        &mut self,
        status_effect_data: &StatusEffectData,
        duration: Duration,
        value: i32,
    ) -> bool {
        let status_effect_type = status_effect_data.status_effect_type;
        match status_effect_type {
            StatusEffectType::ClearGood
            | StatusEffectType::ClearBad
            | StatusEffectType::ClearAll
            | StatusEffectType::ClearInvisible
            | StatusEffectType::DecreaseLifeTime => {
                warn!(
                    "Unimplemented apply_status_effect for type {:?}",
                    status_effect_type
                );
                false
            }
            _ => {
                self.active[status_effect_type] = Some(ActiveStatusEffect {
                    value,
                    time_remaining: duration,
                });
                true
            }
        }
    }

    pub fn get_status_effect_value(&self, status_effect_type: StatusEffectType) -> i32 {
        match &self.active[status_effect_type] {
            Some(status_effect) => status_effect.value,
            None => 0,
        }
    }
}
