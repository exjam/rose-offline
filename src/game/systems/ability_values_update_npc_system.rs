use bevy::ecs::{
    prelude::{Changed, Entity, Or, Res},
    query::WorldQuery,
    system::Query,
};

use crate::game::{
    components::{AbilityValues, Npc, StatusEffects},
    GameData,
};

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct AbilityValuesNpcQuery<'w> {
    ability_values: &'w mut AbilityValues,
    npc: &'w Npc,
    status_effects: &'w StatusEffects,
}

pub fn ability_values_update_npc_system(
    mut query: Query<AbilityValuesNpcQuery, Or<(Changed<Npc>, Changed<StatusEffects>)>>,
    game_data: Res<GameData>,
) {
    for mut npc in query.iter_mut() {
        *npc.ability_values = game_data
            .ability_value_calculator
            .calculate_npc(
                npc.npc.id,
                npc.status_effects,
                npc.ability_values.summon_owner_level,
                npc.ability_values.summon_skill_level,
            )
            .unwrap();
    }
}
