use std::collections::HashMap;

use log::debug;
use num_traits::FromPrimitive;

use crate::{
    data::{
        formats::{FileReader, StbFile, VfsIndex},
        SkillAddAbility, SkillData, SkillDatabase, SkillPage, SkillType,
    },
    stb_column,
};

pub struct StbSkill(pub StbFile);

#[allow(dead_code)]
impl StbSkill {
    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    stb_column! { 1, get_base_skill_index, u32 }
    stb_column! { 2, get_skill_level, u32 }
    stb_column! { 3, get_levelup_skill_points, u32 }
    stb_column! { 4, get_page, u32 }
    stb_column! { 5, get_skill_type, u32 }
    stb_column! { 6, get_cast_range, u32 }
    stb_column! { 6, get_require_planet_index, u32 }
    stb_column! { 51, get_icon_number, u32 }

    pub fn get_add_ability(&self, id: usize) -> Vec<SkillAddAbility> {
        let mut add_ability = Vec::new();
        for i in 0..2 {
            if let Some(ability_type) = self
                .0
                .try_get_int(id, 21 + i * 3)
                .and_then(FromPrimitive::from_i32)
            {
                let ability_value = self.0.try_get_int(id, 22 + i * 3).unwrap_or(0);
                let ability_rate = self.0.try_get_int(id, 23 + i * 3).unwrap_or(0);
                if ability_rate != 0 {
                    add_ability.push(SkillAddAbility::Rate(ability_type, ability_rate));
                } else {
                    add_ability.push(SkillAddAbility::Value(ability_type, ability_value));
                }
            }
        }
        add_ability
    }
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
        add_ability: data.get_add_ability(id),
        skill_type: FromPrimitive::from_u32(data.get_skill_type(id).unwrap_or(0))
            .unwrap_or(SkillType::Unknown),
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

    debug!("Loaded {} skills", skills.len());
    Some(SkillDatabase::new(skills))
}
