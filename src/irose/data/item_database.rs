use arrayvec::ArrayVec;
use num_traits::FromPrimitive;
use std::{collections::HashMap, str::FromStr, time::Duration};

use crate::data::{
    formats::{FileReader, StbFile, StlFile, VfsIndex},
    item::ItemClass,
    AbilityType, BackItemData, BaseItemData, BodyItemData, ConsumableItemData, FaceItemData,
    FeetItemData, GemItemData, HandsItemData, HeadItemData, ItemDatabase, ItemGradeData,
    JewelleryItemData, MaterialItemData, QuestItemData, SkillId, SubWeaponItemData,
    VehicleItemData, WeaponItemData,
};
pub struct StbItem(pub StbFile);
pub struct StbItemGrades(pub StbFile);

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

    pub fn get_equip_union_requirement(&self, id: usize) -> ArrayVec<u32, 2> {
        let mut requirements = ArrayVec::new();
        for i in 0..2 {
            if let Some(union) = self.0.try_get_int(id, 17 + i) {
                if union != 0 {
                    requirements.push(union as u32);
                }
            }
        }
        requirements
    }

    pub fn get_equip_ability_requirement(&self, id: usize) -> ArrayVec<(AbilityType, u32), 2> {
        let mut requirements = ArrayVec::new();
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

    pub fn get_add_ability_union_requirement(&self, id: usize) -> ArrayVec<u32, 2> {
        let mut requirements = ArrayVec::new();
        for i in 0..2 {
            if let Some(union) = self.0.try_get_int(id, 23 + i * 3) {
                if union != 0 {
                    requirements.push(union as u32);
                }
            }
        }
        requirements
    }

    pub fn get_add_ability(&self, id: usize) -> ArrayVec<(AbilityType, i32), 2> {
        let mut add_ability = ArrayVec::new();
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
    stb_column! { 33, get_weapon_attack_range, i32 }
    stb_column! { 34, get_weapon_motion_type, u32 }
    stb_column! { 35, get_weapon_attack_power, i32 }
    stb_column! { 36, get_weapon_attack_speed, i32 }
    stb_column! { 37, get_weapon_is_magic_damage, bool }
    stb_column! { 38, get_weapon_bullet_effect_index, u32 }
    stb_column! { 39, get_weapon_default_effect_index, u32 }
    stb_column! { 40, get_weapon_attack_start_sound_index, u32 }
    stb_column! { 41, get_weapon_attack_fire_sound_index, u32 }
    stb_column! { 42, get_weapon_attack_hit_sound_index, u32 }
    stb_column! { 43, get_weapon_gem_position, u32 }

    // LIST_SUBWEAPON
    stb_column! { 34, get_subweapon_gem_position, u32 }

    // LIST_USEITEM
    stb_column! { 8, get_consumeable_store_skin, i32 }
    stb_column! { 22, get_consumeable_confile_index, usize }

    pub fn get_consumeable_ability_requirement(&self, id: usize) -> Option<(AbilityType, i32)> {
        let ability_type: Option<AbilityType> =
            self.0.try_get_int(id, 17).and_then(FromPrimitive::from_i32);
        let ability_value = self.0.try_get_int(id, 18);

        ability_type.and_then(|ability_type| {
            ability_value.map(|ability_value| (ability_type, ability_value))
        })
    }

    pub fn get_consumeable_add_ability(&self, id: usize) -> Option<(AbilityType, i32)> {
        let ability_type: Option<AbilityType> =
            self.0.try_get_int(id, 19).and_then(FromPrimitive::from_i32);
        let ability_value = self.0.try_get_int(id, 20);

        ability_type.and_then(|ability_type| {
            ability_value.map(|ability_value| (ability_type, ability_value))
        })
    }

    stb_column! { 20, get_consumeable_learn_skill_id, SkillId }
    stb_column! { 20, get_consumeable_use_skill_id, SkillId }
    stb_column! { 21, get_consumeable_use_script_index, usize }
    stb_column! { 22, get_consumeable_use_effect_index, usize }
    stb_column! { 23, get_consumeable_use_sound_index, usize }
    stb_column! { 24, get_consumeable_apply_status_effect_id, usize }
    stb_column! { 25, get_consumeable_cooldown_type_id, usize }
    stb_column! { 26, get_consumeable_cooldown_duration_seconds, u32 }

    // LIST_JEMITEM
    pub fn get_gem_add_ability(&self, id: usize) -> ArrayVec<(AbilityType, i32), 2> {
        let mut add_ability = ArrayVec::new();
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

#[allow(dead_code)]
impl StbItemGrades {
    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    stb_column! { 0, get_attack, i32 }
    stb_column! { 1, get_hit, i32 }
    stb_column! { 2, get_defence, i32 }
    stb_column! { 3, get_resistance, i32 }
    stb_column! { 4, get_avoid, i32 }

    pub fn get_glow_colour(&self, id: usize) -> (f32, f32, f32) {
        let mut colour = self.0.try_get_int(id, 5).unwrap_or(0);

        let red = colour / 1000000;
        colour %= 1000000;

        let green = colour / 1000;
        colour %= 1000;

        let blue = colour;

        (
            red as f32 / 255.0,
            green as f32 / 255.0,
            blue as f32 / 255.0,
        )
    }
}

fn load_base_item(
    data: &StbItem,
    stl: &StlFile,
    id: usize,
    check_valid: bool,
) -> Option<BaseItemData> {
    let icon_index = data.get_icon_number(id).unwrap_or(0);
    if check_valid && icon_index == 0 {
        return None;
    }

    Some(BaseItemData {
        name: stl
            .get_text_string(1, data.0.get(id, data.0.columns() - 1))
            .unwrap_or(&"")
            .to_string(),
        class: data.get_item_class(id).unwrap_or(ItemClass::Unknown),
        base_price: data.get_base_price(id).unwrap_or(0),
        price_rate: data.get_price_rate(id).unwrap_or(0),
        weight: data.get_weight(id).unwrap_or(0),
        quality: data.get_quality(id).unwrap_or(0),
        icon_index,
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

fn load_back_item(data: &StbItem, stl: &StlFile, id: usize) -> Option<BackItemData> {
    let base_item_data = load_base_item(data, stl, id, true)?;
    Some(BackItemData {
        item_data: base_item_data,
        move_speed: data.get_back_move_speed(id).unwrap_or(0),
    })
}

fn load_feet_item(data: &StbItem, stl: &StlFile, id: usize) -> Option<FeetItemData> {
    // Feet item id == 0 is used for base move speed
    let base_item_data = load_base_item(data, stl, id, id != 0)?;
    Some(FeetItemData {
        item_data: base_item_data,
        move_speed: data.get_feet_move_speed(id).unwrap_or(0),
    })
}

fn load_weapon_item(data: &StbItem, stl: &StlFile, id: usize) -> Option<WeaponItemData> {
    // Weapon item id == 0 is used for unarmed attack data
    let base_item_data = load_base_item(data, stl, id, id != 0)?;
    Some(WeaponItemData {
        item_data: base_item_data,
        attack_range: data.get_weapon_attack_range(id).unwrap_or(0),
        attack_power: data.get_weapon_attack_power(id).unwrap_or(0),
        attack_speed: data.get_weapon_attack_speed(id).unwrap_or(0),
        motion_type: data.get_weapon_motion_type(id).unwrap_or(0),
        is_magic_damage: data.get_weapon_is_magic_damage(id).unwrap_or(false),
    })
}

fn load_consumeable_item(data: &StbItem, stl: &StlFile, id: usize) -> Option<ConsumableItemData> {
    let base_item_data = load_base_item(data, stl, id, true)?;
    Some(ConsumableItemData {
        item_data: base_item_data,
        store_skin: data.get_consumeable_store_skin(id).unwrap_or(0),
        confile_index: data.get_consumeable_confile_index(id).unwrap_or(0),
        ability_requirement: data.get_consumeable_ability_requirement(id),
        add_ability: data.get_consumeable_add_ability(id),
        learn_skill_id: data.get_consumeable_learn_skill_id(id),
        use_skill_id: data.get_consumeable_use_skill_id(id),
        apply_status_effect_id: data
            .get_consumeable_apply_status_effect_id(id)
            .filter(|value| *value != 0),
        cooldown_type_id: data.get_consumeable_cooldown_type_id(id).unwrap_or(0),
        cooldown_duration: Duration::from_secs(
            data.get_consumeable_cooldown_duration_seconds(id)
                .unwrap_or(0) as u64,
        ),
    })
}

fn load_gem_item(data: &StbItem, stl: &StlFile, id: usize) -> Option<GemItemData> {
    let base_item_data = load_base_item(data, stl, id, true)?;
    Some(GemItemData {
        item_data: base_item_data,
        gem_add_ability: data.get_gem_add_ability(id),
    })
}

macro_rules! load_items {
    ($vfs:ident, $path:literal, $stl_path:literal, load_base_item, $item_data_type:ident) => {{
        let mut items: HashMap<u16, $item_data_type> = HashMap::new();
        let file = $vfs.open_file($stl_path)?;
        let stl = StlFile::read(FileReader::from(&file)).ok()?;
        let file = $vfs.open_file($path)?;
        let data = StbItem(StbFile::read(FileReader::from(&file)).ok()?);
        for id in 0..data.rows() {
            if let Some(item) = load_base_item(&data, &stl, id, true) {
                items.insert(id as u16, $item_data_type { item_data: item });
            }
        }
        items
    };};
    ($vfs:ident, $path:literal, $stl_path:literal, $load_item_fn:ident, $item_data_type:ident) => {{
        let mut items: HashMap<u16, $item_data_type> = HashMap::new();
        let file = $vfs.open_file($stl_path)?;
        let stl = StlFile::read(FileReader::from(&file)).ok()?;
        let file = $vfs.open_file($path)?;
        let data = StbItem(StbFile::read(FileReader::from(&file)).ok()?);
        for id in 0..data.rows() {
            if let Some(item) = $load_item_fn(&data, &stl, id) {
                items.insert(id as u16, item);
            }
        }
        items
    };};
}

pub fn get_item_database(vfs: &VfsIndex) -> Option<ItemDatabase> {
    let face = load_items! { vfs, "3DDATA/STB/LIST_FACEITEM.STB", "3DDATA/STB/LIST_FACEITEM_S.STL", load_base_item, FaceItemData };
    let head = load_items! { vfs, "3DDATA/STB/LIST_CAP.STB", "3DDATA/STB/LIST_CAP_S.STL", load_base_item, HeadItemData };
    let body = load_items! { vfs, "3DDATA/STB/LIST_BODY.STB", "3DDATA/STB/LIST_BODY_S.STL", load_base_item, BodyItemData };
    let hands = load_items! { vfs, "3DDATA/STB/LIST_ARMS.STB", "3DDATA/STB/LIST_ARMS_S.STL", load_base_item, HandsItemData };
    let feet = load_items! { vfs, "3DDATA/STB/LIST_FOOT.STB", "3DDATA/STB/LIST_FOOT_S.STL",load_feet_item, FeetItemData };
    let back = load_items! { vfs, "3DDATA/STB/LIST_BACK.STB", "3DDATA/STB/LIST_BACK_S.STL", load_back_item, BackItemData };
    let jewellery = load_items! { vfs, "3DDATA/STB/LIST_JEWEL.STB", "3DDATA/STB/LIST_JEWEL_S.STL", load_base_item, JewelleryItemData };
    let weapon = load_items! { vfs, "3DDATA/STB/LIST_WEAPON.STB", "3DDATA/STB/LIST_WEAPON_S.STL", load_weapon_item, WeaponItemData };
    let subweapon = load_items! { vfs, "3DDATA/STB/LIST_SUBWPN.STB", "3DDATA/STB/LIST_SUBWPN_S.STL", load_base_item, SubWeaponItemData };
    let consumable = load_items! { vfs, "3DDATA/STB/LIST_USEITEM.STB", "3DDATA/STB/LIST_USEITEM_S.STL", load_consumeable_item, ConsumableItemData };
    let gem = load_items! { vfs, "3DDATA/STB/LIST_JEMITEM.STB", "3DDATA/STB/LIST_JEMITEM_S.STL",load_gem_item, GemItemData };
    let material = load_items! { vfs, "3DDATA/STB/LIST_NATURAL.STB", "3DDATA/STB/LIST_NATURAL_S.STL", load_base_item, MaterialItemData };
    let quest = load_items! { vfs, "3DDATA/STB/LIST_QUESTITEM.STB", "3DDATA/STB/LIST_QUESTITEM_S.STL", load_base_item, QuestItemData };
    let vehicle = load_items! { vfs, "3DDATA/STB/LIST_PAT.STB", "3DDATA/STB/LIST_PAT_S.STL", load_base_item, VehicleItemData };

    let mut item_grades = Vec::new();
    if let Some(file) = vfs.open_file("3DDATA/STB/LIST_GRADE.STB") {
        if let Ok(data) = StbFile::read(FileReader::from(&file)) {
            let data = StbItemGrades(data);
            for i in 0..data.rows() {
                item_grades.push(ItemGradeData {
                    attack: data.get_attack(i).unwrap_or(0),
                    hit: data.get_hit(i).unwrap_or(0),
                    defence: data.get_defence(i).unwrap_or(0),
                    resistance: data.get_resistance(i).unwrap_or(0),
                    avoid: data.get_avoid(i).unwrap_or(0),
                    glow_colour: data.get_glow_colour(i),
                });
            }
        }
    }

    Some(ItemDatabase::new(
        face,
        head,
        body,
        hands,
        feet,
        back,
        jewellery,
        weapon,
        subweapon,
        consumable,
        gem,
        material,
        quest,
        vehicle,
        item_grades,
    ))
}
