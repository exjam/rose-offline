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
