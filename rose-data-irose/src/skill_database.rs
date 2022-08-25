use arrayvec::ArrayVec;
use log::debug;
use std::{
    num::{NonZeroU32, NonZeroUsize},
    sync::Arc,
    time::Duration,
};

use rose_data::{
    AbilityType, EffectFileId, EffectId, ItemClass, JobClassId, MotionId, NpcId, SkillActionMode,
    SkillAddAbility, SkillCastingEffect, SkillCooldown, SkillCooldownGroup, SkillData,
    SkillDatabase, SkillId, SkillPageType, SkillTargetFilter, SoundId, StatusEffectId,
    StringDatabase, ZoneId,
};
use rose_file_readers::{stb_column, StbFile, VirtualFilesystem};

use crate::data_decoder::{
    decode_item_class, IroseAbilityType, IroseSkillActionMode, IroseSkillBasicCommand,
    IroseSkillPageType, IroseSkillTargetFilter, IroseSkillType,
};

pub const SKILL_PAGE_SIZE: usize = 30;

pub struct StbSkill(pub StbFile);

#[allow(dead_code)]
impl StbSkill {
    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    stb_column! { 1, get_base_skill_id, SkillId }
    stb_column! { 2, get_skill_level, u32 }
    stb_column! { 3, get_learn_skill_points, u32 }
    stb_column! { 4, get_page, IroseSkillPageType }
    stb_column! { 5, get_skill_type, IroseSkillType }
    stb_column! { 6, get_cast_range, u32 }
    stb_column! { 6, get_require_planet_index, NonZeroUsize }
    stb_column! { 6, get_basic_command, IroseSkillBasicCommand }
    stb_column! { 7, get_target_filter, IroseSkillTargetFilter }
    stb_column! { 8, get_scope, u32 }
    stb_column! { 9, get_power, u32 }
    stb_column! { 9, get_item_make_number, u32 }
    stb_column! { 10, get_harm, u32 }
    stb_column! { 11..=12, get_status_effects, [Option<StatusEffectId>; 2] }

    stb_column! { 13, get_success_ratio, i32 }
    stb_column! { 14, get_status_effect_duration_ms, i32 }
    stb_column! { 15, get_damage_type, i32 }

    stb_column! { (16..=19).step_by(2), get_use_ability_type, [Option<IroseAbilityType>; 2] }
    stb_column! { (17..=19).step_by(2), get_use_ability_value, [Option<i32>; 2] }

    pub fn get_use_abilities(&self, id: usize) -> ArrayVec<(AbilityType, i32), 2> {
        self.get_use_ability_type(id)
            .map(|x| x.and_then(|x| x.try_into().ok()))
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

    stb_column! { (21..=26).step_by(3), get_add_ability_type, [Option<IroseAbilityType>; 2] }
    stb_column! { (22..=26).step_by(3), get_add_ability_value, [i32; 2] }
    stb_column! { (23..=26).step_by(3), get_add_ability_rate, [i32; 2] }

    pub fn get_add_ability(&self, id: usize) -> [Option<SkillAddAbility>; 2] {
        let mut result: [Option<SkillAddAbility>; 2] = Default::default();
        let ability_types = self.get_add_ability_type(id);
        let ability_values = self.get_add_ability_value(id);
        let ability_rates = self.get_add_ability_rate(id);

        for (index, skill_add_ability) in result.iter_mut().enumerate() {
            *skill_add_ability = ability_types[index].and_then(|x| {
                x.try_into().ok().map(|ability_type| SkillAddAbility {
                    ability_type,
                    rate: ability_rates[index],
                    value: ability_values[index],
                })
            });
        }

        result
    }

    stb_column! { 27, get_cooldown_group, NonZeroUsize }
    stb_column! { 28, get_summon_pet_npc_id, NpcId }
    stb_column! { 29, get_action_mode, IroseSkillActionMode }

    pub fn get_required_weapon_class(&self, row: usize) -> ArrayVec<ItemClass, 5> {
        let mut result = ArrayVec::<_, 5>::new();

        for column in 30..=34 {
            if let Some(value) = self
                .0
                .try_get_int(row, column)
                .and_then(|x| decode_item_class(x as usize))
            {
                result.push(value);
            }
        }

        result
    }

    stb_column! { 35, get_required_job_class, JobClassId }
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

    stb_column! { (45..=48).step_by(2), get_required_ability_type, [Option<IroseAbilityType>; 2] }
    stb_column! { (46..=48).step_by(2), get_required_ability_value, [Option<i32>; 2] }

    pub fn get_required_abilities(&self, id: usize) -> ArrayVec<(AbilityType, i32), 2> {
        self.get_required_ability_type(id)
            .map(|x| x.and_then(|x| <AbilityType>::try_from(x).ok()))
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

    stb_column! { (56..=67).step_by(3), get_casting_effect_file_ids, [Option<EffectFileId>; 4] }
    stb_column! { (57..=67).step_by(3), get_casting_effect_bone_index, [Option<usize>; 4] }
    stb_column! { (58..=67).step_by(3), get_casting_sound_id, [Option<usize>; 4] }

    pub fn get_casting_effects(&self, id: usize) -> [Option<SkillCastingEffect>; 4] {
        let mut result: [Option<SkillCastingEffect>; 4] = Default::default();
        let effect_file_ids = self.get_casting_effect_file_ids(id);
        let bone_ids = self.get_casting_effect_bone_index(id);

        for i in 0..4 {
            if let Some(effect_file_id) = effect_file_ids[i] {
                result[i] = Some(SkillCastingEffect {
                    effect_file_id,
                    effect_dummy_bone_id: bone_ids[i].filter(|x| *x != 999),
                });
            }
        }

        result
    }

    stb_column! { 68, get_action_motion_id, MotionId }
    stb_column! { 69, get_action_motion_speed, NonZeroU32 }
    stb_column! { 70, get_action_motion_hit_count, i32 }
    stb_column! { 71, get_bullet_effect_id, EffectId }
    stb_column! { 72, get_bullet_link_dummy_bone_id, u32 }
    stb_column! { 73, get_bullet_fire_sound_id, SoundId }
    stb_column! { 74, get_hit_effect_id, EffectFileId }
    stb_column! { 75, get_hit_effect_dummy_bone_id, u32 }
    stb_column! { 76, get_hit_sound_id, SoundId }

    stb_column! { (77..=82).step_by(3), get_hit_dummy_effect_id, [Option<EffectId>; 2] }
    stb_column! { (78..=82).step_by(3), get_hit_dummy_effect_dummy_bone_index, [Option<u32>; 2] }
    stb_column! { (79..=82).step_by(3), get_hit_dummy_sound_id, [Option<SoundId>; 2] }

    stb_column! { 83, get_area_hit_effect, i32 }
    stb_column! { 84, get_area_hit_sound, i32 }
    stb_column! { 85, get_learn_money_cost, u32 }
    stb_column! { 86, get_attribute, i32 }

    pub fn get_cooldown(&self, id: usize) -> SkillCooldown {
        let duration =
            Duration::from_millis(self.get_cooldown_time_5ms(id).unwrap_or(0) as u64 * 200);
        match self.get_cooldown_group(id) {
            Some(group) => SkillCooldown::Group(SkillCooldownGroup(group), duration),
            None => SkillCooldown::Skill(duration),
        }
    }
}

fn load_skill(data: &StbSkill, string_database: &StringDatabase, id: usize) -> Option<SkillData> {
    let skill_id = SkillId::new(id as u16)?;
    let icon_number = data.get_icon_number(id)?;
    let skill_type = data.get_skill_type(id).and_then(|x| x.try_into().ok())?;
    let skill_strings = string_database.get_skill(data.0.get(id, data.0.columns() - 1));

    Some(SkillData {
        id: skill_id,
        name: skill_strings
            .as_ref()
            .map_or("", |x| unsafe { std::mem::transmute(x.name) }),
        description: skill_strings
            .as_ref()
            .map_or("", |x| unsafe { std::mem::transmute(x.description) }),
        base_skill_id: data.get_base_skill_id(id),
        action_mode: data
            .get_action_mode(id)
            .and_then(|x| x.try_into().ok())
            .unwrap_or(SkillActionMode::Stop),
        action_motion_id: data.get_action_motion_id(id),
        action_motion_speed: data
            .get_action_motion_speed(id)
            .map(|x| x.get())
            .unwrap_or(100) as f32
            / 100.0,
        add_ability: data.get_add_ability(id),
        basic_command: data.get_basic_command(id).and_then(|x| x.try_into().ok()),
        bullet_effect_id: data.get_bullet_effect_id(id),
        bullet_link_dummy_bone_id: data.get_bullet_link_dummy_bone_id(id).unwrap_or(0),
        bullet_fire_sound_id: data.get_bullet_fire_sound_id(id),
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
        casting_effects: data.get_casting_effects(id),
        cooldown: data.get_cooldown(id),
        damage_type: data.get_damage_type(id).unwrap_or(0),
        harm: data.get_harm(id).unwrap_or(0),
        hit_effect_file_id: data.get_hit_effect_id(id),
        hit_link_dummy_bone_id: data
            .get_hit_effect_dummy_bone_id(id)
            .filter(|x| *x != 999)
            .map(|x| x as usize),
        hit_sound_id: data.get_hit_sound_id(id),
        icon_number,
        item_make_number: data.get_item_make_number(id).unwrap_or(0),
        level: data.get_skill_level(id).unwrap_or(1),
        learn_money_cost: data.get_learn_money_cost(id).unwrap_or(0),
        page: data.get_page(id).map(|x| x as SkillPageType).unwrap_or(0),
        learn_point_cost: data.get_learn_skill_points(id).unwrap_or(0),
        power: data.get_power(id).unwrap_or(0),
        required_ability: data.get_required_abilities(id),
        required_job_class: data.get_required_job_class(id),
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
            .and_then(|x| x.try_into().ok())
            .unwrap_or(SkillTargetFilter::OnlySelf),
        use_ability: data.get_use_abilities(id),
        warp_zone_id: data.get_warp_zone_id(id),
        warp_zone_x: data.get_warp_zone_xpos(id).unwrap_or(0) as f32 * 1000.0,
        warp_zone_y: data.get_warp_zone_ypos(id).unwrap_or(0) as f32 * 1000.0,
    })
}

pub fn get_skill_database(
    vfs: &VirtualFilesystem,
    string_database: Arc<StringDatabase>,
) -> Result<SkillDatabase, anyhow::Error> {
    let data = StbSkill(vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_SKILL.STB")?);
    let mut skills = Vec::with_capacity(data.rows());
    skills.push(None); // SkillId 0
    for id in 1..data.rows() {
        skills.push(load_skill(&data, &string_database, id));
    }

    debug!("Loaded {} skills", skills.len());
    Ok(SkillDatabase::new(string_database, skills))
}
