use std::collections::HashMap;

use crate::{
    data::{
        formats::{FileReader, StbFile, VfsIndex},
        SkillData, SkillDatabase, SkillPage,
    },
    stb_column,
};

pub struct StbSkill(pub StbFile);

#[allow(dead_code)]
impl StbSkill {
    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    stb_column! { 4, get_page, u32 }
    stb_column! { 51, get_icon_number, u32 }
}

fn decode_skill_page(value: u32) -> Option<SkillPage> {
    match value {
        0 => Some(SkillPage::Basic),
        1 => Some(SkillPage::Active),
        2 => Some(SkillPage::Passive),
        3 => Some(SkillPage::Clan),
        _ => None,
    }
}

fn load_skill(data: &StbSkill, id: usize) -> Option<SkillData> {
    let icon_number = data.get_icon_number(id).unwrap_or(0);
    if icon_number == 0 {
        return None;
    }

    Some(SkillData {
        page: decode_skill_page(data.get_page(id)?)?,
        icon_number,
    })
}

pub fn get_skill_database(vfs: &VfsIndex) -> Option<SkillDatabase> {
    let file = vfs.open_file("3DDATA/STB/LIST_SKILL.STB")?;
    let data = StbSkill(StbFile::read(FileReader::from(&file)).ok()?);
    let mut skills = HashMap::new();
    for id in 0..data.rows() {
        if let Some(skill) = load_skill(&data, id) {
            skills.insert(id as u16, skill);
        }
    }
    Some(SkillDatabase::new(skills))
}
