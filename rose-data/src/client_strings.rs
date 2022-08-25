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

    pub _string_database: Arc<StringDatabase>,
}
