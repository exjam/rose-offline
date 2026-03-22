use bevy::prelude::Component;
use rand::seq::SliceRandom;

use rose_data::{
    AbilityType, AmmoIndex, EquipmentIndex, EquipmentItem, ItemClass, ItemReference, ItemType,
    JobId, SkillId, SkillIds, StackableItem,
};
use rose_game_common::components::{
    AbilityValues, BasicStatType, BasicStats, CharacterGender, StatPoints, StatusEffects,
};

use crate::game::{
    bundles::{skill_list_try_learn_skill, skill_list_try_level_up_skill, SkillListBundle},
    storage::character::CharacterStorage,
    GameData,
};

const BOT_GENDERS: &[CharacterGender] = &[CharacterGender::Male, CharacterGender::Female];
const BOT_FACES: &[u8] = &[1, 8, 15, 22, 29, 36, 43];
const BOT_HAIRS: &[u8] = &[0, 5, 10, 15, 20];

#[derive(Component)]
pub struct BotBuild {
    pub job_id: JobId,
    pub basic_stat_ratios: Vec<(BasicStatType, f32)>,
    pub weapon_type: ItemClass,
    pub subweapon_type: Option<ItemClass>,
    pub skills: Vec<SkillId>,
}

fn calculate_ratios(target: &[(BasicStatType, i32)]) -> Vec<(BasicStatType, f32)> {
    let total: i32 = target.iter().map(|(_, v)| *v).sum();

    target
        .iter()
        .map(|(ability_type, value)| (*ability_type, *value as f32 / total as f32))
        .collect()
}

pub fn bot_build_knight() -> BotBuild {
    BotBuild {
        job_id: JobId::new(121),
        basic_stat_ratios: calculate_ratios(&[
            (BasicStatType::Strength, 210),
            (BasicStatType::Sense, 175),
            (BasicStatType::Concentration, 90),
            (BasicStatType::Intelligence, 25),
        ]),
        weapon_type: ItemClass::OneHandedSword,
        subweapon_type: Some(ItemClass::Shield),
        skills: [
            SkillId::new(SkillIds::MeleeWeaponMastery as u16).unwrap(),
            SkillId::new(SkillIds::PhysicalTraining as u16).unwrap(),
            SkillId::new(SkillIds::QuickStep as u16).unwrap(),
            SkillId::new(SkillIds::DefenceTraining as u16).unwrap(),
            SkillId::new(SkillIds::OneHandedMastery as u16).unwrap(),
            SkillId::new(SkillIds::DoubleAttack as u16).unwrap(),
            SkillId::new(SkillIds::LeapAttack as u16).unwrap(),
            SkillId::new(SkillIds::HeavyAttack as u16).unwrap(),
            SkillId::new(SkillIds::LightningCrusher as u16).unwrap(),
            SkillId::new(SkillIds::Taunt as u16).unwrap(),
            SkillId::new(SkillIds::SpiritualTraining as u16).unwrap(),
            SkillId::new(SkillIds::DivineForce as u16).unwrap(),
        ]
        .into(),
    }
}

pub fn bot_build_champion() -> BotBuild {
    BotBuild {
        job_id: JobId::new(122),
        basic_stat_ratios: calculate_ratios(&[
            (BasicStatType::Sense, 195),
            (BasicStatType::Strength, 190),
            (BasicStatType::Concentration, 90),
            (BasicStatType::Intelligence, 25),
        ]),
        weapon_type: ItemClass::TwoHandedSword,
        subweapon_type: None,
        skills: [
            SkillId::new(SkillIds::MeleeWeaponMastery as u16).unwrap(),
            SkillId::new(SkillIds::PhysicalTraining as u16).unwrap(),
            SkillId::new(SkillIds::QuickStep as u16).unwrap(),
            SkillId::new(SkillIds::DefenceTraining as u16).unwrap(),
            SkillId::new(SkillIds::TwoHandedMastery as u16).unwrap(),
            SkillId::new(SkillIds::DoubleAttack as u16).unwrap(),
            SkillId::new(SkillIds::LeapAttack as u16).unwrap(),
            SkillId::new(SkillIds::HeavyAttack as u16).unwrap(),
            SkillId::new(SkillIds::ChampionHit as u16).unwrap(),
            SkillId::new(SkillIds::SpaceAttack as u16).unwrap(),
            SkillId::new(SkillIds::SpiritualTraining as u16).unwrap(),
            SkillId::new(SkillIds::DivineForce as u16).unwrap(),
        ]
        .into(),
    }
}

pub fn bot_build_mage() -> BotBuild {
    BotBuild {
        job_id: JobId::new(221),
        basic_stat_ratios: calculate_ratios(&[
            (BasicStatType::Intelligence, 231),
            (BasicStatType::Sense, 130),
            (BasicStatType::Concentration, 90),
            (BasicStatType::Strength, 70),
        ]),
        weapon_type: ItemClass::MagicStaff,
        subweapon_type: None,
        skills: [
            SkillId::new(SkillIds::ManaBolt as u16).unwrap(),
            SkillId::new(SkillIds::StaffStun as u16).unwrap(),
            SkillId::new(SkillIds::FireRing as u16).unwrap(),
            SkillId::new(SkillIds::Lightning as u16).unwrap(),
            SkillId::new(SkillIds::Weaken as u16).unwrap(),
            SkillId::new(SkillIds::Silence as u16).unwrap(),
            SkillId::new(SkillIds::IceBolt as u16).unwrap(),
            SkillId::new(SkillIds::FireBurn as u16).unwrap(),
            SkillId::new(SkillIds::PhantomSword as u16).unwrap(),
            SkillId::new(SkillIds::CallFiregon as u16).unwrap(),
            SkillId::new(SkillIds::SpellMastery as u16).unwrap(),
            SkillId::new(SkillIds::StaffMastery as u16).unwrap(),
        ]
        .into(),
    }
}

pub fn bot_build_cleric() -> BotBuild {
    BotBuild {
        job_id: JobId::new(222),
        basic_stat_ratios: calculate_ratios(&[(BasicStatType::Intelligence, 300)]),
        weapon_type: ItemClass::MagicWand,
        subweapon_type: Some(ItemClass::SupportTool),
        skills: [
            SkillId::new(SkillIds::Cure as u16).unwrap(),
            SkillId::new(SkillIds::FireRing as u16).unwrap(),
            SkillId::new(SkillIds::Bonfire as u16).unwrap(),
            SkillId::new(SkillIds::Resurrection as u16).unwrap(),
            SkillId::new(SkillIds::LesserHaste as u16).unwrap(),
            SkillId::new(SkillIds::PowerSupport as u16).unwrap(),
            SkillId::new(SkillIds::HitSupport as u16).unwrap(),
            SkillId::new(SkillIds::CriticalSupport as u16).unwrap(),
            SkillId::new(SkillIds::Meditation as u16).unwrap(),
            SkillId::new(SkillIds::SpellMastery as u16).unwrap(),
        ]
        .into(),
    }
}

pub fn bot_build_raider() -> BotBuild {
    BotBuild {
        job_id: JobId::new(321),
        basic_stat_ratios: calculate_ratios(&[
            (BasicStatType::Dexterity, 210),
            (BasicStatType::Sense, 175),
            (BasicStatType::Concentration, 90),
            (BasicStatType::Intelligence, 25),
        ]),
        weapon_type: ItemClass::Katar,
        subweapon_type: None,
        skills: [
            SkillId::new(SkillIds::KnuckleMastery as u16).unwrap(),
            SkillId::new(SkillIds::CombatMastery as u16).unwrap(),
            SkillId::new(SkillIds::DoubleSlash as u16).unwrap(),
            SkillId::new(SkillIds::ScrewAttack as u16).unwrap(),
            SkillId::new(SkillIds::PrimeHit as u16).unwrap(),
            SkillId::new(SkillIds::ManaBlood as u16).unwrap(),
            SkillId::new(SkillIds::Stealth as u16).unwrap(),
            SkillId::new(SkillIds::PoisonKnife as u16).unwrap(),
            SkillId::new(SkillIds::MagicKnife as u16).unwrap(),
            SkillId::new(SkillIds::BloodyDust as u16).unwrap(),
        ]
        .into(),
    }
}

pub fn bot_build_scout() -> BotBuild {
    BotBuild {
        job_id: JobId::new(322),
        basic_stat_ratios: calculate_ratios(&[
            (BasicStatType::Dexterity, 210),
            (BasicStatType::Sense, 175),
            (BasicStatType::Concentration, 90),
            (BasicStatType::Intelligence, 25),
        ]),
        weapon_type: ItemClass::Bow,
        subweapon_type: None,
        skills: [
            SkillId::new(SkillIds::BowMastery as u16).unwrap(),
            SkillId::new(SkillIds::CombatMastery as u16).unwrap(),
            SkillId::new(SkillIds::DoubleShot as u16).unwrap(),
            SkillId::new(SkillIds::AimShot as u16).unwrap(),
            SkillId::new(SkillIds::HawkShot as u16).unwrap(),
            SkillId::new(SkillIds::FlameHawk as u16).unwrap(),
            SkillId::new(SkillIds::TrapShot as u16).unwrap(),
            SkillId::new(SkillIds::Detect as u16).unwrap(),
            SkillId::new(SkillIds::HawkerSpirit as u16).unwrap(),
            SkillId::new(SkillIds::HeartHit as u16).unwrap(),
            SkillId::new(SkillIds::Relax as u16).unwrap(),
            SkillId::new(SkillIds::PoisonArrow as u16).unwrap(),
        ]
        .into(),
    }
}

pub fn bot_build_bourgeois() -> BotBuild {
    BotBuild {
        job_id: JobId::new(421),
        basic_stat_ratios: calculate_ratios(&[
            (BasicStatType::Concentration, 220),
            (BasicStatType::Sense, 155),
            (BasicStatType::Strength, 105),
        ]),
        weapon_type: ItemClass::Launcher,
        subweapon_type: None,
        skills: [
            SkillId::new(SkillIds::Stockpile as u16).unwrap(),
            SkillId::new(SkillIds::Marksmanship as u16).unwrap(),
            SkillId::new(SkillIds::ArmsMastery as u16).unwrap(),
            SkillId::new(SkillIds::TwinBullets as u16).unwrap(),
            SkillId::new(SkillIds::SnipingShot as u16).unwrap(),
            SkillId::new(SkillIds::Discount as u16).unwrap(),
            SkillId::new(SkillIds::Overcharge as u16).unwrap(),
            SkillId::new(SkillIds::MarketResearch as u16).unwrap(),
            SkillId::new(SkillIds::BagpackMastery as u16).unwrap(),
            SkillId::new(SkillIds::MightyShot as u16).unwrap(),
        ]
        .into(),
    }
}

pub fn bot_build_artisan() -> BotBuild {
    BotBuild {
        job_id: JobId::new(422),
        basic_stat_ratios: calculate_ratios(&[
            (BasicStatType::Concentration, 230),
            (BasicStatType::Sense, 172),
        ]),
        weapon_type: ItemClass::Gun,
        subweapon_type: None,
        skills: [
            SkillId::new(SkillIds::Stockpile as u16).unwrap(),
            SkillId::new(SkillIds::Marksmanship as u16).unwrap(),
            SkillId::new(SkillIds::ArmsMastery as u16).unwrap(),
            SkillId::new(SkillIds::TwinBullets as u16).unwrap(),
            SkillId::new(SkillIds::SnipingShot as u16).unwrap(),
            SkillId::new(SkillIds::WeaponResearch as u16).unwrap(),
            SkillId::new(SkillIds::ArmorResearch as u16).unwrap(),
            SkillId::new(SkillIds::MarketResearch as u16).unwrap(),
            SkillId::new(SkillIds::BagpackMastery as u16).unwrap(),
            SkillId::new(SkillIds::MightyShot as u16).unwrap(),
        ]
        .into(),
    }
}

pub fn spend_stat_points(
    game_data: &GameData,
    bot_build: &BotBuild,
    stat_points: &mut StatPoints,
    basic_stats: &mut BasicStats,
) {
    loop {
        let current_stat_total = bot_build
            .basic_stat_ratios
            .iter()
            .map(|(t, _)| basic_stats.get(*t))
            .sum::<i32>() as f32;

        let mut largest_delta_ratio = None;
        for (basic_stat_type, desired_ratio) in bot_build.basic_stat_ratios.iter() {
            let delta_ratio =
                *desired_ratio - (basic_stats.get(*basic_stat_type) as f32 / current_stat_total);

            if let Some((largest_delta, _)) = largest_delta_ratio {
                if delta_ratio > largest_delta {
                    largest_delta_ratio = Some((delta_ratio, *basic_stat_type));
                }
            } else {
                largest_delta_ratio = Some((delta_ratio, *basic_stat_type));
            }
        }

        let Some((_, basic_stat_type)) = largest_delta_ratio else {
            break;
        };
        let Some(increase_cost) = game_data
            .ability_value_calculator
            .calculate_basic_stat_increase_cost(basic_stats, basic_stat_type)
        else {
            // TODO: Handle max stat
            break;
        };

        if increase_cost > stat_points.points {
            break;
        }

        let new_value = basic_stats.get(basic_stat_type) + 1;
        basic_stats.set(basic_stat_type, new_value);
        stat_points.points -= increase_cost;
    }
}

pub fn spend_skill_points(
    game_data: &GameData,
    bot_build: &BotBuild,
    bot_data: &mut CharacterStorage,
    ability_values: &mut AbilityValues,
) {
    let mut skill_list_bundle = SkillListBundle {
        skill_list: &mut bot_data.skill_list,
        skill_points: Some(&mut bot_data.skill_points),
        game_client: None,
        ability_values,
        level: &bot_data.level,
        move_speed: None,
        team: None,
        character_info: Some(&bot_data.info),
        experience_points: Some(&bot_data.experience_points),
        inventory: Some(&bot_data.inventory),
        stamina: Some(&bot_data.stamina),
        stat_points: Some(&bot_data.stat_points),
        union_membership: Some(&bot_data.union_membership),
        health_points: Some(&bot_data.health_points),
        mana_points: Some(&bot_data.mana_points),
    };

    'next_skill: loop {
        // Try find first skill that we can learn
        for base_skill_id in bot_build.skills.iter() {
            if skill_list_bundle
                .skill_list
                .find_skill_level(&game_data.skills, *base_skill_id)
                .is_some()
            {
                continue; // Already learnt
            }

            if skill_list_try_learn_skill(game_data, &mut skill_list_bundle, *base_skill_id).is_ok()
            {
                continue 'next_skill;
            }
        }

        // Find the first skill that we can level up
        for base_skill_id in bot_build.skills.iter() {
            let Some((skill_slot, _, _)) = skill_list_bundle
                .skill_list
                .find_skill_level(&game_data.skills, *base_skill_id)
            else {
                continue; // Not learnt yet
            };

            if skill_list_try_level_up_skill(game_data, &mut skill_list_bundle, skill_slot).is_ok()
            {
                continue 'next_skill;
            }
        }

        // No more skills we can learn or level up
        break;
    }
}

fn level_up_bot(game_data: &GameData, level: u32, bot_data: &mut CharacterStorage) {
    while bot_data.level.level < level {
        bot_data.level.level += 1;

        bot_data.skill_points.points += game_data
            .ability_value_calculator
            .calculate_levelup_reward_skill_points(bot_data.level.level);

        bot_data.stat_points.points += game_data
            .ability_value_calculator
            .calculate_levelup_reward_stat_points(bot_data.level.level);
    }
}

fn choose_highest_level_item_by_class(
    game_data: &GameData,
    item_type: ItemType,
    item_class: ItemClass,
    level: u32,
) -> Option<EquipmentItem> {
    let mut best_item = None;
    let mut best_item_level = 0;

    for item_reference in game_data.items.iter_items(item_type) {
        let Some(item) = game_data.items.get_base_item(item_reference) else {
            continue;
        };

        if item.class != item_class {
            continue;
        }

        if let Some((_, item_level)) = item
            .equip_ability_requirement
            .iter()
            .find(|(ability_type, _)| *ability_type == AbilityType::Level)
        {
            if best_item_level < *item_level && *item_level < level {
                best_item = Some(item);
                best_item_level = *item_level;
            }
        }
    }

    best_item.and_then(|item_data| EquipmentItem::new(item_data.id, item_data.durability))
}

fn choose_equipment_items(
    game_data: &GameData,
    bot_build: &BotBuild,
    bot_data: &mut CharacterStorage,
) {
    let equipment = &mut bot_data.equipment;

    // Create a list of JobClassId which applies to selected job
    let mut valid_job_classes = Vec::new();
    for job_class in game_data.job_class.iter() {
        if job_class.jobs.contains(&bot_build.job_id) {
            valid_job_classes.push(job_class.id);
        }
    }

    // Choose armour items
    for equipment_index in [
        EquipmentIndex::Head,
        EquipmentIndex::Body,
        EquipmentIndex::Hands,
        EquipmentIndex::Feet,
    ] {
        let mut best_item = None;
        let mut best_item_level = 0;

        for item_reference in game_data.items.iter_items(equipment_index.into()) {
            let Some(item) = game_data.items.get_base_item(item_reference) else {
                continue;
            };

            // Find item which requires our job
            if !item
                .equip_job_class_requirement
                .map_or(false, |job_class| valid_job_classes.contains(&job_class))
            {
                continue;
            }

            // Choose item with highest level which we can equip
            if let Some((_, level)) = item
                .equip_ability_requirement
                .iter()
                .find(|(ability_type, _)| *ability_type == AbilityType::Level)
            {
                if best_item_level < *level && *level < bot_data.level.level {
                    best_item = Some(item);
                    best_item_level = *level;
                }
            }
        }

        if let Some(item_data) = best_item {
            equipment.equipped_items[equipment_index] =
                EquipmentItem::new(item_data.id, item_data.durability);
        }
    }

    // Choose weapon item
    equipment.equipped_items[EquipmentIndex::Weapon] = choose_highest_level_item_by_class(
        game_data,
        ItemType::Weapon,
        bot_build.weapon_type,
        bot_data.level.level,
    )
    .or_else(|| {
        // Fallback to Wooden Sword if not appropriate weapon was found
        game_data
            .items
            .get_base_item(ItemReference::weapon(1))
            .and_then(EquipmentItem::from_item_data)
    });

    if let Some(subweapon_type) = bot_build.subweapon_type {
        equipment.equipped_items[EquipmentIndex::SubWeapon] = choose_highest_level_item_by_class(
            game_data,
            ItemType::SubWeapon,
            subweapon_type,
            bot_data.level.level,
        );
    }

    // TODO: Face ?
    // TODO: Back ?
    // TODO: Necklace ?
    // TODO: Ring ?
    // TODO: Earring ?

    // Claw Arrow
    equipment.equipped_ammo[AmmoIndex::Arrow] =
        StackableItem::new(ItemReference::material(304), 999);

    // Lead Bullet
    equipment.equipped_ammo[AmmoIndex::Bullet] =
        StackableItem::new(ItemReference::material(323), 999);

    // Lead Cannon
    equipment.equipped_ammo[AmmoIndex::Throw] =
        StackableItem::new(ItemReference::material(342), 999);
}

pub fn bot_create_random_build(
    game_data: &GameData,
    name: String,
    level: u32,
) -> (BotBuild, CharacterStorage) {
    let mut rng = rand::thread_rng();
    let bot_build = [
        bot_build_knight,
        bot_build_champion,
        bot_build_cleric,
        bot_build_mage,
        bot_build_scout,
        bot_build_raider,
        bot_build_artisan,
        bot_build_bourgeois,
    ]
    .choose(&mut rng)
    .unwrap()();

    let bot_data = bot_create_with_build(game_data, name, level, &bot_build);
    (bot_build, bot_data)
}

pub fn bot_create_with_build(
    game_data: &GameData,
    name: String,
    level: u32,
    bot_build: &BotBuild,
) -> CharacterStorage {
    let mut rng = rand::thread_rng();
    let mut bot_data = game_data
        .character_creator
        .create(
            name,
            *BOT_GENDERS.choose(&mut rng).unwrap(),
            1,
            *BOT_FACES.choose(&mut rng).unwrap(),
            *BOT_HAIRS.choose(&mut rng).unwrap(),
        )
        .unwrap();

    level_up_bot(game_data, level, &mut bot_data);

    if level >= 70 {
        bot_data.info.job = bot_build.job_id.get();
    } else if level >= 10 {
        bot_data.info.job = (bot_build.job_id.get() / 100) * 100 + 11;
    }

    spend_stat_points(
        game_data,
        bot_build,
        &mut bot_data.stat_points,
        &mut bot_data.basic_stats,
    );

    let mut ability_values = game_data.ability_value_calculator.calculate(
        &bot_data.info,
        &bot_data.level,
        &bot_data.equipment,
        &bot_data.basic_stats,
        &bot_data.skill_list,
        &StatusEffects::new(),
    );

    spend_skill_points(game_data, bot_build, &mut bot_data, &mut ability_values);
    choose_equipment_items(game_data, bot_build, &mut bot_data);

    bot_data
}
