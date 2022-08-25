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
        _string_database: string_database,
    }))
}
