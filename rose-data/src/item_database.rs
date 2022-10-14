use arrayvec::ArrayVec;
use enum_map::Enum;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

use crate::{
    AbilityType, EffectFileId, EffectId, JobClassId, SkillId, SoundId, StatusEffectId,
    StringDatabase, VehiclePartIndex,
};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ItemReference {
    pub item_type: ItemType,
    pub item_number: usize,
}

impl ItemReference {
    pub fn new(item_type: ItemType, item_number: usize) -> Self {
        Self {
            item_type,
            item_number,
        }
    }
}

#[derive(Clone, Copy, Debug, Enum, Deserialize, Serialize, PartialEq, Eq)]
pub enum ItemType {
    Face,
    Head,
    Body,
    Hands,
    Feet,
    Back,
    Jewellery,
    Weapon,
    SubWeapon,
    Consumable,
    Gem,
    Material,
    Quest,
    Vehicle,
}

impl ItemType {
    pub fn is_stackable_item(self) -> bool {
        matches!(
            self,
            ItemType::Consumable | ItemType::Gem | ItemType::Material | ItemType::Quest
        )
    }

    #[allow(dead_code)]
    pub fn is_quest_item(self) -> bool {
        matches!(self, ItemType::Quest)
    }

    pub fn is_equipment_item(self) -> bool {
        matches!(
            self,
            ItemType::Face
                | ItemType::Head
                | ItemType::Body
                | ItemType::Hands
                | ItemType::Feet
                | ItemType::Back
                | ItemType::Jewellery
                | ItemType::Weapon
                | ItemType::SubWeapon
                | ItemType::Vehicle
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ItemClass {
    Unknown,

    FaceMask,
    FaceGlasses,
    FaceEtc,

    Helmet,
    MagicHat,
    Hat,
    HairAccessory,

    CombatUniform,
    MagicClothes,
    CasualClothes,

    Gauntlet,
    MagicGlove,
    Glove,

    Boots,
    MagicBoots,
    Shoes,

    BackArmor,
    Bag,
    Wings,
    ArrowBox,
    BulletBox,
    ShellBox,

    Ring,
    Necklace,
    Earring,

    OneHandedSword,
    OneHandedBlunt,

    TwoHandedSword,
    Spear,
    TwoHandedAxe,

    Bow,
    Gun,
    Launcher,

    MagicStaff,
    MagicWand,

    Katar,
    DualSwords,
    DualGuns,

    Shield,
    SupportTool,

    Crossbow,

    Medicine,
    Food,
    MagicItem,
    SkillBook,
    RepairTool,
    QuestScroll,
    EngineFuel,
    AutomaticConsumption,
    TimeCoupon,

    Jewel,
    WorkOfArt,

    Metal,
    OtherworldlyMetal,
    StoneMaterial,
    WoodenMaterial,
    Leather,
    Cloth,
    RefiningMaterial,
    Chemicals,
    Material,
    GatheredGoods,

    Arrow,
    Bullet,
    Shell,

    QuestItems,
    Certification,

    CartBody,
    CastleGearBody,

    CartEngine,
    CastleGearEngine,

    CartWheels,
    CastleGearLeg,

    CartAccessory,
    CastleGearWeapon,
}

impl ItemClass {
    pub fn is_two_handed_weapon(&self) -> bool {
        matches!(
            *self,
            ItemClass::TwoHandedSword
                | ItemClass::Spear
                | ItemClass::TwoHandedAxe
                | ItemClass::Bow
                | ItemClass::Gun
                | ItemClass::Launcher
                | ItemClass::MagicStaff
                | ItemClass::Katar
                | ItemClass::DualSwords
                | ItemClass::DualGuns
        )
    }
}

#[derive(Debug)]
pub struct BaseItemData {
    pub id: ItemReference,
    pub name: &'static str,
    pub description: &'static str,
    pub class: ItemClass,
    pub base_price: u32,
    pub price_rate: u32,
    pub weight: u32,
    pub quality: u32,
    pub icon_index: u32,
    pub field_model_index: u32,
    pub equip_sound_id: Option<SoundId>,
    pub craft_skill_type: u32,
    pub craft_skill_level: u32,
    pub craft_material: u32,
    pub craft_difficulty: u32,
    pub equip_job_class_requirement: Option<JobClassId>,
    pub equip_union_requirement: ArrayVec<u32, 2>,
    pub equip_ability_requirement: ArrayVec<(AbilityType, u32), 2>,
    pub add_ability_union_requirement: ArrayVec<u32, 2>,
    pub add_ability: ArrayVec<(AbilityType, i32), 2>,
    pub durability: u8,
    pub rare_type: u32,
    pub defence: u32,
    pub resistance: u32,
}

#[derive(Debug)]
pub struct FaceItemData {
    pub item_data: BaseItemData,
}

#[derive(Debug)]
pub struct HeadItemData {
    pub item_data: BaseItemData,
    pub hair_type: u32,
}

#[derive(Debug)]
pub struct BodyItemData {
    pub item_data: BaseItemData,
}

#[derive(Debug)]
pub struct HandsItemData {
    pub item_data: BaseItemData,
}

#[derive(Debug)]
pub struct BackItemData {
    pub item_data: BaseItemData,
    pub move_speed: u32,
}

#[derive(Debug)]
pub struct FeetItemData {
    pub item_data: BaseItemData,
    pub move_speed: u32,
}

#[derive(Debug)]
pub struct JewelleryItemData {
    pub item_data: BaseItemData,
}

#[derive(Debug)]
pub struct GemItemData {
    pub item_data: BaseItemData,
    pub gem_add_ability: ArrayVec<(AbilityType, i32), 2>,
    pub gem_effect_id: Option<EffectId>,
    pub gem_sprite_id: u32,
}

#[derive(Debug)]
pub struct WeaponItemData {
    pub item_data: BaseItemData,
    pub attack_range: i32,
    pub attack_power: i32,
    pub attack_speed: i32,
    pub motion_type: u32,
    pub is_magic_damage: bool,
    pub bullet_effect_id: Option<EffectId>,
    pub effect_id: Option<EffectId>,
    pub attack_start_sound_id: Option<SoundId>,
    pub attack_fire_sound_id: Option<SoundId>,
    pub attack_hit_sound_index: u32,
    pub gem_position: u32,
}

#[derive(Debug)]
pub struct SubWeaponItemData {
    pub item_data: BaseItemData,
    pub gem_position: u32,
}

#[derive(Debug)]
pub struct ConsumableItemData {
    pub item_data: BaseItemData,
    pub store_skin: i32,
    pub confile_index: usize,
    pub ability_requirement: Option<(AbilityType, i32)>,
    pub add_ability: Option<(AbilityType, i32)>,
    pub learn_skill_id: Option<SkillId>,
    pub use_skill_id: Option<SkillId>,
    pub apply_status_effect: Option<(StatusEffectId, i32)>,
    pub cooldown_type_id: usize,
    pub cooldown_duration: Duration,
    pub effect_file_id: Option<EffectFileId>,
    pub effect_sound_id: Option<SoundId>,
}

#[derive(Debug)]
pub struct MaterialItemData {
    pub item_data: BaseItemData,
    pub bullet_effect_id: Option<EffectId>,
}

#[derive(Debug)]

pub struct QuestItemData {
    pub item_data: BaseItemData,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VehicleType {
    Cart,
    CastleGear,
}

#[derive(Debug)]
pub struct VehicleItemData {
    pub item_data: BaseItemData,
    pub vehicle_type: VehicleType,
    pub version: u32,
    pub vehicle_part: VehiclePartIndex,
    pub move_speed: u32,
    pub max_fuel: u32,
    pub fuel_use_rate: u32,
    pub attack_range: i32,
    pub attack_power: i32,
    pub attack_speed: i32,
    pub base_motion_index: u32,
    pub base_avatar_motion_index: u32,
    pub ability_requirement: Option<(AbilityType, i32)>,
    pub skill_requirement: Option<(SkillId, i32)>,
    pub ride_effect_id: Option<EffectId>,
    pub ride_sound_id: Option<SoundId>,
    pub dismount_effect_id: Option<EffectId>,
    pub dismount_sound_id: Option<SoundId>,
    pub dead_effect_id: Option<EffectId>,
    pub dead_sound_id: Option<SoundId>,
    pub stop_sound_id: Option<SoundId>,
    pub move_effect_id: Option<EffectId>,
    pub move_sound_id: Option<SoundId>,
    pub attack_effect_id: Option<EffectId>,
    pub attack_sound_id: Option<SoundId>,
    pub hit_effect_id: Option<EffectId>,
    pub hit_sound_id: Option<SoundId>,
    pub bullet_effect_id: Option<EffectId>,
    pub bullet_fire_point: u32,
}

#[derive(Debug)]
pub struct ItemGradeData {
    pub attack: i32,
    pub hit: i32,
    pub defence: i32,
    pub resistance: i32,
    pub avoid: i32,
    pub glow_colour: (f32, f32, f32),
}

#[allow(dead_code)]
#[derive(Debug)]
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
    _string_database: Arc<StringDatabase>,
    face: Vec<Option<FaceItemData>>,
    head: Vec<Option<HeadItemData>>,
    body: Vec<Option<BodyItemData>>,
    hands: Vec<Option<HandsItemData>>,
    feet: Vec<Option<FeetItemData>>,
    back: Vec<Option<BackItemData>>,
    jewellery: Vec<Option<JewelleryItemData>>,
    weapon: Vec<Option<WeaponItemData>>,
    subweapon: Vec<Option<SubWeaponItemData>>,
    consumable: Vec<Option<ConsumableItemData>>,
    gem: Vec<Option<GemItemData>>,
    material: Vec<Option<MaterialItemData>>,
    quest: Vec<Option<QuestItemData>>,
    vehicle: Vec<Option<VehicleItemData>>,
    item_grades: Vec<ItemGradeData>,
}

#[allow(dead_code)]
impl ItemDatabase {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        string_database: Arc<StringDatabase>,
        face: Vec<Option<FaceItemData>>,
        head: Vec<Option<HeadItemData>>,
        body: Vec<Option<BodyItemData>>,
        hands: Vec<Option<HandsItemData>>,
        feet: Vec<Option<FeetItemData>>,
        back: Vec<Option<BackItemData>>,
        jewellery: Vec<Option<JewelleryItemData>>,
        weapon: Vec<Option<WeaponItemData>>,
        subweapon: Vec<Option<SubWeaponItemData>>,
        consumable: Vec<Option<ConsumableItemData>>,
        gem: Vec<Option<GemItemData>>,
        material: Vec<Option<MaterialItemData>>,
        quest: Vec<Option<QuestItemData>>,
        vehicle: Vec<Option<VehicleItemData>>,
        item_grades: Vec<ItemGradeData>,
    ) -> Self {
        Self {
            _string_database: string_database,
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
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(ItemData::Face),
            ItemType::Head => self
                .head
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(ItemData::Head),
            ItemType::Body => self
                .body
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(ItemData::Body),
            ItemType::Hands => self
                .hands
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(ItemData::Hands),
            ItemType::Feet => self
                .feet
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(ItemData::Feet),
            ItemType::Back => self
                .back
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(ItemData::Back),
            ItemType::Jewellery => self
                .jewellery
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(ItemData::Jewellery),
            ItemType::Weapon => self
                .weapon
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(ItemData::Weapon),
            ItemType::SubWeapon => self
                .subweapon
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(ItemData::SubWeapon),
            ItemType::Consumable => self
                .consumable
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(ItemData::Consumable),
            ItemType::Gem => self
                .gem
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(ItemData::Gem),
            ItemType::Material => self
                .material
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(ItemData::Material),
            ItemType::Quest => self
                .quest
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(ItemData::Quest),
            ItemType::Vehicle => self
                .vehicle
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(ItemData::Vehicle),
        }
    }

    pub fn get_base_item(&self, item: ItemReference) -> Option<&BaseItemData> {
        match item.item_type {
            ItemType::Face => self
                .face
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(|x| &x.item_data),
            ItemType::Head => self
                .head
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(|x| &x.item_data),
            ItemType::Body => self
                .body
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(|x| &x.item_data),
            ItemType::Hands => self
                .hands
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(|x| &x.item_data),
            ItemType::Feet => self
                .feet
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(|x| &x.item_data),
            ItemType::Back => self
                .back
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(|x| &x.item_data),
            ItemType::Jewellery => self
                .jewellery
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(|x| &x.item_data),
            ItemType::Weapon => self
                .weapon
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(|x| &x.item_data),
            ItemType::SubWeapon => self
                .subweapon
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(|x| &x.item_data),
            ItemType::Consumable => self
                .consumable
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(|x| &x.item_data),
            ItemType::Gem => self
                .gem
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(|x| &x.item_data),
            ItemType::Material => self
                .material
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(|x| &x.item_data),
            ItemType::Quest => self
                .quest
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(|x| &x.item_data),
            ItemType::Vehicle => self
                .vehicle
                .get(item.item_number)
                .and_then(|x| x.as_ref())
                .map(|x| &x.item_data),
        }
    }

    pub fn get_face_item(&self, id: usize) -> Option<&FaceItemData> {
        self.face.get(id).and_then(|x| x.as_ref())
    }

    pub fn get_head_item(&self, id: usize) -> Option<&HeadItemData> {
        self.head.get(id).and_then(|x| x.as_ref())
    }

    pub fn get_body_item(&self, id: usize) -> Option<&BodyItemData> {
        self.body.get(id).and_then(|x| x.as_ref())
    }

    pub fn get_hands_item(&self, id: usize) -> Option<&HandsItemData> {
        self.hands.get(id).and_then(|x| x.as_ref())
    }

    pub fn get_feet_item(&self, id: usize) -> Option<&FeetItemData> {
        self.feet.get(id).and_then(|x| x.as_ref())
    }

    pub fn get_back_item(&self, id: usize) -> Option<&BackItemData> {
        self.back.get(id).and_then(|x| x.as_ref())
    }

    pub fn get_jewellery_item(&self, id: usize) -> Option<&JewelleryItemData> {
        self.jewellery.get(id).and_then(|x| x.as_ref())
    }

    pub fn get_sub_weapon_item(&self, id: usize) -> Option<&SubWeaponItemData> {
        self.subweapon.get(id).and_then(|x| x.as_ref())
    }

    pub fn get_weapon_item(&self, id: usize) -> Option<&WeaponItemData> {
        self.weapon.get(id).and_then(|x| x.as_ref())
    }

    pub fn get_consumable_item(&self, id: usize) -> Option<&ConsumableItemData> {
        self.consumable.get(id).and_then(|x| x.as_ref())
    }

    pub fn get_gem_item(&self, id: usize) -> Option<&GemItemData> {
        self.gem.get(id).and_then(|x| x.as_ref())
    }

    pub fn get_material_item(&self, id: usize) -> Option<&MaterialItemData> {
        self.material.get(id).and_then(|x| x.as_ref())
    }

    pub fn get_quest_item(&self, id: usize) -> Option<&QuestItemData> {
        self.quest.get(id).and_then(|x| x.as_ref())
    }

    pub fn get_vehicle_item(&self, id: usize) -> Option<&VehicleItemData> {
        self.vehicle.get(id).and_then(|x| x.as_ref())
    }

    pub fn iter_items(
        &self,
        item_type: ItemType,
    ) -> Box<dyn std::iter::Iterator<Item = ItemReference> + '_> {
        match item_type {
            ItemType::Face => {
                Box::new(self.face.iter().enumerate().filter_map(|(id, item_data)| {
                    item_data
                        .as_ref()
                        .map(|_| ItemReference::new(ItemType::Face, id))
                }))
            }
            ItemType::Head => {
                Box::new(self.head.iter().enumerate().filter_map(|(id, item_data)| {
                    item_data
                        .as_ref()
                        .map(|_| ItemReference::new(ItemType::Head, id))
                }))
            }
            ItemType::Body => {
                Box::new(self.body.iter().enumerate().filter_map(|(id, item_data)| {
                    item_data
                        .as_ref()
                        .map(|_| ItemReference::new(ItemType::Body, id))
                }))
            }
            ItemType::Hands => {
                Box::new(self.hands.iter().enumerate().filter_map(|(id, item_data)| {
                    item_data
                        .as_ref()
                        .map(|_| ItemReference::new(ItemType::Hands, id))
                }))
            }
            ItemType::Feet => {
                Box::new(self.feet.iter().enumerate().filter_map(|(id, item_data)| {
                    item_data
                        .as_ref()
                        .map(|_| ItemReference::new(ItemType::Feet, id))
                }))
            }
            ItemType::Back => {
                Box::new(self.back.iter().enumerate().filter_map(|(id, item_data)| {
                    item_data
                        .as_ref()
                        .map(|_| ItemReference::new(ItemType::Back, id))
                }))
            }
            ItemType::Jewellery => Box::new(self.jewellery.iter().enumerate().filter_map(
                |(id, item_data)| {
                    item_data
                        .as_ref()
                        .map(|_| ItemReference::new(ItemType::Jewellery, id))
                },
            )),
            ItemType::Weapon => Box::new(self.weapon.iter().enumerate().filter_map(
                |(id, item_data)| {
                    item_data
                        .as_ref()
                        .map(|_| ItemReference::new(ItemType::Weapon, id))
                },
            )),
            ItemType::SubWeapon => Box::new(self.subweapon.iter().enumerate().filter_map(
                |(id, item_data)| {
                    item_data
                        .as_ref()
                        .map(|_| ItemReference::new(ItemType::SubWeapon, id))
                },
            )),
            ItemType::Consumable => Box::new(self.consumable.iter().enumerate().filter_map(
                |(id, item_data)| {
                    item_data
                        .as_ref()
                        .map(|_| ItemReference::new(ItemType::Consumable, id))
                },
            )),
            ItemType::Gem => Box::new(self.gem.iter().enumerate().filter_map(|(id, item_data)| {
                item_data
                    .as_ref()
                    .map(|_| ItemReference::new(ItemType::Gem, id))
            })),
            ItemType::Material => Box::new(self.material.iter().enumerate().filter_map(
                |(id, item_data)| {
                    item_data
                        .as_ref()
                        .map(|_| ItemReference::new(ItemType::Material, id))
                },
            )),
            ItemType::Quest => {
                Box::new(self.quest.iter().enumerate().filter_map(|(id, item_data)| {
                    item_data
                        .as_ref()
                        .map(|_| ItemReference::new(ItemType::Quest, id))
                }))
            }
            ItemType::Vehicle => Box::new(self.vehicle.iter().enumerate().filter_map(
                |(id, item_data)| {
                    item_data
                        .as_ref()
                        .map(|_| ItemReference::new(ItemType::Vehicle, id))
                },
            )),
        }
    }
}
