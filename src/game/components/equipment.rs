use crate::game::data::items::*;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Clone, Copy)]
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
    Max,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum VehiclePartIndex {
    Body = 0,
    Engine = 1,
    Leg = 2,
    Ability = 3,
    Arms = 4,
    Max,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum AmmoType {
    Arrow = 0,
    Bullet = 1,
    Throw = 2,
    Max,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Equipment {
    pub equipped_items: [Option<EquipmentItem>; EquipmentIndex::Max as usize],
    pub equipped_vehicle: [Option<EquipmentItem>; VehiclePartIndex::Max as usize],
    pub equipped_ammo: [Option<StackableItem>; AmmoType::Max as usize],
}

impl Equipment {
    pub fn equip_item(
        &mut self,
        item: EquipmentItem,
    ) -> Result<(EquipmentIndex, Option<EquipmentItem>), EquipmentItem> {
        // TODO: Equip ammo, equip vehicles
        let equipment_index = match item.item_type {
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
                    if remaining.is_some() {
                        remaining_items.push(remaining.unwrap());
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
