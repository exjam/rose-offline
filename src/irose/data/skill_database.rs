use arrayvec::ArrayVec;
use log::debug;
use num_traits::FromPrimitive;
use rose_file_readers::{stb_column, StbFile, StlFile, VfsIndex};
use std::{
    collections::HashMap,
    num::{NonZeroU32, NonZeroUsize},
    str::FromStr,
    time::Duration,
};

use crate::data::{
    AbilityType, ItemClass, MotionId, NpcId, SkillActionMode, SkillAddAbility, SkillCooldown,
    SkillCooldownGroup, SkillData, SkillDatabase, SkillId, SkillPageType, SkillTargetFilter,
    SkillType, StatusEffectId, ZoneId,
};

pub struct StbSkill(pub StbFile);

impl FromStr for SkillActionMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u32>().map_err(|_| ())?;
        FromPrimitive::from_u32(value).ok_or(())
    }
}

impl FromStr for SkillCooldownGroup {
    type Err = <NonZeroUsize as std::str::FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SkillCooldownGroup(s.parse::<NonZeroUsize>()?))
    }
}

impl FromStr for SkillType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u32>().map_err(|_| ())?;
        FromPrimitive::from_u32(value).ok_or(())
    }
}

impl FromStr for SkillTargetFilter {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u32>().map_err(|_| ())?;
        FromPrimitive::from_u32(value).ok_or(())
    }
}

impl FromStr for SkillPageType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u32>().map_err(|_| ())?;
        match value {
            0 => Ok(SkillPageType::Basic),
            1 => Ok(SkillPageType::Active),
            2 => Ok(SkillPageType::Passive),
            3 => Ok(SkillPageType::Clan),
            _ => Err(()),
        }
    }
}

#[allow(dead_code)]
impl StbSkill {
    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    stb_column! { 1, get_base_skill_id, SkillId }
    stb_column! { 2, get_skill_level, u32 }
    stb_column! { 3, get_learn_skill_points, u32 }
    stb_column! { 4, get_page, SkillPageType }
    stb_column! { 5, get_skill_type, SkillType }
    stb_column! { 6, get_cast_range, u32 }
    stb_column! { 6, get_require_planet_index, NonZeroUsize }
    stb_column! { 7, get_target_filter, SkillTargetFilter }
    stb_column! { 8, get_scope, u32 }
    stb_column! { 9, get_power, u32 }
    stb_column! { 9, get_item_make_number, u32 }
    stb_column! { 10, get_harm, u32 }
    stb_column! { 11..=12, get_status_effects, [Option<StatusEffectId>; 2] }

    stb_column! { 13, get_success_ratio, i32 }
    stb_column! { 14, get_status_effect_duration_ms, i32 }
    stb_column! { 15, get_damage_type, i32 }

    stb_column! { (16..=19).step_by(2), get_use_ability_type, [Option<AbilityType>; 2] }
    stb_column! { (17..=19).step_by(2), get_use_ability_value, [Option<i32>; 2] }

    pub fn get_use_abilities(&self, id: usize) -> ArrayVec<(AbilityType, i32), 2> {
        self.get_use_ability_type(id)
            .iter()
            .zip(self.get_use_ability_value(id).iter())
            .filter(|(a, b)| a.is_some() && b.is_some())
            .map(|(a, b)| (a.unwrap(), b.unwrap()))
            .collect()
    }

    stb_column! { 20, get_cooldown_time_5ms, i32 }
    stb_column! { 21, get_warp_zone_id, ZoneId }
    stb_column! { 22, get_warp_zone_xpos, i32 }
    stb_column! { 23, get_warp_zone_ypos, i32 }

    stb_column! { (21..=26).step_by(3), get_add_ability_type, [Option<AbilityType>; 2] }
    stb_column! { (22..=26).step_by(3), get_add_ability_value, [i32; 2] }
    stb_column! { (23..=26).step_by(3), get_add_ability_rate, [i32; 2] }

    pub fn get_add_ability(&self, id: usize) -> [Option<SkillAddAbility>; 2] {
        let mut result: [Option<SkillAddAbility>; 2] = Default::default();
        let ability_types = self.get_add_ability_type(id);
        let ability_values = self.get_add_ability_value(id);
        let ability_rates = self.get_add_ability_rate(id);

        for (index, skill_add_ability) in result.iter_mut().enumerate() {
            *skill_add_ability = ability_types[index].map(|ability_type| SkillAddAbility {
                ability_type,
                rate: ability_rates[index],
                value: ability_values[index],
            });
        }

        result
    }

    stb_column! { 27, get_cooldown_group, SkillCooldownGroup }
    stb_column! { 28, get_summon_pet_npc_id, NpcId }
    stb_column! { 29, get_action_mode, SkillActionMode }

    stb_column! { 30..=34, get_required_weapon_class, ArrayVec<ItemClass, 5> }
    stb_column! { 35, get_required_job_set_index, NonZeroUsize }
    stb_column! { 36..=38, get_required_union, ArrayVec<NonZeroUsize, 3> }

    stb_column! { (39..=44).step_by(2), get_required_skill_id, [Option<SkillId>; 3] }
    stb_column! { (40..=44).step_by(2), get_required_skill_level, [Option<i32>; 3] }

    pub fn get_required_skills(&self, id: usize) -> ArrayVec<(SkillId, i32), 3> {
        self.get_required_skill_id(id)
            .iter()
            .zip(self.get_required_skill_level(id).iter())
            .filter(|(a, b)| a.is_some() && b.is_some())
            .map(|(a, b)| (a.unwrap(), b.unwrap()))
            .collect()
    }

    stb_column! { (45..=48).step_by(2), get_required_ability_type, [Option<AbilityType>; 2] }
    stb_column! { (46..=48).step_by(2), get_required_ability_value, [Option<i32>; 2] }

    pub fn get_required_abilities(&self, id: usize) -> ArrayVec<(AbilityType, i32), 2> {
        self.get_required_ability_type(id)
            .iter()
            .zip(self.get_required_ability_value(id).iter())
            .filter(|(a, b)| a.is_some() && b.is_some())
            .map(|(a, b)| (a.unwrap(), b.unwrap()))
            .collect()
    }

    stb_column! { 49, get_script1, i32 }
    stb_column! { 50, get_reserve_02, i32 }
    stb_column! { 51, get_icon_number, u32 }
    stb_column! { 52, get_casting_motion_id, MotionId }
    stb_column! { 53, get_casting_motion_speed, NonZeroU32 }
    stb_column! { 54, get_casting_repeat_motion_id, MotionId }
    stb_column! { 55, get_casting_repeat_motion_count, NonZeroU32 }

    stb_column! { (56..=67).step_by(3), get_casting_effect_index, [Option<NonZeroUsize>; 4] }
    stb_column! { (57..=67).step_by(3), get_casting_effect_bone_index, [Option<usize>; 4] }
    stb_column! { (58..=67).step_by(3), get_casting_sound_index, [Option<usize>; 4] }

    stb_column! { 68, get_action_motion_id, MotionId }
    stb_column! { 69, get_action_motion_speed, NonZeroU32 }
    stb_column! { 70, get_action_motion_hit_count, i32 }
    stb_column! { 71, get_bullet_no, i32 }
    stb_column! { 72, get_bullet_linked_point, i32 }
    stb_column! { 73, get_bullet_fire_sound, i32 }
    stb_column! { 74, get_hit_effect, i32 }
    stb_column! { 75, get_hit_effect_linked_point, i32 }
    stb_column! { 76, get_hit_sound, i32 }

    stb_column! { (77..=82).step_by(3), get_hit_dummy_effect_index, [Option<NonZeroUsize>; 2] }
    stb_column! { (78..=82).step_by(3), get_hit_dummy_effect_bone_index, [Option<usize>; 2] }
    stb_column! { (79..=82).step_by(3), get_hit_dummy_sound_index, [Option<usize>; 2] }

    stb_column! { 83, get_area_hit_effect, i32 }
    stb_column! { 84, get_area_hit_sound, i32 }
    stb_column! { 85, get_learn_money_cost, u32 }
    stb_column! { 86, get_attribute, i32 }

    pub fn get_cooldown(&self, id: usize) -> SkillCooldown {
        let duration =
            Duration::from_millis(self.get_cooldown_time_5ms(id).unwrap_or(0) as u64 * 200);
        match self.get_cooldown_group(id) {
            Some(group) => SkillCooldown::Group(group, duration),
            None => SkillCooldown::Skill(duration),
        }
    }
}

fn load_skill(data: &StbSkill, stl: &StlFile, id: usize) -> Option<SkillData> {
    let skill_id = SkillId::new(id as u16)?;
    let icon_number = data.get_icon_number(id).filter(|icon| *icon != 0)?;
    let skill_type = data.get_skill_type(id)?;

    Some(SkillData {
        id: skill_id,
        name: stl
            .get_text_string(1, data.0.get(id, data.0.columns() - 1))
            .unwrap_or("")
            .to_string(),
        base_skill_id: data.get_base_skill_id(id),
        action_mode: data.get_action_mode(id).unwrap_or(SkillActionMode::Stop),
        action_motion_id: data.get_action_motion_id(id),
        action_motion_speed: data
            .get_action_motion_speed(id)
            .map(|x| x.get())
            .unwrap_or(100) as f32
            / 100.0,
        add_ability: data.get_add_ability(id),
        cast_range: data.get_cast_range(id).unwrap_or(0),
        casting_motion_id: data.get_casting_motion_id(id),
        casting_motion_speed: data
            .get_casting_motion_speed(id)
            .map(|x| x.get())
            .unwrap_or(100) as f32
            / 100.0,
        casting_repeat_motion_count: data
            .get_casting_repeat_motion_count(id)
            .map(|x| x.get())
            .unwrap_or(1),
        casting_repeat_motion_id: data.get_casting_repeat_motion_id(id),
        cooldown: data.get_cooldown(id),
        damage_type: data.get_damage_type(id).unwrap_or(0),
        harm: data.get_harm(id).unwrap_or(0),
        icon_number,
        item_make_number: data.get_item_make_number(id).unwrap_or(0),
        level: data.get_skill_level(id).unwrap_or(1),
        learn_money_cost: data.get_learn_money_cost(id).unwrap_or(0),
        page: data.get_page(id).unwrap_or(SkillPageType::Basic),
        learn_point_cost: data.get_learn_skill_points(id).unwrap_or(0),
        power: data.get_power(id).unwrap_or(0),
        required_ability: data.get_required_abilities(id),
        required_job_set_index: data.get_required_job_set_index(id),
        required_planet: data.get_require_planet_index(id),
        required_skills: data.get_required_skills(id),
        required_union: data.get_required_union(id),
        required_weapon_class: data.get_required_weapon_class(id),
        scope: data.get_scope(id).unwrap_or(0),
        skill_type,
        status_effect_duration: Duration::from_secs(
            data.get_status_effect_duration_ms(id).unwrap_or(0) as u64,
        ),
        status_effects: data.get_status_effects(id),
        success_ratio: data.get_success_ratio(id).unwrap_or(0),
        summon_npc_id: data.get_summon_pet_npc_id(id),
        target_filter: data
            .get_target_filter(id)
            .unwrap_or(SkillTargetFilter::OnlySelf),
        use_ability: data.get_use_abilities(id),
        warp_zone_id: data.get_warp_zone_id(id),
        warp_zone_x: data.get_warp_zone_xpos(id).unwrap_or(0) as f32 * 1000.0,
        warp_zone_y: data.get_warp_zone_ypos(id).unwrap_or(0) as f32 * 1000.0,
    })
}

pub fn get_skill_database(vfs: &VfsIndex) -> Option<SkillDatabase> {
    let stl = vfs
        .read_file::<StlFile, _>("3DDATA/STB/LIST_SKILL_S.STL")
        .ok()?;
    let data = StbSkill(
        vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_SKILL.STB")
            .ok()?,
    );
    let mut skills = HashMap::new();
    for id in 1..data.rows() {
        if let Some(skill) = load_skill(&data, &stl, id) {
            skills.insert(id as u16, skill);
        }
    }

    debug!("Loaded {} skills", skills.len());
    Some(SkillDatabase::new(skills))
}
