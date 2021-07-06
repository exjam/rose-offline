use crate::data::item::Item;

#[derive(Clone)]
pub enum DroppedItem {
    Item(Item),
    Money(usize),
}
