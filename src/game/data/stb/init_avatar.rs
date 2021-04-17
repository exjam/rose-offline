use crate::game::components::BasicStats;
use crate::game::data::formats::StbFile;
use crate::game::data::items::{EquipmentItem, Item};

pub struct StbInitAvatar(pub StbFile);

impl StbInitAvatar {
    pub fn get_basic_stats(&self, row: usize) -> Option<BasicStats> {
        Some(BasicStats {
            strength: self.0.try_get_int(row, 0)? as u16,
            dexterity: self.0.try_get_int(row, 1)? as u16,
            intelligence: self.0.try_get_int(row, 2)? as u16,
            concentration: self.0.try_get_int(row, 3)? as u16,
            charm: self.0.try_get_int(row, 4)? as u16,
            sense: self.0.try_get_int(row, 5)? as u16,
        })
    }

    pub fn get_equipment(&self, row: usize) -> Vec<EquipmentItem> {
        let mut items = Vec::new();
        for i in 6..=13 {
            let item = self.0.get(row, i).parse().unwrap_or(0);
            if let Some(item) = EquipmentItem::from_integer(item) {
                items.push(item);
            }
        }
        items
    }

    pub fn get_inventory_equipment(&self, row: usize) -> Vec<Item> {
        let mut items = Vec::new();
        for i in 0..10 {
            if let Some(item) = Item::from_integer(self.0.get(row, 14 + i).parse().unwrap_or(0), 1)
            {
                items.push(item);
            }
        }
        items
    }

    pub fn get_inventory_consumables(&self, row: usize) -> Vec<Item> {
        let mut items = Vec::new();
        for i in 0..5 {
            let item = self.0.get(row, 24 + i * 2).parse().unwrap_or(0);
            let quantity = self.0.get(row, 25 + i * 2).parse().unwrap_or(0);
            if let Some(item) = Item::from_integer(item, quantity) {
                items.push(item);
            }
        }
        items
    }

    pub fn get_inventory_materials(&self, row: usize) -> Vec<Item> {
        let mut items = Vec::new();
        for i in 0..5 {
            let item = self.0.get(row, 34 + i * 2).parse().unwrap_or(0);
            let quantity = self.0.get(row, 35 + i * 2).parse().unwrap_or(0);
            if let Some(item) = Item::from_integer(item, quantity) {
                items.push(item);
            }
        }
        items
    }
}
