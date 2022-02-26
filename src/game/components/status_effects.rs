use std::time::{Duration, Instant};

use bevy_ecs::prelude::Component;
use enum_map::EnumMap;
use log::warn;

use crate::data::{StatusEffectData, StatusEffectId, StatusEffectType};

#[derive(Clone)]
pub struct ActiveStatusEffect {
    pub id: StatusEffectId,
    pub value: i32,
    pub expire_time: Instant,
}

#[derive(Component, Clone)]
pub struct StatusEffects {
    pub active: EnumMap<StatusEffectType, Option<ActiveStatusEffect>>,
}

#[derive(Clone)]
pub struct ActiveStatusEffectRegen {
    pub total_value: i32,
    pub value_per_second: i32,
    pub applied_value: i32,
    pub applied_duration: Duration,
}

// This is stored in a separate component as it must change every tick, and we want
// Changed<StatusEffects> to only be triggered when effects have been added / removed
#[derive(Component, Clone)]
pub struct StatusEffectsRegen {
    pub regens: EnumMap<StatusEffectType, Option<ActiveStatusEffectRegen>>,
    pub per_second_tick_counter: Duration,
}

impl StatusEffectsRegen {
    pub fn new() -> Self {
        Self {
            regens: Default::default(),
            per_second_tick_counter: Duration::from_secs(0),
        }
    }
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
                status_effect_data.can_be_reapplied && value > status_effect.value
            }
            None => true,
        }
    }

    pub fn apply_status_effect(
        &mut self,
        status_effect_data: &StatusEffectData,
        expire_time: Instant,
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
                    id: status_effect_data.id,
                    value,
                    expire_time,
                });
                true
            }
        }
    }

    pub fn apply_summon_decrease_life_status_effect(
        &mut self,
        status_effect_data: &StatusEffectData,
    ) -> bool {
        self.active[status_effect_data.status_effect_type] = Some(ActiveStatusEffect {
            id: status_effect_data.id,
            value: 0,
            expire_time: Instant::now() + Duration::from_secs(10000000),
        });
        true
    }

    pub fn apply_potion(
        &mut self,
        status_effects_regen: &mut StatusEffectsRegen,
        status_effect_data: &StatusEffectData,
        expire_time: Instant,
        total_value: i32,
        value_per_second: i32,
    ) -> bool {
        let status_effect_type = status_effect_data.status_effect_type;
        match status_effect_type {
            StatusEffectType::IncreaseHp | StatusEffectType::IncreaseMp => {
                self.apply_status_effect(
                    status_effect_data,
                    expire_time,
                    status_effect_data.id.get() as i32,
                );
                status_effects_regen.regens[status_effect_type] = Some(ActiveStatusEffectRegen {
                    total_value,
                    value_per_second,
                    applied_value: 0,
                    applied_duration: Duration::from_secs(0),
                });
                true
            }
            _ => false,
        }
    }

    pub fn get_status_effect_value(&self, status_effect_type: StatusEffectType) -> Option<i32> {
        self.active[status_effect_type]
            .as_ref()
            .map(|status_effect| status_effect.value)
    }
}
