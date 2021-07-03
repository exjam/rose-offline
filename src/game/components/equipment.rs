use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

use crate::data::{
    item::{EquipmentItem, ItemType, StackableItem},
    ItemDatabase, ItemReference, WeaponItemData,
};

#[allow(dead_code)]
#[derive(Clone, Copy, FromPrimitive)]
pub enum EquipmentIndex {
    Face = 1,
    Head = 2,
    Body = 3,
    Back = 4,
    Hands = 5,
    Feet = 6,
    WeaponRight = 7,
    WeaponLeft = 8,
    Necklace = 9,
    Ring = 10,
    Earring = 11,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum VehiclePartIndex {
    Body = 0,
    Engine = 1,
    Leg = 2,
    Ability = 3,
    Arms = 4,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum AmmoIndex {
    Arrow = 0,
    Bullet = 1,
    Throw = 2,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Equipment {
    pub equipped_items: [Option<EquipmentItem>; EquipmentIndex::Earring as usize + 1],
    pub equipped_vehicle: [Option<EquipmentItem>; VehiclePartIndex::Arms as usize + 1],
    pub equipped_ammo: [Option<StackableItem>; AmmoIndex::Throw as usize + 1],
}

pub trait EquipmentItemReference {
    fn from_equipment(equipment: &Equipment, index: EquipmentIndex) -> Option<ItemReference>;
}

impl EquipmentItemReference for ItemReference {
    fn from_equipment(equipment: &Equipment, index: EquipmentIndex) -> Option<ItemReference> {
        equipment.get_equipment_item(index).map(|item| item.item)
    }
}

pub trait EquipmentItemDatabase {
    fn get_equipped_weapon_item_data(
        &self,
        equipment: &Equipment,
        index: EquipmentIndex,
    ) -> Option<&WeaponItemData>;
}

impl EquipmentItemDatabase for ItemDatabase {
    fn get_equipped_weapon_item_data(
        &self,
        equipment: &Equipment,
        index: EquipmentIndex,
    ) -> Option<&WeaponItemData> {
        self.get_weapon_item(
            equipment
                .get_equipment_item(index)
                .map(|item| item.item.item_number)
                .unwrap_or(0),
        )
    }
}

impl Equipment {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_equipment_item(&self, index: EquipmentIndex) -> Option<&EquipmentItem> {
        self.equipped_items[index as usize].as_ref()
    }

    pub fn get_vehicle_item(&self, index: VehiclePartIndex) -> Option<&EquipmentItem> {
        self.equipped_vehicle[index as usize].as_ref()
    }

    pub fn get_ammo_item(&self, index: AmmoIndex) -> Option<&StackableItem> {
        self.equipped_ammo[index as usize].as_ref()
    }

    pub fn get_equipment_slot_mut(&mut self, index: EquipmentIndex) -> &mut Option<EquipmentItem> {
        &mut self.equipped_items[index as usize]
    }

    pub fn equip_item(
        &mut self,
        item: EquipmentItem,
    ) -> Result<(EquipmentIndex, Option<EquipmentItem>), EquipmentItem> {
        // TODO: Equip ammo, equip vehicles
        let equipment_index = match item.item.item_type {
            ItemType::Face => EquipmentIndex::Face,
            ItemType::Head => EquipmentIndex::Head,
            ItemType::Body => EquipmentIndex::Body,
            ItemType::Hands => EquipmentIndex::Hands,
            ItemType::Feet => EquipmentIndex::Feet,
            ItemType::Back => EquipmentIndex::Back,
            ItemType::Jewellery => {
                // TODO: Lookup in STB which type of jewellery this is
                return Err(item);
            }
            // TODO: Support dual wielding of weapons
            ItemType::Weapon => EquipmentIndex::WeaponRight,
            ItemType::SubWeapon => EquipmentIndex::WeaponLeft,
            _ => return Err(item),
        };

        let previous = self.equipped_items[equipment_index as usize].take();
        self.equipped_items[equipment_index as usize] = Some(item);
        Ok((equipment_index, previous))
    }

    pub fn equip_items(
        &mut self,
        items: Vec<EquipmentItem>,
    ) -> (Vec<EquipmentIndex>, Vec<EquipmentItem>) {
        let mut updated_slots: Vec<EquipmentIndex> = Vec::new();
        let mut remaining_items: Vec<EquipmentItem> = Vec::new();

        for item in items {
            match self.equip_item(item) {
                Ok((slot, remaining)) => {
                    updated_slots.push(slot);
                    if let Some(remaining) = remaining {
                        remaining_items.push(remaining);
                    }
                }
                Err(item) => {
                    remaining_items.push(item);
                }
            }
        }

        (updated_slots, remaining_items)
    }
}
