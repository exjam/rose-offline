use bevy_ecs::prelude::{Changed, Commands, Entity, Or, Query, Res};

use crate::game::{
    components::{Equipment, Inventory, Weight},
    GameData,
};

pub fn weight_system(
    mut commands: Commands,
    calculate_weight_query: Query<
        (Entity, &Inventory, &Equipment),
        Or<(Changed<Inventory>, Changed<Equipment>)>,
    >,
    game_data: Res<GameData>,
) {
    calculate_weight_query.for_each(|(entity, inventory, equipment)| {
        let mut weight = 0;

        for item in inventory.iter().filter_map(|slot| slot.as_ref()) {
            weight += game_data
                .items
                .get_base_item(item.get_item_reference())
                .map(|item_data| item_data.weight)
                .unwrap_or(0)
                * item.get_quantity();
        }

        for item in equipment.iter_items().filter_map(|slot| slot.as_ref()) {
            weight += game_data
                .items
                .get_base_item(item.item)
                .map(|item_data| item_data.weight)
                .unwrap_or(0);
        }

        for item in equipment.iter_vehicles().filter_map(|slot| slot.as_ref()) {
            weight += game_data
                .items
                .get_base_item(item.item)
                .map(|item_data| item_data.weight)
                .unwrap_or(0);
        }

        for item in equipment.iter_ammo().filter_map(|slot| slot.as_ref()) {
            weight += game_data
                .items
                .get_base_item(item.item)
                .map(|item_data| item_data.weight)
                .unwrap_or(0)
                * item.quantity;
        }

        commands.entity(entity).insert(Weight::new(weight));
    });
}
