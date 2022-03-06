use crate::data::{
    item::{ItemClass, ItemType},
    AbilityType, ItemReference,
};

pub trait DataDecoder {
    fn decode_item_base1000(&self, id: usize) -> Option<ItemReference>;
    fn decode_ability_type(&self, id: usize) -> Option<AbilityType>;
    fn decode_item_type(&self, id: usize) -> Option<ItemType>;
    fn decode_item_class(&self, id: usize) -> Option<ItemClass>;
}
