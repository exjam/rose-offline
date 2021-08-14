use std::num::NonZeroUsize;

use bevy_ecs::prelude::Mut;
use log::warn;
use num_traits::{AsPrimitive, Saturating, Signed};

use crate::{
    data::AbilityType,
    game::{
        components::{
            AbilityValues, BasicStats, CharacterInfo, ExperiencePoints, GameClient, Inventory,
            Level, Money, MoveSpeed, SkillPoints, Stamina, StatPoints, Team, UnionMembership,
            MAX_STAMINA,
        },
        messages::server::{ServerMessage, UpdateAbilityValue, UpdateMoney},
    },
};

fn add_value<T: Saturating + Copy + 'static, U: Signed + AsPrimitive<T>>(
    value: T,
    add_value: U,
) -> T {
    if add_value.is_negative() {
        value.saturating_sub(add_value.abs().as_())
    } else {
        value.saturating_add(add_value.as_())
    }
}

// TODO: Should all ability values be i64? XP + Money potentially do not fit into i32
pub fn ability_values_get_value(
    ability_type: AbilityType,
    ability_values: &AbilityValues,
    level: &Level,
    move_speed: &MoveSpeed,
    team_number: &Team,
    // For characters only
    character_info: Option<&CharacterInfo>,
    experience_points: Option<&ExperiencePoints>,
    inventory: Option<&Inventory>,
    skill_points: Option<&SkillPoints>,
    stamina: Option<&Stamina>,
    stat_points: Option<&StatPoints>,
    union_membership: Option<&UnionMembership>,
) -> Option<i32> {
    match ability_type {
        AbilityType::Gender => character_info.map(|x| (x.gender % 2) as i32),
        AbilityType::Race => character_info.map(|x| (x.gender / 2) as i32),
        AbilityType::Birthstone => character_info.map(|x| x.birth_stone as i32),
        AbilityType::Class => character_info.map(|x| x.job as i32),
        AbilityType::Rank => character_info.map(|x| x.rank as i32),
        AbilityType::Fame => character_info.map(|x| x.fame as i32),
        AbilityType::FameB => character_info.map(|x| x.fame_b as i32),
        AbilityType::FameG => character_info.map(|x| x.fame_g as i32),
        AbilityType::Face => character_info.map(|x| x.face as i32),
        AbilityType::Hair => character_info.map(|x| x.hair as i32),
        AbilityType::Strength => Some(ability_values.get_strength()),
        AbilityType::Dexterity => Some(ability_values.get_dexterity()),
        AbilityType::Intelligence => Some(ability_values.get_intelligence()),
        AbilityType::Concentration => Some(ability_values.get_concentration()),
        AbilityType::Charm => Some(ability_values.get_charm()),
        AbilityType::Sense => Some(ability_values.get_sense()),
        AbilityType::Attack => Some(ability_values.get_attack_power()),
        AbilityType::Defence => Some(ability_values.get_defence()),
        AbilityType::Hit => Some(ability_values.get_hit()),
        AbilityType::Resistance => Some(ability_values.get_resistance()),
        AbilityType::Avoid => Some(ability_values.get_avoid()),
        AbilityType::AttackSpeed => Some(ability_values.get_attack_speed()),
        AbilityType::Critical => Some(ability_values.get_critical()),
        AbilityType::Speed => Some(move_speed.speed as i32),
        AbilityType::Skillpoint => skill_points.map(|x| x.points as i32),
        AbilityType::BonusPoint => stat_points.map(|x| x.points as i32),
        AbilityType::Experience => experience_points.map(|x| x.xp as i32),
        AbilityType::Level => Some(level.level as i32),
        AbilityType::Money => inventory.map(|x| x.money.0 as i32),
        AbilityType::TeamNumber => Some(team_number.id as i32),
        AbilityType::Union => {
            union_membership.map(|x| x.current_union.map(|x| x.get() as i32).unwrap_or(0))
        }
        AbilityType::UnionPoint1 => union_membership.map(|x| x.points[0] as i32),
        AbilityType::UnionPoint2 => union_membership.map(|x| x.points[1] as i32),
        AbilityType::UnionPoint3 => union_membership.map(|x| x.points[2] as i32),
        AbilityType::UnionPoint4 => union_membership.map(|x| x.points[3] as i32),
        AbilityType::UnionPoint5 => union_membership.map(|x| x.points[4] as i32),
        AbilityType::UnionPoint6 => union_membership.map(|x| x.points[5] as i32),
        AbilityType::UnionPoint7 => union_membership.map(|x| x.points[6] as i32),
        AbilityType::UnionPoint8 => union_membership.map(|x| x.points[7] as i32),
        AbilityType::UnionPoint9 => union_membership.map(|x| x.points[8] as i32),
        AbilityType::UnionPoint10 => union_membership.map(|x| x.points[9] as i32),
        AbilityType::Stamina => stamina.map(|x| x.stamina as i32),
        AbilityType::MaxHealth => Some(ability_values.get_max_health()),
        AbilityType::MaxMana => Some(ability_values.get_max_mana()),
        /*
        TODO: Implement remaining get ability types.
        AbilityType::Health => todo!(),
        AbilityType::Mana => todo!(),
        AbilityType::Weight => todo!(),
        AbilityType::SaveMana => todo!(),
        AbilityType::PvpFlag => todo!(),
        AbilityType::HeadSize => todo!(),
        AbilityType::BodySize => todo!(),
        AbilityType::DropRate => todo!(),
        AbilityType::CurrentPlanet => todo!(),
        AbilityType::GuildNumber => todo!(),
        AbilityType::GuildScore => todo!(),
        AbilityType::GuildPosition => todo!(),
        */
        _ => {
            warn!(
                "ability_values_get_value unimplemented for ability type {:?}",
                ability_type
            );
            None
        }
    }
}

pub fn ability_values_add_value(
    ability_type: AbilityType,
    value: i32,
    mut basic_stats: Option<&mut Mut<BasicStats>>,
    mut inventory: Option<&mut Mut<Inventory>>,
    mut skill_points: Option<&mut Mut<SkillPoints>>,
    mut stamina: Option<&mut Mut<Stamina>>,
    mut stat_points: Option<&mut Mut<StatPoints>>,
    mut union_membership: Option<&mut Mut<UnionMembership>>,
    game_client: Option<&GameClient>,
) -> bool {
    let result = match ability_type {
        AbilityType::Strength => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.strength = add_value(basic_stats.strength, value);
                true
            } else {
                false
            }
        }
        AbilityType::Dexterity => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.dexterity = add_value(basic_stats.dexterity, value);
                true
            } else {
                false
            }
        }
        AbilityType::Intelligence => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.intelligence = add_value(basic_stats.intelligence, value);
                true
            } else {
                false
            }
        }
        AbilityType::Concentration => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.concentration = add_value(basic_stats.concentration, value);
                true
            } else {
                false
            }
        }
        AbilityType::Charm => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.charm = add_value(basic_stats.charm, value);
                true
            } else {
                false
            }
        }
        AbilityType::Sense => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.sense = add_value(basic_stats.sense, value);
                true
            } else {
                false
            }
        }
        AbilityType::BonusPoint => {
            if let Some(stat_points) = stat_points.as_mut() {
                stat_points.points = add_value(stat_points.points, value);
                true
            } else {
                false
            }
        }
        AbilityType::Skillpoint => {
            if let Some(skill_points) = skill_points.as_mut() {
                skill_points.points = add_value(skill_points.points, value);
                true
            } else {
                false
            }
        }
        AbilityType::Money => {
            if let Some(inventory) = inventory.as_mut() {
                inventory.try_add_money(Money(value as i64)).is_ok()
            } else {
                false
            }
        }
        AbilityType::UnionPoint1 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[0] = add_value(union_membership.points[0], value);
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint2 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[1] = add_value(union_membership.points[1], value);
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint3 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[2] = add_value(union_membership.points[2], value);
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint4 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[3] = add_value(union_membership.points[3], value);
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint5 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[4] = add_value(union_membership.points[4], value);
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint6 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[5] = add_value(union_membership.points[5], value);
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint7 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[6] = add_value(union_membership.points[6], value);
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint8 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[7] = add_value(union_membership.points[7], value);
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint9 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[8] = add_value(union_membership.points[8], value);
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint10 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[9] = add_value(union_membership.points[9], value);
                true
            } else {
                false
            }
        }
        AbilityType::Stamina => {
            if let Some(stamina) = stamina.as_mut() {
                stamina.stamina = u32::min(add_value(stamina.stamina, value), MAX_STAMINA);
                true
            } else {
                false
            }
        }
        /*
        TODO: Implement remaining add ability types
        AbilityType::Health => false,
        AbilityType::Mana => false,
        AbilityType::Experience => false,
        AbilityType::Level => false,
        */
        _ => {
            warn!(
                "ability_values_add_value unimplemented for ability type {:?}",
                ability_type
            );
            false
        }
    };

    if result {
        if let Some(game_client) = game_client {
            if matches!(ability_type, AbilityType::Money) {
                game_client
                    .server_message_tx
                    .send(ServerMessage::UpdateMoney(UpdateMoney {
                        is_reward: true,
                        money: inventory.unwrap().money,
                    }))
                    .ok();
            } else {
                game_client
                    .server_message_tx
                    .send(ServerMessage::UpdateAbilityValue(
                        UpdateAbilityValue::RewardAdd(ability_type, value),
                    ))
                    .ok();
            }
        }
    }

    result
}

pub fn ability_values_set_value(
    ability_type: AbilityType,
    value: i32,
    mut basic_stats: Option<&mut Mut<BasicStats>>,
    mut character_info: Option<&mut Mut<CharacterInfo>>,
    mut union_membership: Option<&mut Mut<UnionMembership>>,
    game_client: Option<&GameClient>,
) -> bool {
    let result = match ability_type {
        AbilityType::Gender => {
            if let Some(character_info) = character_info.as_mut() {
                character_info.gender = value as u8;
            }

            true
        }
        AbilityType::Face => {
            if let Some(character_info) = character_info.as_mut() {
                character_info.face = value as u8;
            }

            true
        }
        AbilityType::Hair => {
            if let Some(character_info) = character_info.as_mut() {
                character_info.hair = value as u8;
            }

            true
        }
        AbilityType::Class => {
            if let Some(character_info) = character_info.as_mut() {
                character_info.job = value as u16;
            }

            true
        }
        AbilityType::Strength => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.strength = value;
                true
            } else {
                false
            }
        }
        AbilityType::Dexterity => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.dexterity = value;
                true
            } else {
                false
            }
        }
        AbilityType::Intelligence => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.intelligence = value;
                true
            } else {
                false
            }
        }
        AbilityType::Concentration => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.concentration = value;
                true
            } else {
                false
            }
        }
        AbilityType::Charm => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.charm = value;
                true
            } else {
                false
            }
        }
        AbilityType::Sense => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.sense = value;
                true
            } else {
                false
            }
        }
        AbilityType::Union => {
            if let Some(union_membership) = union_membership.as_mut() {
                if value == 0 {
                    union_membership.current_union = None;
                } else {
                    union_membership.current_union = NonZeroUsize::new(value as usize);
                }
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint1 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[0] = value as u32;
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint2 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[1] = value as u32;
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint3 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[2] = value as u32;
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint4 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[3] = value as u32;
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint5 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[4] = value as u32;
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint6 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[5] = value as u32;
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint7 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[6] = value as u32;
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint8 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[7] = value as u32;
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint9 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[8] = value as u32;
                true
            } else {
                false
            }
        }
        AbilityType::UnionPoint10 => {
            if let Some(union_membership) = union_membership.as_mut() {
                union_membership.points[9] = value as u32;
                true
            } else {
                false
            }
        }
        /*
        TODO: Implement remaining set ability types
        AbilityType::Health => false,
        AbilityType::Mana => false,
        AbilityType::Experience => false,
        AbilityType::Level => false,
        AbilityType::PvpFlag => false,
        AbilityType::TeamNumber => false,
        */
        _ => {
            warn!(
                "ability_values_set_value unimplemented for ability type {:?}",
                ability_type
            );
            false
        }
    };

    if result {
        if let Some(game_client) = game_client {
            game_client
                .server_message_tx
                .send(ServerMessage::UpdateAbilityValue(
                    UpdateAbilityValue::RewardSet(ability_type, value),
                ))
                .ok();
        }
    }

    result
}
