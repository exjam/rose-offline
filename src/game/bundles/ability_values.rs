use log::warn;
use num_traits::{AsPrimitive, Saturating, Signed};

use crate::{
    data::ability::AbilityType,
    game::{
        components::{
            AbilityValues, BasicStats, CharacterInfo, ExperiencePoints, GameClient, Inventory,
            Level, Money, MoveSpeed, SkillPoints, StatPoints, Team, UnionMembership,
        },
        messages::server::{ServerMessage, UpdateAbilityValue},
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
    ability_values: Option<&AbilityValues>,
    character_info: Option<&CharacterInfo>,
    experience_points: Option<&ExperiencePoints>,
    inventory: Option<&Inventory>,
    level: Option<&Level>,
    move_speed: Option<&MoveSpeed>,
    stat_points: Option<&StatPoints>,
    skill_points: Option<&SkillPoints>,
    team_number: Option<&Team>,
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
        AbilityType::Strength => ability_values.map(|x| x.strength as i32),
        AbilityType::Dexterity => ability_values.map(|x| x.dexterity as i32),
        AbilityType::Intelligence => ability_values.map(|x| x.intelligence as i32),
        AbilityType::Concentration => ability_values.map(|x| x.concentration as i32),
        AbilityType::Charm => ability_values.map(|x| x.charm as i32),
        AbilityType::Sense => ability_values.map(|x| x.sense as i32),
        AbilityType::Attack => ability_values.map(|x| x.attack_power as i32),
        AbilityType::Defence => ability_values.map(|x| x.defence as i32),
        AbilityType::Hit => ability_values.map(|x| x.hit as i32),
        AbilityType::Resistance => ability_values.map(|x| x.resistance as i32),
        AbilityType::Avoid => ability_values.map(|x| x.avoid as i32),
        AbilityType::AttackSpeed => ability_values.map(|x| x.attack_speed as i32),
        AbilityType::Critical => ability_values.map(|x| x.critical as i32),
        AbilityType::Speed => move_speed.map(|x| x.speed as i32),
        AbilityType::Skillpoint => skill_points.map(|x| x.points as i32),
        AbilityType::BonusPoint => stat_points.map(|x| x.points as i32),
        AbilityType::Experience => experience_points.map(|x| x.xp as i32),
        AbilityType::Level => level.map(|x| x.level as i32),
        AbilityType::Money => inventory.map(|x| x.money.0 as i32),
        AbilityType::TeamNumber => team_number.map(|x| x.id as i32),
        AbilityType::Union => union_membership
            .and_then(|x| x.current_union)
            .map(|x| x as i32),
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
        /*
        TODO: Implement remaining get ability types.
        AbilityType::Health => todo!(),
        AbilityType::Mana => todo!(),
        AbilityType::Weight => todo!(),
        AbilityType::SaveMana => todo!(),
        AbilityType::PvpFlag => todo!(),
        AbilityType::HeadSize => todo!(),
        AbilityType::BodySize => todo!(),
        AbilityType::MaxHealth => todo!(),
        AbilityType::MaxMana => todo!(),
        AbilityType::DropRate => todo!(),
        AbilityType::CurrentPlanet => todo!(),
        AbilityType::Stamina => todo!(),
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
    mut basic_stats: Option<&mut BasicStats>,
    mut inventory: Option<&mut Inventory>,
    mut stat_points: Option<&mut StatPoints>,
    mut skill_points: Option<&mut SkillPoints>,
    mut union_membership: Option<&mut UnionMembership>,
    game_client: &Option<&GameClient>,
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
            game_client
                .server_message_tx
                .send(ServerMessage::UpdateAbilityValue(
                    UpdateAbilityValue::RewardAdd(ability_type, value),
                ))
                .ok();
        }
    }

    result
}

pub fn ability_values_set_value(
    ability_type: AbilityType,
    value: i32,
    mut basic_stats: Option<&mut BasicStats>,
    mut character_info: Option<&mut CharacterInfo>,
    mut union_membership: Option<&mut UnionMembership>,
    game_client: &Option<&GameClient>,
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
                basic_stats.strength = value as u16;
                true
            } else {
                false
            }
        }
        AbilityType::Dexterity => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.dexterity = value as u16;
                true
            } else {
                false
            }
        }
        AbilityType::Intelligence => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.intelligence = value as u16;
                true
            } else {
                false
            }
        }
        AbilityType::Concentration => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.concentration = value as u16;
                true
            } else {
                false
            }
        }
        AbilityType::Charm => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.charm = value as u16;
                true
            } else {
                false
            }
        }
        AbilityType::Sense => {
            if let Some(basic_stats) = basic_stats.as_mut() {
                basic_stats.sense = value as u16;
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
                    union_membership.current_union = Some(value as usize);
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
        AbilityType::Stamina => false,
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
