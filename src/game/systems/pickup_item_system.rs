use bevy::{
    ecs::query::WorldQuery,
    prelude::{Commands, Entity, EventReader, EventWriter, Query, Res, ResMut},
};
use rose_data::{ItemClass, ItemType};
use rose_game_common::{
    components::{DroppedItem, Inventory, ItemDrop, Money},
    messages::{
        server::{PickupItemDropContent, PickupItemDropError, PickupItemDropResult, ServerMessage},
        PartyItemSharing,
    },
};

use crate::game::{
    bundles::client_entity_leave_zone,
    components::{
        ClientEntity, ClientEntitySector, GameClient, Owner, Party, PartyMember, PartyMembership,
        PartyOwner, Position,
    },
    events::{PickupItemEvent, UseItemEvent},
    resources::ClientEntityList,
    GameData,
};

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct PickupItemQuery<'w> {
    client_entity: &'w ClientEntity,
    client_entity_sector: &'w ClientEntitySector,
    item_drop: &'w mut ItemDrop,
    position: &'w Position,
    owner: Option<&'w Owner>,
    party_owner: Option<&'w PartyOwner>,
}

#[allow(clippy::unnecessary_unwrap)]
pub fn pickup_item_system(
    mut commands: Commands,
    mut pickup_item_events: EventReader<PickupItemEvent>,
    mut query_pickup_item: Query<PickupItemQuery>,
    mut query_party: Query<&mut Party>,
    mut query_inventory: Query<(&mut Inventory, Option<&GameClient>)>,
    query_game_client: Query<&GameClient>,
    query_client_entity: Query<&ClientEntity>,
    query_party_membership: Query<&PartyMembership>,
    mut client_entity_list: ResMut<ClientEntityList>,
    game_data: Res<GameData>,
    mut use_item_events: EventWriter<UseItemEvent>,
) {
    for pickup_item_event in pickup_item_events.iter() {
        let mut pickup_item =
            if let Ok(pickup_item) = query_pickup_item.get_mut(pickup_item_event.item_entity) {
                pickup_item
            } else {
                continue;
            };

        let dropped_item = if let Some(item) = pickup_item.item_drop.item.as_ref() {
            item
        } else {
            continue;
        };

        let pickup_party = query_party_membership
            .get(pickup_item_event.pickup_entity)
            .ok()
            .and_then(|party_membership| party_membership.party());
        let mut pickup_entity = None;

        // Can we pickup the item on behalf of our party?
        if pickup_party.is_some()
            && pickup_party
                == pickup_item
                    .party_owner
                    .map(|party_owner| party_owner.entity)
        {
            let pickup_party = pickup_party.unwrap();

            if let Ok(mut party) = query_party.get_mut(pickup_party) {
                // Try pickup using party rules
                pickup_entity = Some(match party.item_sharing {
                    PartyItemSharing::EqualLootDistribution => {
                        match dropped_item {
                            DroppedItem::Item(_) => {
                                // Give item to whoever picked it up
                                pickup_item_event.pickup_entity
                            }
                            DroppedItem::Money(money) => {
                                // Split money evenly
                                let num_online_party_members = party
                                    .members
                                    .iter()
                                    .filter(|party_member| party_member.get_entity().is_some())
                                    .count()
                                    as i64;
                                let money_per_member = (money.0 / num_online_party_members) + 1;

                                for party_member in party.members.iter() {
                                    if let PartyMember::Online(party_member_entity) = party_member {
                                        if let Ok((mut inventory, game_client)) =
                                            query_inventory.get_mut(*party_member_entity)
                                        {
                                            if inventory
                                                .try_add_money(Money(money_per_member))
                                                .is_ok()
                                            {
                                                if let Some(game_client) = &game_client {
                                                    game_client
                                                        .server_message_tx
                                                        .send(ServerMessage::RewardMoney(
                                                            inventory.money,
                                                        ))
                                                        .ok();
                                                }
                                            }
                                        }
                                    }
                                }

                                pickup_item.item_drop.item =
                                    Some(DroppedItem::Money(Money(money_per_member)));
                                pickup_item_event.pickup_entity
                            }
                        }
                    }
                    PartyItemSharing::AcquisitionOrder => match dropped_item {
                        DroppedItem::Item(item) => {
                            // Take turns in getting item - per item type
                            let party = &mut *party;
                            let acquire_item_order =
                                &mut party.acquire_item_order[item.get_item_type().into()];

                            loop {
                                *acquire_item_order =
                                    (*acquire_item_order + 1) % party.members.len();
                                if let PartyMember::Online(party_member_entity) =
                                    party.members[*acquire_item_order]
                                {
                                    break party_member_entity;
                                }
                            }
                        }
                        DroppedItem::Money(_) => loop {
                            // Take turns in getting money
                            party.acquire_money_order =
                                (party.acquire_money_order + 1) % party.members.len();
                            if let PartyMember::Online(party_member_entity) =
                                party.members[party.acquire_money_order]
                            {
                                break party_member_entity;
                            }
                        },
                    },
                });
            }
        }

        // Can we pickup the item for ourself?
        if pickup_entity.is_none()
            && (pickup_item.owner.is_none()
                || pickup_item.owner.map(|owner| owner.entity)
                    == Some(pickup_item_event.pickup_entity))
        {
            pickup_entity = Some(pickup_item_event.pickup_entity);
        }

        if let Some(pickup_entity) = pickup_entity {
            match pickup_item.item_drop.item.take() {
                Some(DroppedItem::Item(item)) => {
                    if matches!(item.get_item_type(), ItemType::Consumable)
                        && game_data
                            .items
                            .get_consumable_item(item.get_item_number())
                            .map_or(false, |item_data| {
                                matches!(item_data.item_data.class, ItemClass::AutomaticConsumption)
                            })
                    {
                        use_item_events.send(UseItemEvent::from_item(pickup_entity, item));
                    } else if let Ok((mut inventory, game_client)) =
                        query_inventory.get_mut(pickup_entity)
                    {
                        let result = match inventory.try_add_item(item.clone()) {
                            Ok((slot, item)) => Ok(PickupItemDropContent::Item(slot, item.clone())),
                            Err(item) => {
                                pickup_item.item_drop.item = Some(DroppedItem::Item(item));
                                Err(PickupItemDropError::InventoryFull)
                            }
                        };

                        if let Some(game_client) = &game_client {
                            game_client
                                .server_message_tx
                                .send(ServerMessage::PickupItemDropResult(PickupItemDropResult {
                                    item_entity_id: pickup_item.client_entity.id,
                                    result,
                                }))
                                .ok();
                        }

                        if let Ok(client_entity_id) = query_client_entity
                            .get(pickup_entity)
                            .map(|client_entity| client_entity.id)
                        {
                            if let Some(party) = query_party_membership
                                .get(pickup_entity)
                                .ok()
                                .and_then(|party_membership| party_membership.party())
                                .and_then(|party_entity| query_party.get(party_entity).ok())
                            {
                                for party_member in party.members.iter() {
                                    if let &PartyMember::Online(party_member_entity) = party_member
                                    {
                                        if party_member_entity != pickup_entity {
                                            if let Ok(game_client) =
                                                query_game_client.get(party_member_entity)
                                            {
                                                game_client
                                                    .server_message_tx
                                                    .send(ServerMessage::PartyMemberRewardItem {
                                                        client_entity_id,
                                                        item: item.clone(),
                                                    })
                                                    .ok();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Some(DroppedItem::Money(money)) => {
                    if let Ok((mut inventory, game_client)) = query_inventory.get_mut(pickup_entity)
                    {
                        if inventory.try_add_money(money).is_ok() {
                            if let Some(game_client) = &game_client {
                                game_client
                                    .server_message_tx
                                    .send(ServerMessage::PickupItemDropResult(
                                        PickupItemDropResult {
                                            item_entity_id: pickup_item.client_entity.id,
                                            result: Ok(PickupItemDropContent::Money(money)),
                                        },
                                    ))
                                    .ok();
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }

            if pickup_item.item_drop.item.is_none() {
                // Delete picked up item
                client_entity_leave_zone(
                    &mut commands,
                    &mut client_entity_list,
                    pickup_item_event.item_entity,
                    pickup_item.client_entity,
                    pickup_item.client_entity_sector,
                    pickup_item.position,
                );
                commands.entity(pickup_item_event.item_entity).despawn();
            }
        } else if let Ok(game_client) = query_game_client.get(pickup_item_event.pickup_entity) {
            game_client
                .server_message_tx
                .send(ServerMessage::PickupItemDropResult(PickupItemDropResult {
                    item_entity_id: pickup_item.client_entity.id,
                    result: Err(PickupItemDropError::NoPermission),
                }))
                .ok();
        }
    }
}
