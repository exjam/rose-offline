use crate::{data::item::Item, game::components::Money};

#[derive(Clone)]
pub enum DroppedItem {
    Item(Item),
    Money(Money),
}
