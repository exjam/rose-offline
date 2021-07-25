use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

use super::ItemReference;

const MAX_STACKABLE_ITEM_QUANTITY: u32 = 999;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, FromPrimitive, PartialEq)]
pub enum ItemType {
    Face = 1,
    Head = 2,
    Body = 3,
    Hands = 4,
    Feet = 5,
    Back = 6,
    Jewellery = 7,
    Weapon = 8,
    SubWeapon = 9,
    Consumable = 10,
    Gem = 11,
    Material = 12,
    Quest = 13,
    Vehicle = 14,
    Money = 31,
}

impl ItemType {
    pub fn is_stackable(self) -> bool {
        matches!(
            self,
            ItemType::Consumable | ItemType::Gem | ItemType::Material | ItemType::Quest
        )
    }

    #[allow(dead_code)]
    pub fn is_money(self) -> bool {
        matches!(self, ItemType::Money)
    }

    #[allow(dead_code)]
    pub fn is_quest_item(self) -> bool {
        matches!(self, ItemType::Quest)
    }

    pub fn is_equipment(self) -> bool {
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
        )
    }
}

// TODO: The number mappings for this should move to irose stb loading code?
#[derive(Copy, Clone, FromPrimitive)]
pub enum ItemClass {
    Unknown = 0,

    FaceMask = 111,
    FaceGlasses = 112,
    FaceEtc = 113,

    Helmet = 121,
    MagicHat = 122,
    Hat = 123,
    HairAccessory = 124,

    CombatUniform = 131,
    MagicClothes = 132,
    CasualClothes = 133,

    Gauntlet = 141,
    MagicGlove = 142,
    Glove = 143,

    Boots = 151,
    MagicBoots = 152,
    Shoes = 153,

    BackArmor = 161,
    Bag = 162,
    Wings = 163,
    ArrowBox = 164,
    BulletBox = 165,
    ShellBox = 166,

    Ring = 171,
    Necklace = 172,
    Earring = 173,

    OneHandedSword = 211,
    OneHandedBlunt = 212,

    TwoHandedSword = 221,
    Spear = 222,
    TwoHandedAxe = 223,

    Bow = 231,
    Gun = 232,
    Launcher = 233,

    MagicStaff = 241,
    MagicWand = 242,

    Katar = 251,
    DualSwords = 252,
    DualGuns = 253,

    Shield = 261,
    SupportTool = 262,

    Crossbow = 271,

    Medicine = 311,
    Food = 312,
    MagicItem = 313,
    SkillBook = 314,
    RepairTool = 315,
    QuestScroll = 316,
    EngineFuel = 317,
    AutomaticConsumption = 320,
    TimeCoupon = 321,

    Jewel = 411,
    WorkOfArt = 412,

    Metal = 421,
    OtherworldlyMetal = 422,
    StoneMaterial = 423,
    WoodenMaterial = 424,
    Leather = 425,
    Cloth = 426,
    RefiningMaterial = 427,
    Chemicals = 428,
    Material = 429,
    GatheredGoods = 430,

    Arrow = 431,
    Bullet = 432,
    Shell = 433,

    QuestItems = 441,
    Certification = 442,

    CartBody = 511,
    CastleGearBody = 512,

    CartEngine = 521,
    CastleGearEngine = 522,

    CartWheels = 531,
    CastleGearLeg = 532,

    CartAccessory = 551,
    CastleGearWeapon = 552,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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
        if item.item_type.is_equipment() {
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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
        if item.item_type.is_stackable() && quantity > 0 {
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

    pub fn stack_with(&mut self, stackable: StackableItem) -> Result<(), StackError> {
        self.can_stack_with(&stackable)?;
        self.quantity += stackable.quantity;
        Ok(())
    }
}

impl From<&StackableItem> for ItemReference {
    fn from(stackable: &StackableItem) -> Self {
        stackable.item
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Item {
    Equipment(EquipmentItem),
    Stackable(StackableItem),
}

impl Item {
    pub fn new(item: &ItemReference, quantity: u32) -> Option<Item> {
        if item.item_type.is_stackable() {
            StackableItem::new(item, quantity).map(Item::Stackable)
        } else if item.item_type.is_equipment() {
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

    pub fn stack_with(&mut self, stackable: StackableItem) -> Result<(), StackError> {
        match self {
            Item::Equipment(_) => Err(StackError::NotStackable),
            Item::Stackable(item) => item.stack_with(stackable),
        }
    }

    pub fn stack_with_item(&mut self, with_item: Item) -> Result<(), StackError> {
        match self {
            Item::Equipment(_) => Err(StackError::NotStackable),
            Item::Stackable(self_stackable) => match with_item {
                Item::Equipment(_) => Err(StackError::NotStackable),
                Item::Stackable(with_stackable) => self_stackable.stack_with(with_stackable),
            },
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

    pub fn is_same_item(&self, item_reference: ItemReference) -> bool {
        match self {
            Item::Equipment(item) => item.item == item_reference,
            Item::Stackable(item) => item.item == item_reference,
        }
    }
}
