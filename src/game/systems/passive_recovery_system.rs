use std::time::Duration;

use bevy_ecs::prelude::{Query, Res};

use crate::{
    data::PassiveRecoveryState,
    game::{
        components::{AbilityValues, Command, HealthPoints, ManaPoints, PassiveRecoveryTime},
        resources::ServerTime,
        GameData,
    },
};

const RECOVERY_INTERVAL: Duration = Duration::from_secs(4);

pub fn passive_recovery_system(
    mut query: Query<(
        &mut PassiveRecoveryTime,
        &AbilityValues,
        &Command,
        &mut HealthPoints,
        &mut ManaPoints,
    )>,
    game_data: Res<GameData>,
    time: Res<ServerTime>,
) {
    query.for_each_mut(
        |(
            mut passive_recovery_time,
            ability_values,
            command,
            mut health_points,
            mut mana_points,
        )| {
            passive_recovery_time.time += time.delta;

            if passive_recovery_time.time > RECOVERY_INTERVAL {
                passive_recovery_time.time -= RECOVERY_INTERVAL;

                let recovery_state = if command.is_sit() {
                    PassiveRecoveryState::Sitting
                } else {
                    PassiveRecoveryState::Normal
                };

                let recover_hp = game_data
                    .ability_value_calculator
                    .calculate_passive_recover_hp(ability_values, recovery_state);
                let recover_mp = game_data
                    .ability_value_calculator
                    .calculate_passive_recover_mp(ability_values, recovery_state);

                health_points.hp = i32::min(
                    health_points.hp + recover_hp,
                    ability_values.get_max_health(),
                );
                mana_points.mp =
                    i32::min(mana_points.mp + recover_mp, ability_values.get_max_mana());
            }
        },
    );
}
