use bevy::prelude::{EventReader, Query, ResMut};

use rose_data::{Item, StackableSlotBehaviour};
use rose_game_common::{
    components::{Equipment, ItemSlot},
    messages::server::ServerMessage,
};

use crate::{
    game::components::{ClientEntity, GameClient},
    game::{events::UseAmmoEvent, resources::ServerMessages},
};

pub fn use_ammo_system(
    mut query: Query<(&ClientEntity, &mut Equipment, Option<&GameClient>)>,
    mut use_ammo_events: EventReader<UseAmmoEvent>,
    mut server_messages: ResMut<ServerMessages>,
) {
    for event in use_ammo_events.iter() {
        let Ok((client_entity, mut equipment, game_client)) = query.get_mut(event.entity) else {
            continue;
        };

        equipment
            .get_ammo_slot_mut(event.ammo_index)
            .try_take_quantity(event.quantity as u32);

        if let Some(game_client) = game_client {
            match equipment.get_ammo_item(event.ammo_index) {
                Some(ammo_item) => {
                    if (ammo_item.quantity & 0x0F) == 0 {
                        game_client
                            .server_message_tx
                            .send(ServerMessage::UpdateInventory {
                                items: vec![(
                                    ItemSlot::Ammo(event.ammo_index),
                                    Some(Item::Stackable(ammo_item.clone())),
                                )],
                                money: None,
                            })
                            .ok();
                    }
                }
                None => {
                    server_messages.send_entity_message(
                        client_entity,
                        ServerMessage::UpdateAmmo {
                            entity_id: client_entity.id,
                            ammo_index: event.ammo_index,
                            item: None,
                        },
                    );
                }
            }
        }
    }
}
