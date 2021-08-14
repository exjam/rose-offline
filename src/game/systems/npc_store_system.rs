use std::collections::HashSet;

use bevy_ecs::prelude::{Entity, EventReader, Mut, Query, Res, ResMut};
use log::warn;

use crate::{
    data::item::Item,
    game::{
        components::{
            AbilityValues, GameClient, Inventory, ItemSlot, Money, Npc, Position, UnionMembership,
        },
        events::NpcStoreEvent,
        messages::{
            client::NpcStoreBuyItem,
            server::{NpcStoreTransactionError, ServerMessage, UpdateInventory},
        },
        resources::WorldRates,
        GameData,
    },
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
        // if npc_data.store_union_number != union_membership.current_union {
        return Err(NpcStoreTransactionError::NotSameUnion);
    }

    if npc_position.zone_id != position.zone_id
        || nalgebra::distance(&npc_position.position.xy(), &position.position.xy())
            > NPC_STORE_TRANSACTION_MAX_DISTANCE
    {
        return Err(NpcStoreTransactionError::NpcTooFarAway);
    }

    if !sell_items.is_empty() {
        warn!("Unimplemented selling items to NPC store");
    }

    let mut total_buy_cost = 0i64;
    let mut transaction_inventory = inventory.clone();
    let mut updated_inventory_slots = HashSet::new();

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

        let item_data = game_data
            .items
            .get_base_item(store_item_reference)
            .ok_or(NpcStoreTransactionError::NpcNotFound)?;

        let item_price = game_data
            .ability_value_calculator
            .calculate_npc_store_item_buy_price(
                store_item_reference,
                item_data,
                ability_values.get_npc_store_buy_rate(),
                world_rates.prices_rate,
            ) as i64;

        let buy_quantity = if store_item_reference.item_type.is_stackable() {
            buy_item.quantity
        } else {
            1
        } as i64;

        let item = Item::new(&store_item_reference, buy_quantity as u32)
            .ok_or(NpcStoreTransactionError::NpcNotFound)?;

        let (inventory_slot, _) = transaction_inventory
            .try_add_item(item)
            .map_err(|_| NpcStoreTransactionError::NpcNotFound)?;

        updated_inventory_slots.insert(inventory_slot);
        total_buy_cost += item_price * buy_quantity;
    }

    if transaction_inventory
        .try_take_money(Money(total_buy_cost))
        .is_err()
    {
        return Err(NpcStoreTransactionError::NotEnoughMoney);
    }

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
                            .send(ServerMessage::UpdateInventory(UpdateInventory {
                                is_reward: false,
                                items: updated_items
                                    .iter()
                                    .map(|slot| (*slot, inventory.get_item(*slot).cloned()))
                                    .collect(),
                                with_money: Some(inventory.money),
                            }))
                            .ok();
                    }
                }
                Err(NpcStoreTransactionError::InvalidTransactionEntity) => {}
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
