use num_derive::FromPrimitive;

use crate::{
    data::EquipmentItem,
    game::{components::EquipmentIndex, messages::client::CharacterListItem},
    protocol::{Packet, PacketWriter},
};

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

pub struct PacketServerCharacterList<'a> {
    pub characters: &'a [CharacterListItem],
}

impl<'a> From<&'a PacketServerCharacterList<'a>> for Packet {
    fn from(packet: &'a PacketServerCharacterList<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::CharacterListReply as u16);
        writer.write_u8(packet.characters.len() as u8);

        for (slot, character) in packet.characters.iter().enumerate() {
            writer.write_null_terminated_utf8(&character.info.name);
            writer.write_u8(character.info.gender as u8);
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
                EquipmentIndex::WeaponLeft,
                EquipmentIndex::WeaponRight,
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
#[derive(Clone, Copy, Debug, PartialEq)]
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
