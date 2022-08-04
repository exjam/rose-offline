use bevy::math::Vec3;
use enum_map::EnumMap;
use rose_game_common::components::CharacterGender;
use std::sync::Arc;

use rose_data::{
    EquipmentItem, ItemDatabase, ItemReference, QuestTriggerHash, SkillDatabase, SkillId,
    StackableItem, ZoneDatabase, ZoneId,
};
use rose_data_irose::decode_item_base1000;
use rose_file_readers::{stb_column, StbFile, VirtualFilesystem};

use crate::game::{
    components::{
        BasicStats, CharacterInfo, Equipment, ExperiencePoints, HealthPoints, Hotbar, Inventory,
        Level, ManaPoints, Position, QuestState, SkillList, SkillPoints, Stamina, StatPoints,
        UnionMembership,
    },
    storage::character::{CharacterCreator, CharacterCreatorError, CharacterStorage},
};

struct CharacterGenderData {
    basic_stats: BasicStats,
    equipped_items: Vec<ItemReference>,
    inventory_equipment: Vec<ItemReference>,
    inventory_consumables: Vec<(ItemReference, usize)>,
    inventory_materials: Vec<(ItemReference, usize)>,
}

struct CharacterCreatorData {
    item_database: Arc<ItemDatabase>,
    skill_database: Arc<SkillDatabase>,
    gender_data: EnumMap<CharacterGender, CharacterGenderData>,
    skills: Vec<SkillId>,
    start_position: Position,
    revive_position: Position,
}

pub struct StbInitAvatar(pub StbFile);

impl StbInitAvatar {
    stb_column! { 0, get_strength, i32 }
    stb_column! { 1, get_dexterity, i32 }
    stb_column! { 2, get_intelligence, i32 }
    stb_column! { 3, get_concentration, i32 }
    stb_column! { 4, get_charm, i32 }
    stb_column! { 5, get_sense, i32 }

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

    pub fn get_equipment(&self, row: usize) -> Vec<ItemReference> {
        let mut items = Vec::new();
        for i in 6..=13 {
            let item = self.0.try_get_int(row, i).unwrap_or(0) as usize;
            if let Some(item) = decode_item_base1000(item) {
                items.push(item);
            }
        }
        items
    }

    fn get_inventory_equipment(&self, row: usize) -> Vec<ItemReference> {
        let mut items = Vec::new();
        for i in 0..10 {
            let item = self.0.try_get_int(row, 14 + i).unwrap_or(0) as usize;
            if let Some(item) = decode_item_base1000(item) {
                items.push(item);
            }
        }
        items
    }

    fn get_inventory_consumables(&self, row: usize) -> Vec<(ItemReference, usize)> {
        let mut items = Vec::new();
        for i in 0..5 {
            let item = self.0.try_get_int(row, 24 + i * 2).unwrap_or(0) as usize;
            let quantity = self.0.try_get_int(row, 25 + i * 2).unwrap_or(0) as usize;
            if let Some(item) = decode_item_base1000(item) {
                items.push((item, quantity));
            }
        }
        items
    }

    fn get_inventory_materials(&self, row: usize) -> Vec<(ItemReference, usize)> {
        let mut items = Vec::new();
        for i in 0..5 {
            let item = self.0.try_get_int(row, 34 + i * 2).unwrap_or(0) as usize;
            let quantity = self.0.try_get_int(row, 35 + i * 2).unwrap_or(0) as usize;
            if let Some(item) = decode_item_base1000(item) {
                items.push((item, quantity));
            }
        }
        items
    }
}

impl CharacterCreator for CharacterCreatorData {
    fn create(
        &self,
        name: String,
        gender: CharacterGender,
        birth_stone: u8,
        face: u8,
        hair: u8,
    ) -> Result<CharacterStorage, CharacterCreatorError> {
        let gender_data = &self.gender_data[gender];

        // TODO: For now we just make a hash of name to use as unique id
        let unique_id = QuestTriggerHash::from(name.as_str()).hash;

        let mut character = CharacterStorage {
            info: CharacterInfo {
                name,
                unique_id,
                gender,
                race: 0,
                birth_stone,
                job: 0,
                face,
                hair,
                revive_zone_id: self.revive_position.zone_id,
                revive_position: self.revive_position.position,
                fame: 0,
                fame_b: 0,
                fame_g: 0,
                rank: 0,
            },
            basic_stats: gender_data.basic_stats.clone(),
            equipment: Equipment::default(),
            inventory: Inventory::default(),
            level: Level::new(1),
            experience_points: ExperiencePoints::default(),
            position: self.start_position.clone(),
            skill_list: SkillList::default(),
            hotbar: Hotbar::default(),
            delete_time: None,
            health_points: HealthPoints::new(0),
            mana_points: ManaPoints::new(0),
            stat_points: StatPoints::default(),
            skill_points: SkillPoints::default(),
            quest_state: QuestState::default(),
            union_membership: UnionMembership::default(),
            stamina: Stamina::default(),
        };

        for &skill_id in &self.skills {
            if let Some(skill_data) = self.skill_database.get_skill(skill_id) {
                character.skill_list.add_skill(skill_data);
            }
        }

        for item_reference in gender_data.equipped_items.iter().cloned() {
            if let Some(item_data) = self.item_database.get_base_item(item_reference) {
                if let Some(item) = EquipmentItem::from_item_data(item_data) {
                    character.equipment.equip_item(item).ok();
                }
            }
        }

        for item_reference in gender_data.inventory_equipment.iter().cloned() {
            if let Some(item_data) = self.item_database.get_base_item(item_reference) {
                if let Some(item) = EquipmentItem::from_item_data(item_data) {
                    character.inventory.try_add_item(item.into()).ok();
                }
            }
        }

        for (item_reference, quantity) in gender_data.inventory_consumables.iter().cloned() {
            if let Some(item_data) = self.item_database.get_base_item(item_reference) {
                if let Some(item) = StackableItem::from_item_data(item_data, quantity as u32) {
                    character.inventory.try_add_item(item.into()).ok();
                }
            }
        }

        for (item_reference, quantity) in gender_data.inventory_materials.iter().cloned() {
            if let Some(item_data) = self.item_database.get_base_item(item_reference) {
                if let Some(item) = StackableItem::from_item_data(item_data, quantity as u32) {
                    character.inventory.try_add_item(item.into()).ok();
                }
            }
        }

        Ok(character)
    }

    fn get_basic_stats(
        &self,
        gender: CharacterGender,
    ) -> Result<BasicStats, CharacterCreatorError> {
        let gender_data = &self.gender_data[gender];
        Ok(gender_data.basic_stats.clone())
    }
}

fn load_gender(data: &StbInitAvatar, id: usize) -> Option<CharacterGenderData> {
    Some(CharacterGenderData {
        basic_stats: data.get_basic_stats(id)?,
        equipped_items: data.get_equipment(id),
        inventory_consumables: data.get_inventory_consumables(id),
        inventory_equipment: data.get_inventory_equipment(id),
        inventory_materials: data.get_inventory_materials(id),
    })
}

pub fn get_character_creator(
    vfs: &VirtualFilesystem,
    item_database: Arc<ItemDatabase>,
    skill_database: Arc<SkillDatabase>,
    zone_database: &ZoneDatabase,
) -> Option<Box<impl CharacterCreator + Send + Sync>> {
    let data = StbInitAvatar(
        vfs.read_file::<StbFile, _>("3DDATA/STB/INIT_AVATAR.STB")
            .ok()?,
    );
    let gender_data = EnumMap::from_array([
        load_gender(&data, 0).unwrap(),
        load_gender(&data, 1).unwrap(),
    ]);
    let skills = vec![
        SkillId::new(11).unwrap(), // Sit
        SkillId::new(12).unwrap(), // Pick Up
        SkillId::new(16).unwrap(), // Attack
        SkillId::new(20).unwrap(), // Trade
    ];

    let start_zone = ZoneId::new(20).unwrap();
    let zone_data = zone_database
        .get_zone(start_zone)
        .expect("Could not find start zone");

    let revive_position = zone_data
        .get_closest_revive_position(zone_data.start_position)
        .unwrap_or(zone_data.start_position);
    let start_position = Vec3::new(530500.0, 539500.0, 0.0);

    Some(Box::new(CharacterCreatorData {
        item_database,
        skill_database,
        gender_data,
        skills,
        start_position: Position::new(start_position, start_zone),
        revive_position: Position::new(revive_position, start_zone),
    }))
}
