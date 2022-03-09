use rose_data::{Item, ItemDatabase, ItemReference, NpcId, SkillAddAbility, SkillData};

use crate::game::components::{
    AbilityValues, BasicStatType, BasicStats, CharacterInfo, Equipment, Level, SkillList,
    StatusEffects,
};

#[derive(Copy, Clone, Debug)]
pub enum AbilityType {
    Gender,
    Birthstone,
    Class,
    Union,
    Rank,
    Fame,
    Face,
    Hair,

    Strength,
    Dexterity,
    Intelligence,
    Concentration,
    Charm,
    Sense,

    Health,
    Mana,
    Attack,
    Defence,
    Hit,
    Resistance,
    Avoid,
    Speed,
    AttackSpeed,
    Weight,
    Critical,
    RecoverHealth,
    RecoverMana,

    SaveMana,
    Experience,
    Level,
    BonusPoint,
    PvpFlag,
    TeamNumber,
    HeadSize,
    BodySize,
    Skillpoint,
    MaxHealth,
    MaxMana,
    Money,

    PassiveAttackPowerUnarmed,
    PassiveAttackPowerOneHanded,
    PassiveAttackPowerTwoHanded,
    PassiveAttackPowerBow,
    PassiveAttackPowerGun,
    PassiveAttackPowerStaffWand,
    PassiveAttackPowerAutoBow,
    PassiveAttackPowerKatarPair,

    PassiveAttackSpeedBow,
    PassiveAttackSpeedGun,
    PassiveAttackSpeedPair,

    PassiveMoveSpeed,
    PassiveDefence,
    PassiveMaxHealth,
    PassiveMaxMana,
    PassiveRecoverHealth,
    PassiveRecoverMana,
    PassiveWeight,

    PassiveBuySkill,
    PassiveSellSkill,
    PassiveSaveMana,
    PassiveMaxSummons,
    PassiveDropRate,

    Race,
    DropRate,
    FameG,
    FameB,
    CurrentPlanet,
    Stamina,
    Fuel,
    Immunity,

    UnionPoint1,
    UnionPoint2,
    UnionPoint3,
    UnionPoint4,
    UnionPoint5,
    UnionPoint6,
    UnionPoint7,
    UnionPoint8,
    UnionPoint9,
    UnionPoint10,

    GuildNumber,
    GuildScore,
    GuildPosition,

    BankFree,
    BankAddon,
    StoreSkin,
    VehicleHealth,

    PassiveResistance,
    PassiveHit,
    PassiveCritical,
    PassiveAvoid,
    PassiveShieldDefence,
    PassiveImmunity,
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
