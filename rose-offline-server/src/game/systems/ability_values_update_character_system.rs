use bevy::ecs::{
    prelude::{Changed, Or, Res},
    query::WorldQuery,
    system::Query,
};

use crate::game::{
    components::{
        AbilityValues, BasicStats, CharacterInfo, Equipment, Level, SkillList, StatusEffects,
    },
    GameData,
};

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct AbilityValuesCharacterQuery<'w> {
    ability_values: &'w mut AbilityValues,
    basic_stats: &'w BasicStats,
    character_info: &'w CharacterInfo,
    equipment: &'w Equipment,
    level: &'w Level,
    skill_list: &'w SkillList,
    status_effects: &'w StatusEffects,
}

pub fn ability_values_update_character_system(
    mut query: Query<
        AbilityValuesCharacterQuery,
        Or<(
            Changed<CharacterInfo>,
            Changed<Level>,
            Changed<Equipment>,
            Changed<BasicStats>,
            Changed<SkillList>,
            Changed<StatusEffects>,
        )>,
    >,
    game_data: Res<GameData>,
) {
    for mut character in query.iter_mut() {
        *character.ability_values = game_data.ability_value_calculator.calculate(
            character.character_info,
            character.level,
            character.equipment,
            character.basic_stats,
            character.skill_list,
            character.status_effects,
        );
    }
}
