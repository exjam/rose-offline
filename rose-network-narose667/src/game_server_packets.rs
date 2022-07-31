use std::num::NonZeroUsize;

use bevy::math::Vec3;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use rose_data::{ItemType, WorldTicks, ZoneId};
use rose_game_common::{
    components::{
        BasicStats, CharacterInfo, Equipment, ExperiencePoints, HealthPoints, Hotbar, Level,
        ManaPoints, SkillList, SkillPoints, Stamina, StatPoints, Team, UnionMembership,
    },
    messages::ClientEntityId,
};
use rose_network_common::{Packet, PacketError, PacketReader, PacketWriter};

use crate::common_packets::{PacketReadCharacterGender, PacketReadItems};

#[derive(FromPrimitive)]
pub enum ServerPackets {
    ConnectReply = 0x70c,
    SelectCharacter = 0x715,
    JoinZone = 0x753,
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

pub struct PacketServerSelectCharacter {
    pub character_info: CharacterInfo,
    pub position: Vec3,
    pub zone_id: ZoneId,
    pub equipment: Equipment,
    pub basic_stats: BasicStats,
    pub level: Level,
    pub experience_points: ExperiencePoints,
    pub skill_list: SkillList,
    pub hotbar: Hotbar,
    pub health_points: HealthPoints,
    pub mana_points: ManaPoints,
    pub stat_points: StatPoints,
    pub skill_points: SkillPoints,
    pub union_membership: UnionMembership,
    pub stamina: Stamina,
}

impl TryFrom<&Packet> for PacketServerSelectCharacter {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::SelectCharacter as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut union_membership = UnionMembership::default();

        let mut reader = PacketReader::from(packet);
        let gender = reader.read_character_gender_u8()?;

        let zone_id = ZoneId::new(reader.read_u32()? as u16).ok_or(PacketError::InvalidPacket)?;
        let position_x = reader.read_f32()?;
        let position_y = reader.read_f32()?;
        let revive_zone_id =
            ZoneId::new(reader.read_u32()? as u16).ok_or(PacketError::InvalidPacket)?;

        let _face = reader
            .read_equipment_item_part(ItemType::Face)?
            .map_or(0, |i| i.item.item_number);
        let _head = reader
            .read_equipment_item_part(ItemType::Head)?
            .map_or(0, |i| i.item.item_number);
        let equipment = reader.read_equipment_visible_part()?;

        // tagBasicInfo
        let _hair_color = reader.read_u8()?;
        let face = reader.read_u8()?;
        let hair = reader.read_u8()?;
        let job = reader.read_u16()?;
        union_membership.current_union = NonZeroUsize::new(reader.read_u8()? as usize);
        let rank = reader.read_u8()?;
        let fame = reader.read_u8()?;

        // tagBasicAbility
        let strength = reader.read_u16()? as i32;
        let dexterity = reader.read_u16()? as i32;
        let intelligence = reader.read_u16()? as i32;
        let concentration = reader.read_u16()? as i32;
        let charm = reader.read_u16()? as i32;
        let sense = reader.read_u16()? as i32;

        // tagGrowAbility
        let health_points = HealthPoints::new(reader.read_i32()?);
        let mana_points = ManaPoints::new(reader.read_i32()?);
        let experience_points = ExperiencePoints::new(reader.read_u32()? as u64);
        let level = Level::new(reader.read_u16()? as u32);
        let stat_points = StatPoints::new(reader.read_u16()? as u32);
        let skill_points = SkillPoints::new(reader.read_u16()? as u32);
        let _penalty_xp = reader.read_u32()?;
        let stamina = Stamina::new(reader.read_u16()? as u32);
        let _pat_hp = reader.read_u32()?;
        let _pat_cooldown = reader.read_u32()?;

        // tagSkillAbility
        let skill_list = SkillList::default(); // TODO: Read skill list
        for _ in 0..90 {
            reader.read_u32()?;
        }

        // CHotIcons
        let hotbar = Hotbar::default(); // TODO: Read hotbar
        for _ in 0..48 {
            reader.read_u16()?;
        }

        let unique_id = reader.read_u32()?;

        for _ in 0..20 {
            reader.read_u32()?; // TODO: Read cooldowns
        }

        let name = reader.read_null_terminated_utf8()?.to_string();

        Ok(PacketServerSelectCharacter {
            character_info: CharacterInfo {
                name,
                gender,
                race: 0,
                birth_stone: 0,
                job,
                face,
                hair,
                rank,
                fame,
                fame_b: 0,
                fame_g: 0,
                revive_zone_id,
                revive_position: Vec3::new(0.0, 0.0, 0.0),
                unique_id,
            },
            position: Vec3::new(position_x, position_y, 0.0),
            zone_id,
            equipment,
            basic_stats: BasicStats {
                strength,
                dexterity,
                intelligence,
                concentration,
                charm,
                sense,
            },
            level,
            experience_points,
            skill_list,
            hotbar,
            health_points,
            mana_points,
            stat_points,
            skill_points,
            union_membership,
            stamina,
        })
    }
}

pub struct PacketServerJoinZone {
    pub entity_id: ClientEntityId,
    pub experience_points: ExperiencePoints,
    pub team: Team,
    pub health_points: HealthPoints,
    pub mana_points: ManaPoints,
    pub world_ticks: WorldTicks,
    pub craft_rate: i32,
    pub world_price_rate: i32,
    pub item_price_rate: i32,
    pub town_price_rate: i32,
}

impl TryFrom<&Packet> for PacketServerJoinZone {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::JoinZone as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = ClientEntityId(reader.read_u16()? as usize);
        let health_points = HealthPoints::new(reader.read_u16()? as i32);
        let mana_points = ManaPoints::new(reader.read_u16()? as i32);

        let experience_points = ExperiencePoints::new(reader.read_u32()? as u64);
        let _penalty_xp = reader.read_u32();

        // tagVAR_GLOBAL
        // TODO: This is not correct, but it doesn't break anything right now
        let craft_rate = reader.read_u16()? as i32;
        let _update_time = reader.read_u32()?;
        let world_price_rate = reader.read_u16()? as i32;
        let town_price_rate = reader.read_u8()? as i32;
        let item_price_rate_0 = reader.read_u8()? as i32;
        for _ in 1..11 {
            let _item_price_rate_n = reader.read_u8()?;
        }
        let _global_flags = reader.read_u32()?;

        let world_ticks = WorldTicks(reader.read_u32()? as u64);
        let team = Team::new(reader.read_u32()?);

        Ok(PacketServerJoinZone {
            entity_id,
            experience_points,
            team,
            health_points,
            mana_points,
            world_ticks,
            craft_rate,
            world_price_rate,
            town_price_rate,
            item_price_rate: item_price_rate_0,
        })
    }
}
