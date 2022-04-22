use bevy::ecs::prelude::{Entity, EventReader, Mut, Query, Res};
use bevy::math::Vec3Swizzles;
use log::warn;
use std::collections::HashSet;

use rose_data::Item;

use crate::game::{
    components::{
        AbilityValues, GameClient, Inventory, ItemSlot, Money, Npc, Position, UnionMembership,
    },
    events::NpcStoreEvent,
    messages::{
        client::NpcStoreBuyItem,
        server::{NpcStoreTransactionError, ServerMessage},
    },
    resources::WorldRates,
    GameData,
};

pub const NPC_STORE_TRANSACTION_MAX_DISTANCE: f32 = 6000.0;

fn npc_store_do_transaction(
    npc_query: &Query<(&Npc, &Position)>,
    game_data: &GameData,
    world_rates: &WorldRates,
    store_entity: Entity,
    buy_items: &[NpcStoreBuyItem],
    sell_items: &[(ItemSlot, usize)],
    ability_values: &AbilityValues,
    inventory: &mut Mut<Inventory>,
    position: &Position,
    _union_membership: &UnionMembership,
) -> Result<HashSet<ItemSlot>, NpcStoreTransactionError> {
    let (npc, npc_position) = npc_query
        .get(store_entity)
        .map_err(|_| NpcStoreTransactionError::NpcNotFound)?;

    let npc_data = game_data
        .npcs
        .get_npc(npc.id)
        .ok_or(NpcStoreTransactionError::NpcNotFound)?;

    if npc_data.store_union_number.is_some() {
        warn!("Unimplemented union NPC store");
        // TODO: if npc_data.store_union_number != union_membership.current_union { ... etc
        return Err(NpcStoreTransactionError::NotSameUnion);
    }

    if npc_position.zone_id != position.zone_id
        || position.position.xy().distance(npc_position.position.xy())
            > NPC_STORE_TRANSACTION_MAX_DISTANCE
    {
        return Err(NpcStoreTransactionError::NpcTooFarAway);
    }

    let mut total_buy_cost = 0i64;
    let mut total_sell_value = 0i64;
    let mut transaction_inventory = inventory.clone();
    let mut updated_inventory_slots = HashSet::new();

    // First process sell items
    for &(sell_item_slot, sell_item_quantity) in sell_items {
        let sell_item_quantity = usize::min(
            sell_item_quantity,
            transaction_inventory
                .get_item(sell_item_slot)
                .map(|item| item.get_quantity() as usize)
                .unwrap_or(0),
        );

        let sell_item = transaction_inventory
            .try_take_quantity(sell_item_slot, sell_item_quantity as u32)
            .ok_or(NpcStoreTransactionError::NpcNotFound)?;

        let item_price = game_data
            .ability_value_calculator
            .calculate_npc_store_item_sell_price(
                &game_data.items,
                &sell_item,
                ability_values.get_npc_store_sell_rate(),
                world_rates.world_price_rate,
                world_rates.item_price_rate,
                world_rates.town_price_rate,
            )
            .ok_or(NpcStoreTransactionError::NpcNotFound)? as i64;

        log::trace!(target: "npc_store", "Sell item {:?}, price: {}", sell_item.get_item_reference(), item_price);
        updated_inventory_slots.insert(sell_item_slot);
        total_sell_value += item_price * sell_item.get_quantity() as i64;
    }

    // Process buy items
    for buy_item in buy_items {
        let store_tab_id = npc_data
            .store_tabs
            .get(buy_item.tab_index)
            .and_then(|x| *x)
            .ok_or(NpcStoreTransactionError::NpcNotFound)?;

        let store_tab_data = game_data
            .npcs
            .get_store_tab(store_tab_id)
            .ok_or(NpcStoreTransactionError::NpcNotFound)?;

        let store_item_reference = *store_tab_data
            .items
            .get(&(buy_item.item_index as u16))
            .ok_or(NpcStoreTransactionError::NpcNotFound)?;

        let store_item_data = game_data
            .items
            .get_base_item(store_item_reference)
            .ok_or(NpcStoreTransactionError::NpcNotFound)?;

        let item_price = game_data
            .ability_value_calculator
            .calculate_npc_store_item_buy_price(
                &game_data.items,
                store_item_reference,
                ability_values.get_npc_store_buy_rate(),
                world_rates.item_price_rate,
                world_rates.town_price_rate,
            )
            .ok_or(NpcStoreTransactionError::NpcNotFound)? as i64;

        let buy_quantity = if store_item_reference.item_type.is_stackable_item() {
            buy_item.quantity
        } else {
            1
        } as i64;

        let item = Item::from_item_data(store_item_data, buy_quantity as u32)
            .ok_or(NpcStoreTransactionError::NpcNotFound)?;

        let (inventory_slot, _) = transaction_inventory
            .try_add_item(item)
            .map_err(|_| NpcStoreTransactionError::NpcNotFound)?;

        log::trace!(target: "npc_store", "Buy item {:?}, price: {}", store_item_reference, item_price);
        updated_inventory_slots.insert(inventory_slot);
        total_buy_cost += item_price * buy_quantity;
    }

    transaction_inventory
        .try_add_money(Money(total_sell_value))
        .map_err(|_| NpcStoreTransactionError::NotEnoughMoney)?;

    transaction_inventory
        .try_take_money(Money(total_buy_cost))
        .map_err(|_| NpcStoreTransactionError::NotEnoughMoney)?;

    **inventory = transaction_inventory;
    Ok(updated_inventory_slots)
}

pub fn npc_store_system(
    npc_query: Query<(&Npc, &Position)>,
    mut transaction_entity_query: Query<(
        &AbilityValues,
        &mut Inventory,
        &Position,
        &UnionMembership,
        Option<&GameClient>,
    )>,
    mut npc_store_events: EventReader<NpcStoreEvent>,
    game_data: Res<GameData>,
    world_rates: Res<WorldRates>,
) {
    for event in npc_store_events.iter() {
        if let Ok((ability_values, mut inventory, position, union_membership, game_client)) =
            transaction_entity_query.get_mut(event.transaction_entity)
        {
            match npc_store_do_transaction(
                &npc_query,
                &game_data,
                &world_rates,
                event.store_entity,
                &event.buy_items,
                &event.sell_items,
                ability_values,
                &mut inventory,
                position,
                union_membership,
            ) {
                Ok(updated_items) => {
                    if let Some(game_client) = game_client {
                        game_client
                            .server_message_tx
                            .send(ServerMessage::UpdateInventory(
                                updated_items
                                    .iter()
                                    .map(|slot| (*slot, inventory.get_item(*slot).cloned()))
                                    .collect(),
                                Some(inventory.money),
                            ))
                            .ok();
                    }
                }
                Err(error) => {
                    if let Some(game_client) = game_client {
                        game_client
                            .server_message_tx
                            .send(ServerMessage::NpcStoreTransactionError(error))
                            .ok();
                    }
                }
            }
        }
    }
}
