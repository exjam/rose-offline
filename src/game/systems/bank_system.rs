use bevy::prelude::{EventReader, Query};

use rose_data::ItemSlotBehaviour;
use rose_game_common::messages::server::ServerMessage;

use crate::game::{
    components::{Bank, GameClient, Inventory},
    events::BankEvent,
};

pub fn bank_system(
    mut bank_events: EventReader<BankEvent>,
    mut query_entity: Query<(&GameClient, &mut Bank, &mut Inventory)>,
) {
    for event in bank_events.iter() {
        match *event {
            BankEvent::Open { entity } => {
                let (game_client, mut bank) =
                    if let Ok((game_client, bank, _)) = query_entity.get_mut(entity) {
                        (game_client, bank)
                    } else {
                        continue;
                    };

                if bank.sent_to_client {
                    game_client
                        .server_message_tx
                        .send(ServerMessage::BankOpen)
                        .ok();
                    continue;
                } else {
                    game_client
                        .server_message_tx
                        .send(ServerMessage::BankSetItems {
                            items: bank
                                .slots
                                .iter()
                                .enumerate()
                                .filter(|(_, item)| item.is_some())
                                .map(|(i, item)| (i as u8, item.clone()))
                                .collect(),
                        })
                        .ok();
                    bank.sent_to_client = true;
                }
            }
            BankEvent::DepositItem {
                entity,
                item_slot,
                ref item,
                .. // TODO: is_premium,
            } => {
                let (game_client, mut bank, mut inventory) =
                    if let Ok((game_client, bank, inventory)) = query_entity.get_mut(entity) {
                        (game_client, bank, inventory)
                    } else {
                        continue;
                    };

                if inventory.get_item(item_slot).map_or(false, |inventory_item| inventory_item.is_same_item(item)) {
                    if let Some(inventory_slot) = inventory.get_item_slot_mut(item_slot) {
                        if let Some(deposit_item) =
                            inventory_slot.try_take_quantity(item.get_quantity())
                        {
                            match bank.try_add_item(deposit_item) {
                                Ok((bank_slot, bank_item)) => {
                                    game_client
                                        .server_message_tx
                                        .send(ServerMessage::BankTransaction {
                                            inventory_item_slot: item_slot,
                                            inventory_item: inventory
                                                .get_item(item_slot)
                                                .cloned(),
                                            inventory_money: Some(inventory.money),
                                            bank_slot,
                                            bank_item: Some(bank_item.clone()),
                                        })
                                        .ok();
                                }
                                Err(deposit_item) => {
                                    inventory_slot
                                        .try_stack_with_item(deposit_item)
                                        .expect("bad things happened");
                                }
                            }
                        }
                    }
                }
            }
            BankEvent::WithdrawItem {
                entity,
                bank_slot: bank_slot_index,
                ref item,
                .. // TODO: is_premium,
            } => {
                let (game_client, mut bank, mut inventory) =
                    if let Ok((game_client, bank, inventory)) = query_entity.get_mut(entity) {
                        (game_client, bank, inventory)
                    } else {
                        continue;
                    };

                if bank.slots.get(bank_slot_index).and_then(|slot| slot.as_ref()).map_or(false, |bank_item| bank_item.is_same_item(item)) {
                    if let Some(bank_slot) = bank.slots.get_mut(bank_slot_index) {
                        if let Some(withdraw_item) = bank_slot.try_take_quantity(item.get_quantity()) {
                            match inventory.try_add_item(withdraw_item) {
                                Ok((inventory_item_slot, inventory_item)) => {
                                    game_client
                                        .server_message_tx
                                        .send(ServerMessage::BankTransaction {
                                            inventory_item_slot,
                                            inventory_item: Some(inventory_item.clone()),
                                            inventory_money: Some(inventory.money),
                                            bank_slot: bank_slot_index,
                                            bank_item: bank.slots.get(bank_slot_index).unwrap().clone(),
                                        })
                                        .ok();
                                },
                                Err(withdraw_item) => {
                                    bank_slot.try_stack_with_item(withdraw_item)
                                    .expect("bad things happened");
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}
