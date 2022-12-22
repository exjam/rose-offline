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
        invalid_name: get_string(348),
        duration_seconds: get_string(315),

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

        bank_tab: get_string(344),
        bank_tab_premium: get_string(590),

        clan_name: get_string(44),
        clan_level: get_string(45),
        clan_point: get_string(46),
        clan_slogan: get_string(47),
        clan_money: get_string(48),
        clan_ally: get_string(49),
        clan_member_contribution: get_string(51),
        clan_member_count: get_string(53),
        clan_promote_error: get_string(54),
        clan_created: get_string(55),
        clan_joined: get_string(56),
        clan_destroy_success: get_string(57),
        clan_create_error: get_string(58),
        clan_create_error_name: get_string(59),
        clan_create_error_permission: get_string(60),
        clan_destroyed: get_string(61),
        clan_destroy_error: get_string(62),
        clan_destroy_error_permission: get_string(63),
        clan_join_member_accepted: get_string(64),
        clan_join_error: get_string(65),
        clan_join_error_permission: get_string(66),
        clan_join_error_already_in_clan: get_string(67),
        clan_kick_success: get_string(68),
        clan_kicked: get_string(69),
        clan_quit: get_string(70),
        clan_invited: get_string(71),
        clan_invite_rejected: get_string(72),
        clan_create_error_condition: get_string(77),
        clan_create_conditions: get_string(98),
        clan_create_error_slogan: get_string(78),
        clan_error_permission: get_string(76),

        _string_database: string_database,
    }))
}
