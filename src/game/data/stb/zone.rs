use crate::game::data::formats::StbFile;

pub struct StbZone(pub StbFile);

#[allow(dead_code)]
impl StbZone {
    pub fn get_zone_file(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 1)
    }

    pub fn get_zone_start_event_object_name(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 2)
    }

    pub fn get_zone_respawn_event_object_name(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 3)
    }

    pub fn get_zone_is_underground(&self, row: usize) -> Option<bool> {
        self.0.try_get_int(row, 4).map(|x| x != 0)
    }

    pub fn get_zone_background_music_day(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 5)
    }

    pub fn get_zone_background_music_night(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 6)
    }

    pub fn get_zone_skybox_index(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 7)
    }

    pub fn get_zone_minimap_filename(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 8)
    }

    pub fn get_zone_minimap_start_x(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 9)
    }

    pub fn get_zone_minimap_start_y(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 10)
    }

    pub fn get_zone_object_table(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 11)
    }

    pub fn get_zone_cnst_table(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 12)
    }

    pub fn get_zone_day_cycle_time(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 13)
    }

    pub fn get_zone_morning_time(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 14)
    }

    pub fn get_zone_day_time(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 15)
    }

    pub fn get_zone_evening_time(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 16)
    }

    pub fn get_zone_night_time(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 17)
    }

    pub fn get_zone_pvp_state(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 18)
    }

    pub fn get_zone_planet(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 19)
    }

    pub fn get_zone_footstep_type(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 20)
    }

    pub fn get_zone_camera_type(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 21)
    }

    pub fn get_zone_join_trigger(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 22)
    }

    pub fn get_zone_kill_trigger(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 23)
    }

    pub fn get_zone_dead_trigger(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 24)
    }

    pub fn get_zone_sector_size(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 25)
    }

    pub fn get_zone_string_id(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 26)
    }

    pub fn get_zone_weather_type(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 27)
    }

    pub fn get_zone_party_xp_a(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 28)
    }

    pub fn get_zone_party_xp_b(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 29)
    }

    pub fn get_zone_vehicle_use_flags(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 30)
    }

    pub fn get_zone_revive_zone_no(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 31)
    }

    pub fn get_zone_revive_pos_x(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 32)
    }

    pub fn get_zone_revive_pos_y(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 33)
    }
}
