use crate::data::StatusEffectType;

use super::StatusEffects;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DamageCategory {
    Character,
    Npc,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DamageType {
    Physical,
    Magic,
}

#[derive(Clone, Debug)]
pub struct AbilityValuesAdjust {
    pub additional_damage_multiplier: f32,
    pub attack_speed: i32,
    pub attack_power: i32,
    pub avoid: i32,
    pub critical: i32,
    pub defence: i32,
    pub hit: i32,
    pub resistance: i32,
    pub max_health: i32,
    pub max_mana: i32,
    pub run_speed: f32,
}

impl From<&StatusEffects> for AbilityValuesAdjust {
    fn from(status_effects: &StatusEffects) -> Self {
        Self {
            additional_damage_multiplier: status_effects
                .get_status_effect_value(StatusEffectType::AdditionalDamageRate)
                .unwrap_or(100) as f32
                / 100.0,
            attack_speed: status_effects
                .get_status_effect_value(StatusEffectType::IncreaseAttackSpeed)
                .unwrap_or(0)
                - status_effects
                    .get_status_effect_value(StatusEffectType::DecreaseAttackSpeed)
                    .unwrap_or(0),
            attack_power: status_effects
                .get_status_effect_value(StatusEffectType::IncreaseAttackPower)
                .unwrap_or(0)
                - status_effects
                    .get_status_effect_value(StatusEffectType::DecreaseAttackPower)
                    .unwrap_or(0),
            avoid: status_effects
                .get_status_effect_value(StatusEffectType::IncreaseAvoid)
                .unwrap_or(0)
                - status_effects
                    .get_status_effect_value(StatusEffectType::DecreaseAvoid)
                    .unwrap_or(0),
            critical: status_effects
                .get_status_effect_value(StatusEffectType::IncreaseCritical)
                .unwrap_or(0)
                - status_effects
                    .get_status_effect_value(StatusEffectType::DecreaseCritical)
                    .unwrap_or(0),
            defence: status_effects
                .get_status_effect_value(StatusEffectType::IncreaseDefence)
                .unwrap_or(0)
                - status_effects
                    .get_status_effect_value(StatusEffectType::DecreaseDefence)
                    .unwrap_or(0),
            hit: status_effects
                .get_status_effect_value(StatusEffectType::IncreaseHit)
                .unwrap_or(0)
                - status_effects
                    .get_status_effect_value(StatusEffectType::DecreaseHit)
                    .unwrap_or(0),
            resistance: status_effects
                .get_status_effect_value(StatusEffectType::IncreaseResistance)
                .unwrap_or(0)
                - status_effects
                    .get_status_effect_value(StatusEffectType::DecreaseResistance)
                    .unwrap_or(0),
            max_health: status_effects
                .get_status_effect_value(StatusEffectType::IncreaseMaxHp)
                .unwrap_or(0),
            max_mana: status_effects
                .get_status_effect_value(StatusEffectType::IncreaseMaxMp)
                .unwrap_or(0),
            run_speed: (status_effects
                .get_status_effect_value(StatusEffectType::IncreaseMoveSpeed)
                .unwrap_or(0)
                - status_effects
                    .get_status_effect_value(StatusEffectType::DecreaseMoveSpeed)
                    .unwrap_or(0)) as f32,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AbilityValues {
    pub damage_category: DamageCategory,
    pub level: i32,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub strength: i32,
    pub dexterity: i32,
    pub intelligence: i32,
    pub concentration: i32,
    pub charm: i32,
    pub sense: i32,
    pub max_health: i32,
    pub max_mana: i32,
    pub additional_health_recovery: i32,
    pub additional_mana_recovery: i32,
    pub attack_damage_type: DamageType,
    pub attack_power: i32,
    pub attack_speed: i32,
    pub passive_attack_speed: i32,
    pub attack_range: i32,
    pub hit: i32,
    pub defence: i32,
    pub resistance: i32,
    pub critical: i32,
    pub avoid: i32,
    pub max_damage_sources: usize,
    pub drop_rate: i32,
    pub max_weight: i32,
    pub summon_owner_level: Option<i32>,
    pub summon_skill_level: Option<i32>,
    pub adjust: AbilityValuesAdjust,
}

#[allow(dead_code)]
impl AbilityValues {
    pub fn get_damage_category(&self) -> DamageCategory {
        self.damage_category
    }

    pub fn get_level(&self) -> i32 {
        self.level
    }

    pub fn get_walk_speed(&self) -> f32 {
        self.walk_speed
    }

    pub fn get_strength(&self) -> i32 {
        self.strength
    }

    pub fn get_dexterity(&self) -> i32 {
        self.dexterity
    }

    pub fn get_intelligence(&self) -> i32 {
        self.intelligence
    }

    pub fn get_concentration(&self) -> i32 {
        self.concentration
    }

    pub fn get_charm(&self) -> i32 {
        self.charm
    }

    pub fn get_sense(&self) -> i32 {
        self.sense
    }

    pub fn get_additional_health_recovery(&self) -> i32 {
        self.additional_health_recovery
    }

    pub fn get_additional_mana_recovery(&self) -> i32 {
        self.additional_mana_recovery
    }

    pub fn get_attack_damage_type(&self) -> DamageType {
        self.attack_damage_type
    }

    pub fn get_passive_attack_speed(&self) -> i32 {
        self.passive_attack_speed
    }

    pub fn get_attack_range(&self) -> i32 {
        self.attack_range
    }

    pub fn get_max_damage_sources(&self) -> usize {
        self.max_damage_sources
    }

    pub fn get_drop_rate(&self) -> i32 {
        self.drop_rate
    }

    pub fn max_weight(&self) -> i32 {
        self.max_weight
    }

    pub fn get_additional_damage_multipler(&self) -> f32 {
        self.adjust.additional_damage_multiplier
    }

    pub fn get_attack_speed(&self) -> i32 {
        self.attack_speed + self.adjust.attack_speed
    }

    pub fn get_attack_power(&self) -> i32 {
        self.attack_power + self.adjust.attack_power
    }

    pub fn get_avoid(&self) -> i32 {
        self.avoid + self.adjust.avoid
    }

    pub fn get_critical(&self) -> i32 {
        self.critical + self.adjust.critical
    }

    pub fn get_defence(&self) -> i32 {
        self.defence + self.adjust.defence
    }

    pub fn get_hit(&self) -> i32 {
        self.hit + self.adjust.hit
    }

    pub fn get_resistance(&self) -> i32 {
        self.resistance + self.adjust.resistance
    }

    pub fn get_max_health(&self) -> i32 {
        self.max_health + self.adjust.max_health
    }

    pub fn get_max_mana(&self) -> i32 {
        self.max_mana + self.adjust.max_mana
    }

    pub fn get_run_speed(&self) -> f32 {
        self.run_speed + self.adjust.run_speed
    }
}
