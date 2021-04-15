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
    JoinZone = 0x753,
    MoveEntity = 0x79a,
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
pub struct PacketEquipmentItemPart {
    #[skip(getters)]
    item_number: B10,
    #[skip(getters)]
    gem: B9,
    #[skip(getters)]
    has_socket: bool,
    #[skip(getters)]
    grade: B4,
}

fn write_equipment_item_part(writer: &mut PacketWriter, item: &Option<EquipmentItem>) {
    if let Some(item) = item {
        let part = PacketEquipmentItemPart::new()
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
pub struct PacketEquipmentItemFull {
    #[skip(getters)]
    item_type: B5,
    #[skip(getters)]
    item_number: B10,
    #[skip(getters)]
    is_crafted: bool,
    #[skip(getters)]
    gem: B9,
    #[skip(getters)]
    durability: B7,
    #[skip(getters)]
    life: B10,
    #[skip(getters)]
    has_socket: bool,
    #[skip(getters)]
    is_appraised: bool,
    #[skip(getters)]
    grade: B4,
}

fn write_equipment_item_full(writer: &mut PacketWriter, equipment: Option<&EquipmentItem>) {
    match equipment {
        Some(equipment) => {
            let item = PacketEquipmentItemFull::new()
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

#[bitfield]
#[derive(Clone, Copy)]
pub struct PacketStackableItemFull {
    #[skip(getters)]
    item_type: B5,
    #[skip(getters)]
    item_number: B10,
    #[skip]
    __: B1,
    #[skip(getters)]
    quantity: B32,
}

fn write_stackable_item_full(writer: &mut PacketWriter, stackable: Option<&StackableItem>) {
    match stackable {
        Some(stackable) => {
            let item = PacketStackableItemFull::new()
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
            write_equipment_item_full(writer, Some(equipment));
        }
        Some(Item::Stackable(stackable)) => {
            write_stackable_item_full(writer, Some(stackable));
        }
        Some(Item::Money(money)) => {
            let item = PacketStackableItemFull::new()
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
        writer.write_f32(packet.position.position.x);
        writer.write_f32(packet.position.position.y);
        writer.write_u16(packet.position.respawn_zone);

        writer.write_u32(character_info.face as u32);
        writer.write_u32(character_info.hair as u32);

        // tagPartITEM * N
        let equipped_items = &packet.equipment.equipped_items;
        write_equipment_item_part(&mut writer, &equipped_items[EquipmentIndex::Head as usize]);
        write_equipment_item_part(&mut writer, &equipped_items[EquipmentIndex::Body as usize]);
        write_equipment_item_part(&mut writer, &equipped_items[EquipmentIndex::Hands as usize]);
        write_equipment_item_part(&mut writer, &equipped_items[EquipmentIndex::Feet as usize]);
        write_equipment_item_part(&mut writer, &equipped_items[EquipmentIndex::Face as usize]);
        write_equipment_item_part(&mut writer, &equipped_items[EquipmentIndex::Back as usize]);
        write_equipment_item_part(
            &mut writer,
            &equipped_items[EquipmentIndex::WeaponRight as usize],
        );
        write_equipment_item_part(
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
        for _ in 0..10 {
            writer.write_u16(0); // Union points
        }
        writer.write_u32(0); // Guild ID
        writer.write_u16(0); // Guild contribution
        writer.write_u8(0); // Guild pos
        writer.write_u16(0); // PK flag
        writer.write_u16(100); // Stamina

        for _ in 0..40 {
            writer.write_u32(0); // seconds remaining
            writer.write_u16(0); // buff id
            writer.write_u16(0); // reserved
        }

        // tagSkillAbility
        for _ in 0..120 {
            writer.write_u16(0); // skill id
        }

        // CHotIcons
        for _ in 0..32 {
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
            write_equipment_item_full(&mut writer, item.as_ref());
        }

        for item in &inventory.equipment.slots {
            write_equipment_item_full(&mut writer, item.as_ref());
        }

        for item in &inventory.consumables.slots {
            write_stackable_item_full(&mut writer, item.as_ref());
        }

        for item in &inventory.materials.slots {
            write_stackable_item_full(&mut writer, item.as_ref());
        }

        for item in &inventory.vehicles.slots {
            write_equipment_item_full(&mut writer, item.as_ref());
        }

        for item in &equipment.equipped_ammo {
            write_stackable_item_full(&mut writer, item.as_ref());
        }

        for item in &equipment.equipped_vehicle {
            write_equipment_item_full(&mut writer, item.as_ref());
        }

        writer.into()
    }
}

pub struct PacketServerCharacterQuestData {}

impl From<&PacketServerCharacterQuestData> for Packet {
    fn from(_packet: &PacketServerCharacterQuestData) -> Self {
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

pub struct PacketMoveEntity {
    pub entity_id: u16,
    pub target_entity_id: u16,
    pub distance: u16,
    pub x: f32,
    pub y: f32,
    pub z: u16,
}

impl From<&PacketMoveEntity> for Packet {
    fn from(packet: &PacketMoveEntity) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::MoveEntity as u16);
        writer.write_u16(packet.entity_id);
        writer.write_u16(packet.target_entity_id);
        writer.write_u16(packet.distance);
        writer.write_f32(packet.x);
        writer.write_f32(packet.y);
        writer.write_u16(packet.z);
        writer.into()
    }
}

pub struct PacketServerJoinZone<'a> {
    pub entity_id: u16,
    pub level: &'a Level,
}

impl<'a> From<&'a PacketServerJoinZone<'a>> for Packet {
    fn from(packet: &'a PacketServerJoinZone<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::JoinZone as u16);
        writer.write_u16(packet.entity_id);
        writer.write_u16(100); // hp
        writer.write_u16(50); // mp

        writer.write_u32(packet.level.xp as u32);
        writer.write_u32(0); // penalty xp

        // tagVAR_GLOBAL
        writer.write_u16(100); // craft rate
        writer.write_u32(0); // update time
        writer.write_u16(100); // world price rate
        writer.write_u8(100); // town rate
        for _ in 0..11 {
            writer.write_u8(100); // item rate
        }
        writer.write_u32(0); // global flags (0x1 = pvp allowed)

        writer.write_u32(0); // account world time
        writer.write_u32(0); // team number
        writer.into()
    }
}
