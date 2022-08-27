use std::sync::Arc;

use crate::StringDatabase;

pub struct ClientStrings {
    pub equip_require_job: &'static str,
    pub item_class: &'static str,
    pub item_durability: &'static str,
    pub item_life: &'static str,
    pub item_quality: &'static str,
    pub item_attack_range: &'static str,
    pub item_attack_speed_fast: &'static str,
    pub item_attack_speed_normal: &'static str,
    pub item_attack_speed_slow: &'static str,
    pub item_move_speed: &'static str,
    pub item_weight: &'static str,
    pub item_requires_appraisal: &'static str,

    pub skill_level: &'static str,
    pub skill_damage_type_0: &'static str,
    pub skill_damage_type_1: &'static str,
    pub skill_damage_type_2: &'static str,
    pub skill_damage_type_3: &'static str,
    pub skill_cast_range: &'static str,
    pub skill_aoe_range: &'static str,
    pub skill_cost_ability: &'static str,
    pub skill_learn_point_cost: &'static str,
    pub skill_require_ability: &'static str,
    pub skill_summon_point_cost: &'static str,
    pub skill_steal_ability: &'static str,
    pub skill_require_equipment: &'static str,
    pub skill_require_job: &'static str,
    pub skill_require_skill: &'static str,
    pub skill_status_effects: &'static str,
    pub skill_success_rate: &'static str,
    pub skill_duration: &'static str,
    pub skill_recover_xp: &'static str,
    pub skill_passive_ability: &'static str,
    pub skill_next_level_info: &'static str,
    pub skill_power: &'static str,
    pub skill_target: &'static str,
    pub skill_type: &'static str,

    pub duration_seconds: &'static str,

    pub _string_database: Arc<StringDatabase>,
}
