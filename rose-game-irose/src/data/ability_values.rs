use core::f32;
use log::error;
use rand::Rng;
use rose_data_irose::IroseSkillPageType;
use std::{num::NonZeroU32, sync::Arc};

use rose_data::{
    AbilityType, AmmoIndex, EquipmentIndex, EquipmentItem, Item, ItemClass, ItemDatabase,
    ItemReference, ItemType, ItemWeaponType, NpcDatabase, NpcId, SkillAddAbility, SkillData,
    SkillDatabase, VehiclePartIndex,
};
use rose_game_common::{
    components::{
        AbilityValues, BasicStatType, BasicStats, CharacterInfo, DamageCategory, DamageType,
        Equipment, EquipmentItemDatabase, ItemSlot, Level, Money, SkillList, StatusEffects,
    },
    data::{AbilityValueCalculator, Damage, PassiveRecoveryState},
};

const MAX_BASIC_STAT_VALUE: i32 = 300;

pub struct AbilityValuesData {
    item_database: Arc<ItemDatabase>,
    skill_database: Arc<SkillDatabase>,
    npc_database: Arc<NpcDatabase>,
}

pub fn get_ability_value_calculator(
    item_database: Arc<ItemDatabase>,
    skill_database: Arc<SkillDatabase>,
    npc_database: Arc<NpcDatabase>,
) -> Box<impl AbilityValueCalculator + Send + Sync> {
    Box::new(AbilityValuesData {
        item_database,
        skill_database,
        npc_database,
    })
}

impl AbilityValueCalculator for AbilityValuesData {
    fn calculate_npc(
        &self,
        npc_id: NpcId,
        status_effects: &StatusEffects,
        owner_level: Option<i32>,
        summon_skill_level: Option<i32>,
    ) -> Option<AbilityValues> {
        let npc_data = self.npc_database.get_npc(npc_id)?;
        let mut level = npc_data.level;
        let mut max_health = npc_data.level * npc_data.health_points;
        let mut attack_power = npc_data.attack;
        let mut hit = npc_data.hit;
        let mut defence = npc_data.defence;
        let mut resistance = npc_data.resistance;
        let mut avoid = npc_data.avoid;

        if let Some(owner_level) = owner_level {
            let summon_skill_level = summon_skill_level.unwrap_or(1);

            level = owner_level;
            max_health =
                (npc_data.health_points * (summon_skill_level + 16) * (owner_level + 85)) / 2600;
            attack_power = (attack_power * (summon_skill_level + 22) * (owner_level + 100)) / 4000;
            hit = (hit * (summon_skill_level + 30) * (owner_level + 50)) / 3200;
            defence = (defence * (summon_skill_level + 30) * (owner_level + 80)) / 4400;
            resistance = (resistance * (summon_skill_level + 24) * (owner_level + 90)) / 3600;
            avoid = (avoid * (summon_skill_level + 22) * (owner_level + 90)) / 3400;
        }

        Some(AbilityValues {
            is_driving: false,
            damage_category: DamageCategory::Npc,
            walk_speed: npc_data.walk_speed as f32,
            run_speed: npc_data.run_speed as f32,
            vehicle_move_speed: 0.0,
            level,
            strength: 0,
            dexterity: 0,
            intelligence: npc_data.level,
            concentration: 0,
            charm: 0,
            sense: npc_data.level,
            max_health,
            max_mana: 100,
            additional_health_recovery: 0,
            additional_mana_recovery: 0,
            attack_damage_type: if npc_data.is_attack_magic_damage {
                DamageType::Magic
            } else {
                DamageType::Physical
            },
            attack_power,
            attack_speed: npc_data.attack_speed,
            passive_attack_speed: 0,
            attack_range: npc_data.attack_range,
            hit,
            defence,
            resistance,
            critical: (npc_data.level as f32 * 2.5) as i32,
            avoid,
            vehicle_attack_power: 0,
            vehicle_attack_range: 0,
            vehicle_attack_speed: 0,
            vehicle_hit: 0,
            vehicle_defence: 0,
            vehicle_critical: 0,
            vehicle_avoid: 0,
            max_damage_sources: ((npc_data.health_points / 8) + 4) as usize,
            drop_rate: 0,
            max_weight: 0,
            summon_owner_level: owner_level,
            summon_skill_level,
            adjust: status_effects.into(),
            npc_store_buy_rate: 0,
            npc_store_sell_rate: 0,
            save_mana: 0,
        })
    }

    fn calculate(
        &self,
        character_info: &CharacterInfo,
        level: &Level,
        equipment: &Equipment,
        basic_stats: &BasicStats,
        skill_list: &SkillList,
        status_effects: &StatusEffects,
    ) -> AbilityValues {
        let equipment_ability_values =
            calculate_equipment_ability_values(&self.item_database, equipment);
        let vehicle_ability_values =
            calculate_vehicle_ability_values(&self.item_database, equipment);
        let passive_ability_values =
            calculate_passive_skill_ability_values(&self.skill_database, skill_list);

        let vehicle_basic_stats = basic_stats.clone();
        let basic_stats = BasicStats {
            strength: (basic_stats.strength
                + passive_ability_values.value.strength
                + passive_ability_values.rate.strength
                + equipment_ability_values.strength),
            dexterity: (basic_stats.dexterity
                + passive_ability_values.value.dexterity
                + passive_ability_values.rate.dexterity
                + equipment_ability_values.dexterity),
            intelligence: (basic_stats.intelligence
                + passive_ability_values.value.intelligence
                + passive_ability_values.rate.intelligence
                + equipment_ability_values.intelligence),
            concentration: (basic_stats.concentration
                + passive_ability_values.value.concentration
                + passive_ability_values.rate.concentration
                + equipment_ability_values.concentration),
            charm: (basic_stats.charm
                + passive_ability_values.value.charm
                + passive_ability_values.rate.charm
                + equipment_ability_values.charm),
            sense: (basic_stats.sense
                + passive_ability_values.value.sense
                + passive_ability_values.rate.sense
                + equipment_ability_values.sense),
        };

        /*
        TODO:
        Cal_MaxWEIGHT ();
        m_fRateUseMP
        job based += stats + immunity
        */
        let (attack_speed, passive_attack_speed) = calculate_attack_speed(
            &self.item_database,
            equipment,
            &equipment_ability_values,
            &passive_ability_values,
        );

        let (job_add_max_health, job_add_attack, job_add_defence, job_add_resistance) =
            match character_info.job {
                121 | 122 | 221 | 222 | 321 | 322 | 421 | 422 => (300, 30, 25, 20),
                _ => (0, 0, 0, 0),
            };

        AbilityValues {
            is_driving: false,
            damage_category: DamageCategory::Character,
            walk_speed: 200.0,
            run_speed: calculate_run_speed(
                &self.item_database,
                &basic_stats,
                &equipment_ability_values,
                equipment,
                &passive_ability_values,
            ),
            vehicle_move_speed: calculate_vehicle_move_speed(
                &self.item_database,
                &vehicle_ability_values,
                equipment,
            ),
            max_health: calculate_max_health(
                character_info,
                level,
                &basic_stats,
                &equipment_ability_values,
                &passive_ability_values,
            ) + job_add_max_health,
            max_mana: calculate_max_mana(
                character_info,
                level,
                &basic_stats,
                &equipment_ability_values,
                &passive_ability_values,
            ),
            level: level.level as i32,
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
            attack_damage_type: self
                .item_database
                .get_equipped_weapon_item_data(equipment, EquipmentIndex::Weapon)
                .map(|item| {
                    if item.is_magic_damage {
                        DamageType::Magic
                    } else {
                        DamageType::Physical
                    }
                })
                .unwrap_or(DamageType::Physical),
            attack_power: calculate_attack_power(
                &self.item_database,
                &basic_stats,
                level,
                &equipment_ability_values,
                equipment,
                &passive_ability_values,
            ) + job_add_attack,
            attack_speed,
            passive_attack_speed,
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
                level,
                &equipment_ability_values,
                equipment,
                &passive_ability_values,
                false,
            ) + job_add_defence,
            resistance: calculate_resistance(
                &self.item_database,
                &basic_stats,
                level,
                &equipment_ability_values,
                equipment,
                &passive_ability_values,
            ) + job_add_resistance,
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
            vehicle_attack_power: calculate_vehicle_attack_power(
                &self.item_database,
                &vehicle_basic_stats,
                level,
                &vehicle_ability_values,
                equipment,
            ) + job_add_attack,
            vehicle_attack_range: calculate_vehicle_attack_range(&self.item_database, equipment),
            vehicle_attack_speed: calculate_vehicle_attack_speed(
                &self.item_database,
                equipment,
                &vehicle_ability_values,
            ),
            vehicle_hit: calculate_vehicle_hit(
                &self.item_database,
                &vehicle_basic_stats,
                level,
                &vehicle_ability_values,
                equipment,
                &passive_ability_values,
            ),
            vehicle_defence: calculate_defence(
                &self.item_database,
                &basic_stats,
                level,
                &equipment_ability_values,
                equipment,
                &passive_ability_values,
                true,
            ) + job_add_defence,
            vehicle_critical: calculate_vehicle_critical(
                &vehicle_basic_stats,
                level,
                &vehicle_ability_values,
                &passive_ability_values,
            ),
            vehicle_avoid: calculate_vehicle_avoid(
                &vehicle_basic_stats,
                level,
                &vehicle_ability_values,
                &passive_ability_values,
            ),
            max_damage_sources: 0,
            drop_rate: calculate_drop_rate(&equipment_ability_values, &passive_ability_values),
            save_mana: calculate_save_mana(&equipment_ability_values, &passive_ability_values),
            max_weight: calculate_max_weight(
                &self.item_database,
                level,
                &basic_stats,
                equipment,
                &equipment_ability_values,
                &passive_ability_values,
            ),
            summon_owner_level: None,
            summon_skill_level: None,
            adjust: status_effects.into(),
            npc_store_buy_rate: passive_ability_values.value.buy_skill,
            npc_store_sell_rate: passive_ability_values.value.sell_skill,
        }
    }

    fn calculate_damage(
        &self,
        attacker: &AbilityValues,
        defender: &AbilityValues,
        hit_count: i32,
    ) -> Damage {
        let mut rng = rand::thread_rng();
        let success_rate = calculate_damage_success_rate(&mut rng, attacker, defender);
        if success_rate < 20
            && (rng.gen_range(1..=100)
                + (0.6 * (attacker.get_level() - defender.get_level()) as f32) as i32)
                < 94
        {
            Damage {
                amount: 0,
                apply_hit_stun: false,
                is_critical: false,
            }
        } else {
            match attacker.get_attack_damage_type() {
                DamageType::Magic => calculate_attack_damage_magic(
                    &mut rng,
                    attacker,
                    defender,
                    hit_count,
                    success_rate,
                ),
                DamageType::Physical => calculate_attack_damage_physical(
                    &mut rng,
                    attacker,
                    defender,
                    hit_count,
                    success_rate,
                ),
            }
        }
    }

    fn calculate_skill_adjust_value(
        &self,
        skill_add_ability: &SkillAddAbility,
        caster_intelligence: i32,
        ability_value: i32,
    ) -> i32 {
        ((ability_value * skill_add_ability.rate) as f32 / 100.0
            + skill_add_ability.value as f32 * (caster_intelligence as f32 + 300.0) / 315.0)
            as i32
    }

    fn calculate_skill_damage(
        &self,
        attacker: &AbilityValues,
        defender: &AbilityValues,
        skill_data: &SkillData,
        hit_count: i32,
    ) -> Damage {
        let mut rng = rand::thread_rng();
        let mut damage = match skill_data.damage_type {
            1 => {
                let success = ((attacker.get_level() + 20) - defender.get_level()
                    + rng.gen_range(1..=60)) as f32
                    * (attacker.get_hit() as f32 - defender.get_avoid() as f32 * 0.6
                        + rng.gen_range(1..=70) as f32
                        + 10.0)
                    / 110.0;

                if success < 10.0 {
                    0.0
                } else if success < 20.0 {
                    (skill_data.power as f32
                        * 0.4
                        * (attacker.get_attack_power() as f32 + 50.0)
                        * (rng.gen_range(1..=30) as f32
                            + attacker.get_sense() as f32 * 1.2
                            + 340.0))
                        / (defender.get_defence() + defender.get_resistance() + 20) as f32
                        / (250 + defender.get_level() - attacker.get_level()) as f32
                        + 20.0
                } else if matches!(attacker.damage_category, DamageCategory::Character)
                    && matches!(defender.damage_category, DamageCategory::Character)
                {
                    ((skill_data.power as f32 + attacker.get_attack_power() as f32 * 0.2)
                        * (attacker.get_attack_power() as f32 + 60.0)
                        * (rng.gen_range(1..=30) as f32
                            + attacker.get_sense() as f32 * 0.7
                            + 370.0))
                        * 0.01
                        * (320 - defender.get_level() + attacker.get_level()) as f32
                        / (defender.get_defence() as f32
                            + defender.get_resistance() as f32 * 0.8
                            + defender.get_avoid() as f32 * 0.4
                            + 40.0)
                        / 1600.0
                        + 60.0
                } else {
                    ((skill_data.power as f32 + attacker.get_attack_power() as f32 * 0.2)
                        * (attacker.get_attack_power() as f32 + 60.0)
                        * (rng.gen_range(1..=30) as f32
                            + attacker.get_sense() as f32 * 0.7
                            + 370.0))
                        * 0.01
                        * (120 - defender.get_level() + attacker.get_level()) as f32
                        / (defender.get_defence() as f32
                            + defender.get_resistance() as f32 * 0.8
                            + defender.get_avoid() as f32 * 0.4
                            + 20.0)
                        / 270.0
                        + 20.0
                }
            }
            2 => {
                let success = ((attacker.get_level() + 30) - defender.get_level()
                    + rng.gen_range(1..=50)) as f32
                    * (attacker.get_hit() as f32 - defender.get_avoid() as f32 * 0.56
                        + rng.gen_range(1..=70) as f32
                        + 10.0)
                    / 110.0;

                if success < 8.0 {
                    0.0
                } else if success < 20.0 {
                    (skill_data.power as f32
                        * (attacker.get_attack_power() as f32 * 0.8
                            + attacker.get_intelligence() as f32
                            + 80.0)
                        * (rng.gen_range(1..=30) as f32
                            + attacker.get_sense() as f32 * 1.3
                            + 280.0)
                        * 0.2)
                        / (defender.get_defence() as f32 * 0.3
                            + defender.get_resistance() as f32
                            + 30.0)
                        / (250 + defender.get_level() - attacker.get_level()) as f32
                        + 20.0
                } else if matches!(attacker.damage_category, DamageCategory::Character)
                    && matches!(defender.damage_category, DamageCategory::Character)
                {
                    ((skill_data.power as f32 + 50.0)
                        * (attacker.get_attack_power() as f32 * 0.8
                            + (attacker.get_intelligence() as f32 * 1.2)
                            + 100.0)
                        * (rng.gen_range(1..=30) as f32
                            + attacker.get_sense() as f32 * 0.7
                            + 350.0)
                        * 0.01)
                        * (380 - defender.get_level() + attacker.get_level()) as f32
                        / (defender.get_defence() as f32 * 0.4
                            + defender.get_resistance() as f32
                            + defender.get_avoid() as f32 * 0.3
                            + 60.0)
                        / 2500.0
                        + 60.0
                } else {
                    (skill_data.power as f32
                        * (attacker.get_attack_power() as f32 * 0.8
                            + (attacker.get_intelligence() as f32 * 1.2)
                            + 100.0)
                        * (rng.gen_range(1..=30) as f32
                            + attacker.get_sense() as f32 * 0.7
                            + 350.0)
                        * 0.01)
                        * (150 - defender.get_level() + attacker.get_level()) as f32
                        / (defender.get_defence() as f32 * 0.3
                            + defender.get_resistance() as f32
                            + defender.get_avoid() as f32 * 0.3
                            + 60.0)
                        / 350.0
                        + 20.0
                }
            }
            3 => {
                let success = ((attacker.get_level() + 10) - defender.get_level()
                    + rng.gen_range(1..=80)) as f32
                    * (attacker.get_hit() as f32 - defender.get_avoid() as f32 * 0.5
                        + rng.gen_range(1..=50) as f32
                        + 50.0)
                    / 90.0;
                if success < 6.0 {
                    0.0
                } else if success < 20.0 {
                    (skill_data.power as f32
                        * (skill_data.power as f32 + attacker.get_intelligence() as f32 + 80.0)
                        * (rng.gen_range(1..=30) + attacker.get_sense() * 2 + 290) as f32
                        * 0.2)
                        / (defender.get_defence() as f32 * 0.2
                            + defender.get_resistance() as f32
                            + 30.0)
                        / (250 + defender.get_level() - attacker.get_level()) as f32
                        + 20.0
                } else if matches!(attacker.damage_category, DamageCategory::Character)
                    && matches!(defender.damage_category, DamageCategory::Character)
                {
                    ((skill_data.power as f32 + 35.0)
                        * (skill_data.power as f32 + attacker.get_intelligence() as f32 + 140.0)
                        * (rng.gen_range(1..=30) + attacker.get_sense() + 380) as f32
                        * 0.01)
                        * (400 - defender.get_level() + attacker.get_level()) as f32
                        / (defender.get_defence() as f32 * 0.5
                            + defender.get_resistance() as f32 * 1.2
                            + defender.get_avoid() as f32 * 0.4
                            + 20.0)
                        / 3400.0
                        + 40.0
                } else {
                    ((skill_data.power as f32 + 35.0)
                        * (skill_data.power as f32 + attacker.get_intelligence() as f32 + 140.0)
                        * (rng.gen_range(1..=30) + attacker.get_sense() + 380) as f32
                        * 0.01)
                        * (150 - defender.get_level() + attacker.get_level()) as f32
                        / (defender.get_defence() as f32 * 0.35
                            + defender.get_resistance() as f32 * 1.2
                            + defender.get_avoid() as f32 * 0.4
                            + 10.0)
                        / 730.0
                        + 20.0
                }
            }
            _ => {
                let success = ((attacker.get_level() + 8) - defender.get_level()
                    + rng.gen_range(1..=80)) as f32
                    * (attacker.get_hit() as f32 - defender.get_avoid() as f32 * 0.6
                        + rng.gen_range(1..=50) as f32
                        + 50.0)
                    / 90.0;
                if success < 10.0 {
                    0.0
                } else if success < 20.0 {
                    ((skill_data.power as f32 + 40.0)
                        * (attacker.get_attack_power() as f32 + 40.0)
                        * (rng.gen_range(1..=30) as f32
                            + attacker.get_critical() as f32 * 0.2
                            + 40.0))
                        * 0.4
                        / (defender.get_defence() as f32
                            + defender.get_resistance() as f32 * 0.3
                            + defender.get_avoid() as f32 * 0.4
                            + 10.0)
                        / 80.0
                        + 5.0
                } else if matches!(attacker.damage_category, DamageCategory::Character)
                    && matches!(defender.damage_category, DamageCategory::Character)
                {
                    ((skill_data.power as f32 + attacker.get_critical() as f32 * 0.15 + 40.0)
                        * attacker.get_attack_power() as f32
                        * (rng.gen_range(1..=30) as f32
                            + attacker.get_critical() as f32 * 0.32
                            + 35.0))
                        * 0.01
                        * (350 - defender.get_level() + attacker.get_level()) as f32
                        / (defender.get_defence() as f32
                            + defender.get_resistance() as f32 * 0.3
                            + defender.get_avoid() as f32 * 0.4
                            + 35.0)
                        / 400.0
                        + 20.0
                } else {
                    ((skill_data.power as f32 + attacker.get_critical() as f32 * 0.15 + 40.0)
                        * attacker.get_attack_power() as f32
                        * (rng.gen_range(1..=30) as f32
                            + attacker.get_critical() as f32 * 0.32
                            + 35.0))
                        * 0.01
                        * (120 - defender.get_level() + attacker.get_level()) as f32
                        / (defender.get_defence() as f32
                            + defender.get_resistance() as f32 * 0.3
                            + defender.get_avoid() as f32 * 0.4
                            + 10.0)
                        / 100.0
                        + 20.0
                }
            }
        };

        damage *= attacker.get_additional_damage_multipler();
        damage = f32::max(damage, 5.0) * hit_count as f32;

        if attacker.get_damage_category() == DamageCategory::Character
            && defender.get_damage_category() == DamageCategory::Character
        {
            damage = f32::min(damage, defender.get_max_health() as f32 * 0.45);
        }

        damage = f32::min(damage, 2047.0);

        let apply_hit_stun = (damage * (rng.gen_range(1..=100) as f32 + 100.0)
            / (defender.get_avoid() as f32 + 40.0)
            / 14.0)
            >= 10.0;

        Damage {
            amount: damage as u32,
            is_critical: false,
            apply_hit_stun,
        }
    }

    fn calculate_give_xp(
        &self,
        attacker_level: i32,
        attacker_damage: i32,
        defender_level: i32,
        defender_max_hp: i32,
        defender_reward_xp: i32,
        world_xp_rate: i32,
    ) -> i32 {
        let level_difference = attacker_level - defender_level;
        let attacker_damage = attacker_damage as f32;
        let defender_level = defender_level as f32;
        let defender_max_hp = defender_max_hp as f32;
        let defender_reward_xp = defender_reward_xp as f32;
        let world_xp_rate = world_xp_rate as f32;

        (if level_difference < 3 {
            ((defender_level + 3.0)
                * defender_reward_xp
                * (attacker_damage + defender_max_hp / 15.0 + 30.0)
                * world_xp_rate)
                / defender_max_hp
                / 370.0
        } else {
            ((defender_level + 3.0)
                * defender_reward_xp
                * (attacker_damage + defender_max_hp / 15.0 + 30.0)
                * world_xp_rate)
                / defender_max_hp
                / (level_difference + 3) as f32
                / 60.0
        }) as i32
    }

    fn calculate_give_stamina(
        &self,
        experience_points: i32,
        level: i32,
        world_stamina_rate: i32,
    ) -> i32 {
        (((experience_points + 100) as f32 / (level + 6) as f32)
            * (world_stamina_rate as f32 / 80.0)) as i32
    }

    fn calculate_basic_stat_increase_cost(
        &self,
        basic_stats: &BasicStats,
        basic_stat_type: BasicStatType,
    ) -> Option<u32> {
        let current = match basic_stat_type {
            BasicStatType::Strength => basic_stats.strength,
            BasicStatType::Dexterity => basic_stats.dexterity,
            BasicStatType::Intelligence => basic_stats.intelligence,
            BasicStatType::Concentration => basic_stats.concentration,
            BasicStatType::Charm => basic_stats.charm,
            BasicStatType::Sense => basic_stats.sense,
        };

        if current > MAX_BASIC_STAT_VALUE {
            None
        } else {
            Some((current as f32 * 0.2) as u32)
        }
    }

    fn calculate_levelup_require_xp(&self, level: u32) -> u64 {
        match level as u64 {
            0..=15 => (((level + 3) * (level + 5) * (level + 10)) as f64 * 0.7) as u64,
            16..=60 => (((level - 5) * (level + 2) * (level + 2)) as f64 * 2.2) as u64,
            61..=113 => (((level - 11) * (level) * (level + 4)) as f64 * 2.5) as u64,
            114..=150 => (((level - 31) * (level - 20) * (level + 4)) as f64 * 3.8) as u64,
            151..=189 => (((level - 67) * (level - 20) * (level - 10)) as f64 * 6.0) as u64,
            190..=u64::MAX => {
                ((level - 90) * (level - 120) * (level - 60) * (level - 170) * (level - 188)) as u64
            }
        }
    }

    fn calculate_levelup_reward_skill_points(&self, level: u32) -> u32 {
        (level + 2) / 2
    }

    fn calculate_levelup_reward_stat_points(&self, level: u32) -> u32 {
        (level as f32 * 0.8) as u32 + 10
    }

    fn calculate_reward_value(
        &self,
        equation_id: usize,
        base_reward_value: i32,
        dup_count: i32,
        level: i32,
        charm: i32,
        fame: i32,
        world_reward_rate: i32,
    ) -> i32 {
        match equation_id {
            0 => {
                ((base_reward_value + 30) * (charm + 10) * world_reward_rate * (fame + 20)
                    / (level + 70))
                    / 30000
                    + base_reward_value
            }
            1 => {
                base_reward_value * (level + 3) * (level + charm / 2 + 40) * world_reward_rate
                    / 10000
            }
            2 => base_reward_value * dup_count,
            3 | 5 => {
                ((base_reward_value + 20) * (charm + 10) * world_reward_rate * (fame + 20)
                    / (level + 70))
                    / 30000
                    + base_reward_value
            }
            4 => {
                ((base_reward_value + 2) * (level + charm + 40) * (fame + 40) * world_reward_rate)
                    / 140000
            }
            6 => {
                ((base_reward_value + 20) * (level + charm) * (fame + 20) * world_reward_rate)
                    / 3000000
                    + base_reward_value
            }
            _ => 0,
        }
    }

    fn calculate_npc_store_item_buy_price(
        &self,
        item_database: &ItemDatabase,
        item: ItemReference,
        buy_skill_value: i32,
        item_rate: i32,
        town_rate: i32,
    ) -> Option<i32> {
        let item_data = item_database.get_base_item(item)?;

        match item.item_type {
            ItemType::Face
            | ItemType::Head
            | ItemType::Body
            | ItemType::Hands
            | ItemType::Feet
            | ItemType::Back
            | ItemType::Weapon
            | ItemType::SubWeapon
            | ItemType::Vehicle => Some(
                (item_data.base_price as f32
                    * (item_data.quality as f32 + 50.0)
                    * (1.0 - buy_skill_value as f32 * 0.01)
                    / 100.0
                    + 0.5) as i32,
            ),
            ItemType::Consumable
            | ItemType::Material
            | ItemType::Jewellery
            | ItemType::Gem
            | ItemType::Quest => {
                let item_rate = if matches!(
                    item_data.class,
                    ItemClass::Medicine
                        | ItemClass::Food
                        | ItemClass::Metal
                        | ItemClass::OtherworldlyMetal
                        | ItemClass::StoneMaterial
                        | ItemClass::WoodenMaterial
                        | ItemClass::Leather
                        | ItemClass::Cloth
                        | ItemClass::RefiningMaterial
                        | ItemClass::Chemicals
                ) {
                    item_rate
                } else {
                    town_rate
                };

                Some(
                    (item_data.base_price as f32
                        * (1.0 + (item_rate as f32 - 50.0) * item_data.price_rate as f32 / 1000.0)
                        * (1.0 - buy_skill_value as f32 * 0.01)
                        + 0.5) as i32,
                )
            }
        }
    }

    fn calculate_npc_store_item_sell_price(
        &self,
        item_database: &ItemDatabase,
        item: &Item,
        sell_skill_value: i32,
        world_rate: i32,
        item_rate: i32,
        town_rate: i32,
    ) -> Option<i32> {
        let item_data = item_database.get_base_item(item.get_item_reference())?;
        match item.get_item_type() {
            ItemType::Face
            | ItemType::Head
            | ItemType::Body
            | ItemType::Hands
            | ItemType::Feet
            | ItemType::Back
            | ItemType::Weapon
            | ItemType::SubWeapon
            | ItemType::Vehicle => {
                let item = item.as_equipment().unwrap();
                let gem_base_price = if item.is_appraised {
                    item_database
                        .get_base_item(ItemReference::gem(item.gem as usize))
                        .map(|gem_item_data| gem_item_data.base_price)
                        .unwrap_or(0)
                } else {
                    0
                } as f32;
                Some(
                    ((item_data.base_price as f32
                        * (40.0 + item.grade as f32)
                        * (200.0 + item.durability as f32)
                        * (200.0 - world_rate as f32)
                        * (1.0 + sell_skill_value as f32 * 0.01)
                        * ((4000.0 + item.life as f32) / 14000.0)
                        / 1000000.0)
                        + gem_base_price * 0.2) as i32,
                )
            }
            ItemType::Consumable
            | ItemType::Material
            | ItemType::Jewellery
            | ItemType::Gem
            | ItemType::Quest => {
                let item_rate = if matches!(
                    item_data.class,
                    ItemClass::Medicine
                        | ItemClass::Food
                        | ItemClass::Metal
                        | ItemClass::OtherworldlyMetal
                        | ItemClass::StoneMaterial
                        | ItemClass::WoodenMaterial
                        | ItemClass::Leather
                        | ItemClass::Cloth
                        | ItemClass::RefiningMaterial
                        | ItemClass::Chemicals
                ) {
                    item_rate
                } else {
                    town_rate
                };

                Some(
                    (item_data.base_price as f32
                        * (1000.0 + (item_rate as f32 - 50.0) * item_data.price_rate as f32)
                        * (1.0 + sell_skill_value as f32 * 0.01)
                        * (200.0 - world_rate as f32)
                        / 180000.0) as i32,
                )
            }
        }
    }

    fn calculate_passive_recover_hp(
        &self,
        ability_values: &AbilityValues,
        recovery_state: PassiveRecoveryState,
    ) -> i32 {
        match recovery_state {
            PassiveRecoveryState::Normal => {
                ((ability_values.get_additional_health_recovery() as f32
                    + (ability_values.get_concentration() as f32 + 40.0) / 6.0)
                    / 6.0) as i32
            }
            PassiveRecoveryState::Sitting => {
                (ability_values.get_additional_health_recovery() as f32
                    + ((((ability_values.get_concentration() as f32 + 30.0) / 8.0) * 23.0) / 10.0))
                    as i32
            }
        }
    }

    fn calculate_passive_recover_mp(
        &self,
        ability_values: &AbilityValues,
        recovery_state: PassiveRecoveryState,
    ) -> i32 {
        match recovery_state {
            PassiveRecoveryState::Normal => 0,
            PassiveRecoveryState::Sitting => {
                (ability_values.get_additional_mana_recovery() as f32
                    + ((((ability_values.get_concentration() as f32 + 20.0) / 10.0) * 20.0) / 7.0))
                    as i32
            }
        }
    }

    fn calculate_decrease_weapon_life(
        &self,
        is_driving: bool,
        equipment: &Equipment,
    ) -> Option<ItemSlot> {
        let (equipment_item, item_slot) = if is_driving {
            (
                equipment.get_vehicle_item(VehiclePartIndex::Arms),
                ItemSlot::Vehicle(VehiclePartIndex::Arms),
            )
        } else {
            (
                equipment.get_equipment_item(EquipmentIndex::Weapon),
                ItemSlot::Equipment(EquipmentIndex::Weapon),
            )
        };

        let equipment_item = equipment_item?;
        if equipment_item.life == 0 {
            return None;
        }

        let mut rng = rand::thread_rng();
        if rng.gen_range(1..=710) >= equipment_item.durability as i32 + 600 {
            Some(item_slot)
        } else {
            None
        }
    }

    fn calculate_decrease_armour_life(
        &self,
        is_driving: bool,
        equipment: &Equipment,
        damage: &Damage,
    ) -> Option<ItemSlot> {
        let mut rng = rand::thread_rng();
        let rand_value = rng.gen_range(1..=400);

        if rand_value >= 101 {
            return None;
        }

        let (equipment_item, item_slot) = if is_driving {
            let vehicle_part_index = if rand_value >= 51 {
                VehiclePartIndex::Body
            } else if rand_value >= 21 {
                VehiclePartIndex::Leg
            } else {
                VehiclePartIndex::Arms
            };

            (
                equipment.get_vehicle_item(vehicle_part_index),
                ItemSlot::Vehicle(vehicle_part_index),
            )
        } else {
            let equipment_index = if equipment
                .get_equipment_item(EquipmentIndex::SubWeapon)
                .filter(|item| item.life > 0)
                .and_then(|item| self.item_database.get_base_item(item.item))
                .map_or(false, |item_data| {
                    matches!(item_data.class, ItemClass::Shield)
                }) {
                if rand_value > 60 {
                    EquipmentIndex::SubWeapon
                } else if rand_value > 30 {
                    EquipmentIndex::Body
                } else if rand_value > 22 {
                    EquipmentIndex::Feet
                } else if rand_value > 16 {
                    EquipmentIndex::Head
                } else if rand_value > 9 {
                    EquipmentIndex::Back
                } else if rand_value > 5 {
                    EquipmentIndex::Hands
                } else {
                    EquipmentIndex::Face
                }
            } else if rand_value > 64 {
                EquipmentIndex::Body
            } else if rand_value > 46 {
                EquipmentIndex::Feet
            } else if rand_value > 31 {
                EquipmentIndex::Hands
            } else if rand_value > 15 {
                EquipmentIndex::Back
            } else if rand_value > 7 {
                EquipmentIndex::Head
            } else {
                EquipmentIndex::Face
            };

            (
                equipment.get_equipment_item(equipment_index),
                ItemSlot::Equipment(equipment_index),
            )
        };

        let equipment_item = equipment_item?;
        if equipment_item.life == 0 {
            return None;
        }

        if rng.gen_range(1..=120)
            - (equipment_item.durability as i32 + 10 - (damage.amount / 10) as i32)
            >= 0
        {
            Some(item_slot)
        } else {
            None
        }
    }

    fn calculate_repair_from_npc_price(&self, item: &EquipmentItem) -> Money {
        let base_item_price = self
            .item_database
            .get_base_item(item.item)
            .map_or(0, |item_data| item_data.base_price);

        Money(
            (((base_item_price + 1000) as f32 / 400000.0)
                * (item.durability + 10) as f32
                * (1100 - item.life) as f32) as i64,
        )
    }

    fn calculate_clan_max_members(&self, level: NonZeroU32) -> usize {
        match level.get() {
            1 => 15,
            2 => 20,
            3 => 25,
            4 => 30,
            5 => 36,
            6 => 43,
            _ => 50,
        }
    }
}

fn calculate_damage_success_rate(
    rng: &mut impl Rng,
    attacker: &AbilityValues,
    defender: &AbilityValues,
) -> i32 {
    if attacker.get_damage_category() == DamageCategory::Character
        && defender.get_damage_category() == DamageCategory::Character
    {
        40 - 60 * ((attacker.get_hit() + defender.get_avoid()) / attacker.get_avoid())
            + rng.gen_range(1..=100)
    } else {
        let value = (attacker.get_level() + 10) - (defender.get_level() as f32 * 1.1) as i32
            + rng.gen_range(1..=50);
        if value <= 0 {
            0
        } else {
            (value as f32
                * ((attacker.get_hit() as f32 * 1.1 - defender.get_avoid() as f32 * 0.93
                    + rng.gen_range(1..=60) as f32
                    + 5.0
                    + attacker.get_level() as f32 * 0.2)
                    / 80.0)) as i32
        }
    }
}

fn calculate_attack_damage_physical(
    rng: &mut impl Rng,
    attacker: &AbilityValues,
    defender: &AbilityValues,
    hit_count: i32,
    success_rate: i32,
) -> Damage {
    let crit_success_rate = 16 * (3 * rng.gen_range(1..=100) + attacker.get_level() + 30)
        / (attacker.get_critical() + 70);
    let apply_hit_stun = ((28 - crit_success_rate) * (attacker.get_attack_power() + 20)
        / (defender.get_defence() + 5))
        >= 10;

    if crit_success_rate < 20 {
        // Critical physical damage
        let mut damage = if attacker.get_damage_category() == DamageCategory::Character
            && defender.get_damage_category() == DamageCategory::Character
        {
            attacker.get_attack_power() as f32
                * (success_rate as f32 * 0.05 + 35.0)
                * ((attacker.get_attack_power() - defender.get_defence() + 430) as f32
                    / (300.0
                        * (defender.get_defence() as f32
                            + defender.get_avoid() as f32 * 0.4
                            + 10.0)))
                + 25.0
        } else {
            attacker.get_attack_power() as f32
                * (success_rate as f32 * 0.05 + 29.0)
                * ((attacker.get_attack_power() - defender.get_defence() + 230) as f32
                    / (100.0
                        * (defender.get_defence() as f32
                            + defender.get_avoid() as f32 * 0.3
                            + 5.0)))
        };

        damage *= attacker.get_additional_damage_multipler();
        damage = f32::max(damage * hit_count as f32, 10.0);

        if attacker.get_damage_category() == DamageCategory::Character
            && defender.get_damage_category() == DamageCategory::Character
        {
            damage = f32::min(damage, defender.get_max_health() as f32 * 0.35);
        }

        damage = f32::min(damage, 2047.0);

        Damage {
            amount: damage as u32,
            is_critical: true,
            apply_hit_stun,
        }
    } else {
        // Normal physical damage
        let mut damage = if attacker.get_damage_category() == DamageCategory::Character
            && defender.get_damage_category() == DamageCategory::Character
        {
            attacker.get_attack_power() as f32
                * (success_rate as f32 * 0.05 + 25.0)
                * ((attacker.get_attack_power() - defender.get_defence() + 400) as f32
                    / (420.0
                        * (defender.get_defence() as f32
                            + defender.get_avoid() as f32 * 0.4
                            + 5.0)))
                + 20.0
        } else {
            attacker.get_attack_power() as f32
                * (success_rate as f32 * 0.03 + 26.0)
                * ((attacker.get_attack_power() - defender.get_defence() + 250) as f32
                    / (145.0
                        * (defender.get_defence() as f32
                            + defender.get_avoid() as f32 * 0.4
                            + 5.0)))
        };

        damage *= attacker.get_additional_damage_multipler();
        damage = f32::max(damage * hit_count as f32, 5.0);

        if attacker.get_damage_category() == DamageCategory::Character
            && defender.get_damage_category() == DamageCategory::Character
        {
            damage = f32::min(damage, defender.get_max_health() as f32 * 0.25);
        }

        damage = f32::min(damage, 2047.0);

        Damage {
            amount: damage as u32,
            is_critical: false,
            apply_hit_stun,
        }
    }
}

fn calculate_attack_damage_magic(
    rng: &mut impl Rng,
    attacker: &AbilityValues,
    defender: &AbilityValues,
    hit_count: i32,
    success_rate: i32,
) -> Damage {
    let crit_success_rate = 16 * (3 * rng.gen_range(1..=100) + attacker.get_level() + 30)
        / (attacker.get_critical() + 70);
    let apply_hit_stun = ((28 - crit_success_rate) * (attacker.get_attack_power() + 20)
        / (defender.get_defence() + 5))
        >= 10;

    if crit_success_rate < 20 {
        // Critical magic damage
        let mut damage = if attacker.get_damage_category() == DamageCategory::Character
            && defender.get_damage_category() == DamageCategory::Character
        {
            attacker.get_attack_power() as f32
                * (success_rate as f32 * 0.05 + 33.0)
                * ((attacker.get_attack_power() - defender.get_defence() + 340) as f32
                    / (360.0
                        * (defender.get_resistance() as f32
                            + defender.get_avoid() as f32 * 0.3
                            + 20.0)))
                + 25.0
        } else {
            attacker.get_attack_power() as f32
                * (success_rate as f32 * 0.05 + 33.0)
                * ((attacker.get_attack_power() as f32 - defender.get_defence() as f32 * 0.8
                    + 310.0)
                    / (200.0
                        * (defender.get_resistance() as f32
                            + defender.get_avoid() as f32 * 0.3
                            + 5.0)))
        };

        damage *= attacker.get_additional_damage_multipler();
        damage = f32::max(damage * hit_count as f32, 10.0);

        if attacker.get_damage_category() == DamageCategory::Character
            && defender.get_damage_category() == DamageCategory::Character
        {
            damage = f32::min(damage, defender.get_max_health() as f32 * 0.35);
        }

        damage = f32::min(damage, 2047.0);

        Damage {
            amount: damage as u32,
            is_critical: true,
            apply_hit_stun,
        }
    } else {
        // Normal magic damage
        let mut damage = if attacker.get_damage_category() == DamageCategory::Character
            && defender.get_damage_category() == DamageCategory::Character
        {
            attacker.get_attack_power() as f32
                * (success_rate as f32 * 0.06 + 29.0)
                * ((attacker.get_attack_power() as f32 - defender.get_defence() as f32 * 0.8
                    + 350.0)
                    / (640.0
                        * (defender.get_resistance() as f32
                            + defender.get_avoid() as f32 * 0.3
                            + 5.0)))
                + 20.0
        } else {
            attacker.get_attack_power() as f32
                * (success_rate as f32 * 0.03 + 30.0)
                * ((attacker.get_attack_power() as f32 - defender.get_defence() as f32 * 0.8
                    + 280.0)
                    / (280.0
                        * (defender.get_resistance() as f32
                            + defender.get_avoid() as f32 * 0.3
                            + 5.0)))
        };

        damage *= attacker.get_additional_damage_multipler();
        damage = f32::max(damage * hit_count as f32, 5.0);

        if attacker.get_damage_category() == DamageCategory::Character
            && defender.get_damage_category() == DamageCategory::Character
        {
            damage = f32::min(damage, defender.get_max_health() as f32 * 0.25);
        }

        damage = f32::min(damage, 2047.0);

        Damage {
            amount: damage as u32,
            is_critical: false,
            apply_hit_stun,
        }
    }
}

#[derive(Default)]
struct EquipmentAbilityValue {
    pub gender: i32,
    pub birthstone: i32,
    pub job: i32,
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
    pub max_weight: i32,
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
            AbilityType::Job => self.job += value,
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
            AbilityType::Weight => self.max_weight += value,
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
                error!("Item has unimplemented ability type {:?}", ability_type)
            }
        }
    }
}

fn calculate_equipment_ability_values(
    item_database: &ItemDatabase,
    equipment: &Equipment,
) -> EquipmentAbilityValue {
    let mut result = EquipmentAbilityValue::new();

    for item in equipment.iter_equipped_items() {
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

    result
}

fn calculate_vehicle_ability_values(
    item_database: &ItemDatabase,
    equipment: &Equipment,
) -> EquipmentAbilityValue {
    let mut result = EquipmentAbilityValue::new();

    for item in equipment.iter_equipped_vehicles() {
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
    max_weight: i32,
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
            AbilityType::PassiveWeight => abilities.max_weight += value,
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
                error!(
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

fn calculate_passive_skill_ability_values(
    skill_database: &SkillDatabase,
    skill_list: &SkillList,
) -> PassiveSkillAbilityValues {
    let mut result = PassiveSkillAbilityValues::new();

    if let Some(passive_skills) = skill_list.get_page(IroseSkillPageType::Passive as usize) {
        for skill_id in passive_skills.skills.iter().filter_map(|x| *x) {
            if let Some(skill_data) = skill_database.get_skill(skill_id) {
                for add_ability in skill_data.add_ability.iter().filter_map(|x| x.as_ref()) {
                    if add_ability.rate != 0 {
                        result.add_ability_rate(add_ability.ability_type, add_ability.rate);
                    } else {
                        result.add_ability_value(add_ability.ability_type, add_ability.value);
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
        .and_then(|item| item_database.get_back_item(item.item.item_number))
        .map(|item_data| item_data.move_speed)
        .unwrap_or(0) as f32;

    let item_run_speed = item_speed * (basic_stats.dexterity as f32 + 500.0) / 100.0
        + equipment_ability_values.move_speed as f32;

    let passive_run_speed = passive_ability_values.value.move_speed as f32
        + item_run_speed * (passive_ability_values.rate.move_speed as f32 / 100.0);

    item_run_speed + passive_run_speed
}

fn calculate_vehicle_move_speed(
    item_database: &ItemDatabase,
    vehicle_ability_values: &EquipmentAbilityValue,
    equipment: &Equipment,
) -> f32 {
    let engine_item = equipment.get_vehicle_item(VehiclePartIndex::Engine);
    let leg_item = equipment.get_vehicle_item(VehiclePartIndex::Leg);

    let broken = engine_item.map_or(true, |item| item.is_broken())
        || leg_item.map_or(true, |item| item.is_broken());

    let item_speed = if broken {
        200.0
    } else {
        engine_item
            .and_then(|item| item_database.get_vehicle_item(item.item.item_number))
            .map(|item_data| item_data.move_speed)
            .unwrap_or(0) as f32
            * leg_item
                .and_then(|item| item_database.get_vehicle_item(item.item.item_number))
                .map(|item_data| item_data.move_speed)
                .unwrap_or(0) as f32
            / 10.0
    };

    // TODO: Limit speed to 300 if > max weight

    item_speed + vehicle_ability_values.move_speed as f32
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
        + basic_stats.strength * strength_multipler
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
        + basic_stats.intelligence * int_multipler
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
            .and_then(|item| item_database.get_material_item(item.item.item_number))
            .map(|item| item.item_data.quality)
            .unwrap_or(0) as f32
    };

    let weapon = equipment
        .get_equipment_item(EquipmentIndex::Weapon)
        .filter(|item| !item.is_broken())
        .and_then(|item| {
            item_database
                .get_weapon_item(item.item.item_number)
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
        + (attack_power * passive_attack_rate);

    (attack_power + passive_attack_power) as i32
}

fn calculate_vehicle_attack_power(
    item_database: &ItemDatabase,
    basic_stats: &BasicStats,
    level: &Level,
    vehicle_ability_values: &EquipmentAbilityValue,
    equipment: &Equipment,
) -> i32 {
    let concentration = basic_stats.concentration as f32;
    let level = level.level as f32;

    let arms_attack_power = equipment
        .get_vehicle_item(VehiclePartIndex::Arms)
        .filter(|item| !item.is_broken())
        .and_then(|item| {
            item_database
                .get_vehicle_item(item.item.item_number)
                .map(|item_data| item_data.attack_power as f32)
        })
        .unwrap_or(0.0);

    let attack_power =
        level * 3.0 + concentration + arms_attack_power + vehicle_ability_values.attack as f32;

    attack_power as i32
}

fn calculate_attack_speed(
    item_database: &ItemDatabase,
    equipment: &Equipment,
    equipment_ability_values: &EquipmentAbilityValue,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> (i32, i32) {
    let (weapon_attack_speed, weapon_item_class) = item_database
        .get_weapon_item(
            equipment
                .get_equipment_item(EquipmentIndex::Weapon)
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

    (
        (attack_speed + passive_attack_speed + equipment_ability_values.attack_speed as f32) as i32,
        passive_attack_speed as i32,
    )
}

fn calculate_vehicle_attack_speed(
    item_database: &ItemDatabase,
    equipment: &Equipment,
    vehicle_ability_values: &EquipmentAbilityValue,
) -> i32 {
    let arms_attack_speed = equipment
        .get_vehicle_item(VehiclePartIndex::Arms)
        .filter(|item| !item.is_broken())
        .and_then(|item| {
            item_database
                .get_vehicle_item(item.item.item_number)
                .map(|item_data| item_data.attack_speed as f32)
        })
        .unwrap_or(0.0);

    let attack_speed = 1500.0 / (arms_attack_speed + 5.0);

    (attack_speed + vehicle_ability_values.attack_speed as f32) as i32
}

fn calculate_attack_range(item_database: &ItemDatabase, equipment: &Equipment) -> i32 {
    let weapon_attack_range = item_database
        .get_weapon_item(
            equipment
                .get_equipment_item(EquipmentIndex::Weapon)
                .map(|item| item.item.item_number)
                .unwrap_or(0),
        )
        .map(|weapon| weapon.attack_range)
        .unwrap_or(70);

    let scale = 1.0;

    weapon_attack_range + (scale * 120.0) as i32
}

fn calculate_vehicle_attack_range(item_database: &ItemDatabase, equipment: &Equipment) -> i32 {
    let arms_attack_range = equipment
        .get_vehicle_item(VehiclePartIndex::Arms)
        .and_then(|item| {
            item_database
                .get_vehicle_item(item.item.item_number)
                .map(|item_data| item_data.attack_range)
        })
        .unwrap_or(0);

    let scale = 1.0;

    arms_attack_range + (scale * 120.0) as i32
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
        .get_equipment_item(EquipmentIndex::Weapon)
        .filter(|item| !item.is_broken())
        .and_then(|item| {
            item_database
                .get_weapon_item(item.item.item_number)
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
    let passive_hit = passive_ability_values.value.hit as f32 + (hit * passive_hit_rate);

    (hit + passive_hit) as i32
}

fn calculate_vehicle_hit(
    item_database: &ItemDatabase,
    basic_stats: &BasicStats,
    level: &Level,
    vehicle_ability_values: &EquipmentAbilityValue,
    equipment: &Equipment,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> i32 {
    let concentration = basic_stats.concentration as f32;
    let level = level.level as f32;

    let hit = if let Some(arms_item_quality) = equipment
        .get_vehicle_item(VehiclePartIndex::Arms)
        .filter(|item| !item.is_broken())
        .and_then(|item| {
            item_database
                .get_vehicle_item(item.item.item_number)
                .map(|item_data| item_data.item_data.quality as f32)
        }) {
        (concentration + 10.0) * 0.8 + level * 0.5 + arms_item_quality * 1.2
    } else {
        0.0
    } + vehicle_ability_values.hit as f32;

    let passive_hit_rate = passive_ability_values.rate.hit as f32 / 100.0;
    let passive_hit = passive_ability_values.value.hit as f32 + (hit * passive_hit_rate);

    (hit + passive_hit) as i32
}

fn calculate_defence(
    item_database: &ItemDatabase,
    basic_stats: &BasicStats,
    level: &Level,
    equipment_ability_values: &EquipmentAbilityValue,
    equipment: &Equipment,
    passive_ability_values: &PassiveSkillAbilityValues,
    for_vehicle: bool,
) -> i32 {
    let mut item_defence = 0;

    for item in equipment.iter_equipped_items().filter(|item| item.life > 0) {
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

    if for_vehicle {
        for item in equipment
            .iter_equipped_vehicles()
            .filter(|item| item.life > 0)
        {
            item_defence += item_database
                .get_item_grade(item.grade)
                .map(|grade| grade.defence)
                .unwrap_or(0);
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
        passive_ability_values.value.defence as f32 + (defence * passive_defence_rate);

    let mut defence = (defence + passive_defence) as i32;

    if let Some(offhand_item) = equipment.get_equipment_item(EquipmentIndex::SubWeapon) {
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

    for item in equipment.iter_equipped_items().filter(|item| item.life > 0) {
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
    let passive_resistance =
        passive_ability_values.value.resistance as f32 + (resistance * passive_resistance_rate);

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
        passive_ability_values.value.critical as f32 + (critical * passive_critical_rate);

    (critical + passive_critical) as i32
}

fn calculate_vehicle_critical(
    basic_stats: &BasicStats,
    level: &Level,
    vehicle_ability_values: &EquipmentAbilityValue,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> i32 {
    let level = level.level as f32;
    let sense = basic_stats.sense as f32;
    let critical = sense * 0.8 + level * 0.3 + vehicle_ability_values.critical as f32;

    let passive_critical_rate = passive_ability_values.rate.critical as f32 / 100.0;
    let passive_critical =
        passive_ability_values.value.critical as f32 + (critical * passive_critical_rate);

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
        EquipmentIndex::SubWeapon,
    ];

    // Get total durability for specific set of equipment
    let equipment_durability: i32 = AVOID_DURABILITY_ITEMS
        .iter()
        .filter_map(|x| equipment.get_equipment_item(*x))
        .filter(|x| x.life > 0)
        .map(|item| item.durability as i32)
        .sum();

    // Count grade on all items which have defence stat > 0
    let mut equipment_total_grade = 0;
    for item in equipment.iter_equipped_items().filter(|item| item.life > 0) {
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
    let passive_avoid = passive_ability_values.value.avoid as f32 + (avoid * passive_avoid_rate);

    (avoid + passive_avoid) as i32
}

fn calculate_vehicle_avoid(
    basic_stats: &BasicStats,
    level: &Level,
    vehicle_ability_values: &EquipmentAbilityValue,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> i32 {
    let dexterity = basic_stats.dexterity as f32;
    let level = level.level as f32;
    let avoid = (dexterity + 10.0) * 0.8 + level * 0.5 + vehicle_ability_values.avoid as f32;

    let passive_avoid_rate = passive_ability_values.rate.avoid as f32 / 100.0;
    let passive_avoid = passive_ability_values.value.avoid as f32 + (avoid * passive_avoid_rate);

    (avoid + passive_avoid) as i32
}

fn calculate_drop_rate(
    equipment_ability_values: &EquipmentAbilityValue,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> i32 {
    let drop_rate = equipment_ability_values.drop_rate as f32;
    let passive_drop_rate = passive_ability_values.value.drop_rate as f32
        + (drop_rate * passive_ability_values.rate.drop_rate as f32 / 100.0);

    (drop_rate + passive_drop_rate) as i32
}

fn calculate_save_mana(
    equipment_ability_values: &EquipmentAbilityValue,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> i32 {
    let save_mana = equipment_ability_values.save_mana as f32;
    let passive_save_mana = passive_ability_values.value.save_mana as f32
        + (save_mana * passive_ability_values.rate.save_mana as f32 / 100.0);

    (save_mana + passive_save_mana) as i32
}

fn calculate_max_weight(
    item_database: &ItemDatabase,
    level: &Level,
    basic_stats: &BasicStats,
    equipment: &Equipment,
    equipment_ability_values: &EquipmentAbilityValue,
    passive_ability_values: &PassiveSkillAbilityValues,
) -> i32 {
    let mut max_weight = 1100
        + level.level as i32 * 5
        + basic_stats.strength * 6
        + equipment_ability_values.max_weight;

    // If user has a bag equipped, apply max weight passives
    if equipment
        .get_equipment_item(EquipmentIndex::Back)
        .filter(|x| x.life > 0)
        .and_then(|x| item_database.get_base_item(x.item))
        .filter(|x| matches!(x.class, ItemClass::Bag))
        .is_some()
    {
        max_weight += (passive_ability_values.value.max_weight as f32
            + max_weight as f32 * passive_ability_values.rate.max_weight as f32 / 100.0)
            as i32;
    }

    max_weight
}
