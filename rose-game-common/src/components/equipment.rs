use bevy_ecs::prelude::Component;
use enum_map::EnumMap;
use serde::{Deserialize, Serialize};

use rose_data::{
    AmmoIndex, EquipmentIndex, EquipmentItem, ItemDatabase, ItemReference, ItemType, StackableItem,
    VehiclePartIndex, WeaponItemData,
};

#[derive(Component, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Equipment {
    pub equipped_items: EnumMap<EquipmentIndex, Option<EquipmentItem>>,
    pub equipped_vehicle: EnumMap<VehiclePartIndex, Option<EquipmentItem>>,
    pub equipped_ammo: EnumMap<AmmoIndex, Option<StackableItem>>,
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
        self.equipped_items[index].as_ref()
    }

    pub fn get_vehicle_item(&self, index: VehiclePartIndex) -> Option<&EquipmentItem> {
        self.equipped_vehicle[index].as_ref()
    }

    pub fn get_ammo_item(&self, index: AmmoIndex) -> Option<&StackableItem> {
        self.equipped_ammo[index].as_ref()
    }

    pub fn get_ammo_slot_mut(&mut self, index: AmmoIndex) -> &mut Option<StackableItem> {
        &mut self.equipped_ammo[index]
    }

    pub fn get_equipment_slot_mut(&mut self, index: EquipmentIndex) -> &mut Option<EquipmentItem> {
        &mut self.equipped_items[index]
    }

    pub fn get_vehicle_slot_mut(&mut self, index: VehiclePartIndex) -> &mut Option<EquipmentItem> {
        &mut self.equipped_vehicle[index]
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

        let previous = self.equipped_items[equipment_index].take();
        self.equipped_items[equipment_index] = Some(item);
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

    pub fn iter_equipped_items(&self) -> impl Iterator<Item = &EquipmentItem> {
        self.equipped_items
            .iter()
            .filter_map(|(_, slot)| slot.as_ref())
    }

    pub fn iter_equipped_vehicles(&self) -> impl Iterator<Item = &EquipmentItem> {
        self.equipped_vehicle
            .iter()
            .filter_map(|(_, slot)| slot.as_ref())
    }

    pub fn iter_equipped_ammo(&self) -> impl Iterator<Item = &StackableItem> {
        self.equipped_ammo
            .iter()
            .filter_map(|(_, slot)| slot.as_ref())
    }
}
