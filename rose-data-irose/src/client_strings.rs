use std::fmt::Write;
use std::sync::Arc;

use arrayvec::ArrayString;

use rose_data::{ClientStrings, StringDatabase};

pub fn get_client_strings(
    string_database: Arc<StringDatabase>,
) -> Result<Arc<ClientStrings>, anyhow::Error> {
    let get_string = |id: u16| -> &'static str {
        let mut key = ArrayString::<16>::new();
        write!(&mut key, "{}", id).ok();
        unsafe {
            std::mem::transmute(
                string_database
                    .client_strings
                    .get_text_string(string_database.language, &key)
                    .unwrap_or(""),
            )
        }
    };

    Ok(Arc::new(ClientStrings {
        equip_require_job: get_string(170),
        item_class: get_string(106),
        item_durability: get_string(434),
        item_life: get_string(433),
        item_quality: get_string(125),
        item_attack_range: get_string(110),
        item_attack_speed_fast: get_string(436),
        item_attack_speed_normal: get_string(435),
        item_attack_speed_slow: get_string(437),
        item_move_speed: get_string(171),
        item_weight: get_string(107),
        item_requires_appraisal: get_string(430),
        skill_level: get_string(313),
        skill_cast_range: get_string(309),
        skill_aoe_range: get_string(310),
        skill_cost_ability: get_string(319),
        skill_learn_point_cost: get_string(506),
        skill_recover_xp: get_string(272),
        skill_require_ability: get_string(323),
        skill_require_equipment: get_string(320),
        skill_require_job: get_string(321),
        skill_require_skill: get_string(322),
        skill_passive_ability: get_string(515),
        skill_power: get_string(317),
        skill_damage_type_0: get_string(80),
        skill_damage_type_1: get_string(81),
        skill_damage_type_2: get_string(82),
        skill_damage_type_3: get_string(83),
        skill_summon_point_cost: get_string(34),
        skill_steal_ability: get_string(514),
        skill_status_effects: get_string(516),
        skill_success_rate: get_string(318),
        skill_duration: get_string(314),
        skill_next_level_info: get_string(316),
        skill_target: get_string(307),
        skill_type: get_string(106),
        duration_seconds: get_string(315),
        bank_tab: get_string(344),
        bank_tab_premium: get_string(590),
        _string_database: string_database,
    }))
}
