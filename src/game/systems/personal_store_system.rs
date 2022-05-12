use bevy::{
    ecs::{
        prelude::{EventReader, Query},
        query::WorldQuery,
    },
    prelude::Mut,
};

use rose_data::{Item, ItemSlotBehaviour};
use rose_game_common::{
    components::{ItemSlot, Money},
    messages::server::PersonalStoreTransactionCancelled,
};

use crate::game::{
    components::{ClientEntity, GameClient, Inventory, PersonalStore},
    events::{PersonalStoreEvent, PersonalStoreEventBuyItem, PersonalStoreEventListItems},
    messages::server::{
        PersonalStoreItemList, PersonalStoreTransactionResult, PersonalStoreTransactionSoldOut,
        PersonalStoreTransactionSuccess, ServerMessage,
    },
};

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct PersonalStoreEntityQuery<'w> {
    client_entity: &'w ClientEntity,
    inventory: &'w mut Inventory,
    game_client: Option<&'w GameClient>,
}

fn personal_store_list_items(
    store: &PersonalStore,
    seller: &PersonalStoreEntityQueryReadOnlyItem,
    buyer: &PersonalStoreEntityQueryReadOnlyItem,
) {
    let mut buy_items = Vec::new();
    let mut sell_items = Vec::new();

    for (store_slot, slot) in store.buy_items.iter().enumerate() {
        if let Some((item, price)) = slot {
            buy_items.push((store_slot as u8, item.clone(), *price));
        }
    }

    for (store_slot, slot) in store.sell_items.iter().enumerate() {
        if let Some((item_slot, price)) = slot {
            if let Some(item) = seller.inventory.get_item(*item_slot) {
                sell_items.push((store_slot as u8, item.clone(), *price));
            }
        }
    }

    if let Some(game_client) = buyer.game_client {
        game_client
            .server_message_tx
            .send(ServerMessage::PersonalStoreItemList(
                PersonalStoreItemList {
                    sell_items,
                    buy_items,
                },
            ))
            .ok();
    }
}

enum BuyError {
    InvalidStoreSlotIndex,
    ItemSoldOut,
    NotEnoughMoney,
    InventoryFull,
}

fn personal_store_buy_item(
    store: &mut Mut<PersonalStore>,
    seller: &mut PersonalStoreEntityQueryItem,
    buyer: &mut PersonalStoreEntityQueryItem,
    store_slot_index: usize,
    buy_item: &Item,
) -> Result<(ItemSlot, ItemSlot), BuyError> {
    // Try get the item from the personal store
    let (store_item_slot, item_price) = store
        .sell_items
        .get(store_slot_index)
        .and_then(|x| x.as_ref())
        .ok_or(BuyError::InvalidStoreSlotIndex)
        .cloned()?;

    let store_inventory_slot = seller
        .inventory
        .get_item_slot_mut(store_item_slot)
        .ok_or(BuyError::InvalidStoreSlotIndex)?;
    if !store_inventory_slot.contains_same_item(buy_item) {
        return Err(BuyError::ItemSoldOut);
    }

    let item_price = Money(item_price.0 * buy_item.get_quantity() as i64);
    if buyer.inventory.money < item_price {
        return Err(BuyError::NotEnoughMoney);
    }

    let transaction_item = store_inventory_slot.try_take_quantity(buy_item.get_quantity());
    if transaction_item.is_none() {
        return Err(BuyError::ItemSoldOut);
    }

    let transaction_item = transaction_item.unwrap();
    let transaction_money = buyer.inventory.try_take_money(item_price).unwrap();

    match buyer.inventory.try_add_item(transaction_item) {
        Ok((buyer_item_slot, _)) => {
            // Success, give money to seller
            if store_inventory_slot.is_none() {
                *store.sell_items.get_mut(store_slot_index).unwrap() = None;
            }

            seller.inventory.try_add_money(transaction_money).ok();

            Ok((buyer_item_slot, store_item_slot))
        }
        Err(rejected_item) => {
            // Failed, rollback by returning item to seller and money to buyer
            store_inventory_slot
                .try_stack_with_item(rejected_item)
                .expect("Unexpected failure rolling back personal store transaction");

            buyer
                .inventory
                .try_add_money(transaction_money)
                .expect("Unexpected failure rolling back personal store transaction");

            Err(BuyError::InventoryFull)
        }
    }
}

pub fn personal_store_system(
    mut entity_query: Query<PersonalStoreEntityQuery>,
    mut store_query: Query<&mut PersonalStore>,
    mut personal_store_events: EventReader<PersonalStoreEvent>,
) {
    for event in personal_store_events.iter() {
        match *event {
            PersonalStoreEvent::ListItems(PersonalStoreEventListItems {
                store_entity,
                list_entity,
            }) => {
                if let Ok([seller, buyer]) = entity_query.get_many([store_entity, list_entity]) {
                    if let Ok(store) = store_query.get(store_entity) {
                        personal_store_list_items(store, &seller, &buyer);
                    }
                }
            }
            PersonalStoreEvent::BuyItem(PersonalStoreEventBuyItem {
                store_entity,
                buyer_entity,
                store_slot_index,
                ref buy_item,
            }) => {
                if let Ok([mut seller, mut buyer]) =
                    entity_query.get_many_mut([store_entity, buyer_entity])
                {
                    if let Ok(mut store) = store_query.get_mut(store_entity) {
                        match personal_store_buy_item(
                            &mut store,
                            &mut seller,
                            &mut buyer,
                            store_slot_index,
                            buy_item,
                        ) {
                            Ok((buyer_item_slot, seller_item_slot)) => {
                                if let Some(seller_game_client) = seller.game_client {
                                    seller_game_client
                                        .server_message_tx
                                        .send(ServerMessage::PersonalStoreTransactionResult(
                                            PersonalStoreTransactionResult::BoughtFromStore(
                                                PersonalStoreTransactionSuccess {
                                                    store_entity_id: seller.client_entity.id,
                                                    money: seller.inventory.money,
                                                    store_slot_index,
                                                    store_slot_item: seller
                                                        .inventory
                                                        .get_item(seller_item_slot)
                                                        .cloned(),
                                                    inventory_slot: seller_item_slot,
                                                    inventory_item: seller
                                                        .inventory
                                                        .get_item(seller_item_slot)
                                                        .cloned(),
                                                },
                                            ),
                                        ))
                                        .ok();
                                }

                                if let Some(buyer_game_client) = buyer.game_client {
                                    buyer_game_client
                                        .server_message_tx
                                        .send(ServerMessage::PersonalStoreTransactionResult(
                                            PersonalStoreTransactionResult::BoughtFromStore(
                                                PersonalStoreTransactionSuccess {
                                                    store_entity_id: seller.client_entity.id,
                                                    money: buyer.inventory.money,
                                                    store_slot_index,
                                                    store_slot_item: seller
                                                        .inventory
                                                        .get_item(seller_item_slot)
                                                        .cloned(),
                                                    inventory_slot: buyer_item_slot,
                                                    inventory_item: buyer
                                                        .inventory
                                                        .get_item(buyer_item_slot)
                                                        .cloned(),
                                                },
                                            ),
                                        ))
                                        .ok();
                                }
                            }
                            Err(BuyError::ItemSoldOut) => {
                                if let Some(buyer_game_client) = buyer.game_client {
                                    buyer_game_client
                                        .server_message_tx
                                        .send(ServerMessage::PersonalStoreTransactionResult(
                                            PersonalStoreTransactionResult::SoldOut(
                                                PersonalStoreTransactionSoldOut {
                                                    store_entity_id: seller.client_entity.id,
                                                    store_slot_index,
                                                    item: None,
                                                },
                                            ),
                                        ))
                                        .ok();
                                }
                            }
                            Err(BuyError::InvalidStoreSlotIndex)
                            | Err(BuyError::InventoryFull)
                            | Err(BuyError::NotEnoughMoney) => {
                                if let Some(buyer_game_client) = buyer.game_client {
                                    buyer_game_client
                                        .server_message_tx
                                        .send(ServerMessage::PersonalStoreTransactionResult(
                                            PersonalStoreTransactionResult::Cancelled(
                                                PersonalStoreTransactionCancelled {
                                                    store_entity_id: seller.client_entity.id,
                                                },
                                            ),
                                        ))
                                        .ok();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
