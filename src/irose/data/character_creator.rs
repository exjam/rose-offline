use nalgebra::Point3;

use crate::data::{
    formats::{FileReader, VfsIndex},
    SkillDatabase,
};

use crate::{
    data::{
        character::{CharacterCreator, CharacterCreatorError, CharacterStorage},
        formats::StbFile,
        item::{EquipmentItem, Item},
        SkillPage, SkillReference,
    },
    game::components::{
        BasicStats, CharacterInfo, Equipment, HealthPoints, Hotbar, Inventory, Level, ManaPoints,
        Position, SkillList,
    },
    stb_column,
};

use super::decode_item_reference;

struct CharacterGenderData {
    basic_stats: BasicStats,
    inventory_items: Vec<Item>,
    equipped_items: Vec<EquipmentItem>,
}

struct CharacterCreatorData {
    gender_data: Vec<CharacterGenderData>,
    skills: Vec<(SkillReference, SkillPage)>,
}

impl CharacterCreatorData {
    pub fn new(
        gender_data: Vec<CharacterGenderData>,
        skills: Vec<(SkillReference, SkillPage)>,
    ) -> Self {
        Self {
            gender_data,
            skills,
        }
    }
}

pub struct StbInitAvatar(pub StbFile);

impl StbInitAvatar {
    stb_column! { 0, get_strength, u16 }
    stb_column! { 1, get_dexterity, u16 }
    stb_column! { 2, get_intelligence, u16 }
    stb_column! { 3, get_concentration, u16 }
    stb_column! { 4, get_charm, u16 }
    stb_column! { 5, get_sense, u16 }

    pub fn get_basic_stats(&self, row: usize) -> Option<BasicStats> {
        Some(BasicStats {
            strength: self.get_strength(row)?,
            dexterity: self.get_dexterity(row)?,
            intelligence: self.get_intelligence(row)?,
            concentration: self.get_concentration(row)?,
            charm: self.get_charm(row)?,
            sense: self.get_sense(row)?,
        })
    }

    pub fn get_equipment(&self, row: usize) -> Vec<EquipmentItem> {
        let mut items = Vec::new();
        for i in 6..=13 {
            let item = self.0.try_get_int(row, i).unwrap_or(0) as u32;
            if let Some(item) = decode_item_reference(item)
                .ok()
                .and_then(|item| EquipmentItem::new(&item))
            {
                items.push(item);
            }
        }
        items
    }

    fn get_inventory_equipment(&self, row: usize) -> Vec<Item> {
        let mut items = Vec::new();
        for i in 0..10 {
            let item = self.0.try_get_int(row, 14 + i).unwrap_or(0) as u32;
            if let Some(item) = decode_item_reference(item)
                .ok()
                .and_then(|item| Item::new(&item, 1))
            {
                items.push(item);
            }
        }
        items
    }

    fn get_inventory_consumables(&self, row: usize) -> Vec<Item> {
        let mut items = Vec::new();
        for i in 0..5 {
            let item = self.0.try_get_int(row, 24 + i * 2).unwrap_or(0) as u32;
            let quantity = self.0.try_get_int(row, 25 + i * 2).unwrap_or(0) as u32;
            if let Some(item) = decode_item_reference(item)
                .ok()
                .and_then(|item| Item::new(&item, quantity))
            {
                items.push(item);
            }
        }
        items
    }

    fn get_inventory_materials(&self, row: usize) -> Vec<Item> {
        let mut items = Vec::new();
        for i in 0..5 {
            let item = self.0.try_get_int(row, 34 + i * 2).unwrap_or(0) as u32;
            let quantity = self.0.try_get_int(row, 35 + i * 2).unwrap_or(0) as u32;
            if let Some(item) = decode_item_reference(item)
                .ok()
                .and_then(|item| Item::new(&item, quantity))
            {
                items.push(item);
            }
        }
        items
    }

    pub fn get_inventory_items(&self, row: usize) -> Vec<Item> {
        let mut items = self.get_inventory_equipment(row);
        items.append(&mut self.get_inventory_consumables(row));
        items.append(&mut self.get_inventory_materials(row));
        items
    }
}

impl CharacterCreator for CharacterCreatorData {
    fn create(
        &self,
        name: String,
        gender: u8,
        birth_stone: u8,
        face: u8,
        hair: u8,
    ) -> Result<CharacterStorage, CharacterCreatorError> {
        let gender_data = self
            .gender_data
            .get(gender as usize)
            .ok_or(CharacterCreatorError::InvalidGender)?;

        let mut character = CharacterStorage {
            info: CharacterInfo {
                name,
                gender,
                birth_stone,
                job: 0,
                face,
                hair,
                respawn_zone: 20,
            },
            basic_stats: gender_data.basic_stats.clone(),
            equipment: Equipment::new(),
            inventory: Inventory::new(),
            level: Level::new(1),
            position: Position::new(Point3::new(530500.0, 539500.0, 0.0), 20),
            skill_list: SkillList::new(),
            hotbar: Hotbar::new(),
            delete_time: None,
            health_points: HealthPoints::new(50),
            mana_points: ManaPoints::new(40),
        };

        for (skill, page) in &self.skills {
            character.skill_list.add_skill(*skill, *page);
        }

        character
            .equipment
            .equip_items(gender_data.equipped_items.clone());

        for item in gender_data.inventory_items.clone() {
            character.inventory.try_add_item(item).ok();
        }

        Ok(character)
    }
}

fn load_gender(data: &StbInitAvatar, id: usize) -> Option<CharacterGenderData> {
    let basic_stats = data.get_basic_stats(id)?;
    let inventory_items = data.get_inventory_items(id);
    let equipped_items = data.get_equipment(id);

    Some(CharacterGenderData {
        basic_stats,
        inventory_items,
        equipped_items,
    })
}

pub fn get_character_creator(
    vfs: &VfsIndex,
    skill_database: &SkillDatabase,
) -> Option<Box<impl CharacterCreator + Send + Sync>> {
    let file = vfs.open_file("3DDATA/STB/INIT_AVATAR.STB")?;
    let data = StbInitAvatar(StbFile::read(FileReader::from(&file)).ok()?);
    let mut gender_data = Vec::new();
    for id in 0..data.0.rows() {
        if let Some(gender) = load_gender(&data, id) {
            gender_data.insert(id, gender);
        }
    }

    let mut skills = Vec::new();
    for skill in [11, 12, 16, 19, 20, 21]
        .iter()
        .map(|id| SkillReference(*id as usize))
    {
        if let Some(skill_data) = skill_database.get_skill(&skill) {
            skills.push((skill, skill_data.page));
        }
    }

    Some(Box::new(CharacterCreatorData::new(gender_data, skills)))
}