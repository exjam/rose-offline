use crate::game::data::formats::StbFile;

pub struct StbSkill(pub StbFile);

#[allow(dead_code)]
impl StbSkill {
    pub fn get_skill_tab_type(&self, row: usize) -> Option<i32> {
        self.0.try_get_int(row, 4)
    }

    // TODO: The rest of the owl
}
