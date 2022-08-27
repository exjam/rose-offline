use std::sync::Arc;

use enum_map::enum_map;

use rose_data::{ItemType, StringDatabase};
use rose_file_readers::{StlFile, StlReadOptions, VirtualFilesystem};

use crate::{
    encode_ability_type, encode_item_class, encode_skill_target_filter, encode_skill_type,
};

pub fn get_string_database(
    vfs: &VirtualFilesystem,
    language: usize,
) -> Result<Arc<StringDatabase>, anyhow::Error> {
    let stl_read_options = StlReadOptions {
        language_filter: Some(vec![language]),
    };

    Ok(Arc::new(StringDatabase {
        language,
        encode_ability_type: Box::new(encode_ability_type),
        encode_item_class: Box::new(encode_item_class),
        encode_skill_target_filter: Box::new(encode_skill_target_filter),
        encode_skill_type: Box::new(encode_skill_type),
        ability: vfs
            .read_file_with::<StlFile, _>("3DDATA/STB/STR_ABILITY.STL", &stl_read_options)?,
        clan: vfs.read_file_with::<StlFile, _>("3DDATA/STB/STR_CLAN.STL", &stl_read_options)?,
        client_strings: vfs
            .read_file_with::<StlFile, _>("3DDATA/STB/LIST_STRING.STL", &stl_read_options)?,
        item: enum_map! {
            ItemType::Face => vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_FACEITEM_S.STL", &stl_read_options)?,
            ItemType::Head => vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_CAP_S.STL", &stl_read_options)?,
            ItemType::Body => vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_BODY_S.STL", &stl_read_options)?,
            ItemType::Hands => vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_ARMS_S.STL", &stl_read_options)?,
            ItemType::Feet => vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_FOOT_S.STL", &stl_read_options)?,
            ItemType::Back => vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_BACK_S.STL", &stl_read_options)?,
            ItemType::Jewellery => vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_JEWEL_S.STL", &stl_read_options)?,
            ItemType::Weapon => vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_WEAPON_S.STL", &stl_read_options)?,
            ItemType::SubWeapon => vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_SUBWPN_S.STL", &stl_read_options)?,
            ItemType::Consumable => vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_USEITEM_S.STL", &stl_read_options)?,
            ItemType::Gem => vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_JEMITEM_S.STL", &stl_read_options)?,
            ItemType::Material => vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_NATURAL_S.STL", &stl_read_options)?,
            ItemType::Quest => vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_QUESTITEM_S.STL", &stl_read_options)?,
            ItemType::Vehicle => vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_PAT_S.STL", &stl_read_options)?,
        },
        item_prefix: vfs
            .read_file_with::<StlFile, _>("3DDATA/STB/STR_ITEMPREFIX.STL", &stl_read_options)?,
        item_class: vfs
            .read_file_with::<StlFile, _>("3DDATA/STB/STR_ITEMTYPE.STL", &stl_read_options)?,
        job: vfs.read_file_with::<StlFile, _>("3DDATA/STB/STR_JOB.STL", &stl_read_options)?,
        job_class: vfs
            .read_file_with::<StlFile, _>("3DDATA/STB/LIST_CLASS_S.STL", &stl_read_options)?,
        npc: vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_NPC_S.STL", &stl_read_options)?,
        npc_store_tabs: vfs
            .read_file_with::<StlFile, _>("3DDATA/STB/LIST_SELL_S.STL", &stl_read_options)?,
        planet: vfs.read_file_with::<StlFile, _>("3DDATA/STB/STR_PLANET.STL", &stl_read_options)?,
        quest: vfs
            .read_file_with::<StlFile, _>("3DDATA/STB/LIST_QUEST_S.STL", &stl_read_options)?,
        skill: vfs
            .read_file_with::<StlFile, _>("3DDATA/STB/LIST_SKILL_S.STL", &stl_read_options)?,
        skill_target: vfs
            .read_file_with::<StlFile, _>("3DDATA/STB/STR_SKILLTARGET.STL", &stl_read_options)?,
        skill_type: vfs
            .read_file_with::<StlFile, _>("3DDATA/STB/STR_SKILLTYPE.STL", &stl_read_options)?,
        status_effect: vfs
            .read_file_with::<StlFile, _>("3DDATA/STB/LIST_STATUS_S.STL", &stl_read_options)?,
        union: vfs
            .read_file_with::<StlFile, _>("3DDATA/STB/LIST_UNION_S.STL", &stl_read_options)?,
        zone: vfs.read_file_with::<StlFile, _>("3DDATA/STB/LIST_ZONE_S.STL", &stl_read_options)?,
    }))
}
