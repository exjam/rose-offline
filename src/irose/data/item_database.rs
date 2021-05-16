use num_traits::FromPrimitive;
use std::{collections::HashMap, str::FromStr};

use crate::data::{
    formats::{FileReader, StbFile, VfsIndex},
    item::{AbilityType, ItemClass},
    BackItemData, BaseItemData, BodyItemData, ConsumableItemData, FaceItemData, FeetItemData,
    GemItemData, HandsItemData, HeadItemData, ItemDatabase, JewelleryItemData, MaterialItemData,
    QuestItemData, SubWeaponItemData, VehicleItemData, WeaponItemData,
};
pub struct StbItem(pub StbFile);

use crate::stb_column;

impl FromStr for ItemClass {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u32>().map_err(|_| ())?;
        FromPrimitive::from_u32(value).ok_or(())
    }
}

#[allow(dead_code)]
impl StbItem {
    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    stb_column! { 4, get_item_class, ItemClass }
    stb_column! { 5, get_base_price, u32 }
    stb_column! { 6, get_price_rate, u32 }
    stb_column! { 7, get_weight, u32 }
    stb_column! { 8, get_quality, u32 }
    stb_column! { 9, get_icon_number, u32 }
    stb_column! { 10, get_field_model, u32 }
    stb_column! { 11, get_equip_sound, u32 }
    stb_column! { 12, get_craft_skill_type, u32 }
    stb_column! { 13, get_craft_skill_level, u32 }
    stb_column! { 14, get_craft_material, u32 }
    stb_column! { 15, get_craft_difficulty, u32 }
    stb_column! { 16, get_equip_class_requirement, u32 }

    pub fn get_equip_union_requirement(&self, id: usize) -> Vec<u32> {
        let mut requirements = Vec::new();
        for i in 0..2 {
            if let Some(union) = self.0.try_get_int(id, 17 + i) {
                if union != 0 {
                    requirements.push(union as u32);
                }
            }
        }
        requirements
    }

    pub fn get_equip_ability_requirement(&self, id: usize) -> Vec<(AbilityType, u32)> {
        let mut requirements = Vec::new();
        for i in 0..2 {
            let ability_type: Option<AbilityType> = self
                .0
                .try_get_int(id, 19 + i * 2)
                .and_then(FromPrimitive::from_i32);
            let ability_value = self.0.try_get_int(id, 20 + i * 2);

            ability_type.map(|ability_type| {
                ability_value
                    .map(|ability_value| requirements.push((ability_type, ability_value as u32)))
            });
        }
        requirements
    }

    pub fn get_add_ability_union_requirement(&self, id: usize) -> Vec<u32> {
        let mut requirements = Vec::new();
        for i in 0..2 {
            if let Some(union) = self.0.try_get_int(id, 23 + i * 3) {
                if union != 0 {
                    requirements.push(union as u32);
                }
            }
        }
        requirements
    }

    pub fn get_add_ability(&self, id: usize) -> Vec<(AbilityType, i32)> {
        let mut add_ability = Vec::new();
        for i in 0..2 {
            let ability_type: Option<AbilityType> = self
                .0
                .try_get_int(id, 24 + i * 3)
                .and_then(FromPrimitive::from_i32);
            let ability_value = self.0.try_get_int(id, 25 + i * 3);

            ability_type.map(|ability_type| {
                ability_value.map(|ability_value| add_ability.push((ability_type, ability_value)))
            });
        }
        add_ability
    }

    stb_column! { 29, get_durability, u32 }
    stb_column! { 30, get_rare_type, u32 }
    stb_column! { 31, get_defence, u32 }
    stb_column! { 32, get_resistance, u32 }

    // LIST_BACK
    stb_column! { 33, get_back_move_speed, u32 }

    // LIST_FOOT
    stb_column! { 33, get_feet_move_speed, u32 }

    // LIST_WEAPON
    stb_column! { 4, get_weapon_type, u32 }
    stb_column! { 33, get_weapon_attack_range, u32 }
    stb_column! { 34, get_weapon_motion_type, u32 }
    stb_column! { 35, get_weapon_attack_power, u32 }
    stb_column! { 36, get_weapon_attack_speed, u32 }
    stb_column! { 37, get_weapon_is_magic_damage, bool }
    stb_column! { 38, get_weapon_bullet_effect_index, u32 }
    stb_column! { 39, get_weapon_default_effect_index, u32 }
    stb_column! { 40, get_weapon_attack_start_sound_index, u32 }
    stb_column! { 41, get_weapon_attack_fire_sound_index, u32 }
    stb_column! { 42, get_weapon_attack_hit_sound_index, u32 }
    stb_column! { 43, get_weapon_gem_position, u32 }

    // LIST_SUBWEAPON
    stb_column! { 4, get_subweapon_type, u32 }
    stb_column! { 34, get_subweapon_gem_position, u32 }

    // LIST_JEMITEM
    pub fn get_gem_add_ability(&self, id: usize) -> Vec<(AbilityType, i32)> {
        let mut add_ability = Vec::new();
        for i in 0..2 {
            let ability_type: Option<AbilityType> = self
                .0
                .try_get_int(id, 16 + i * 2)
                .and_then(FromPrimitive::from_i32);
            let ability_value = self.0.try_get_int(id, 17 + i * 2);

            ability_type.map(|ability_type| {
                ability_value.map(|ability_value| add_ability.push((ability_type, ability_value)))
            });
        }
        add_ability
    }
}

fn load_base_item(data: &StbItem, id: usize) -> Option<BaseItemData> {
    let icon_number = data.get_icon_number(id)?;

    Some(BaseItemData {
        class: data.get_item_class(id)?,
        base_price: data.get_base_price(id).unwrap_or(0),
        price_rate: data.get_price_rate(id).unwrap_or(0),
        weight: data.get_weight(id).unwrap_or(0),
        quality: data.get_quality(id).unwrap_or(0),
        icon_index: icon_number,
        equip_sound_index: data.get_equip_sound(id).unwrap_or(0),
        craft_skill_type: data.get_craft_skill_type(id).unwrap_or(0),
        craft_skill_level: data.get_craft_skill_level(id).unwrap_or(0),
        craft_material: data.get_craft_material(id).unwrap_or(0),
        craft_difficulty: data.get_craft_difficulty(id).unwrap_or(0),
        equip_class_requirement: data.get_equip_class_requirement(id).unwrap_or(0),
        equip_union_requirement: data.get_equip_union_requirement(id),
        equip_ability_requirement: data.get_equip_ability_requirement(id),
        add_ability_union_requirement: data.get_add_ability_union_requirement(id),
        add_ability: data.get_add_ability(id),
        durability: data.get_durability(id).unwrap_or(0),
        rare_type: data.get_rare_type(id).unwrap_or(0),
        defence: data.get_defence(id).unwrap_or(0),
        resistance: data.get_resistance(id).unwrap_or(0),
        field_model_index: data.get_field_model(id).unwrap_or(0),
    })
}

fn load_back_item(data: &StbItem, id: usize) -> Option<BackItemData> {
    let base_item_data = load_base_item(data, id)?;
    Some(BackItemData {
        item_data: base_item_data,
        move_speed: data.get_back_move_speed(id).unwrap_or(0),
    })
}

fn load_feet_item(data: &StbItem, id: usize) -> Option<FeetItemData> {
    let base_item_data = load_base_item(data, id)?;
    Some(FeetItemData {
        item_data: base_item_data,
        move_speed: data.get_feet_move_speed(id).unwrap_or(0),
    })
}

fn load_weapon_item(data: &StbItem, id: usize) -> Option<WeaponItemData> {
    let base_item_data = load_base_item(data, id)?;
    Some(WeaponItemData {
        item_data: base_item_data,
        weapon_type: data.get_weapon_type(id).unwrap_or(0),
        attack_range: data.get_weapon_attack_range(id).unwrap_or(0),
        attack_power: data.get_weapon_attack_power(id).unwrap_or(0),
        attack_speed: data.get_weapon_attack_speed(id).unwrap_or(0),
        is_magic_damage: data.get_weapon_is_magic_damage(id).unwrap_or(false),
    })
}

fn load_subweapon_item(data: &StbItem, id: usize) -> Option<SubWeaponItemData> {
    let base_item_data = load_base_item(data, id)?;
    Some(SubWeaponItemData {
        item_data: base_item_data,
        weapon_type: data.get_subweapon_type(id).unwrap_or(0),
    })
}

fn load_gem_item(data: &StbItem, id: usize) -> Option<GemItemData> {
    let base_item_data = load_base_item(data, id)?;
    Some(GemItemData {
        item_data: base_item_data,
        gem_add_ability: data.get_gem_add_ability(id),
    })
}

macro_rules! load_item_stb {
    ($vfs:ident, $path:literal, load_base_item, $item_data_type:ident) => {{
        let mut items: HashMap<u16, $item_data_type> = HashMap::new();
        let file = $vfs.open_file($path)?;
        let data = StbItem(StbFile::read(FileReader::from(&file)).ok()?);
        for id in 0..data.rows() {
            if let Some(item) = load_base_item(&data, id) {
                items.insert(id as u16, $item_data_type { item_data: item });
            }
        }
        items
    };};
    ($vfs:ident, $path:literal, $load_item_fn:ident, $item_data_type:ident) => {{
        let mut items: HashMap<u16, $item_data_type> = HashMap::new();
        let file = $vfs.open_file($path)?;
        let data = StbItem(StbFile::read(FileReader::from(&file)).ok()?);
        for id in 0..data.rows() {
            if let Some(item) = $load_item_fn(&data, id) {
                items.insert(id as u16, item);
            }
        }
        items
    };};
}

pub fn get_item_database(vfs: &VfsIndex) -> Option<ItemDatabase> {
    let face = load_item_stb! { vfs, "3DDATA/STB/LIST_FACEITEM.STB", load_base_item, FaceItemData };
    let head = load_item_stb! { vfs, "3DDATA/STB/LIST_CAP.STB", load_base_item, HeadItemData };
    let body = load_item_stb! { vfs, "3DDATA/STB/LIST_BODY.STB", load_base_item, BodyItemData };
    let hands = load_item_stb! { vfs, "3DDATA/STB/LIST_ARMS.STB", load_base_item, HandsItemData };
    let feet = load_item_stb! { vfs, "3DDATA/STB/LIST_FOOT.STB", load_feet_item, FeetItemData };
    let back = load_item_stb! { vfs, "3DDATA/STB/LIST_BACK.STB", load_back_item, BackItemData };
    let jewellery =
        load_item_stb! { vfs, "3DDATA/STB/LIST_JEWEL.STB", load_base_item, JewelleryItemData };
    let weapon =
        load_item_stb! { vfs, "3DDATA/STB/LIST_WEAPON.STB", load_weapon_item, WeaponItemData };
    let subweapon = load_item_stb! { vfs, "3DDATA/STB/LIST_SUBWPN.STB", load_subweapon_item, SubWeaponItemData };
    let consumable =
        load_item_stb! { vfs, "3DDATA/STB/LIST_USEITEM.STB", load_base_item, ConsumableItemData };
    let gem = load_item_stb! { vfs, "3DDATA/STB/LIST_JEMITEM.STB", load_gem_item, GemItemData };
    let material =
        load_item_stb! { vfs, "3DDATA/STB/LIST_NATURAL.STB", load_base_item, MaterialItemData };
    let quest =
        load_item_stb! { vfs, "3DDATA/STB/LIST_QUESTITEM.STB", load_base_item, QuestItemData };
    let vehicle =
        load_item_stb! { vfs, "3DDATA/STB/LIST_PAT.STB", load_base_item, VehicleItemData };

    Some(ItemDatabase::new(
        face, head, body, hands, feet, back, jewellery, weapon, subweapon, consumable, gem,
        material, quest, vehicle,
    ))
}
