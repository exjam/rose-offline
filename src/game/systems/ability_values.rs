use bevy_ecs::prelude::{Changed, Or, Query, QuerySet, Res};

use crate::game::{
    components::{
        AbilityValues, BasicStats, CharacterInfo, Equipment, HealthPoints, Level, ManaPoints,
        MoveMode, MoveSpeed, Npc, SkillList, StatusEffects,
    },
    GameData,
};

#[allow(clippy::type_complexity)]
pub fn ability_values_system(
    mut query_set: QuerySet<(
        Query<
            (
                &mut AbilityValues,
                &CharacterInfo,
                &Level,
                &Equipment,
                &BasicStats,
                &SkillList,
                &StatusEffects,
            ),
            Or<(
                Changed<CharacterInfo>,
                Changed<Level>,
                Changed<Equipment>,
                Changed<BasicStats>,
                Changed<SkillList>,
                Changed<StatusEffects>,
            )>,
        >,
        Query<
            (&mut AbilityValues, &Level, &Npc, &StatusEffects),
            Or<(Changed<Level>, Changed<Npc>, Changed<StatusEffects>)>,
        >,
        Query<
            (
                &AbilityValues,
                &MoveMode,
                &mut MoveSpeed,
                &mut HealthPoints,
                Option<&mut ManaPoints>,
            ),
            Changed<AbilityValues>,
        >,
    )>,
    game_data: Res<GameData>,
) {
    query_set.q0_mut().for_each_mut(
        |(
            mut ability_values,
            character_info,
            level,
            equipment,
            basic_stats,
            skill_list,
            status_effects,
        )| {
            *ability_values = game_data.ability_value_calculator.calculate(
                character_info,
                level,
                equipment,
                basic_stats,
                skill_list,
                status_effects,
            );
        },
    );

    query_set
        .q1_mut()
        .for_each_mut(|(mut ability_values, level, npc, status_effects)| {
            *ability_values = game_data
                .ability_value_calculator
                .calculate_npc(npc.id, Some(level), status_effects)
                .unwrap();
        });

    query_set.q2_mut().for_each_mut(
        |(ability_values, move_mode, mut move_speed, mut health_points, mana_points)| {
            // Limit hp to max health
            let max_hp = ability_values.get_max_health() as u32;
            if health_points.hp > max_hp {
                health_points.hp = max_hp;
            }

            // Limit mp to max mana
            if let Some(mut mana_points) = mana_points {
                let max_mp = ability_values.get_max_mana() as u32;
                if mana_points.mp > max_mp {
                    mana_points.mp = max_mp;
                }
            }

            // Update move speed
            let updated_move_speed = match move_mode {
                MoveMode::Run => ability_values.get_run_speed(),
                MoveMode::Walk => ability_values.get_walk_speed(),
            };
            if (move_speed.speed - updated_move_speed).abs() > f32::EPSILON {
                move_speed.speed = updated_move_speed;
            }
        },
    );
}
