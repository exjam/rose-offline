use bevy::{
    ecs::{prelude::Changed, query::WorldQuery, system::Query},
    prelude::Or,
};

use crate::game::components::{AbilityValues, HealthPoints, ManaPoints, MoveMode, MoveSpeed};

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct AbilityValuesChangedQuery<'w> {
    ability_values: &'w mut AbilityValues,
    health_points: &'w mut HealthPoints,
    mana_points: Option<&'w mut ManaPoints>,
    move_mode: &'w MoveMode,
    move_speed: &'w mut MoveSpeed,
}

pub fn ability_values_changed_system(
    mut query: Query<AbilityValuesChangedQuery, Or<(Changed<AbilityValues>, Changed<MoveMode>)>>,
) {
    for mut object in query.iter_mut() {
        // Update is_driving so vehicle stats are used correctly
        object.ability_values.is_driving = matches!(object.move_mode, MoveMode::Drive);

        // Limit hp to max health
        let max_hp = object.ability_values.get_max_health();
        if object.health_points.hp > max_hp {
            object.health_points.hp = max_hp;
        }

        // Limit mp to max mana
        if let Some(mut mana_points) = object.mana_points {
            let max_mp = object.ability_values.get_max_mana();
            if mana_points.mp > max_mp {
                mana_points.mp = max_mp;
            }
        }

        // Update move speed
        let updated_move_speed = object.ability_values.get_move_speed(object.move_mode);
        if (object.move_speed.speed - updated_move_speed).abs() > f32::EPSILON {
            object.move_speed.speed = updated_move_speed;
        }
    }
}
