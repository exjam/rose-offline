use enum_map::Enum;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::data::ItemReference;

const MAX_STACKABLE_ITEM_QUANTITY: u32 = 999;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Enum, Serialize, Deserialize)]
pub enum EquipmentIndex {
    Face,
    Head,
    Body,
    Back,
    Hands,
    Feet,
    WeaponRight,
    WeaponLeft,
    Necklace,
    Ring,
    Earring,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Enum, Serialize, Deserialize)]
pub enum VehiclePartIndex {
    Body,
    Engine,
    Leg,
    Ability,
    Arms,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Enum, Serialize, Deserialize)]
pub enum AmmoIndex {
    Arrow,
    Bullet,
    Throw,
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

#[derive(Copy, Clone)]
pub enum ItemWeaponType {
    OneHanded,
    TwoHanded,
    Bow,
    Gun,
    Launcher,
    MagicMelee,
    MagicRanged,
    Crossbow,
    Katar,
    DualWield,
}

impl ItemWeaponType {
    pub fn from(item_class: ItemClass) -> Option<Self> {
        match item_class {
            ItemClass::OneHandedSword | ItemClass::OneHandedBlunt => {
                Some(ItemWeaponType::OneHanded)
            }
            ItemClass::TwoHandedSword | ItemClass::Spear | ItemClass::TwoHandedAxe => {
                Some(ItemWeaponType::TwoHanded)
            }
            ItemClass::Bow => Some(ItemWeaponType::Bow),
            ItemClass::Gun | ItemClass::DualGuns => Some(ItemWeaponType::Gun),
            ItemClass::Launcher => Some(ItemWeaponType::Launcher),
            ItemClass::MagicStaff => Some(ItemWeaponType::MagicMelee),
            ItemClass::MagicWand => Some(ItemWeaponType::MagicRanged),
            ItemClass::Crossbow => Some(ItemWeaponType::Crossbow),
            ItemClass::Katar => Some(ItemWeaponType::Katar),
            ItemClass::DualSwords => Some(ItemWeaponType::DualWield),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct EquipmentItem {
    pub item: ItemReference,
    pub gem: u16,
    pub durability: u8,
    pub life: u16,
    pub grade: u8,
    pub is_crafted: bool,
    pub has_socket: bool,
    pub is_appraised: bool,
}

impl EquipmentItem {
    pub fn new(item: &ItemReference) -> Option<EquipmentItem> {
        if item.item_type.is_equipment_item() {
            Some(EquipmentItem {
                item: *item,
                gem: 0,
                durability: 100,
                life: 1000,
                grade: 0,
                is_crafted: false,
                has_socket: false,
                is_appraised: false,
            })
        } else {
            None
        }
    }

    pub fn is_broken(&self) -> bool {
        self.life == 0
    }
}

impl From<&EquipmentItem> for ItemReference {
    fn from(equipment: &EquipmentItem) -> Self {
        equipment.item
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct StackableItem {
    pub item: ItemReference,
    pub quantity: u32,
}

#[derive(Debug)]
pub enum StackError {
    NotStackable,
    NotSameItem,
    PartialStack(u32), // usize is how much quantity can stack
}

impl StackableItem {
    pub fn new(item: &ItemReference, quantity: u32) -> Option<StackableItem> {
        if item.item_type.is_stackable_item() && quantity > 0 {
            Some(StackableItem {
                item: *item,
                quantity,
            })
        } else {
            None
        }
    }

    pub fn can_stack_with(&self, stackable: &StackableItem) -> Result<(), StackError> {
        if self.item != stackable.item {
            Err(StackError::NotSameItem)
        } else if self.quantity + stackable.quantity > MAX_STACKABLE_ITEM_QUANTITY {
            Err(StackError::PartialStack(
                MAX_STACKABLE_ITEM_QUANTITY - self.quantity,
            ))
        } else {
            Ok(())
        }
    }

    pub fn try_stack_with(&mut self, stackable: StackableItem) -> Result<(), StackError> {
        self.can_stack_with(&stackable)?;
        self.quantity += stackable.quantity;
        Ok(())
    }

    pub fn try_take_subquantity(&mut self, quantity: u32) -> Option<StackableItem> {
        if self.quantity < quantity {
            None
        } else {
            self.quantity -= quantity;

            let mut item = self.clone();
            item.quantity = quantity;
            Some(item)
        }
    }
}

impl From<&StackableItem> for ItemReference {
    fn from(stackable: &StackableItem) -> Self {
        stackable.item
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Item {
    Equipment(EquipmentItem),
    Stackable(StackableItem),
}

impl Item {
    pub fn new(item: &ItemReference, quantity: u32) -> Option<Item> {
        if item.item_type.is_stackable_item() {
            StackableItem::new(item, quantity).map(Item::Stackable)
        } else if item.item_type.is_equipment_item() {
            EquipmentItem::new(item).map(Item::Equipment)
        } else {
            None
        }
    }

    pub fn can_stack_with(&self, stackable: &StackableItem) -> Result<(), StackError> {
        match self {
            Item::Equipment(_) => Err(StackError::NotStackable),
            Item::Stackable(self_stackable) => self_stackable.can_stack_with(stackable),
        }
    }

    pub fn try_stack_with(&mut self, stackable: StackableItem) -> Result<(), StackError> {
        match self {
            Item::Equipment(_) => Err(StackError::NotStackable),
            Item::Stackable(item) => item.try_stack_with(stackable),
        }
    }

    pub fn try_stack_with_item(&mut self, with_item: Item) -> Result<(), StackError> {
        match self {
            Item::Equipment(_) => Err(StackError::NotStackable),
            Item::Stackable(self_stackable) => match with_item {
                Item::Equipment(_) => Err(StackError::NotStackable),
                Item::Stackable(with_stackable) => self_stackable.try_stack_with(with_stackable),
            },
        }
    }

    // Only succeeds if quantity < self.quantity, this can not take the whole item
    pub fn try_take_subquantity(&mut self, quantity: u32) -> Option<Item> {
        match self {
            Item::Equipment(_) => None,
            Item::Stackable(stackable) => stackable
                .try_take_subquantity(quantity)
                .map(Item::Stackable),
        }
    }

    pub fn get_item_reference(&self) -> ItemReference {
        match self {
            Item::Equipment(item) => item.item,
            Item::Stackable(item) => item.item,
        }
    }

    pub fn get_item_number(&self) -> usize {
        match self {
            Item::Equipment(item) => item.item.item_number,
            Item::Stackable(item) => item.item.item_number,
        }
    }

    pub fn get_item_type(&self) -> ItemType {
        match self {
            Item::Equipment(item) => item.item.item_type,
            Item::Stackable(item) => item.item.item_type,
        }
    }

    pub fn get_quantity(&self) -> u32 {
        match self {
            Item::Equipment(_) => 1,
            Item::Stackable(item) => item.quantity,
        }
    }

    #[allow(dead_code)]
    pub fn as_equipment(&self) -> Option<&EquipmentItem> {
        match self {
            Item::Equipment(equipment) => Some(equipment),
            Item::Stackable(_) => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_equipment_mut(&mut self) -> Option<&mut EquipmentItem> {
        match self {
            Item::Equipment(equipment) => Some(equipment),
            Item::Stackable(_) => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_stackable(&self) -> Option<&StackableItem> {
        match self {
            Item::Equipment(_) => None,
            Item::Stackable(stackable) => Some(stackable),
        }
    }

    #[allow(dead_code)]
    pub fn as_stackable_mut(&mut self) -> Option<&mut StackableItem> {
        match self {
            Item::Equipment(_) => None,
            Item::Stackable(stackable) => Some(stackable),
        }
    }

    pub fn is_same_item_reference(&self, item_reference: ItemReference) -> bool {
        match self {
            Item::Equipment(item) => item.item == item_reference,
            Item::Stackable(item) => item.item == item_reference,
        }
    }

    pub fn is_same_item(&self, compare_item: &Item) -> bool {
        match self {
            Item::Equipment(_) => self == compare_item,
            Item::Stackable(item) => compare_item.is_same_item_reference(item.item),
        }
    }
}

pub trait ItemSlotBehaviour {
    fn try_take_quantity(&mut self, quantity: u32) -> Option<Item>;
    fn try_stack_with_item(&mut self, with_item: Item) -> Result<&Item, StackError>;

    fn contains_same_item(&self, compare_item: &Item) -> bool;
}

impl ItemSlotBehaviour for Option<Item> {
    fn try_take_quantity(&mut self, quantity: u32) -> Option<Item> {
        match self {
            Some(item) => match item.get_quantity().cmp(&quantity) {
                Ordering::Less => None,
                Ordering::Equal => self.take(),
                Ordering::Greater => item.try_take_subquantity(quantity),
            },
            None => None,
        }
    }

    fn try_stack_with_item(&mut self, with_item: Item) -> Result<&Item, StackError> {
        match self {
            Some(item) => match item.try_stack_with_item(with_item) {
                Ok(_) => Ok(self.as_ref().unwrap()),
                Err(err) => Err(err),
            },
            None => {
                *self = Some(with_item);
                Ok(self.as_ref().unwrap())
            }
        }
    }

    fn contains_same_item(&self, compare_item: &Item) -> bool {
        match self {
            Some(item) => item.is_same_item(compare_item),
            None => false,
        }
    }
}

pub trait StackableSlotBehaviour {
    fn try_take_quantity(&mut self, quantity: u32) -> Option<StackableItem>;

    fn can_stack_with(&self, stackable: &StackableItem) -> Result<(), StackError>;
    fn try_stack_with(&mut self, with_item: StackableItem) -> Result<&StackableItem, StackError>;

    fn contains_same_item(&self, compare_item: &StackableItem) -> bool;
}

impl StackableSlotBehaviour for Option<StackableItem> {
    fn try_take_quantity(&mut self, quantity: u32) -> Option<StackableItem> {
        match self {
            Some(item) => match item.quantity.cmp(&quantity) {
                Ordering::Less => None,
                Ordering::Equal => self.take(),
                Ordering::Greater => item.try_take_subquantity(quantity),
            },
            None => None,
        }
    }

    fn can_stack_with(&self, stackable: &StackableItem) -> Result<(), StackError> {
        match self {
            Some(item) => item.can_stack_with(stackable),
            None => Ok(()),
        }
    }

    fn try_stack_with(&mut self, with_item: StackableItem) -> Result<&StackableItem, StackError> {
        match self {
            Some(item) => match item.try_stack_with(with_item) {
                Ok(_) => Ok(self.as_ref().unwrap()),
                Err(err) => Err(err),
            },
            None => {
                *self = Some(with_item);
                Ok(self.as_ref().unwrap())
            }
        }
    }

    fn contains_same_item(&self, compare_item: &StackableItem) -> bool {
        match self {
            Some(item) => item.item == compare_item.item,
            None => false,
        }
    }
}
