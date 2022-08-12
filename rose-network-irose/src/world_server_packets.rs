use bevy::math::Vec3;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use rose_data::{EquipmentIndex, EquipmentItem, ItemReference, ZoneId};
use rose_game_common::{
    components::{CharacterDeleteTime, CharacterInfo, Equipment, Level},
    messages::server::CharacterListItem,
};
use rose_network_common::{Packet, PacketError, PacketReader, PacketWriter};

use crate::common_packets::{PacketReadCharacterGender, PacketWriteCharacterGender};

#[derive(FromPrimitive)]
pub enum ServerPackets {
    ConnectReply = 0x70c,
    CharacterListReply = 0x712,
    CreateCharacterReply = 0x713,
    DeleteCharacterReply = 0x714,
    MoveServer = 0x711,
    ReturnToCharacterSelect = 0x71c,
}

#[allow(dead_code)]
#[derive(Clone, Copy, FromPrimitive)]
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

impl TryFrom<&Packet> for PacketConnectionReply {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::ConnectReply as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let result = FromPrimitive::from_u8(reader.read_u8()?).ok_or(PacketError::InvalidPacket)?;
        let packet_sequence_id = reader.read_u32()?;
        let pay_flags = reader.read_u32()?;
        Ok(PacketConnectionReply {
            result,
            packet_sequence_id,
            pay_flags,
        })
    }
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

pub struct PacketServerCharacterList {
    pub characters: Vec<CharacterListItem>,
}

impl TryFrom<&Packet> for PacketServerCharacterList {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::CharacterListReply as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let num_characters = reader.read_u8()? as usize;
        let mut characters = Vec::with_capacity(num_characters);
        for _ in 0..num_characters {
            let name = reader.read_null_terminated_utf8()?.to_string();
            let gender = reader.read_character_gender_u8()?;
            let level = Level::new(reader.read_u16()? as u32);
            let job = reader.read_u16()?;
            let delete_time = match reader.read_u32()? {
                0 => None,
                seconds => Some(CharacterDeleteTime::from_seconds_remaining(seconds)),
            };
            let _is_premium = reader.read_u8()?;
            let face = reader.read_u16()? as u8;
            reader.read_u16()?;
            let hair = reader.read_u16()? as u8;
            reader.read_u16()?;

            let mut equipment = Equipment::new();
            for index in [
                EquipmentIndex::Head,
                EquipmentIndex::Body,
                EquipmentIndex::Hands,
                EquipmentIndex::Feet,
                EquipmentIndex::Face,
                EquipmentIndex::Back,
                EquipmentIndex::SubWeapon,
                EquipmentIndex::Weapon,
            ] {
                let item_number = reader.read_u16()? as usize;
                let grade = reader.read_u16()?;
                if item_number != 0 {
                    if let Some(mut item) =
                        EquipmentItem::new(ItemReference::new(index.into(), item_number), 0)
                    {
                        item.grade = grade as u8;
                        equipment.equip_item(item).ok();
                    }
                }
            }

            characters.push(CharacterListItem {
                info: CharacterInfo {
                    name,
                    gender,
                    race: 0,
                    birth_stone: 0,
                    job,
                    face,
                    hair,
                    // TODO: We should move some of this stuff out of CharacterInfo
                    rank: 0,
                    fame: 0,
                    fame_b: 0,
                    fame_g: 0,
                    revive_zone_id: ZoneId::new(1).unwrap(),
                    revive_position: Vec3::new(0.0, 0.0, 0.0),
                    unique_id: 0,
                },
                level,
                delete_time,
                equipment,
            });
        }

        Ok(PacketServerCharacterList { characters })
    }
}

impl From<&PacketServerCharacterList> for Packet {
    fn from(packet: &PacketServerCharacterList) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::CharacterListReply as u16);
        writer.write_u8(packet.characters.len() as u8);

        for (slot, character) in packet.characters.iter().enumerate() {
            writer.write_null_terminated_utf8(&character.info.name);
            writer.write_character_gender_u8(character.info.gender);
            writer.write_u16(character.level.level as u16);
            writer.write_u16(character.info.job);
            match &character.delete_time {
                Some(delete_time) => {
                    writer.write_u32(std::cmp::max(
                        delete_time.get_time_until_delete().as_secs() as u32,
                        1u32,
                    ));
                }
                None => {
                    writer.write_u32(0);
                }
            }
            writer.write_u8(if slot >= 3 { 1 } else { 0 });

            writer.write_u16(character.info.face as u16);
            writer.write_u16(0);
            writer.write_u16(character.info.hair as u16);
            writer.write_u16(0);

            for index in [
                EquipmentIndex::Head,
                EquipmentIndex::Body,
                EquipmentIndex::Hands,
                EquipmentIndex::Feet,
                EquipmentIndex::Face,
                EquipmentIndex::Back,
                EquipmentIndex::SubWeapon,
                EquipmentIndex::Weapon,
            ]
            .iter()
            {
                if let Some(&EquipmentItem { item, grade, .. }) =
                    character.equipment.get_equipment_item(*index)
                {
                    writer.write_u16(item.item_number as u16);
                    writer.write_u16(grade as u16);
                } else {
                    writer.write_u16(0);
                    writer.write_u16(0);
                }
            }
        }

        writer.into()
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive)]
pub enum CreateCharacterResult {
    Ok = 0,
    Failed = 1,
    NameAlreadyExists = 2,
    InvalidValue = 3,
    NoMoreSlots = 4,
    Blocked = 5,
}

pub struct PacketServerCreateCharacterReply {
    pub result: CreateCharacterResult,
    pub is_platinum: bool,
}

impl TryFrom<&Packet> for PacketServerCreateCharacterReply {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::CreateCharacterReply as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let result = FromPrimitive::from_u8(reader.read_u8()?).ok_or(PacketError::InvalidPacket)?;
        let is_platinum = reader.read_u8()? != 0;
        Ok(Self {
            result,
            is_platinum,
        })
    }
}

impl From<&PacketServerCreateCharacterReply> for Packet {
    fn from(packet: &PacketServerCreateCharacterReply) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::CreateCharacterReply as u16);
        writer.write_u8(packet.result as u8);
        writer.write_u8(if packet.is_platinum { 1 } else { 0 });
        writer.into()
    }
}

pub struct PacketServerDeleteCharacterReply<'a> {
    pub seconds_until_delete: Option<u32>,
    pub name: &'a str,
}

impl<'a> TryFrom<&'a Packet> for PacketServerDeleteCharacterReply<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::DeleteCharacterReply as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let seconds_until_delete = match reader.read_u32()? {
            0xFFFFFFFF => None,
            seconds_until_delete => Some(seconds_until_delete),
        };
        let name = reader.read_null_terminated_utf8()?;

        Ok(Self {
            seconds_until_delete,
            name,
        })
    }
}

impl<'a> From<&'a PacketServerDeleteCharacterReply<'a>> for Packet {
    fn from(packet: &'a PacketServerDeleteCharacterReply) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::DeleteCharacterReply as u16);
        match packet.seconds_until_delete {
            Some(seconds_until_delete) => writer.write_u32(seconds_until_delete),
            None => writer.write_u32(0xFFFFFFFF),
        }
        writer.write_null_terminated_utf8(packet.name);
        writer.into()
    }
}

pub struct PacketServerMoveServer<'a> {
    pub login_token: u32,
    pub packet_codec_seed: u32,
    pub ip: &'a str,
    pub port: u16,
}

impl<'a> TryFrom<&'a Packet> for PacketServerMoveServer<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::MoveServer as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let port = reader.read_u16()?;
        let login_token = reader.read_u32()?;
        let packet_codec_seed = reader.read_u32()?;
        let ip = reader.read_null_terminated_utf8()?;
        Ok(PacketServerMoveServer {
            login_token,
            packet_codec_seed,
            ip,
            port,
        })
    }
}

impl<'a> From<&PacketServerMoveServer<'a>> for Packet {
    fn from(packet: &PacketServerMoveServer) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::MoveServer as u16);
        writer.write_u16(packet.port);
        writer.write_u32(packet.login_token);
        writer.write_u32(packet.packet_codec_seed);
        writer.write_null_terminated_utf8(packet.ip);
        writer.into()
    }
}

pub struct PacketServerReturnToCharacterSelect {}

impl From<&PacketServerReturnToCharacterSelect> for Packet {
    fn from(_packet: &PacketServerReturnToCharacterSelect) -> Self {
        let writer = PacketWriter::new(ServerPackets::ReturnToCharacterSelect as u16);
        writer.into()
    }
}
