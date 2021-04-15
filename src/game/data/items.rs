use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};

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
        match self {
            ItemType::Consumable
            | ItemType::Gem
            | ItemType::Material
            | ItemType::Quest
            | ItemType::Money => true,
            _ => false,
        }
    }

    pub fn is_money(self) -> bool {
        match self {
            ItemType::Money => true,
            _ => false,
        }
    }

    pub fn is_equipment(self) -> bool {
        match self {
            ItemType::Face
            | ItemType::Head
            | ItemType::Body
            | ItemType::Hands
            | ItemType::Feet
            | ItemType::Back
            | ItemType::Jewellery
            | ItemType::Weapon
            | ItemType::SubWeapon => true,
            _ => false,
        }
    }
}

#[derive(FromPrimitive)]
pub enum ItemClass {
    Ring = 171,
    Necklace = 172,
    Earring = 173,
    Arrow = 231,
    Arrow2 = 271,
    Bullet = 232,
    Throw = 233,
    Bullet2 = 253,
    NotUseBullet = 242,
    Shield = 261,
    SkillDoing = 313,
    SkillLearn = 314,
    RepairItem = 315,
    EventItem = 316,
    Fuel = 317,
    CartBody = 511,
    CastlerGearBody = 512,
    CartEngine = 521,
    CastleGearEngine = 522,
    CastleGearWeapon = 552,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EquipmentItem {
    pub item_type: ItemType,
    pub item_number: u16,
    pub gem: u16,
    pub durability: u8,
    pub life: u16,
    pub grade: u8,
    pub is_crafted: bool,
    pub has_socket: bool,
    pub is_appraised: bool,
}

impl EquipmentItem {
    pub fn from_integer(value: u32) -> Option<EquipmentItem> {
        let item_number: u16 = (value % 1000) as u16;
        let item_type: ItemType = FromPrimitive::from_u32(value / 1000)?;

        if item_type.is_equipment() {
            Some(EquipmentItem {
                item_type,
                item_number,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StackableItem {
    pub item_type: ItemType,
    pub item_number: u16,
    pub quantity: u32,
}

impl StackableItem {
    pub fn from_integer(value: u32, quantity: u32) -> Option<StackableItem> {
        let item_number: u16 = (value % 1000) as u16;
        let item_type: ItemType = FromPrimitive::from_u32(value / 1000)?;

        if item_type.is_stackable() {
            Some(StackableItem {
                item_type,
                item_number,
                quantity,
            })
        } else {
            None
        }
    }

    pub fn try_combine(&mut self, stackable: &StackableItem) -> Result<Option<StackableItem>, ()> {
        if self.item_type != stackable.item_type {
            Err(())
        } else if self.item_number != stackable.item_number {
            Err(())
        } else if self.quantity >= MAX_STACKABLE_ITEM_QUANTITY {
            Err(())
        } else {
            let remaining = MAX_STACKABLE_ITEM_QUANTITY - self.quantity;
            if remaining > stackable.quantity {
                self.quantity += stackable.quantity;
                Ok(None)
            } else {
                self.quantity += remaining;
                Ok(Some(StackableItem {
                    item_type: stackable.item_type,
                    item_number: stackable.item_number,
                    quantity: stackable.quantity - remaining,
                }))
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MoneyItem {
    pub quantity: u32,
}

#[derive(Debug)]
pub enum Item {
    Equipment(EquipmentItem),
    Stackable(StackableItem),
    Money(MoneyItem),
}

impl Item {
    pub fn from_integer(value: u32, quantity: u32) -> Option<Item> {
        let item_type: ItemType = FromPrimitive::from_u32(value / 1000)?;

        if item_type.is_money() {
            Some(Item::Money(MoneyItem { quantity }))
        } else if item_type.is_stackable() {
            match StackableItem::from_integer(value, quantity) {
                Some(stackable) => Some(Item::Stackable(stackable)),
                None => None,
            }
        } else if item_type.is_equipment() {
            match EquipmentItem::from_integer(value) {
                Some(equipment) => Some(Item::Equipment(equipment)),
                None => None,
            }
        } else {
            None
        }
    }
}

// TODO: Probably doesn't belong here, but will do for now.
#[derive(Debug, FromPrimitive)]
pub enum AbilityType {
    Gender = 2,
    Birthstone = 3,
    Class = 4,
    Union = 5,
    Rank = 6,
    Fame = 7,
    Face = 8,
    Hair = 9,

    Strength = 10,
    Dexterity = 11,
    Intelligence = 12,
    Concentration = 13,
    Charm = 14,
    Sense = 15,

    Health = 16,
    Mana = 17,
    Attack = 18,
    Defence = 19,
    Hit = 20,
    Resistence = 21,
    Avoid = 22,
    Speed = 23,
    AttackSpeed = 24,
    Weight = 25,
    Critical = 26,
    RecoverHealth = 27,
    RecoverMana = 28,

    SaveMana = 29,
    Experience = 30,
    Level = 31,
    BonusPoint = 32,
    PvpFlag = 33,
    TeamNumber = 34,
    HeadSize = 35,
    BodySize = 36,
    Skillpoint = 37,
    MaxHealth = 38,
    MaxMana = 39,
    Money = 40,

    PassiveAttackPowerUnarmed = 41,
    PassiveAttackPowerOneHanded = 42,
    PassiveAttackPowerTwoHanded = 43,
    PassiveAttackPowerBow = 44,
    PassiveAttackPowerGun = 45,
    PassiveAttackPowerStaffWand = 46,
    PassiveAttackPowerAutoBow = 47,
    PassiveAttackPowerKatarPair = 48,

    PassiveAttackSpeedBow = 49,
    PassiveAttackSpeedGun = 50,
    PassiveAttackSpeedPair = 51,

    PassiveMoveSpeed = 52,
    PassiveDefence = 53,
    PassiveMaxHealth = 54,
    PassiveMaxMana = 55,
    PassiveRecoverHealth = 56,
    PassiveRecoverMana = 57,
    PassiveWeight = 58,

    PassiveBuySkill = 59,
    PassiveSellSkill = 60,
    PassiveSaveMana = 61,
    PassiveMaxSummons = 62,
    PassiveDropRate = 63,

    Race = 71,
    DropRate = 72,
    FameG = 73,
    FameB = 74,
    CurrentPlanet = 75,
    Stamina = 76,
    Fuel = 77,
    Immunity = 78,

    UnionPoint1 = 81,
    UnionPoint2 = 82,
    UnionPoint3 = 83,
    UnionPoint4 = 84,
    UnionPoint5 = 85,
    UnionPoint6 = 86,
    UnionPoint7 = 87,
    UnionPoint8 = 88,
    UnionPoint9 = 89,
    UnionPoint10 = 90,

    GuildNumber = 91,
    GuildScore = 92,
    GuildPosition = 93,

    BankFree = 94,
    BankAddon = 95,
    StoreSkin = 96,
    VehicleHealth = 97,

    PassiveResistence = 98,
    PassiveHit = 99,
    PassiveCritical = 100,
    PassiveAvoid = 101,
    PassiveShield = 102,
    PassiveImmunity = 103,

    Max = 105,
}
