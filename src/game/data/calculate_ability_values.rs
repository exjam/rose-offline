use crate::game::components::{AbilityValues, BasicStats, Equipment, EquipmentIndex, Inventory};
use crate::game::data::items::{AbilityType, ItemType};
use crate::game::data::stb::StbItem;
use crate::game::data::{
    STB_ITEM_BACK, STB_ITEM_BODY, STB_ITEM_CONSUMABLE, STB_ITEM_FACE, STB_ITEM_FEET, STB_ITEM_GEM,
    STB_ITEM_HANDS, STB_ITEM_HEAD, STB_ITEM_JEWELLERY, STB_ITEM_MATERIAL, STB_ITEM_QUEST,
    STB_ITEM_SUB_WEAPON, STB_ITEM_VEHICLE, STB_ITEM_WEAPON,
};

fn get_item_stb(item_type: ItemType) -> Option<&'static StbItem> {
    match item_type {
        ItemType::Face => Some(&STB_ITEM_FACE),
        ItemType::Head => Some(&STB_ITEM_HEAD),
        ItemType::Body => Some(&STB_ITEM_BODY),
        ItemType::Hands => Some(&STB_ITEM_HANDS),
        ItemType::Feet => Some(&STB_ITEM_FEET),
        ItemType::Back => Some(&STB_ITEM_BACK),
        ItemType::Jewellery => Some(&STB_ITEM_JEWELLERY),
        ItemType::Weapon => Some(&STB_ITEM_WEAPON),
        ItemType::SubWeapon => Some(&STB_ITEM_SUB_WEAPON),
        ItemType::Consumable => Some(&STB_ITEM_CONSUMABLE),
        ItemType::Gem => Some(&STB_ITEM_GEM),
        ItemType::Material => Some(&STB_ITEM_MATERIAL),
        ItemType::Quest => Some(&STB_ITEM_QUEST),
        ItemType::Vehicle => Some(&STB_ITEM_VEHICLE),
        _ => None,
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

fn calculate_equipment_ability_values(equipment: &Equipment) -> EquipmentAbilityValue {
    let mut result = EquipmentAbilityValue::new();

    for item in equipment.equipped_items.iter().filter_map(|x| x.as_ref()) {
        if item.is_appraised || item.has_socket {
            // TODO: STB_ITEM_GEM column 16+17, 18+19
        }

        if let Some(item_stb) = get_item_stb(item.item_type) {
            // TODO: Check item_stb.get_item_union_requirement(item_number)
            for (ability, value) in item_stb.get_item_add_ability(item.item_number) {
                result.add_ability_value(ability, value);
            }
        }
    }

    // TODO: If riding cart, add values from vehicle

    result
}

fn calculate_run_speed(
    basic_stats: &BasicStats,
    equipment_ability_values: &EquipmentAbilityValue,
    equipment: &Equipment,
) -> f32 {
    // TODO: Check if riding cart
    let mut item_speed = 20f32;

    item_speed += equipment
        .get_equipment_item(EquipmentIndex::Feet)
        .filter(|item| !item.is_broken())
        .and_then(|item| STB_ITEM_FEET.get_boots_move_speed(item.item_number))
        .unwrap_or(STB_ITEM_FEET.get_boots_move_speed(0).unwrap_or(0)) as f32;

    item_speed += equipment
        .get_equipment_item(EquipmentIndex::Back)
        .filter(|item| !item.is_broken())
        .and_then(|item| STB_ITEM_BACK.get_back_move_speed(item.item_number))
        .unwrap_or(0) as f32;

    let run_speed = item_speed * (basic_stats.dexterity as f32 + 500.0) / 100.0
        + equipment_ability_values.move_speed as f32;

    // TODO: Adding of passive move speed
    // TODO: run_speed += add_value
    println!("Run speed: {}", run_speed);
    run_speed
}

pub fn calculate_ability_values(
    equipment: &Equipment,
    inventory: &Inventory,
    basic_stats: &BasicStats,
) -> AbilityValues {
    // TODO: Passive skills
    let equipment_ability_values = calculate_equipment_ability_values(equipment);
    // TODO: Add buffs / debuffs

    AbilityValues {
        run_speed: calculate_run_speed(&basic_stats, &equipment_ability_values, &equipment),
    }
}
