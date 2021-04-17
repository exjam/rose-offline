use crate::game::data::formats::StbFile;
use crate::game::data::items::{AbilityType, ItemClass};
use num_traits::FromPrimitive;
use std::ops::Deref;

pub struct StbItem(pub StbFile);

impl Deref for StbItem {
    type Target = StbFile;
    fn deref(&self) -> &StbFile {
        &self.0
    }
}

#[allow(dead_code)]
impl StbItem {
    pub fn get_item_class(&self, item_number: u16) -> Option<ItemClass> {
        self.0
            .try_get_int(item_number as usize, 4)
            .and_then(|x| FromPrimitive::from_i32(x))
    }

    pub fn get_item_base_price(&self, item_number: u16) -> Option<i32> {
        self.0.try_get_int(item_number as usize, 5)
    }

    pub fn get_item_price_rate(&self, item_number: u16) -> Option<i32> {
        self.0.try_get_int(item_number as usize, 6)
    }

    pub fn get_item_weight(&self, item_number: u16) -> Option<i32> {
        self.0.try_get_int(item_number as usize, 7)
    }

    pub fn get_item_quality(&self, item_number: u16) -> Option<i32> {
        self.0.try_get_int(item_number as usize, 8)
    }

    pub fn get_item_icon_number(&self, item_number: u16) -> Option<i32> {
        self.0.try_get_int(item_number as usize, 9)
    }

    pub fn get_item_field_model(&self, item_number: u16) -> Option<&str> {
        self.0.try_get(item_number as usize, 10)
    }

    pub fn get_item_equip_sound(&self, item_number: u16) -> Option<i32> {
        self.0.try_get_int(item_number as usize, 11)
    }

    pub fn get_item_craft_skill_type(&self, item_number: u16) -> Option<i32> {
        self.0.try_get_int(item_number as usize, 12)
    }

    pub fn get_item_craft_skill_level(&self, item_number: u16) -> Option<i32> {
        self.0.try_get_int(item_number as usize, 13)
    }

    pub fn get_item_craft_material(&self, item_number: u16) -> Option<i32> {
        self.0.try_get_int(item_number as usize, 14)
    }

    pub fn get_item_craft_difficulty(&self, item_number: u16) -> Option<i32> {
        self.0.try_get_int(item_number as usize, 15)
    }

    pub fn get_item_equip_class_requirement(&self, item_number: u16) -> Option<i32> {
        self.0.try_get_int(item_number as usize, 16)
    }

    pub fn get_item_equip_union_requirement(&self, item_number: u16) -> Vec<i32> {
        let mut requirements = Vec::new();
        for i in 0..2 {
            if let Some(union) = self.0.try_get_int(item_number as usize, 17 + i) {
                if union != 0 {
                    requirements.push(union);
                }
            }
        }
        requirements
    }

    pub fn get_item_ability_requirement(&self, item_number: u16) -> Vec<(AbilityType, i32)> {
        let mut requirements = Vec::new();
        for i in 0..2 {
            let ability_type: Option<AbilityType> = self
                .0
                .try_get_int(item_number as usize, 19 + i * 2)
                .and_then(|x| FromPrimitive::from_i32(x));
            let ability_value = self.0.try_get_int(item_number as usize, 20 + i * 2);

            ability_type.map(|ability_type| {
                ability_value.map(|ability_value| requirements.push((ability_type, ability_value)))
            });
        }
        requirements
    }

    pub fn get_item_union_requirement(&self, item_number: u16) -> Vec<i32> {
        let mut requirements = Vec::new();
        for i in 0..2 {
            if let Some(union) = self.0.try_get_int(item_number as usize, 23 + i * 3) {
                if union != 0 {
                    requirements.push(union);
                }
            }
        }
        requirements
    }

    pub fn get_item_add_ability(&self, item_number: u16) -> Vec<(AbilityType, i32)> {
        let mut requirements = Vec::new();
        for i in 0..2 {
            let ability_type: Option<AbilityType> = self
                .0
                .try_get_int(item_number as usize, 24 + i * 3)
                .and_then(|x| FromPrimitive::from_i32(x));
            let ability_value = self.0.try_get_int(item_number as usize, 25 + i * 3);

            ability_type.map(|ability_type| {
                ability_value.map(|ability_value| requirements.push((ability_type, ability_value)))
            });
        }
        requirements
    }

    pub fn get_item_durability(&self, item_number: u16) -> Option<i32> {
        self.0.try_get_int(item_number as usize, 29)
    }

    pub fn get_item_rare_type(&self, item_number: u16) -> Option<i32> {
        self.0.try_get_int(item_number as usize, 30)
    }

    pub fn get_item_defence(&self, item_number: u16) -> Option<i32> {
        self.0.try_get_int(item_number as usize, 31)
    }

    pub fn get_item_resistence(&self, item_number: u16) -> Option<i32> {
        self.0.try_get_int(item_number as usize, 32)
    }
}

pub struct StbItemBack(pub StbItem);

impl Deref for StbItemBack {
    type Target = StbItem;
    fn deref(&self) -> &StbItem {
        &self.0
    }
}

impl StbItemBack {
    pub fn get_back_move_speed(&self, item_number: u16) -> Option<i32> {
        self.0 .0.try_get_int(item_number as usize, 33)
    }
}

pub struct StbItemFoot(pub StbItem);

impl Deref for StbItemFoot {
    type Target = StbItem;
    fn deref(&self) -> &StbItem {
        &self.0
    }
}

impl StbItemFoot {
    pub fn get_boots_move_speed(&self, item_number: u16) -> Option<i32> {
        self.0 .0.try_get_int(item_number as usize, 33)
    }
}
