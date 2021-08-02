use num_derive::FromPrimitive;

use crate::{
    data::NpcId,
    game::components::{
        AbilityValues, BasicStatType, BasicStats, CharacterInfo, DamageCategory, DamageType,
        Equipment, Level, SkillList, StatusEffects,
    },
};

use super::StatusEffectType;

#[derive(Copy, Clone, Debug, FromPrimitive)]
pub enum AbilityType {
    Gender = 2,
    Birthstone = 3,
    Class = 4,
    Union = 5,
    Rank = 6,
    Fame = 7,
    Face = 8,
    Hair = 9,

    Strength = 10,
    Dexterity = 11,
    Intelligence = 12,
    Concentration = 13,
    Charm = 14,
    Sense = 15,

    Health = 16,
    Mana = 17,
    Attack = 18,
    Defence = 19,
    Hit = 20,
    Resistance = 21,
    Avoid = 22,
    Speed = 23,
    AttackSpeed = 24,
    Weight = 25,
    Critical = 26,
    RecoverHealth = 27,
    RecoverMana = 28,

    SaveMana = 29,
    Experience = 30,
    Level = 31,
    BonusPoint = 32,
    PvpFlag = 33,
    TeamNumber = 34,
    HeadSize = 35,
    BodySize = 36,
    Skillpoint = 37,
    MaxHealth = 38,
    MaxMana = 39,
    Money = 40,

    PassiveAttackPowerUnarmed = 41,
    PassiveAttackPowerOneHanded = 42,
    PassiveAttackPowerTwoHanded = 43,
    PassiveAttackPowerBow = 44,
    PassiveAttackPowerGun = 45,
    PassiveAttackPowerStaffWand = 46,
    PassiveAttackPowerAutoBow = 47,
    PassiveAttackPowerKatarPair = 48,

    PassiveAttackSpeedBow = 49,
    PassiveAttackSpeedGun = 50,
    PassiveAttackSpeedPair = 51,

    PassiveMoveSpeed = 52,
    PassiveDefence = 53,
    PassiveMaxHealth = 54,
    PassiveMaxMana = 55,
    PassiveRecoverHealth = 56,
    PassiveRecoverMana = 57,
    PassiveWeight = 58,

    PassiveBuySkill = 59,
    PassiveSellSkill = 60,
    PassiveSaveMana = 61,
    PassiveMaxSummons = 62,
    PassiveDropRate = 63,

    Race = 71,
    DropRate = 72,
    FameG = 73,
    FameB = 74,
    CurrentPlanet = 75,
    Stamina = 76,
    Fuel = 77,
    Immunity = 78,

    UnionPoint1 = 81,
    UnionPoint2 = 82,
    UnionPoint3 = 83,
    UnionPoint4 = 84,
    UnionPoint5 = 85,
    UnionPoint6 = 86,
    UnionPoint7 = 87,
    UnionPoint8 = 88,
    UnionPoint9 = 89,
    UnionPoint10 = 90,

    GuildNumber = 91,
    GuildScore = 92,
    GuildPosition = 93,

    BankFree = 94,
    BankAddon = 95,
    StoreSkin = 96,
    VehicleHealth = 97,

    PassiveResistance = 98,
    PassiveHit = 99,
    PassiveCritical = 100,
    PassiveAvoid = 101,
    PassiveShieldDefence = 102,
    PassiveImmunity = 103,

    Max = 105,
}

#[derive(Clone, Copy)]
pub struct Damage {
    pub amount: u32,
    pub is_critical: bool,
    pub apply_hit_stun: bool,
}

pub trait GetAbilityValues {
    fn get_additional_health_recovery(&self) -> i32;
    fn get_additional_mana_recovery(&self) -> i32;
    fn get_attack_damage_type(&self) -> DamageType;
    fn get_attack_range(&self) -> i32;
    fn get_charm(&self) -> i32;
    fn get_concentration(&self) -> i32;
    fn get_damage_category(&self) -> DamageCategory;
    fn get_dexterity(&self) -> i32;
    fn get_drop_rate(&self) -> i32;
    fn get_intelligence(&self) -> i32;
    fn get_level(&self) -> i32;
    fn get_max_damage_sources(&self) -> usize;
    fn get_passive_attack_speed(&self) -> i32;
    fn get_sense(&self) -> i32;
    fn get_strength(&self) -> i32;
    fn get_walk_speed(&self) -> f32;
    fn get_attack_speed(&self) -> i32;
    fn get_attack_power(&self) -> i32;
    fn get_avoid(&self) -> i32;
    fn get_critical(&self) -> i32;
    fn get_defence(&self) -> i32;
    fn get_hit(&self) -> i32;
    fn get_resistance(&self) -> i32;
    fn get_max_health(&self) -> i32;
    fn get_max_mana(&self) -> i32;
    fn get_run_speed(&self) -> f32;
}

impl GetAbilityValues for (&AbilityValues, &StatusEffects) {
    fn get_additional_health_recovery(&self) -> i32 {
        self.0.additional_health_recovery
    }

    fn get_additional_mana_recovery(&self) -> i32 {
        self.0.additional_mana_recovery
    }

    fn get_attack_damage_type(&self) -> DamageType {
        self.0.attack_damage_type
    }

    fn get_attack_range(&self) -> i32 {
        self.0.attack_range
    }

    fn get_charm(&self) -> i32 {
        self.0.charm
    }

    fn get_concentration(&self) -> i32 {
        self.0.concentration
    }

    fn get_damage_category(&self) -> DamageCategory {
        self.0.damage_category
    }

    fn get_dexterity(&self) -> i32 {
        self.0.dexterity
    }

    fn get_drop_rate(&self) -> i32 {
        self.0.drop_rate
    }

    fn get_intelligence(&self) -> i32 {
        self.0.intelligence
    }

    fn get_level(&self) -> i32 {
        self.0.level
    }

    fn get_max_damage_sources(&self) -> usize {
        self.0.max_damage_sources
    }

    fn get_passive_attack_speed(&self) -> i32 {
        self.0.passive_attack_speed
    }

    fn get_sense(&self) -> i32 {
        self.0.sense
    }

    fn get_strength(&self) -> i32 {
        self.0.strength
    }

    fn get_walk_speed(&self) -> f32 {
        self.0.walk_speed
    }

    fn get_attack_speed(&self) -> i32 {
        self.0.attack_speed
            + self
                .1
                .get_status_effect_value(StatusEffectType::IncreaseAttackSpeed)
                .unwrap_or(0)
            - self
                .1
                .get_status_effect_value(StatusEffectType::DecreaseAttackSpeed)
                .unwrap_or(0)
    }

    fn get_attack_power(&self) -> i32 {
        self.0.attack_power
            + self
                .1
                .get_status_effect_value(StatusEffectType::IncreaseAttackPower)
                .unwrap_or(0)
            - self
                .1
                .get_status_effect_value(StatusEffectType::DecreaseAttackPower)
                .unwrap_or(0)
    }

    fn get_avoid(&self) -> i32 {
        self.0.avoid
            + self
                .1
                .get_status_effect_value(StatusEffectType::IncreaseAvoid)
                .unwrap_or(0)
            - self
                .1
                .get_status_effect_value(StatusEffectType::DecreaseAvoid)
                .unwrap_or(0)
    }

    fn get_critical(&self) -> i32 {
        self.0.critical
            + self
                .1
                .get_status_effect_value(StatusEffectType::IncreaseCritical)
                .unwrap_or(0)
            - self
                .1
                .get_status_effect_value(StatusEffectType::DecreaseCritical)
                .unwrap_or(0)
    }

    fn get_defence(&self) -> i32 {
        self.0.defence
            + self
                .1
                .get_status_effect_value(StatusEffectType::IncreaseDefence)
                .unwrap_or(0)
            - self
                .1
                .get_status_effect_value(StatusEffectType::DecreaseDefence)
                .unwrap_or(0)
    }

    fn get_hit(&self) -> i32 {
        self.0.hit
            + self
                .1
                .get_status_effect_value(StatusEffectType::IncreaseHit)
                .unwrap_or(0)
            - self
                .1
                .get_status_effect_value(StatusEffectType::DecreaseHit)
                .unwrap_or(0)
    }

    fn get_resistance(&self) -> i32 {
        self.0.resistance
            + self
                .1
                .get_status_effect_value(StatusEffectType::IncreaseResistance)
                .unwrap_or(0)
            - self
                .1
                .get_status_effect_value(StatusEffectType::DecreaseResistance)
                .unwrap_or(0)
    }

    fn get_max_health(&self) -> i32 {
        self.0.max_health
            + self
                .1
                .get_status_effect_value(StatusEffectType::IncreaseMaxHp)
                .unwrap_or(0)
    }

    fn get_max_mana(&self) -> i32 {
        self.0.max_mana
            + self
                .1
                .get_status_effect_value(StatusEffectType::IncreaseMaxMp)
                .unwrap_or(0)
    }

    fn get_run_speed(&self) -> f32 {
        self.0.run_speed
            + self
                .1
                .get_status_effect_value(StatusEffectType::IncreaseMoveSpeed)
                .unwrap_or(0) as f32
            - self
                .1
                .get_status_effect_value(StatusEffectType::DecreaseMoveSpeed)
                .unwrap_or(0) as f32
    }
}

pub trait AbilityValueCalculator {
    fn calculate(
        &self,
        character_info: &CharacterInfo,
        level: &Level,
        equipment: &Equipment,
        basic_stats: &BasicStats,
        skill_list: &SkillList,
    ) -> AbilityValues;

    fn calculate_npc(&self, npc_id: NpcId) -> Option<AbilityValues>;

    fn calculate_damage(
        &self,
        attacker: (&AbilityValues, &StatusEffects),
        defender: (&AbilityValues, &StatusEffects),
        hit_count: i32,
    ) -> Damage;

    fn calculate_give_xp(
        &self,
        attacker_level: i32,
        attacker_damage: i32,
        defender_level: i32,
        defender_max_hp: i32,
        defender_reward_xp: i32,
        world_xp_rate: i32,
    ) -> i32;

    fn calculate_give_stamina(
        &self,
        experience_points: i32,
        level: i32,
        world_stamina_rate: i32,
    ) -> i32;

    fn calculate_basic_stat_increase_cost(
        &self,
        basic_stats: &BasicStats,
        basic_stat_type: BasicStatType,
    ) -> Option<u32>;

    fn calculate_levelup_require_xp(&self, level: u32) -> u64;
    fn calculate_levelup_reward_skill_points(&self, level: u32) -> u32;
    fn calculate_levelup_reward_stat_points(&self, level: u32) -> u32;

    fn calculate_reward_value(
        &self,
        equation_id: usize,
        base_reward_value: i32,
        dup_count: i32,
        level: i32,
        charm: i32,
        fame: i32,
        world_reward_rate: i32,
    ) -> i32;
}
