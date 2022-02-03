use num_derive::FromPrimitive;

use crate::{
    data::{item::Item, ItemDatabase, ItemReference, NpcId, SkillAddAbility, SkillData},
    game::components::{
        AbilityValues, BasicStatType, BasicStats, CharacterInfo, Equipment, Level, SkillList,
        StatusEffects,
    },
};

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

#[derive(Clone, Copy)]
pub enum PassiveRecoveryState {
    Normal,
    Sitting,
}

pub trait AbilityValueCalculator {
    fn calculate(
        &self,
        character_info: &CharacterInfo,
        level: &Level,
        equipment: &Equipment,
        basic_stats: &BasicStats,
        skill_list: &SkillList,
        status_effects: &StatusEffects,
    ) -> AbilityValues;

    fn calculate_npc(
        &self,
        npc_id: NpcId,
        status_effects: &StatusEffects,
        owner_level: Option<i32>,
        summon_skill_level: Option<i32>,
    ) -> Option<AbilityValues>;

    fn calculate_damage(
        &self,
        attacker: &AbilityValues,
        defender: &AbilityValues,
        hit_count: i32,
    ) -> Damage;

    fn calculate_skill_adjust_value(
        &self,
        skill_add_ability: &SkillAddAbility,
        caster_intelligence: i32,
        ability_value: i32,
    ) -> i32;

    fn calculate_skill_damage(
        &self,
        attacker: &AbilityValues,
        defender: &AbilityValues,
        skill_data: &SkillData,
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

    fn calculate_npc_store_item_buy_price(
        &self,
        item_database: &ItemDatabase,
        item: ItemReference,
        buy_skill_value: i32,
        item_rate: i32,
        town_rate: i32,
    ) -> Option<i32>;

    fn calculate_npc_store_item_sell_price(
        &self,
        item_database: &ItemDatabase,
        item: &Item,
        sell_skill_value: i32,
        world_rate: i32,
        item_rate: i32,
        town_rate: i32,
    ) -> Option<i32>;

    fn calculate_passive_recover_hp(
        &self,
        ability_values: &AbilityValues,
        recovery_state: PassiveRecoveryState,
    ) -> i32;

    fn calculate_passive_recover_mp(
        &self,
        ability_values: &AbilityValues,
        recovery_state: PassiveRecoveryState,
    ) -> i32;
}
