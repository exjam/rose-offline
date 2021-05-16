use std::sync::Arc;

use crate::{
    data::{item::AbilityType, AbilityValueCalculator, ItemDatabase, ItemReference, SkillDatabase},
    game::components::{
        AbilityValues, BasicStats, CharacterInfo, Equipment, EquipmentIndex, Inventory,
    },
};

pub struct AbilityValuesData {
    item_database: Arc<ItemDatabase>,
    skill_database: Arc<SkillDatabase>,
}

impl AbilityValuesData {
    pub fn new(item_database: Arc<ItemDatabase>, skill_database: Arc<SkillDatabase>) -> Self {
        Self {
            item_database,
            skill_database,
        }
    }
}

pub fn get_ability_value_calculator(
    item_database: Arc<ItemDatabase>,
    skill_database: Arc<SkillDatabase>,
) -> Option<Box<impl AbilityValueCalculator + Send + Sync>> {
    Some(Box::new(AbilityValuesData::new(
        item_database,
        skill_database,
    )))
}

impl AbilityValueCalculator for AbilityValuesData {
    fn calculate(
        &self,
        character_info: &CharacterInfo,
        equipment: &Equipment,
        inventory: &Inventory,
        basic_stats: &BasicStats,
    ) -> AbilityValues {
        // TODO: Passive skills
        let equipment_ability_values =
            calculate_equipment_ability_values(&self.item_database, equipment);
        // TODO: Add buffs / debuffs

        AbilityValues {
            run_speed: calculate_run_speed(
                &self.item_database,
                &basic_stats,
                &equipment_ability_values,
                &equipment,
            ),
        }
    }
}

#[derive(Default)]
struct EquipmentAbilityValue {
    pub move_speed: i32,
}

impl EquipmentAbilityValue {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_ability_value(&mut self, ability_type: AbilityType, value: i32) {
        match ability_type {
            AbilityType::Speed => self.move_speed += value,
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

        if let Some(item_data) = item_database.get_base_item(ItemReference::new(
            item.item_type,
            item.item_number as usize,
        )) {
            // TODO: Check item_stb.get_item_union_requirement(item_number)
            for (ability, value) in item_data.add_ability.iter() {
                result.add_ability_value(*ability, *value);
            }
        }
    }

    // TODO: If riding cart, add values from vehicle

    result
}

fn calculate_run_speed(
    item_database: &ItemDatabase,
    basic_stats: &BasicStats,
    equipment_ability_values: &EquipmentAbilityValue,
    equipment: &Equipment,
) -> f32 {
    // TODO: Check if riding cart
    let mut item_speed = 20f32;

    item_speed += equipment
        .get_equipment_item(EquipmentIndex::Feet)
        .filter(|item| !item.is_broken())
        .and_then(|item| item_database.get_feet_item(item.item_number as usize))
        .or_else(|| item_database.get_feet_item(0))
        .map(|item_data| item_data.move_speed)
        .unwrap_or(0) as f32;

    item_speed += equipment
        .get_equipment_item(EquipmentIndex::Back)
        .filter(|item| !item.is_broken())
        .and_then(|item| item_database.get_back_item(item.item_number as usize))
        .map(|item_data| item_data.move_speed)
        .unwrap_or(0) as f32;

    let run_speed = item_speed * (basic_stats.dexterity as f32 + 500.0) / 100.0
        + equipment_ability_values.move_speed as f32;

    // TODO: Adding of passive move speed
    // TODO: run_speed += add_value
    run_speed
}
