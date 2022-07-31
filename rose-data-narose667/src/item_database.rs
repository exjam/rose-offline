use arrayvec::ArrayVec;
use rose_file_readers::{stb_column, StbFile, StlFile, VfsIndex};
use std::time::Duration;

use rose_data::{
    AbilityType, BackItemData, BaseItemData, BodyItemData, ConsumableItemData, EffectFileId,
    EffectId, FaceItemData, FeetItemData, GemItemData, HandsItemData, HeadItemData, ItemClass,
    ItemDatabase, ItemGradeData, ItemReference, ItemType, JewelleryItemData, MaterialItemData,
    QuestItemData, SkillId, SoundId, StatusEffectId, SubWeaponItemData, VehicleItemData,
    VehicleItemPart, WeaponItemData,
};

use crate::data_decoder::{decode_ability_type, IroseItemClass};

pub struct StbItem(pub StbFile);
pub struct StbItemGrades(pub StbFile);

#[allow(dead_code)]
impl StbItem {
    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    stb_column! { 4, get_item_class, IroseItemClass }
    stb_column! { 5, get_base_price, u32 }
    stb_column! { 6, get_price_rate, u32 }
    stb_column! { 7, get_weight, u32 }
    stb_column! { 8, get_quality, u32 }
    stb_column! { 9, get_icon_number, u32 }
    stb_column! { 10, get_field_model, u32 }
    stb_column! { 11, get_equip_sound, SoundId }
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
                .and_then(|id| decode_ability_type(id as usize));
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
                .and_then(|id| decode_ability_type(id as usize));
            let ability_value = self.0.try_get_int(id, 25 + i * 3);

            ability_type.map(|ability_type| {
                ability_value.map(|ability_value| add_ability.push((ability_type, ability_value)))
            });
        }
        add_ability
    }

    stb_column! { 29, get_durability, u8 }
    stb_column! { 30, get_rare_type, u32 }
    stb_column! { 31, get_defence, u32 }
    stb_column! { 32, get_resistance, u32 }

    // LIST_BACK
    stb_column! { 33, get_back_move_speed, u32 }

    // LIST_FOOT
    stb_column! { 33, get_feet_move_speed, u32 }

    // LIST_CAP
    stb_column! { 33, get_head_hair_type, u32 }

    // LIST_WEAPON
    stb_column! { 33, get_weapon_attack_range, i32 }
    stb_column! { 34, get_weapon_motion_type, u32 }
    stb_column! { 35, get_weapon_attack_power, i32 }
    stb_column! { 36, get_weapon_attack_speed, i32 }
    stb_column! { 37, get_weapon_is_magic_damage, bool }
    stb_column! { 38, get_weapon_bullet_effect_id, EffectId }
    stb_column! { 39, get_weapon_effect_id, EffectId }
    stb_column! { 40, get_weapon_attack_start_sound_id, SoundId }
    stb_column! { 41, get_weapon_attack_fire_sound_id, SoundId }
    stb_column! { 42, get_weapon_attack_hit_sound_index, u32 }
    stb_column! { 43, get_weapon_gem_position, u32 }

    // LIST_SUBWEAPON
    stb_column! { 34, get_subweapon_gem_position, u32 }

    // LIST_USEITEM
    stb_column! { 8, get_consumeable_store_skin, i32 }
    stb_column! { 22, get_consumeable_confile_index, usize }

    pub fn get_consumeable_ability_requirement(&self, id: usize) -> Option<(AbilityType, i32)> {
        let ability_type: Option<AbilityType> = self
            .0
            .try_get_int(id, 17)
            .and_then(|id| decode_ability_type(id as usize));
        let ability_value = self.0.try_get_int(id, 18);

        ability_type.and_then(|ability_type| {
            ability_value.map(|ability_value| (ability_type, ability_value))
        })
    }

    pub fn get_consumeable_add_ability(&self, id: usize) -> Option<(AbilityType, i32)> {
        let ability_type: Option<AbilityType> = self
            .0
            .try_get_int(id, 19)
            .and_then(|id| decode_ability_type(id as usize));
        let ability_value = self.0.try_get_int(id, 20);

        ability_type.and_then(|ability_type| {
            ability_value.map(|ability_value| (ability_type, ability_value))
        })
    }

    stb_column! { 20, get_consumeable_learn_skill_id, SkillId }
    stb_column! { 20, get_consumeable_use_skill_id, SkillId }
    stb_column! { 21, get_consumeable_use_script_index, usize }
    stb_column! { 22, get_consumeable_use_effect_file_id, EffectFileId }
    stb_column! { 23, get_consumeable_use_effect_sound_id, SoundId }

    pub fn get_consumeable_apply_status_effect(&self, id: usize) -> Option<(StatusEffectId, i32)> {
        let status_effect_id: Option<StatusEffectId> = self
            .0
            .try_get_int(id, 24)
            .and_then(|x| u16::try_from(x).ok())
            .and_then(StatusEffectId::new);
        let status_effect_value = self.0.try_get_int(id, 20).unwrap_or(0);

        status_effect_id.map(|status_effect_id| (status_effect_id, status_effect_value))
    }

    stb_column! { 25, get_consumeable_cooldown_type_id, usize }
    stb_column! { 26, get_consumeable_cooldown_duration_seconds, u32 }

    // LIST_JEMITEM
    pub fn get_gem_add_ability(&self, id: usize) -> ArrayVec<(AbilityType, i32), 2> {
        let mut add_ability = ArrayVec::new();
        for i in 0..2 {
            let ability_type: Option<AbilityType> = self
                .0
                .try_get_int(id, 16 + i * 2)
                .and_then(|id| decode_ability_type(id as usize));
            let ability_value = self.0.try_get_int(id, 17 + i * 2);

            ability_type.map(|ability_type| {
                ability_value.map(|ability_value| add_ability.push((ability_type, ability_value)))
            });
        }
        add_ability
    }

    // LIST_NATURAL
    stb_column! { 17, get_material_bullet_effect_id, EffectId }

    // LIST_PAT
    pub fn get_vehicle_part(&self, id: usize) -> Option<VehicleItemPart> {
        match self.0.try_get_int(id, 2)? {
            0 => Some(VehicleItemPart::Body),
            1 => Some(VehicleItemPart::Engine),
            2 => Some(VehicleItemPart::Leg),
            3 => Some(VehicleItemPart::Ability),
            4 => Some(VehicleItemPart::Arms),
            _ => None,
        }
    }
    stb_column! { 33, get_vehicle_move_speed, u32 }
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
    item_type: ItemType,
    id: usize,
    check_valid: bool,
) -> Option<BaseItemData> {
    let icon_index = data.get_icon_number(id).unwrap_or(0);
    if check_valid && icon_index == 0 {
        return None;
    }

    Some(BaseItemData {
        id: ItemReference::new(item_type, id),
        name: stl
            .get_text_string(1, data.0.get(id, data.0.columns() - 1))
            .unwrap_or("")
            .to_string(),
        class: data
            .get_item_class(id)
            .unwrap_or(IroseItemClass::Unknown)
            .try_into()
            .unwrap_or(ItemClass::Unknown),
        base_price: data.get_base_price(id).unwrap_or(0),
        price_rate: data.get_price_rate(id).unwrap_or(0),
        weight: data.get_weight(id).unwrap_or(0),
        quality: data.get_quality(id).unwrap_or(0),
        icon_index,
        equip_sound_id: data.get_equip_sound(id),
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
    let base_item_data = load_base_item(data, stl, ItemType::Back, id, true)?;
    Some(BackItemData {
        item_data: base_item_data,
        move_speed: data.get_back_move_speed(id).unwrap_or(0),
    })
}

fn load_feet_item(data: &StbItem, stl: &StlFile, id: usize) -> Option<FeetItemData> {
    // Feet item id == 0 is used for base move speed
    let base_item_data = load_base_item(data, stl, ItemType::Feet, id, id != 0)?;
    Some(FeetItemData {
        item_data: base_item_data,
        move_speed: data.get_feet_move_speed(id).unwrap_or(0),
    })
}

fn load_head_item(data: &StbItem, stl: &StlFile, id: usize) -> Option<HeadItemData> {
    let base_item_data = load_base_item(data, stl, ItemType::Head, id, true)?;
    Some(HeadItemData {
        item_data: base_item_data,
        hair_type: data.get_head_hair_type(id).unwrap_or(0),
    })
}

fn load_weapon_item(data: &StbItem, stl: &StlFile, id: usize) -> Option<WeaponItemData> {
    // Weapon item id == 0 is used for unarmed attack data
    let base_item_data = load_base_item(data, stl, ItemType::Weapon, id, id != 0)?;
    Some(WeaponItemData {
        item_data: base_item_data,
        attack_range: data.get_weapon_attack_range(id).unwrap_or(0),
        attack_power: data.get_weapon_attack_power(id).unwrap_or(0),
        attack_speed: data.get_weapon_attack_speed(id).unwrap_or(0),
        motion_type: data.get_weapon_motion_type(id).unwrap_or(0),
        is_magic_damage: data.get_weapon_is_magic_damage(id).unwrap_or(false),
        bullet_effect_id: data.get_weapon_bullet_effect_id(id),
        effect_id: data.get_weapon_effect_id(id),
        attack_start_sound_id: data.get_weapon_attack_start_sound_id(id),
        attack_fire_sound_id: data.get_weapon_attack_fire_sound_id(id),
        attack_hit_sound_index: data.get_weapon_attack_hit_sound_index(id).unwrap_or(0),
        gem_position: data.get_weapon_gem_position(id).unwrap_or(0),
    })
}

fn load_consumeable_item(data: &StbItem, stl: &StlFile, id: usize) -> Option<ConsumableItemData> {
    let base_item_data = load_base_item(data, stl, ItemType::Consumable, id, true)?;
    Some(ConsumableItemData {
        item_data: base_item_data,
        store_skin: data.get_consumeable_store_skin(id).unwrap_or(0),
        confile_index: data.get_consumeable_confile_index(id).unwrap_or(0),
        ability_requirement: data.get_consumeable_ability_requirement(id),
        add_ability: data.get_consumeable_add_ability(id),
        learn_skill_id: data.get_consumeable_learn_skill_id(id),
        use_skill_id: data.get_consumeable_use_skill_id(id),
        apply_status_effect: data.get_consumeable_apply_status_effect(id),
        cooldown_type_id: data.get_consumeable_cooldown_type_id(id).unwrap_or(0),
        cooldown_duration: Duration::from_secs(
            data.get_consumeable_cooldown_duration_seconds(id)
                .unwrap_or(0) as u64,
        ),
        effect_file_id: data.get_consumeable_use_effect_file_id(id),
        effect_sound_id: data.get_consumeable_use_effect_sound_id(id),
    })
}

fn load_gem_item(data: &StbItem, stl: &StlFile, id: usize) -> Option<GemItemData> {
    let base_item_data = load_base_item(data, stl, ItemType::Gem, id, true)?;
    Some(GemItemData {
        item_data: base_item_data,
        gem_add_ability: data.get_gem_add_ability(id),
    })
}

fn load_material_item(data: &StbItem, stl: &StlFile, id: usize) -> Option<MaterialItemData> {
    let base_item_data = load_base_item(data, stl, ItemType::Material, id, true)?;
    Some(MaterialItemData {
        item_data: base_item_data,
        bullet_effect_id: data.get_material_bullet_effect_id(id),
    })
}

fn load_vehicle_item(data: &StbItem, stl: &StlFile, id: usize) -> Option<VehicleItemData> {
    let base_item_data = load_base_item(data, stl, ItemType::Vehicle, id, true)?;
    Some(VehicleItemData {
        item_data: base_item_data,
        vehicle_part: data.get_vehicle_part(id)?,
        move_speed: data.get_vehicle_move_speed(id).unwrap_or(0),
    })
}

macro_rules! load_items {
    ($vfs:ident, $path:literal, $stl_path:literal, load_base_item, $item_type:expr, $item_data_type:ident) => {{
        let stl = $vfs.read_file::<StlFile, _>($stl_path)?;
        let data = StbItem($vfs.read_file::<StbFile, _>($path)?);
        let mut items: Vec<Option<$item_data_type>> = Vec::with_capacity(data.rows());
        for id in 0..data.rows() {
            if let Some(item) = load_base_item(&data, &stl, $item_type, id, true) {
                items.push(Some($item_data_type { item_data: item }));
            } else {
                items.push(None);
            }
        }
        items
    }};
    ($vfs:ident, $path:literal, $stl_path:literal, $load_item_fn:ident, $item_data_type:ident) => {{
        let stl = $vfs.read_file::<StlFile, _>($stl_path)?;
        let data = StbItem($vfs.read_file::<StbFile, _>($path)?);
        let mut items: Vec<Option<$item_data_type>> = Vec::with_capacity(data.rows());
        for id in 0..data.rows() {
            items.push($load_item_fn(&data, &stl, id));
        }
        items
    }};
}

pub fn get_item_database(vfs: &VfsIndex) -> Result<ItemDatabase, anyhow::Error> {
    let face = load_items! { vfs, "3DDATA/STB/LIST_FACEITEM.STB", "3DDATA/STB/LIST_FACEITEM_S.STL", load_base_item, ItemType::Face, FaceItemData };
    let head = load_items! { vfs, "3DDATA/STB/LIST_CAP.STB", "3DDATA/STB/LIST_CAP_S.STL", load_head_item, HeadItemData };
    let body = load_items! { vfs, "3DDATA/STB/LIST_BODY.STB", "3DDATA/STB/LIST_BODY_S.STL", load_base_item, ItemType::Body, BodyItemData };
    let hands = load_items! { vfs, "3DDATA/STB/LIST_ARMS.STB", "3DDATA/STB/LIST_ARMS_S.STL", load_base_item, ItemType::Hands, HandsItemData };
    let feet = load_items! { vfs, "3DDATA/STB/LIST_FOOT.STB", "3DDATA/STB/LIST_FOOT_S.STL", load_feet_item, FeetItemData };
    let back = load_items! { vfs, "3DDATA/STB/LIST_BACK.STB", "3DDATA/STB/LIST_BACK_S.STL", load_back_item, BackItemData };
    let jewellery = load_items! { vfs, "3DDATA/STB/LIST_JEWEL.STB", "3DDATA/STB/LIST_JEWEL_S.STL", load_base_item, ItemType::Jewellery, JewelleryItemData };
    let weapon = load_items! { vfs, "3DDATA/STB/LIST_WEAPON.STB", "3DDATA/STB/LIST_WEAPON_S.STL", load_weapon_item, WeaponItemData };
    let subweapon = load_items! { vfs, "3DDATA/STB/LIST_SUBWPN.STB", "3DDATA/STB/LIST_SUBWPN_S.STL", load_base_item, ItemType::SubWeapon, SubWeaponItemData };
    let consumable = load_items! { vfs, "3DDATA/STB/LIST_USEITEM.STB", "3DDATA/STB/LIST_USEITEM_S.STL", load_consumeable_item, ConsumableItemData };
    let gem = load_items! { vfs, "3DDATA/STB/LIST_GEMITEM.STB", "3DDATA/STB/LIST_GEMITEM_S.STL", load_gem_item, GemItemData };
    let material = load_items! { vfs, "3DDATA/STB/LIST_NATURAL.STB", "3DDATA/STB/LIST_NATURAL_S.STL", load_material_item, MaterialItemData };
    let quest = load_items! { vfs, "3DDATA/STB/LIST_QUESTITEM.STB", "3DDATA/STB/LIST_QUESTITEM_S.STL", load_base_item, ItemType::Quest, QuestItemData };
    let vehicle = load_items! { vfs, "3DDATA/STB/LIST_PAT.STB", "3DDATA/STB/LIST_PAT_S.STL", load_vehicle_item, VehicleItemData };

    let mut item_grades = Vec::new();
    if let Ok(data) = vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_GRADE.STB") {
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

    log::debug!(
        "Loaded {} items",
        face.len()
            + head.len()
            + body.len()
            + hands.len()
            + feet.len()
            + back.len()
            + jewellery.len()
            + weapon.len()
            + subweapon.len()
            + consumable.len()
            + gem.len()
            + material.len()
            + quest.len()
            + vehicle.len()
            + item_grades.len()
    );
    Ok(ItemDatabase::new(
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
