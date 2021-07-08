use crate::data::item::Item;

use super::Money;

#[derive(Clone)]
pub enum DroppedItem {
    Item(Item),
    Money(Money),
}
