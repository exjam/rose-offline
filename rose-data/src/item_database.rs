use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

use crate::{AbilityType, SkillId, StatusEffectId};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
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

#[derive(Copy, Clone, Debug)]
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
    pub equip_union_requirement: ArrayVec<u32, 2>,
    pub equip_ability_requirement: ArrayVec<(AbilityType, u32), 2>,
    pub add_ability_union_requirement: ArrayVec<u32, 2>,
    pub add_ability: ArrayVec<(AbilityType, i32), 2>,
    pub durability: u32,
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
}

#[derive(Debug)]
pub struct WeaponItemData {
    pub item_data: BaseItemData,
    pub attack_range: i32,
    pub attack_power: i32,
    pub attack_speed: i32,
    pub motion_type: u32,
    pub is_magic_damage: bool,
}

#[derive(Debug)]
pub struct SubWeaponItemData {
    pub item_data: BaseItemData,
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
}

#[derive(Debug)]
pub struct MaterialItemData {
    pub item_data: BaseItemData,
}

#[derive(Debug)]

pub struct QuestItemData {
    pub item_data: BaseItemData,
}

#[derive(Debug)]
pub enum VehicleItemPart {
    Body,
    Engine,
    Leg,
    Ability,
    Arms,
}

#[derive(Debug)]
pub struct VehicleItemData {
    pub item_data: BaseItemData,
    pub vehicle_part: VehicleItemPart,
    pub move_speed: u32,
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
    #[allow(clippy::too_many_arguments)]
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
                .map(ItemData::Face),
            ItemType::Head => self
                .head
                .get(&(item.item_number as u16))
                .map(ItemData::Head),
            ItemType::Body => self
                .body
                .get(&(item.item_number as u16))
                .map(ItemData::Body),
            ItemType::Hands => self
                .hands
                .get(&(item.item_number as u16))
                .map(ItemData::Hands),
            ItemType::Feet => self
                .feet
                .get(&(item.item_number as u16))
                .map(ItemData::Feet),
            ItemType::Back => self
                .back
                .get(&(item.item_number as u16))
                .map(ItemData::Back),
            ItemType::Jewellery => self
                .jewellery
                .get(&(item.item_number as u16))
                .map(ItemData::Jewellery),
            ItemType::Weapon => self
                .weapon
                .get(&(item.item_number as u16))
                .map(ItemData::Weapon),
            ItemType::SubWeapon => self
                .subweapon
                .get(&(item.item_number as u16))
                .map(ItemData::SubWeapon),
            ItemType::Consumable => self
                .consumable
                .get(&(item.item_number as u16))
                .map(ItemData::Consumable),
            ItemType::Gem => self.gem.get(&(item.item_number as u16)).map(ItemData::Gem),
            ItemType::Material => self
                .material
                .get(&(item.item_number as u16))
                .map(ItemData::Material),
            ItemType::Quest => self
                .quest
                .get(&(item.item_number as u16))
                .map(ItemData::Quest),
            ItemType::Vehicle => self
                .vehicle
                .get(&(item.item_number as u16))
                .map(ItemData::Vehicle),
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

    pub fn iter_items(
        &self,
        item_type: ItemType,
    ) -> Box<dyn std::iter::Iterator<Item = ItemReference> + '_> {
        match item_type {
            ItemType::Face => Box::new(
                self.face
                    .keys()
                    .map(|id| ItemReference::new(ItemType::Face, *id as usize)),
            ),
            ItemType::Head => Box::new(
                self.head
                    .keys()
                    .map(|id| ItemReference::new(ItemType::Head, *id as usize)),
            ),
            ItemType::Body => Box::new(
                self.body
                    .keys()
                    .map(|id| ItemReference::new(ItemType::Body, *id as usize)),
            ),
            ItemType::Hands => Box::new(
                self.hands
                    .keys()
                    .map(|id| ItemReference::new(ItemType::Hands, *id as usize)),
            ),
            ItemType::Feet => Box::new(
                self.feet
                    .keys()
                    .map(|id| ItemReference::new(ItemType::Feet, *id as usize)),
            ),
            ItemType::Back => Box::new(
                self.back
                    .keys()
                    .map(|id| ItemReference::new(ItemType::Back, *id as usize)),
            ),
            ItemType::Jewellery => Box::new(
                self.jewellery
                    .keys()
                    .map(|id| ItemReference::new(ItemType::Jewellery, *id as usize)),
            ),
            ItemType::Weapon => Box::new(
                self.weapon
                    .keys()
                    .map(|id| ItemReference::new(ItemType::Weapon, *id as usize)),
            ),
            ItemType::SubWeapon => Box::new(
                self.subweapon
                    .keys()
                    .map(|id| ItemReference::new(ItemType::SubWeapon, *id as usize)),
            ),
            ItemType::Consumable => Box::new(
                self.consumable
                    .keys()
                    .map(|id| ItemReference::new(ItemType::Consumable, *id as usize)),
            ),
            ItemType::Gem => Box::new(
                self.gem
                    .keys()
                    .map(|id| ItemReference::new(ItemType::Gem, *id as usize)),
            ),
            ItemType::Material => Box::new(
                self.material
                    .keys()
                    .map(|id| ItemReference::new(ItemType::Material, *id as usize)),
            ),
            ItemType::Quest => Box::new(
                self.quest
                    .keys()
                    .map(|id| ItemReference::new(ItemType::Quest, *id as usize)),
            ),
            ItemType::Vehicle => Box::new(
                self.vehicle
                    .keys()
                    .map(|id| ItemReference::new(ItemType::Vehicle, *id as usize)),
            ),
        }
    }
}