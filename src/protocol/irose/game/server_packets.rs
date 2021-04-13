use num_derive::FromPrimitive;

use crate::{
    game::{
        components::{
            BasicStats, CharacterInfo, Equipment, EquipmentIndex, Inventory, Level, Position,
        },
        data::items::{EquipmentItem, Item, ItemType, StackableItem},
    },
    protocol::packet::{Packet, PacketWriter},
};
use modular_bitfield::prelude::*;

#[derive(FromPrimitive)]
pub enum ServerPackets {
    ConnectReply = 0x70c,
    SelectCharacter = 0x715,
    CharacterInventory = 0x716,
    QuestData = 0x71b,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum ConnectResult {
    Ok = 0,
    Failed = 1,
    TimeOut = 2,
    InvalidPassword = 3,
    AlreadyLoggedIn = 4,
}

pub struct PacketConnectionReply {
    pub result: ConnectResult,
    pub packet_sequence_id: u32,
    pub pay_flags: u32,
}

impl From<&PacketConnectionReply> for Packet {
    fn from(packet: &PacketConnectionReply) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::ConnectReply as u16);
        writer.write_u8(packet.result as u8);
        writer.write_u32(packet.packet_sequence_id);
        writer.write_u32(packet.pay_flags);
        writer.into()
    }
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct tagPartITEM {
    item_number: B10,
    gem: B9,
    has_socket: bool,
    grade: B4,
}

fn write_part_item(writer: &mut PacketWriter, item: &Option<EquipmentItem>) {
    if let Some(item) = item {
        let part = tagPartITEM::new()
            .with_item_number(item.item_number)
            .with_gem(item.gem)
            .with_has_socket(item.has_socket)
            .with_grade(item.grade);
        for b in part.into_bytes().iter() {
            writer.write_u8(*b);
        }
        writer.write_u8(0);
    } else {
        writer.write_u32(0);
    }
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct tagBaseITEM_Equipment {
    item_type: B5,
    item_number: B10,
    is_crafted: bool,
    gem: B9,
    durability: B7,
    life: B10,
    has_socket: bool,
    is_appraised: bool,
    grade: B4,
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct tagBaseITEM_Stackable {
    item_type: B5,
    item_number: B10,
    #[skip]
    __: B1,
    quantity: B32,
}

fn write_full_equipment_item(writer: &mut PacketWriter, equipment: Option<&EquipmentItem>) {
    match equipment {
        Some(equipment) => {
            let item = tagBaseITEM_Equipment::new()
                .with_item_type(equipment.item_type as u8)
                .with_item_number(equipment.item_number as u16)
                .with_is_crafted(equipment.is_crafted)
                .with_gem(equipment.gem)
                .with_durability(equipment.durability)
                .with_life(equipment.life)
                .with_has_socket(equipment.has_socket)
                .with_is_appraised(equipment.is_appraised)
                .with_grade(equipment.grade);
            writer.write_bytes(&item.into_bytes());
        }
        _ => {
            writer.write_u16(0);
            writer.write_u32(0);
        }
    }
}

fn write_full_stackable_item(writer: &mut PacketWriter, stackable: Option<&StackableItem>) {
    match stackable {
        Some(stackable) => {
            let item = tagBaseITEM_Stackable::new()
                .with_item_type(stackable.item_type as u8)
                .with_item_number(stackable.item_number as u16)
                .with_quantity(stackable.quantity);
            writer.write_bytes(&item.into_bytes());
        }
        _ => {
            writer.write_u16(0);
            writer.write_u32(0);
        }
    }
}

fn write_full_item(writer: &mut PacketWriter, item: &Option<Item>) {
    match item {
        Some(Item::Equipment(equipment)) => {
            write_full_equipment_item(writer, Some(equipment));
        }
        Some(Item::Stackable(stackable)) => {
            write_full_stackable_item(writer, Some(stackable));
        }
        Some(Item::Money(money)) => {
            let item = tagBaseITEM_Stackable::new()
                .with_item_type(ItemType::Money as u8)
                .with_quantity(money.quantity);
            writer.write_bytes(&item.into_bytes());
        }
        _ => {
            writer.write_u16(0);
            writer.write_u32(0);
        }
    }
}

pub struct PacketServerSelectCharacter<'a> {
    pub character_info: &'a CharacterInfo,
    pub position: &'a Position,
    pub equipment: &'a Equipment,
    pub basic_stats: &'a BasicStats,
    pub level: &'a Level,
}

impl<'a> From<&'a PacketServerSelectCharacter<'a>> for Packet {
    fn from(packet: &'a PacketServerSelectCharacter<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::SelectCharacter as u16);
        let character_info = packet.character_info;
        writer.write_u8(character_info.gender);
        writer.write_u16(packet.position.zone);
        writer.write_f32(packet.position.x);
        writer.write_f32(packet.position.y);
        writer.write_u16(packet.position.respawn_zone);

        writer.write_u32(character_info.face as u32);
        writer.write_u32(character_info.hair as u32);

        // tagPartITEM * N
        let equipped_items = &packet.equipment.equipped_items;
        write_part_item(&mut writer, &equipped_items[EquipmentIndex::Head as usize]);
        write_part_item(&mut writer, &equipped_items[EquipmentIndex::Body as usize]);
        write_part_item(&mut writer, &equipped_items[EquipmentIndex::Hands as usize]);
        write_part_item(&mut writer, &equipped_items[EquipmentIndex::Feet as usize]);
        write_part_item(&mut writer, &equipped_items[EquipmentIndex::Face as usize]);
        write_part_item(&mut writer, &equipped_items[EquipmentIndex::Back as usize]);
        write_part_item(
            &mut writer,
            &equipped_items[EquipmentIndex::WeaponRight as usize],
        );
        write_part_item(
            &mut writer,
            &equipped_items[EquipmentIndex::WeaponLeft as usize],
        );

        // tagBasicInfo
        writer.write_u8(character_info.birth_stone);
        writer.write_u8(character_info.face as u8);
        writer.write_u8(character_info.hair as u8);
        writer.write_u16(character_info.job);
        writer.write_u8(0); // union
        writer.write_u8(0); // rank
        writer.write_u8(0); // fame

        // tagBasicAbility
        let basic_stats = packet.basic_stats;
        writer.write_u16(basic_stats.strength);
        writer.write_u16(basic_stats.dexterity);
        writer.write_u16(basic_stats.intelligence);
        writer.write_u16(basic_stats.concentration);
        writer.write_u16(basic_stats.charm);
        writer.write_u16(basic_stats.sense);

        // tagGrowAbility
        writer.write_u16(100); // HP
        writer.write_u16(100); // MP
        writer.write_u32(packet.level.xp as u32); // XP
        writer.write_u16(packet.level.level);
        writer.write_u16(0); // Stat points
        writer.write_u16(0); // Skill points
        writer.write_u8(100); // Body Size
        writer.write_u8(100); // Head Size
        writer.write_u32(0); // Penalty XP
        writer.write_u16(0); // Fame G
        writer.write_u16(0); // Fame B
        for i in 0..10 {
            writer.write_u16(0); // Union points
        }
        writer.write_u32(0); // Guild ID
        writer.write_u16(0); // Guild contribution
        writer.write_u8(0); // Guild pos
        writer.write_u16(0); // PK flag
        writer.write_u16(100); // Stamina

        for i in 0..40 {
            writer.write_u32(0); // seconds remaining
            writer.write_u16(0); // buff id
            writer.write_u16(0); // reserved
        }

        // tagSkillAbility
        for i in 0..120 {
            writer.write_u16(0); // skill id
        }

        // CHotIcons
        for i in 0..32 {
            writer.write_u16(0); // skill id
        }

        writer.write_u32(123); // client id
        writer.write_null_terminated_utf8(&character_info.name);
        writer.into()
    }
}

pub struct PacketServerCharacterInventory<'a> {
    pub equipment: &'a Equipment,
    pub inventory: &'a Inventory,
}

impl<'a> From<&'a PacketServerCharacterInventory<'a>> for Packet {
    fn from(packet: &'a PacketServerCharacterInventory<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::CharacterInventory as u16);
        let inventory = packet.inventory;
        let equipment = packet.equipment;
        writer.write_i64(inventory.money.0);

        for item in &equipment.equipped_items {
            write_full_equipment_item(&mut writer, item.as_ref());
        }

        for item in &inventory.equipment.slots {
            write_full_equipment_item(&mut writer, item.as_ref());
        }

        for item in &inventory.consumables.slots {
            write_full_stackable_item(&mut writer, item.as_ref());
        }

        for item in &inventory.materials.slots {
            write_full_stackable_item(&mut writer, item.as_ref());
        }

        for item in &inventory.vehicles.slots {
            write_full_equipment_item(&mut writer, item.as_ref());
        }

        for item in &equipment.equipped_ammo {
            write_full_stackable_item(&mut writer, item.as_ref());
        }

        for item in &equipment.equipped_vehicle {
            write_full_equipment_item(&mut writer, item.as_ref());
        }

        writer.into()
    }
}

pub struct PacketServerCharacterQuestData {}

impl From<&PacketServerCharacterQuestData> for Packet {
    fn from(packet: &PacketServerCharacterQuestData) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::QuestData as u16);

        for _ in 0..5 {
            writer.write_u16(0); // episode var
        }

        for _ in 0..3 {
            writer.write_u16(0); // job var
        }

        for _ in 0..7 {
            writer.write_u16(0); // planet var
        }

        for _ in 0..10 {
            writer.write_u16(0); // union var
        }

        for _ in 0..10 {
            // Quest data
            writer.write_u16(0); // quest id
            writer.write_u32(0); // seconds until expires
            for _ in 0..10 {
                writer.write_u16(0); // quest vars
            }
            writer.write_u32(0); // switches bitvec
            for _ in 0..6 {
                write_full_item(&mut writer, &None); // quest items
            }
        }

        for _ in 0..30 {
            write_full_item(&mut writer, &None); // wish list items
        }

        writer.into()
    }
}
