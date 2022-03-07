use bevy_ecs::prelude::{EventReader, Query};

use crate::{
    data::ItemSlotBehaviour,
    game::{
        components::{ClientEntity, ClientEntityId, GameClient, Inventory, Money, PersonalStore},
        events::{PersonalStoreEvent, PersonalStoreEventBuyItem, PersonalStoreEventListItems},
        messages::server::{
            PersonalStoreItemList, PersonalStoreTransactionCancelled,
            PersonalStoreTransactionResult, PersonalStoreTransactionSoldOut,
            PersonalStoreTransactionSuccess, ServerMessage,
        },
    },
};

pub fn personal_store_system(
    mut entity_query: Query<(
        &mut Inventory,
        Option<&mut PersonalStore>,
        &ClientEntity,
        Option<&GameClient>,
    )>,
    mut personal_store_events: EventReader<PersonalStoreEvent>,
) {
    for event in personal_store_events.iter() {
        match *event {
            PersonalStoreEvent::ListItems(PersonalStoreEventListItems {
                store_entity,
                list_entity,
            }) => {
                let mut buy_items = Vec::new();
                let mut sell_items = Vec::new();

                if let Ok((store_inventory, Some(personal_store), _, _)) =
                    entity_query.get_mut(store_entity)
                {
                    for (store_slot, slot) in personal_store.buy_items.iter().enumerate() {
                        if let Some((item, price)) = slot {
                            buy_items.push((store_slot as u8, item.clone(), *price));
                        }
                    }

                    for (store_slot, slot) in personal_store.sell_items.iter().enumerate() {
                        if let Some((item_slot, price)) = slot {
                            if let Some(item) = store_inventory.get_item(*item_slot) {
                                sell_items.push((store_slot as u8, item.clone(), *price));
                            }
                        }
                    }
                }

                if let Ok((_, _, _, Some(game_client))) = entity_query.get_mut(list_entity) {
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
            PersonalStoreEvent::BuyItem(PersonalStoreEventBuyItem {
                store_entity,
                buyer_entity,
                store_slot_index,
                ref buy_item,
            }) => {
                let buy_item = buy_item.clone();
                let mut transaction_item = None;
                let mut transaction_money = None;
                let mut store_inventory_slot = None;
                let mut item_price = Money(0);
                let mut store_client_entity_id = ClientEntityId(0);
                let mut store_slot_remaining_item = None;

                // Try get the item from the personal store
                if let Ok((mut store_inventory, Some(personal_store), store_client_entity, _)) =
                    entity_query.get_mut(store_entity)
                {
                    store_client_entity_id = store_client_entity.id;

                    if let Some(Some((inventory_slot, price))) =
                        personal_store.sell_items.get(store_slot_index as usize)
                    {
                        if let Some(item_slot) = store_inventory.get_item_slot_mut(*inventory_slot)
                        {
                            if item_slot.contains_same_item(&buy_item) {
                                transaction_item =
                                    item_slot.try_take_quantity(buy_item.get_quantity());
                                store_slot_remaining_item = item_slot.clone();
                                store_inventory_slot = Some(*inventory_slot);
                                item_price = *price;
                            }
                        }
                    }
                }

                if transaction_item.is_some() {
                    // Try take the buyer's money and give them the item
                    if let Ok((mut buyer_inventory, _, _, buyer_game_client)) =
                        entity_query.get_mut(buyer_entity)
                    {
                        if let Ok(money) = buyer_inventory.try_take_money(item_price) {
                            let buyer_remaining_money = buyer_inventory.money;
                            match buyer_inventory.try_add_item(transaction_item.take().unwrap()) {
                                Ok((buyer_item_slot, buyer_item)) => {
                                    transaction_money = Some(money);

                                    if let Some(buyer_game_client) = buyer_game_client {
                                        buyer_game_client
                                            .server_message_tx
                                            .send(ServerMessage::PersonalStoreTransactionResult(
                                                PersonalStoreTransactionResult::BoughtFromStore(
                                                    PersonalStoreTransactionSuccess {
                                                        store_entity_id: store_client_entity_id,
                                                        money: buyer_remaining_money,
                                                        store_slot_index,
                                                        store_slot_item: store_slot_remaining_item
                                                            .clone(),
                                                        inventory_slot: buyer_item_slot,
                                                        inventory_item: Some(buyer_item.clone()),
                                                    },
                                                ),
                                            ))
                                            .ok();
                                    }
                                }
                                Err(rejected_item) => {
                                    transaction_item = Some(rejected_item);
                                    buyer_inventory.try_add_money(money).expect(
                                        "Unexpected failure undoing personal store transaction",
                                    );

                                    if let Some(buyer_game_client) = buyer_game_client {
                                        buyer_game_client
                                            .server_message_tx
                                            .send(ServerMessage::PersonalStoreTransactionResult(
                                                PersonalStoreTransactionResult::Cancelled(
                                                    PersonalStoreTransactionCancelled {
                                                        store_entity_id: store_client_entity_id,
                                                    },
                                                ),
                                            ))
                                            .ok();
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Item sold out before we could buy it!
                    if let Ok((_, _, _, Some(buyer_game_client))) =
                        entity_query.get_mut(buyer_entity)
                    {
                        buyer_game_client
                            .server_message_tx
                            .send(ServerMessage::PersonalStoreTransactionResult(
                                PersonalStoreTransactionResult::SoldOut(
                                    PersonalStoreTransactionSoldOut {
                                        store_entity_id: store_client_entity_id,
                                        store_slot_index,
                                        item: None,
                                    },
                                ),
                            ))
                            .ok();
                    }
                }

                // If was a success, give money to seller, else return item to seller
                if transaction_item.is_some() || transaction_money.is_some() {
                    if let Ok((mut store_inventory, _, _, store_game_client)) =
                        entity_query.get_mut(store_entity)
                    {
                        let store_inventory_slot = store_inventory_slot.unwrap();
                        if transaction_item.is_some() {
                            // Failed, return item to store inventory
                            if let Some(item_slot) =
                                store_inventory.get_item_slot_mut(store_inventory_slot)
                            {
                                item_slot
                                    .try_stack_with_item(transaction_item.take().unwrap())
                                    .expect(
                                        "Unexpected failure undoing personal store transaction",
                                    );
                            }
                        } else {
                            // Success, give money to store inventory
                            store_inventory
                                .try_add_money(transaction_money.take().unwrap())
                                .ok();

                            // Send packet to store with the result
                            if let Some(store_game_client) = store_game_client {
                                store_game_client
                                    .server_message_tx
                                    .send(ServerMessage::PersonalStoreTransactionResult(
                                        PersonalStoreTransactionResult::BoughtFromStore(
                                            PersonalStoreTransactionSuccess {
                                                store_entity_id: store_client_entity_id,
                                                money: store_inventory.money,
                                                store_slot_index,
                                                store_slot_item: store_slot_remaining_item,
                                                inventory_slot: store_inventory_slot,
                                                inventory_item: store_inventory
                                                    .get_item(store_inventory_slot)
                                                    .cloned(),
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
