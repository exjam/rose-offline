use crate::game::{
    bundles::ItemDropBundle,
    components::{DroppedItem, GameClient, Inventory, Position},
    events::RewardItemEvent,
    messages::server::ServerMessage,
    resources::{ClientEntityList, ServerTime},
};
use bevy_ecs::{
    prelude::{Commands, EventReader, Query, ResMut},
    system::Res,
};

pub fn reward_item_system(
    mut commands: Commands,
    mut query: Query<(&Position, &mut Inventory, Option<&GameClient>)>,
    mut reward_item_events: EventReader<RewardItemEvent>,
    mut client_entity_list: ResMut<ClientEntityList>,
    server_time: Res<ServerTime>,
) {
    for event in reward_item_events.iter() {
        if let Ok((position, mut inventory, game_client)) = query.get_mut(event.entity) {
            match inventory.try_add_item(event.item.clone()) {
                Ok((slot, item)) => {
                    if let Some(game_client) = game_client {
                        game_client
                            .server_message_tx
                            .send(ServerMessage::RewardItems(vec![(slot, Some(item.clone()))]))
                            .ok();
                    }
                }
                Err(item) => {
                    if event.drop_on_full_inventory {
                        ItemDropBundle::spawn(
                            &mut commands,
                            &mut client_entity_list,
                            DroppedItem::Item(item),
                            position,
                            Some(event.entity),
                            &server_time,
                        );
                    }
                }
            }
        }
    }
}
