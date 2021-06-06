use std::sync::Arc;

use crate::{
    data::{
        item::{AbilityType, ItemClass, ItemWeaponType},
        AbilityValueCalculator, ItemDatabase, NpcDatabase, SkillAddAbility, SkillDatabase,
        SkillReference,
    },
    game::components::{
        AbilityValues, AmmoIndex, BasicStats, CharacterInfo, Equipment, EquipmentIndex, Inventory,
        Level, SkillList,
    },
};

pub struct AbilityValuesData {
    item_database: Arc<ItemDatabase>,
    skill_database: Arc<SkillDatabase>,
    npc_database: Arc<NpcDatabase>,
}

impl AbilityValuesData {
    pub fn new(
        item_database: Arc<ItemDatabase>,
        skill_database: Arc<SkillDatabase>,
        npc_database: Arc<NpcDatabase>,
    ) -> Self {
        Self {
            item_database,
            skill_database,
            npc_database,
        }
    }
}

pub fn get_ability_value_calculator(
    item_database: Arc<ItemDatabase>,
    skill_database: Arc<SkillDatabase>,
    npc_database: Arc<NpcDatabase>,
) -> Option<Box<impl AbilityValueCalculator + Send + Sync>> {
    Some(Box::new(AbilityValuesData::new(
        item_database,
        skill_database,
        npc_database,
    )))
}

impl AbilityValueCalculator for AbilityValuesData {
    fn calculate_npc(&self, npc_id: usize) -> Option<AbilityValues> {
        let npc_data = self.npc_database.get_npc(npc_id)?;
        Some(AbilityValues {
            run_speed: npc_data.run_speed as f32,
            strength: 0,
            dexterity: 0,
            intelligence: npc_data.level as u16,
            concentration: 0,
            charm: 0,
            sense: npc_data.level as u16,
            max_health: npc_data.level * npc_data.health_points,
            max_mana: 100,
            additional_health_recovery: 0,
            additional_mana_recovery: 0,
            attack_power: npc_data.attack,
            attack_speed: npc_data.attack_speed,
            attack_range: npc_data.attack_range,
            hit: npc_data.hit,
            defence: npc_data.defence,
            resistance: npc_data.resistance,
            critical: (npc_data.level as f32 * 2.5) as i32,
            avoid: npc_data.avoid,
        })
    }

    fn calculate(
        &self,
        character_info: &CharacterInfo,
        level: &Level,
        equipment: &Equipment,
        _inventory: &Inventory,
        basic_stats: &BasicStats,
        skill_list: &SkillList,
    ) -> AbilityValues {
        let equipment_ability_values =
            calculate_equipment_ability_values(&self.item_database, equipment);
        let passive_ability_values = calculate_passive_skill_ability_values(
            &self.skill_database,
            skill_list.get_passive_skills(),
        );

        // TODO: Apparently we only add these passive_ability_values stats when not on a cart
        let basic_stats = BasicStats {
            strength: (basic_stats.strength as i32
                + passive_ability_values.value.strength
                + passive_ability_values.rate.strength) as u16,
            dexterity: (basic_stats.dexterity as i32
                + passive_ability_values.value.dexterity
                + passive_ability_values.rate.dexterity) as u16,
            intelligence: (basic_stats.intelligence as i32
                + passive_ability_values.value.intelligence
                + passive_ability_values.rate.intelligence) as u16,
            concentration: (basic_stats.concentration as i32
                + passive_ability_values.value.concentration
                + passive_ability_values.rate.concentration) as u16,
            charm: (basic_stats.charm as i32
                + passive_ability_values.value.charm
                + passive_ability_values.rate.charm) as u16,
            sense: (basic_stats.sense as i32
                + passive_ability_values.value.sense
                + passive_ability_values.rate.sense) as u16,
        };

        /*
        TODO:
        Cal_MaxWEIGHT ();
        calculate weight in inventory
        Cal_DropRATE ();
        m_fRateUseMP
        class based += stats + immunity
        */

        // TODO: If riding cart, most stat calculations are different
        AbilityValues {
            run_speed: calculate_run_speed(
                &self.item_database,
                &basic_stats,
                &equipment_ability_values,
                &equipment,
                &passive_ability_values,
            ),
            max_health: calculate_max_health(
                character_info,
                level,
                &basic_stats,
                &equipment_ability_values,
                &passive_ability_values,
            ),
            max_mana: calculate_max_mana(
                character_info,
                level,
                &basic_stats,
                &equipment_ability_values,
                &passive_ability_values,
            ),
            strength: basic_stats.strength,
            dexterity: basic_stats.dexterity,
            intelligence: basic_stats.intelligence,
            concentration: basic_stats.concentration,
            charm: basic_stats.charm,
            sense: basic_stats.sense,
            additional_health_recovery: passive_ability_values.value.recover_health
                + (equipment_ability_values.recover_health as f32
                    * (passive_ability_values.rate.recover_health as f32 / 100.0))
                    as i32,
            additional_mana_recovery: passive_ability_values.value.recover_mana
                + (equipment_ability_values.recover_mana as f32
                    * (passive_ability_values.rate.recover_mana as f32 / 100.0))
                    as i32,
            attack_power: calculate_attack_power(
                &self.item_database,
                &basic_stats,
                level,
                &equipment_ability_values,
                equipment,
                &passive_ability_values,
            ),
            attack_speed: calculate_attack_speed(
                &self.item_database,
                equipment,
                &equipment_ability_values,
                &passive_ability_values,
            ),
            attack_range: calculate_attack_range(&self.item_database, equipment),
            hit: calculate_hit(
                &self.item_database,
                &basic_stats,
                &equipment_ability_values,
                equipment,
                &passive_ability_values,
            ),
            defence: calculate_defence(
                &self.item_database,
                &basic_stats,
                &level,
                &equipment_ability_values,
                equipment,
                &passive_ability_values,
            ),
            resistance: calculate_resistance(
                &self.item_database,
                &basic_stats,
                &level,
                &equipment_ability_values,
                equipment,
                &passive_ability_values,
            ),
            critical: calculate_critical(
                &basic_stats,
                &equipment_ability_values,
                &passive_ability_values,
            ),
            avoid: calculate_avoid(
                &self.item_database,
                &basic_stats,
                level,
                equipment,
                &equipment_ability_values,
                &passive_ability_values,
            ),
        }
    }
}

#[derive(Default)]
struct EquipmentAbilityValue {
    pub gender: i32,
    pub birthstone: i32,
    pub class: i32,
    pub union: i32,
    pub rank: i32,
    pub fame: i32,
    pub face: i32,
    pub hair: i32,
    pub strength: i32,
    pub dexterity: i32,
    pub intelligence: i32,
    pub concentration: i32,
    pub charm: i32,
    pub sense: i32,
    pub health: i32,
    pub mana: i32,
    pub attack: i32,
    pub defence: i32,
    pub hit: i32,
    pub resistance: i32,
    pub avoid: i32,
    pub move_speed: i32,
    pub attack_speed: i32,
    pub weight: i32,
    pub critical: i32,
    pub recover_health: i32,
    pub recover_mana: i32,
    pub save_mana: i32,
    pub experience: i32,
    pub level: i32,
    pub bonus_point: i32,
    pub pvp_flag: i32,
    pub team_number: i32,
    pub head_size: i32,
    pub body_size: i32,
    pub skillpoint: i32,
    pub max_health: i32,
    pub max_mana: i32,
    pub money: i32,
    pub race: i32,
    pub drop_rate: i32,
    pub fame_g: i32,
    pub fame_b: i32,
    pub current_planet: i32,
    pub stamina: i32,
    pub fuel: i32,
    pub immunity: i32,
    pub union_point1: i32,
    pub union_point2: i32,
    pub union_point3: i32,
    pub union_point4: i32,
    pub union_point5: i32,
    pub union_point6: i32,
    pub union_point7: i32,
    pub union_point8: i32,
    pub union_point9: i32,
    pub union_point10: i32,
    pub guild_number: i32,
    pub guild_score: i32,
    pub guild_position: i32,
    pub bank_free: i32,
    pub bank_addon: i32,
    pub store_skin: i32,
    pub vehicle_health: i32,
}

impl EquipmentAbilityValue {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_ability_value(&mut self, ability_type: AbilityType, value: i32) {
        match ability_type {
            AbilityType::Gender => self.gender += value,
            AbilityType::Birthstone => self.birthstone += value,
            AbilityType::Class => self.class += value,
            AbilityType::Union => self.union += value,
            AbilityType::Rank => self.rank += value,
            AbilityType::Fame => self.fame += value,
            AbilityType::Face => self.face += value,
            AbilityType::Hair => self.hair += value,
            AbilityType::Strength => self.strength += value,
            AbilityType::Dexterity => self.dexterity += value,
            AbilityType::Intelligence => self.intelligence += value,
            AbilityType::Concentration => self.concentration += value,
            AbilityType::Charm => self.charm += value,
            AbilityType::Sense => self.sense += value,
            AbilityType::Health => self.health += value,
            AbilityType::Mana => self.mana += value,
            AbilityType::Attack => self.attack += value,
            AbilityType::Defence => self.defence += value,
            AbilityType::Hit => self.hit += value,
            AbilityType::Resistance => self.resistance += value,
            AbilityType::Avoid => self.avoid += value,
            AbilityType::Speed => self.move_speed += value,
            AbilityType::AttackSpeed => self.attack_speed += value,
            AbilityType::Weight => self.weight += value,
            AbilityType::Critical => self.critical += value,
            AbilityType::RecoverHealth => self.recover_health += value,
            AbilityType::RecoverMana => self.recover_mana += value,
            AbilityType::SaveMana => self.save_mana += value,
            AbilityType::Experience => self.experience += value,
            AbilityType::Level => self.level += value,
            AbilityType::BonusPoint => self.bonus_point += value,
            AbilityType::PvpFlag => self.pvp_flag += value,
            AbilityType::TeamNumber => self.team_number += value,
            AbilityType::HeadSize => self.head_size += value,
            AbilityType::BodySize => self.body_size += value,
            AbilityType::Skillpoint => self.skillpoint += value,
            AbilityType::MaxHealth => self.max_health += value,
            AbilityType::MaxMana => self.max_mana += value,
            AbilityType::Money => self.money += value,
            AbilityType::Race => self.race += value,
            AbilityType::DropRate => self.drop_rate += value,
            AbilityType::FameG => self.fame_g += value,
            AbilityType::FameB => self.fame_b += value,
            AbilityType::CurrentPlanet => self.current_planet += value,
            AbilityType::Stamina => self.stamina += value,
            AbilityType::Fuel => self.fuel += value,
            AbilityType::Immunity => self.immunity += value,
            AbilityType::UnionPoint1 => self.union_point1 += value,
            AbilityType::UnionPoint2 => self.union_point2 += value,
            AbilityType::UnionPoint3 => self.union_point3 += value,
            AbilityType::UnionPoint4 => self.union_point4 += value,
            AbilityType::UnionPoint5 => self.union_point5 += value,
            AbilityType::UnionPoint6 => self.union_point6 += value,
            AbilityType::UnionPoint7 => self.union_point7 += value,
            AbilityType::UnionPoint8 => self.union_point8 += value,
            AbilityType::UnionPoint9 => self.union_point9 += value,
            AbilityType::UnionPoint10 => self.union_point10 += value,
            AbilityType::GuildNumber => self.guild_number += value,
            AbilityType::GuildScore => self.guild_score += value,
            AbilityType::GuildPosition => self.guild_position += value,
            AbilityType::BankFree => self.bank_free += value,
            AbilityType::BankAddon => self.bank_addon += value,
            AbilityType::StoreSkin => self.store_skin += value,
            AbilityType::VehicleHealth => self.vehicle_health += value,
            _ => {
                println!("Item has unimplemented ability type {:?}", ability_type)
            }
        }
    }
}

fn calculate_equipment_ability_values(
    item_database: &ItemDatabase,
    equipment: &Equipment,
) -> EquipmentAbilityValue {
    let mut result = EquipmentAbilityValue::new();

    for item in equipment.equipped_items.iter().filter_map(|x| x.as_ref()) {
        if item.is_appraised || item.has_socket {
            if let Some(item_data) = item_database.get_gem_item(item.gem as usize) {
                for (ability, value) in item_data.gem_add_ability.iter() {
                    result.add_ability_value(*ability, *value);
                }
            }
        }

        if let Some(item_data) = item_database.get_base_item(item.into()) {
            // TODO: Check item_stb.get_item_union_requirement(item_number)
            for (ability, value) in item_data.add_ability.iter() {
                result.add_ability_value(*ability, *value);
            }
        }
    }

    // TODO: If riding cart, add values from vehicle

    result
}

#[derive(Default)]
struct PassiveSkillAbilities {
    strength: i32,
    dexterity: i32,
    intelligence: i32,
    concentration: i32,
    charm: i32,
    sense: i32,
    attack_power_unarmed: i32,
    attack_power_one_handed: i32,
    attack_power_two_handed: i32,
    attack_power_bow: i32,
    attack_power_gun: i32,
    attack_power_staff_wand: i32,
    attack_power_auto_bow: i32,
    attack_power_katar_pair: i32,
    attack_speed_bow: i32,
    attack_speed_gun: i32,
    attack_speed_pair: i32,
    move_speed: i32,
    defence: i32,
    max_health: i32,
    max_mana: i32,
    recover_health: i32,
    recover_mana: i32,
    weight: i32,
    buy_skill: i32,
    sell_skill: i32,
    save_mana: i32,
    max_summons: i32,
    drop_rate: i32,
    resistance: i32,
    hit: i32,
    critical: i32,
    avoid: i32,
    shield_defence: i32,
    immunity: i32,
}

impl PassiveSkillAbilities {
    fn get_passive_weapon_attack_power(&self, weapon_type: Option<ItemWeaponType>) -> i32 {
        match weapon_type {
            Some(ItemWeaponType::OneHanded) => self.attack_power_one_handed,
            Some(ItemWeaponType::TwoHanded) => self.attack_power_two_handed,
            Some(ItemWeaponType::Bow) => self.attack_power_bow,
            Some(ItemWeaponType::Gun) | Some(ItemWeaponType::Launcher) => self.attack_power_gun,
            Some(ItemWeaponType::MagicMelee) | Some(ItemWeaponType::MagicRanged) => {
                self.attack_power_staff_wand
            }
            Some(ItemWeaponType::Crossbow) => self.attack_power_auto_bow,
            Some(ItemWeaponType::Katar) | Some(ItemWeaponType::DualWield) => {
                self.attack_power_katar_pair
            }
            None => self.attack_power_unarmed,
        }
    }
}

#[derive(Default)]
struct PassiveSkillAbilityValues {
    pub value: PassiveSkillAbilities,
    pub rate: PassiveSkillAbilities,
}

impl PassiveSkillAbilityValues {
    pub fn new() -> Self {
        Default::default()
    }

    fn get_passive_weapon_attack_power_value(&self, weapon_type: Option<ItemWeaponType>) -> i32 {
        self.value.get_passive_weapon_attack_power(weapon_type)
    }

    fn get_passive_weapon_attack_power_rate(&self, weapon_type: Option<ItemWeaponType>) -> i32 {
        self.rate.get_passive_weapon_attack_power(weapon_type)
    }

    fn add_ability(abilities: &mut PassiveSkillAbilities, ability_type: AbilityType, value: i32) {
        match ability_type {
            AbilityType::Strength => abilities.strength += value,
            AbilityType::Dexterity => abilities.dexterity += value,
            AbilityType::Intelligence => abilities.intelligence += value,
            AbilityType::Concentration => abilities.concentration += value,
            AbilityType::Charm => abilities.charm += value,
            AbilityType::Sense => abilities.sense += value,
            AbilityType::PassiveAttackPowerUnarmed => abilities.attack_power_unarmed += value,
            AbilityType::PassiveAttackPowerOneHanded => abilities.attack_power_one_handed += value,
            AbilityType::PassiveAttackPowerTwoHanded => abilities.attack_power_two_handed += value,
            AbilityType::PassiveAttackPowerBow => abilities.attack_power_bow += value,
            AbilityType::PassiveAttackPowerGun => abilities.attack_power_gun += value,
            AbilityType::PassiveAttackPowerStaffWand => abilities.attack_power_staff_wand += value,
            AbilityType::PassiveAttackPowerAutoBow => abilities.attack_power_auto_bow += value,
            AbilityType::PassiveAttackPowerKatarPair => abilities.attack_power_katar_pair += value,
            AbilityType::PassiveAttackSpeedBow => abilities.attack_speed_bow += value,
            AbilityType::PassiveAttackSpeedGun => abilities.attack_speed_gun += value,
            AbilityType::PassiveAttackSpeedPair => abilities.attack_speed_pair += value,
            AbilityType::PassiveMoveSpeed => abilities.move_speed += value,
            AbilityType::PassiveDefence => abilities.defence += value,
            AbilityType::PassiveMaxHealth => abilities.max_health += value,
            AbilityType::PassiveMaxMana => abilities.max_mana += value,
            AbilityType::PassiveRecoverHealth => abilities.recover_health += value,
            AbilityType::PassiveRecoverMana => abilities.recover_mana += value,
            AbilityType::PassiveWeight => abilities.weight += value,
            AbilityType::PassiveBuySkill => abilities.buy_skill += value,
            AbilityType::PassiveSellSkill => abilities.sell_skill += value,
            AbilityType::PassiveSaveMana => abilities.save_mana += value,
            AbilityType::PassiveMaxSummons => abilities.max_summons += value,
            AbilityType::PassiveDropRate => abilities.drop_rate += value,
            AbilityType::PassiveResistance => abilities.resistance += value,
            AbilityType::PassiveHit => abilities.hit += value,
            AbilityType::PassiveCritical => abilities.critical += value,
            AbilityType::PassiveAvoid => abilities.avoid += value,
            AbilityType::PassiveShieldDefence => abilities.shield_defence += value,
            AbilityType::PassiveImmunity => abilities.immunity += value,
            _ => {
                println!(
                    "Passive skill has unimplemented ability type {:?}",
                    ability_type
                )
            }
        }
    }

    pub fn add_ability_rate(&mut self, ability_type: AbilityType, value: i32) {
        Self::add_ability(&mut self.rate, ability_type, value);
    }

    pub fn add_ability_value(&mut self, ability_type: AbilityType, value: i32) {
        Self::add_ability(&mut self.value, ability_type, value);
    }
}

fn calculate_passive_skill_ability_values<'a>(
    skill_database: &SkillDatabase,
    passive_skills: impl Iterator<Item = &'a SkillReference>,
) -> PassiveSkillAbilityValues {
    let mut result = PassiveSkillAbilityValues::new();

    for skill_reference in passive_skills {
        if let Some(skill_data) = skill_database.get_skill(skill_reference) {
            for add_ability in &skill_data.add_ability {
                match add_ability {
                    SkillAddAbility::Rate(ability_type, rate) => {
                        result.add_ability_rate(*ability_type, *rate);
                    }
                    SkillAddAbility::Value(ability_type, value) => {
                        result.add_ability_value(*ability_type, *value);
                    }
                }
            }
        }
    }

    result
}

fn calculate_run_speed(
    item_database: &ItemDatabase,
    basic_stats: &BasicStats,
    equipment_ability_values: &EquipmentAbilityValue,
    equipment: &Equipment,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> f32 {
    let mut item_speed = 20f32;

    item_speed += equipment
        .get_equipment_item(EquipmentIndex::Feet)
        .filter(|item| !item.is_broken())
        .and_then(|item| item_database.get_feet_item(item.item.item_number))
        .or_else(|| item_database.get_feet_item(0))
        .map(|item_data| item_data.move_speed)
        .unwrap_or(0) as f32;

    item_speed += equipment
        .get_equipment_item(EquipmentIndex::Back)
        .filter(|item| !item.is_broken())
        .and_then(|item| item_database.get_back_item(item.item.item_number as usize))
        .map(|item_data| item_data.move_speed)
        .unwrap_or(0) as f32;

    let item_run_speed = item_speed * (basic_stats.dexterity as f32 + 500.0) / 100.0
        + equipment_ability_values.move_speed as f32;

    let passive_run_speed = passive_ability_values.value.move_speed as f32
        + item_run_speed * (passive_ability_values.rate.move_speed as f32 / 100.0);

    item_run_speed + passive_run_speed
}

fn calculate_max_health(
    character_info: &CharacterInfo,
    level: &Level,
    basic_stats: &BasicStats,
    equipment_ability_values: &EquipmentAbilityValue,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> i32 {
    let (level_add, level_multiplier, strength_multipler) = match character_info.job {
        111 => (7, 12, 2),
        121 => (-3, 14, 2),
        122 => (2, 13, 2),

        211 => (11, 10, 2),
        221 => (11, 10, 2),
        222 => (5, 11, 2),

        311 => (10, 11, 2),
        321 => (2, 13, 2),
        322 => (11, 11, 2),

        411 => (12, 10, 2),
        421 => (13, 10, 2),
        422 => (6, 11, 2),

        _ => (12, 8, 2),
    };

    let max_health = (level.level as i32 + level_add) * level_multiplier
        + (basic_stats.strength as i32) * strength_multipler
        + equipment_ability_values.max_health;

    let passive_max_health = passive_ability_values.value.max_health
        + ((max_health as f32) * ((passive_ability_values.rate.max_health as f32) / 100.0)) as i32;

    max_health + passive_max_health
}

fn calculate_max_mana(
    character_info: &CharacterInfo,
    level: &Level,
    basic_stats: &BasicStats,
    equipment_ability_values: &EquipmentAbilityValue,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> i32 {
    let (level_add, level_multiplier, int_multipler) = match character_info.job {
        111 => (3, 4.0, 4),
        121 => (0, 4.5, 4),
        122 => (-6, 5.0, 4),

        211 => (0, 6.0, 4),
        221 => (-7, 7.0, 4),
        222 => (-4, 6.5, 4),

        311 => (4, 4.0, 4),
        321 => (4, 4.0, 4),
        322 => (0, 4.5, 4),

        411 => (3, 4.0, 4),
        421 => (3, 4.0, 4),
        422 => (0, 4.5, 4),

        _ => (4, 3.0, 4),
    };

    let max_mana = ((level.level as i32 + level_add) as f32 * level_multiplier) as i32
        + (basic_stats.intelligence as i32) * int_multipler
        + equipment_ability_values.max_mana;

    let passive_max_mana = passive_ability_values.value.max_mana
        + ((max_mana as f32) * ((passive_ability_values.rate.max_mana as f32) / 100.0)) as i32;

    max_mana + passive_max_mana
}

fn calculate_attack_power(
    item_database: &ItemDatabase,
    basic_stats: &BasicStats,
    level: &Level,
    equipment_ability_values: &EquipmentAbilityValue,
    equipment: &Equipment,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> i32 {
    let dexterity = basic_stats.dexterity as f32;
    let concentration = basic_stats.concentration as f32;
    let strength = basic_stats.strength as f32;
    let intelligence = basic_stats.intelligence as f32;
    let sense = basic_stats.sense as f32;
    let level = level.level as f32;

    let get_ammo_quality = |item_database: &ItemDatabase, equipment: &Equipment, ammo_index| {
        equipment
            .get_ammo_item(ammo_index)
            .and_then(|item| item_database.get_material_item(item.item.item_number as usize))
            .map(|item| item.item_data.quality)
            .unwrap_or(0) as f32
    };

    let weapon = equipment
        .get_equipment_item(EquipmentIndex::WeaponRight)
        .filter(|item| !item.is_broken())
        .and_then(|item| {
            item_database
                .get_weapon_item(item.item.item_number as usize)
                .map(|item_data| (item, item_data))
        });

    let weapon_attack = weapon
        .map(|(weapon, weapon_data)| {
            weapon_data.attack_power as f32
                + item_database
                    .get_item_grade(weapon.grade)
                    .map(|grade| grade.attack)
                    .unwrap_or(0) as f32
        })
        .unwrap_or(0.0);

    let weapon_type =
        weapon.and_then(|(_, weapon_data)| ItemWeaponType::from(weapon_data.item_data.class));

    let attack_power = match weapon_type {
        Some(ItemWeaponType::Bow) | Some(ItemWeaponType::Crossbow) => {
            let ammo_quality = get_ammo_quality(item_database, equipment, AmmoIndex::Arrow);
            dexterity * 0.62
                + strength * 0.2
                + level * 0.2
                + ammo_quality
                + (weapon_attack + ammo_quality * 0.5 + 8.0)
                    * ((dexterity * 0.04 + sense * 0.03 + 29.0) / 30.0)
        }
        Some(ItemWeaponType::Gun) => {
            let ammo_quality = get_ammo_quality(item_database, equipment, AmmoIndex::Bullet);
            dexterity * 0.4
                + concentration * 0.5
                + level * 0.2
                + ammo_quality
                + (weapon_attack + ammo_quality * 0.6 + 8.0)
                    * ((concentration * 0.03 + sense * 0.05 + 29.0) / 30.0)
        }
        Some(ItemWeaponType::Launcher) => {
            let ammo_quality = get_ammo_quality(item_database, equipment, AmmoIndex::Throw);
            strength * 0.52
                + concentration * 0.5
                + level * 0.2
                + ammo_quality
                + (weapon_attack + ammo_quality + 12.0)
                    * ((concentration * 0.04 + sense * 0.05 + 29.0) / 30.0)
        }
        Some(ItemWeaponType::OneHanded) | Some(ItemWeaponType::TwoHanded) => {
            strength * 0.75 + level * 0.2 + weapon_attack * ((strength * 0.05 + 29.0) / 30.0)
        }
        Some(ItemWeaponType::MagicMelee) => {
            strength * 0.4
                + intelligence * 0.4
                + level * 0.2
                + weapon_attack * ((intelligence * 0.05 + 29.0) / 30.0)
        }
        Some(ItemWeaponType::MagicRanged) => {
            intelligence * 0.6 + level * 0.2 + weapon_attack * ((sense * 0.1 + 26.0) / 27.0)
        }
        Some(ItemWeaponType::DualWield) => {
            strength * 0.63
                + dexterity * 0.45
                + level * 0.2
                + weapon_attack * ((dexterity * 0.05 + 25.0) / 26.0)
        }
        Some(ItemWeaponType::Katar) => {
            strength * 0.42
                + dexterity * 0.55
                + level * 0.2
                + weapon_attack * ((dexterity * 0.05 + 20.0) / 21.0)
        }
        None => strength * 0.5 + dexterity * 0.3 + level * 0.2,
    } + equipment_ability_values.attack as f32;

    let passive_attack_rate =
        passive_ability_values.get_passive_weapon_attack_power_rate(weapon_type) as f32 / 100.0;
    let passive_attack_power = passive_ability_values
        .get_passive_weapon_attack_power_value(weapon_type) as f32
        + (attack_power as f32 * passive_attack_rate);

    (attack_power + passive_attack_power) as i32
}

fn calculate_attack_speed(
    item_database: &ItemDatabase,
    equipment: &Equipment,
    equipment_ability_values: &EquipmentAbilityValue,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> i32 {
    let (weapon_attack_speed, weapon_item_class) = item_database
        .get_weapon_item(
            equipment
                .get_equipment_item(EquipmentIndex::WeaponRight)
                .map(|item| item.item.item_number)
                .unwrap_or(0),
        )
        .map(|weapon| (weapon.attack_speed, Some(weapon.item_data.class)))
        .unwrap_or((0, None));

    let attack_speed = 1500.0 / (weapon_attack_speed + 5) as f32;

    let (passive_value, passive_rate) = match weapon_item_class {
        Some(ItemClass::Bow) => (
            passive_ability_values.value.attack_speed_bow,
            passive_ability_values.rate.attack_speed_bow,
        ),
        Some(ItemClass::Gun) | Some(ItemClass::Launcher) => (
            passive_ability_values.value.attack_speed_gun,
            passive_ability_values.rate.attack_speed_gun,
        ),
        Some(ItemClass::Katar) | Some(ItemClass::DualSwords) => (
            passive_ability_values.value.attack_speed_pair,
            passive_ability_values.rate.attack_speed_pair,
        ),
        _ => (0, 0),
    };

    let passive_attack_speed = passive_value as f32 + attack_speed * (passive_rate as f32 / 100.0);

    (attack_speed + passive_attack_speed + equipment_ability_values.attack_speed as f32) as i32
}

fn calculate_attack_range(item_database: &ItemDatabase, equipment: &Equipment) -> i32 {
    let weapon_attack_range = item_database
        .get_weapon_item(
            equipment
                .get_equipment_item(EquipmentIndex::WeaponRight)
                .map(|item| item.item.item_number)
                .unwrap_or(0),
        )
        .map(|weapon| weapon.attack_range)
        .unwrap_or(70);

    let scale = 1.0;

    weapon_attack_range as i32 + (scale * 120.0) as i32
}

fn calculate_hit(
    item_database: &ItemDatabase,
    basic_stats: &BasicStats,
    equipment_ability_values: &EquipmentAbilityValue,
    equipment: &Equipment,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> i32 {
    let concentration = basic_stats.concentration as f32;

    let hit = if let Some((weapon, weapon_data)) = equipment
        .get_equipment_item(EquipmentIndex::WeaponRight)
        .filter(|item| !item.is_broken())
        .and_then(|item| {
            item_database
                .get_weapon_item(item.item.item_number as usize)
                .map(|item_data| (item, item_data))
        }) {
        let weapon_quality = weapon_data.item_data.quality as f32;
        let weapon_durability = weapon.durability as f32;
        let grade_hit = item_database
            .get_item_grade(weapon.grade)
            .map(|grade| grade.hit)
            .unwrap_or(0) as f32;

        (concentration + 10.0) * 0.8 + weapon_quality * 0.6 + grade_hit + weapon_durability * 0.8
    } else {
        (concentration + 10.0) * 0.5 + 15.0
    } + equipment_ability_values.hit as f32;

    let passive_hit_rate = passive_ability_values.rate.hit as f32 / 100.0;
    let passive_hit = passive_ability_values.value.hit as f32 + (hit as f32 * passive_hit_rate);

    (hit + passive_hit) as i32
}

fn calculate_defence(
    item_database: &ItemDatabase,
    basic_stats: &BasicStats,
    level: &Level,
    equipment_ability_values: &EquipmentAbilityValue,
    equipment: &Equipment,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> i32 {
    let mut item_defence = 0;

    for item in equipment
        .equipped_items
        .iter()
        .filter_map(|x| x.as_ref())
        .filter(|x| x.life > 0)
    {
        if let Some(item_data) = item_database.get_base_item(item.into()) {
            if item_data.defence > 0 {
                let grade_defence = item_database
                    .get_item_grade(item.grade)
                    .map(|grade| grade.defence)
                    .unwrap_or(0);
                item_defence += item_data.defence as i32 + grade_defence;
            }
        }
    }

    let strength = basic_stats.strength as f32;
    let level = level.level as f32;
    let defence = item_defence as f32
        + (strength + 5.0) * 0.35
        + (level + 15.0) * 0.7
        + equipment_ability_values.defence as f32;

    let passive_defence_rate = passive_ability_values.rate.defence as f32 / 100.0;
    let passive_defence =
        passive_ability_values.value.defence as f32 + (defence as f32 * passive_defence_rate);

    let mut defence = (defence + passive_defence) as i32;

    if let Some(offhand_item) = equipment.get_equipment_item(EquipmentIndex::WeaponLeft) {
        if let Some(ItemClass::Shield) = item_database
            .get_base_item(offhand_item.into())
            .map(|x| x.class)
        {
            let passive_shield_defence_rate =
                passive_ability_values.rate.shield_defence as f32 / 100.0;
            let passive_shield_defence = passive_ability_values.value.shield_defence as f32
                + (defence as f32 * passive_shield_defence_rate);
            defence += passive_shield_defence as i32;
        }
    }

    defence
}

fn calculate_resistance(
    item_database: &ItemDatabase,
    basic_stats: &BasicStats,
    level: &Level,
    equipment_ability_values: &EquipmentAbilityValue,
    equipment: &Equipment,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> i32 {
    let mut item_resistance = 0;

    for item in equipment
        .equipped_items
        .iter()
        .filter_map(|x| x.as_ref())
        .filter(|x| x.life > 0)
    {
        if let Some(item_data) = item_database.get_base_item(item.into()) {
            if item_data.resistance > 0 {
                let grade_resistance = item_database
                    .get_item_grade(item.grade)
                    .map(|grade| grade.resistance)
                    .unwrap_or(0);
                item_resistance += item_data.resistance as i32 + grade_resistance;
            }
        }
    }

    let intelligence = basic_stats.intelligence as f32;
    let level = level.level as f32;
    let resistance = item_resistance as f32
        + (intelligence + 5.0) * 0.6
        + (level + 15.0) * 0.8
        + equipment_ability_values.resistance as f32;

    let passive_resistance_rate = passive_ability_values.rate.resistance as f32 / 100.0;
    let passive_resistance = passive_ability_values.value.resistance as f32
        + (resistance as f32 * passive_resistance_rate);

    (resistance + passive_resistance) as i32
}

fn calculate_critical(
    basic_stats: &BasicStats,
    equipment_ability_values: &EquipmentAbilityValue,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> i32 {
    let concentration = basic_stats.concentration as f32;
    let sense = basic_stats.sense as f32;
    let critical = sense + (concentration + 20.0) * 0.2 + equipment_ability_values.critical as f32;

    let passive_critical_rate = passive_ability_values.rate.critical as f32 / 100.0;
    let passive_critical =
        passive_ability_values.value.critical as f32 + (critical as f32 * passive_critical_rate);

    (critical + passive_critical) as i32
}

fn calculate_avoid(
    item_database: &ItemDatabase,
    basic_stats: &BasicStats,
    level: &Level,
    equipment: &Equipment,
    equipment_ability_values: &EquipmentAbilityValue,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> i32 {
    const AVOID_DURABILITY_ITEMS: [EquipmentIndex; 6] = [
        EquipmentIndex::Head,
        EquipmentIndex::Body,
        EquipmentIndex::Back,
        EquipmentIndex::Hands,
        EquipmentIndex::Feet,
        EquipmentIndex::WeaponLeft,
    ];

    // Get total durability for specific set of equipment
    let equipment_durability: i32 = AVOID_DURABILITY_ITEMS
        .iter()
        .map(|x| equipment.get_equipment_item(*x))
        .flatten()
        .filter(|x| x.life > 0)
        .map(|item| item.durability as i32)
        .sum();

    // Count grade on all items which have defence stat > 0
    let mut equipment_total_grade = 0;
    for item in equipment
        .equipped_items
        .iter()
        .filter_map(|x| x.as_ref())
        .filter(|x| x.life > 0)
    {
        if let Some(item_data) = item_database.get_base_item(item.into()) {
            if item_data.defence > 0 {
                equipment_total_grade += item.grade as i32;
            }
        }
    }

    let dexterity = basic_stats.dexterity as f32;
    let level = level.level as f32;
    let avoid = (dexterity * 1.9 + level * 0.3 + 10.0) * 0.4
        + (equipment_durability as f32) * 0.3
        + equipment_total_grade as f32
        + equipment_ability_values.avoid as f32;

    let passive_avoid_rate = passive_ability_values.rate.avoid as f32 / 100.0;
    let passive_avoid =
        passive_ability_values.value.avoid as f32 + (avoid as f32 * passive_avoid_rate);

    (avoid + passive_avoid) as i32
}
