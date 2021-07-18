use std::collections::HashMap;

use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};

use crate::data::{
    ability::AbilityType,
    item::{ItemClass, ItemType},
};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ItemReference {
    pub item_type: ItemType,
    pub item_number: usize,
}

pub enum ItemReferenceDecodeError {
    Empty,
    InvalidItemType,
    InvalidItemNumber,
}

impl ItemReference {
    pub fn new(item_type: ItemType, item_number: usize) -> Self {
        Self {
            item_type,
            item_number,
        }
    }

    pub fn from_base1000(value: u32) -> Result<Self, ItemReferenceDecodeError> {
        if value == 0 {
            Err(ItemReferenceDecodeError::Empty)
        } else {
            let item_type = FromPrimitive::from_u32(value / 1000)
                .ok_or(ItemReferenceDecodeError::InvalidItemType)?;
            let item_number = value % 1000;
            if item_number == 0 {
                Err(ItemReferenceDecodeError::InvalidItemNumber)
            } else {
                Ok(Self::new(item_type, item_number as usize))
            }
        }
    }
}

pub struct BaseItemData {
    pub name: String,
    pub class: ItemClass,
    pub base_price: u32,
    pub price_rate: u32,
    pub weight: u32,
    pub quality: u32,
    pub icon_index: u32,
    pub field_model_index: u32,
    pub equip_sound_index: u32,
    pub craft_skill_type: u32,
    pub craft_skill_level: u32,
    pub craft_material: u32,
    pub craft_difficulty: u32,
    pub equip_class_requirement: u32,
    pub equip_union_requirement: Vec<u32>,
    pub equip_ability_requirement: Vec<(AbilityType, u32)>,
    pub add_ability_union_requirement: Vec<u32>,
    pub add_ability: Vec<(AbilityType, i32)>,
    pub durability: u32,
    pub rare_type: u32,
    pub defence: u32,
    pub resistance: u32,
}

pub struct FaceItemData {
    pub item_data: BaseItemData,
}

pub struct HeadItemData {
    pub item_data: BaseItemData,
}

pub struct BodyItemData {
    pub item_data: BaseItemData,
}

pub struct HandsItemData {
    pub item_data: BaseItemData,
}

pub struct BackItemData {
    pub item_data: BaseItemData,
    pub move_speed: u32,
}

pub struct FeetItemData {
    pub item_data: BaseItemData,
    pub move_speed: u32,
}

pub struct JewelleryItemData {
    pub item_data: BaseItemData,
}

pub struct GemItemData {
    pub item_data: BaseItemData,
    pub gem_add_ability: Vec<(AbilityType, i32)>,
}

pub struct WeaponItemData {
    pub item_data: BaseItemData,
    pub attack_range: i32,
    pub attack_power: i32,
    pub attack_speed: i32,
    pub motion_type: u32,
    pub is_magic_damage: bool,
}

pub struct SubWeaponItemData {
    pub item_data: BaseItemData,
}

pub struct ConsumableItemData {
    pub item_data: BaseItemData,
}

pub struct MaterialItemData {
    pub item_data: BaseItemData,
}

pub struct QuestItemData {
    pub item_data: BaseItemData,
}

pub struct VehicleItemData {
    pub item_data: BaseItemData,
}

pub struct ItemGradeData {
    pub attack: i32,
    pub hit: i32,
    pub defence: i32,
    pub resistance: i32,
    pub avoid: i32,
    pub glow_colour: (f32, f32, f32),
}

#[allow(dead_code)]
pub enum ItemData<'a> {
    Face(&'a FaceItemData),
    Head(&'a HeadItemData),
    Body(&'a BodyItemData),
    Hands(&'a HandsItemData),
    Feet(&'a FeetItemData),
    Back(&'a BackItemData),
    Jewellery(&'a JewelleryItemData),
    Weapon(&'a WeaponItemData),
    SubWeapon(&'a SubWeaponItemData),
    Consumable(&'a ConsumableItemData),
    Gem(&'a GemItemData),
    Material(&'a MaterialItemData),
    Quest(&'a QuestItemData),
    Vehicle(&'a VehicleItemData),
}

pub struct ItemDatabase {
    face: HashMap<u16, FaceItemData>,
    head: HashMap<u16, HeadItemData>,
    body: HashMap<u16, BodyItemData>,
    hands: HashMap<u16, HandsItemData>,
    feet: HashMap<u16, FeetItemData>,
    back: HashMap<u16, BackItemData>,
    jewellery: HashMap<u16, JewelleryItemData>,
    weapon: HashMap<u16, WeaponItemData>,
    subweapon: HashMap<u16, SubWeaponItemData>,
    consumable: HashMap<u16, ConsumableItemData>,
    gem: HashMap<u16, GemItemData>,
    material: HashMap<u16, MaterialItemData>,
    quest: HashMap<u16, QuestItemData>,
    vehicle: HashMap<u16, VehicleItemData>,
    item_grades: Vec<ItemGradeData>,
}

#[allow(dead_code)]
impl ItemDatabase {
    pub fn new(
        face: HashMap<u16, FaceItemData>,
        head: HashMap<u16, HeadItemData>,
        body: HashMap<u16, BodyItemData>,
        hands: HashMap<u16, HandsItemData>,
        feet: HashMap<u16, FeetItemData>,
        back: HashMap<u16, BackItemData>,
        jewellery: HashMap<u16, JewelleryItemData>,
        weapon: HashMap<u16, WeaponItemData>,
        subweapon: HashMap<u16, SubWeaponItemData>,
        consumable: HashMap<u16, ConsumableItemData>,
        gem: HashMap<u16, GemItemData>,
        material: HashMap<u16, MaterialItemData>,
        quest: HashMap<u16, QuestItemData>,
        vehicle: HashMap<u16, VehicleItemData>,
        item_grades: Vec<ItemGradeData>,
    ) -> Self {
        Self {
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
        }
    }

    pub fn get_item_grade(&self, grade: u8) -> Option<&ItemGradeData> {
        self.item_grades.get(grade as usize)
    }

    pub fn get_item(&self, item: ItemReference) -> Option<ItemData> {
        match item.item_type {
            ItemType::Face => self
                .face
                .get(&(item.item_number as u16))
                .map(|x| ItemData::Face(&x)),
            ItemType::Head => self
                .head
                .get(&(item.item_number as u16))
                .map(|x| ItemData::Head(&x)),
            ItemType::Body => self
                .body
                .get(&(item.item_number as u16))
                .map(|x| ItemData::Body(&x)),
            ItemType::Hands => self
                .hands
                .get(&(item.item_number as u16))
                .map(|x| ItemData::Hands(&x)),
            ItemType::Feet => self
                .feet
                .get(&(item.item_number as u16))
                .map(|x| ItemData::Feet(&x)),
            ItemType::Back => self
                .back
                .get(&(item.item_number as u16))
                .map(|x| ItemData::Back(&x)),
            ItemType::Jewellery => self
                .jewellery
                .get(&(item.item_number as u16))
                .map(|x| ItemData::Jewellery(&x)),
            ItemType::Weapon => self
                .weapon
                .get(&(item.item_number as u16))
                .map(|x| ItemData::Weapon(&x)),
            ItemType::SubWeapon => self
                .subweapon
                .get(&(item.item_number as u16))
                .map(|x| ItemData::SubWeapon(&x)),
            ItemType::Consumable => self
                .consumable
                .get(&(item.item_number as u16))
                .map(|x| ItemData::Consumable(&x)),
            ItemType::Gem => self
                .gem
                .get(&(item.item_number as u16))
                .map(|x| ItemData::Gem(&x)),
            ItemType::Material => self
                .material
                .get(&(item.item_number as u16))
                .map(|x| ItemData::Material(&x)),
            ItemType::Quest => self
                .quest
                .get(&(item.item_number as u16))
                .map(|x| ItemData::Quest(&x)),
            ItemType::Vehicle => self
                .vehicle
                .get(&(item.item_number as u16))
                .map(|x| ItemData::Vehicle(&x)),
            _ => None,
        }
    }

    pub fn get_base_item(&self, item: ItemReference) -> Option<&BaseItemData> {
        match item.item_type {
            ItemType::Face => self
                .face
                .get(&(item.item_number as u16))
                .map(|x| &x.item_data),
            ItemType::Head => self
                .head
                .get(&(item.item_number as u16))
                .map(|x| &x.item_data),
            ItemType::Body => self
                .body
                .get(&(item.item_number as u16))
                .map(|x| &x.item_data),
            ItemType::Hands => self
                .hands
                .get(&(item.item_number as u16))
                .map(|x| &x.item_data),
            ItemType::Feet => self
                .feet
                .get(&(item.item_number as u16))
                .map(|x| &x.item_data),
            ItemType::Back => self
                .back
                .get(&(item.item_number as u16))
                .map(|x| &x.item_data),
            ItemType::Jewellery => self
                .jewellery
                .get(&(item.item_number as u16))
                .map(|x| &x.item_data),
            ItemType::Weapon => self
                .weapon
                .get(&(item.item_number as u16))
                .map(|x| &x.item_data),
            ItemType::SubWeapon => self
                .subweapon
                .get(&(item.item_number as u16))
                .map(|x| &x.item_data),
            ItemType::Consumable => self
                .consumable
                .get(&(item.item_number as u16))
                .map(|x| &x.item_data),
            ItemType::Gem => self
                .gem
                .get(&(item.item_number as u16))
                .map(|x| &x.item_data),
            ItemType::Material => self
                .material
                .get(&(item.item_number as u16))
                .map(|x| &x.item_data),
            ItemType::Quest => self
                .quest
                .get(&(item.item_number as u16))
                .map(|x| &x.item_data),
            ItemType::Vehicle => self
                .vehicle
                .get(&(item.item_number as u16))
                .map(|x| &x.item_data),
            _ => None,
        }
    }

    pub fn get_face_item(&self, id: usize) -> Option<&FaceItemData> {
        self.face.get(&(id as u16))
    }

    pub fn get_head_item(&self, id: usize) -> Option<&HeadItemData> {
        self.head.get(&(id as u16))
    }

    pub fn get_body_item(&self, id: usize) -> Option<&BodyItemData> {
        self.body.get(&(id as u16))
    }

    pub fn get_hands_item(&self, id: usize) -> Option<&HandsItemData> {
        self.hands.get(&(id as u16))
    }

    pub fn get_feet_item(&self, id: usize) -> Option<&FeetItemData> {
        self.feet.get(&(id as u16))
    }

    pub fn get_back_item(&self, id: usize) -> Option<&BackItemData> {
        self.back.get(&(id as u16))
    }

    pub fn get_jewellery_item(&self, id: usize) -> Option<&JewelleryItemData> {
        self.jewellery.get(&(id as u16))
    }

    pub fn get_weapon_item(&self, id: usize) -> Option<&WeaponItemData> {
        self.weapon.get(&(id as u16))
    }

    pub fn get_consumable_item(&self, id: usize) -> Option<&ConsumableItemData> {
        self.consumable.get(&(id as u16))
    }

    pub fn get_gem_item(&self, id: usize) -> Option<&GemItemData> {
        self.gem.get(&(id as u16))
    }

    pub fn get_material_item(&self, id: usize) -> Option<&MaterialItemData> {
        self.material.get(&(id as u16))
    }

    pub fn get_quest_item(&self, id: usize) -> Option<&QuestItemData> {
        self.quest.get(&(id as u16))
    }

    pub fn get_vehicle_item(&self, id: usize) -> Option<&VehicleItemData> {
        self.vehicle.get(&(id as u16))
    }
}
