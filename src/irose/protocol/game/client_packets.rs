use std::convert::{TryFrom, TryInto};

use modular_bitfield::{
    bitfield,
    prelude::{B14, B2},
};
use nalgebra::Point2;
use num_derive::FromPrimitive;

use crate::{
    data::{item::Item, MotionId, WarpGateId},
    game::{
        components::{
            AmmoIndex, BasicStatType, ClientEntityId, EquipmentIndex, HotbarSlot, ItemSlot,
            SkillSlot,
        },
        messages::client::{NpcStoreBuyItem, PartyReply, PartyRequest, ReviveRequestType},
    },
    irose::protocol::game::common_packets::{
        decode_ammo_index, decode_item_slot, PacketReadEquipmentIndex, PacketReadHotbarSlot,
        PacketReadItemSlot, PacketReadItems, PacketReadSkillSlot,
    },
    protocol::{Packet, PacketReader, ProtocolError},
};

#[derive(FromPrimitive)]
pub enum ClientPackets {
    LogoutRequest = 0x707,
    ConnectRequest = 0x70b,
    ReturnToCharacterSelectRequest = 0x71C,
    QuestRequest = 0x730,
    JoinZone = 0x753,
    ReviveRequest = 0x755,
    Emote = 0x781,
    MoveToggle = 0x782,
    Chat = 0x783,
    StopMove = 0x796,
    Attack = 0x798,
    Move = 0x79a,
    NpcStoreTransaction = 0x7a1,
    UseItem = 0x7a3,
    DropItemFromInventory = 0x7a4,
    ChangeEquipment = 0x7a5,
    PickupItemDrop = 0x7a7,
    WarpGateRequest = 0x7a8,
    IncreaseBasicStat = 0x7a9,
    SetHotbarSlot = 0x7aa,
    ChangeAmmo = 0x7ab,
    CastSkillSelf = 0x7b2,
    CastSkillTargetEntity = 0x7b3,
    CastSkillTargetPosition = 0x7b4,
    PersonalStoreListItems = 0x7c4,
    PersonalStoreBuyItem = 0x7c5,
    PartyRequest = 0x7d0,
    PartyReply = 0x7d1,
}

#[derive(Debug)]
pub struct PacketClientConnectRequest<'a> {
    pub login_token: u32,
    pub password_md5: &'a str,
}

impl<'a> TryFrom<&'a Packet> for PacketClientConnectRequest<'a> {
    type Error = ProtocolError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::ConnectRequest as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let login_token = reader.read_u32()?;
        let password_md5 = reader.read_fixed_length_utf8(32)?;
        Ok(PacketClientConnectRequest {
            login_token,
            password_md5,
        })
    }
}

#[derive(Debug)]
pub struct PacketClientJoinZone {
    pub weight_rate: u8,
    pub z: u16,
}

impl TryFrom<&Packet> for PacketClientJoinZone {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::JoinZone as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let weight_rate = reader.read_u8()?;
        let z = reader.read_u16()?;
        Ok(PacketClientJoinZone { weight_rate, z })
    }
}

#[derive(Debug)]
pub struct PacketClientMove {
    pub target_entity_id: Option<ClientEntityId>,
    pub x: f32,
    pub y: f32,
    pub z: u16,
}

impl TryFrom<&Packet> for PacketClientMove {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::Move as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let target_entity_id = reader.read_u16()?;
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let z = reader.read_u16()?;
        Ok(PacketClientMove {
            target_entity_id: if target_entity_id == 0 {
                None
            } else {
                Some(ClientEntityId(target_entity_id as usize))
            },
            x,
            y,
            z,
        })
    }
}

#[derive(Debug)]
pub struct PacketClientAttack {
    pub target_entity_id: ClientEntityId,
}

impl TryFrom<&Packet> for PacketClientAttack {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::Attack as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let target_entity_id = ClientEntityId(reader.read_u16()? as usize);
        Ok(PacketClientAttack { target_entity_id })
    }
}

#[derive(Debug)]
pub struct PacketClientChat<'a> {
    pub text: &'a str,
}

impl<'a> TryFrom<&'a Packet> for PacketClientChat<'a> {
    type Error = ProtocolError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::Chat as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let text = reader.read_null_terminated_utf8()?;
        Ok(PacketClientChat { text })
    }
}

#[derive(Debug)]
pub struct PacketClientSetHotbarSlot {
    pub slot_index: u8,
    pub slot: Option<HotbarSlot>,
}

impl TryFrom<&Packet> for PacketClientSetHotbarSlot {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::SetHotbarSlot as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let slot_index = reader.read_u8()?;
        let slot = reader.read_hotbar_slot()?;
        Ok(PacketClientSetHotbarSlot { slot_index, slot })
    }
}

pub struct PacketClientChangeEquipment {
    pub equipment_index: EquipmentIndex,
    pub item_slot: Option<ItemSlot>,
}

impl TryFrom<&Packet> for PacketClientChangeEquipment {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::ChangeEquipment as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let equipment_index = reader.read_equipment_index_u16()?;
        let item_slot = reader.read_item_slot_u16().ok();
        Ok(PacketClientChangeEquipment {
            equipment_index,
            item_slot,
        })
    }
}

pub struct PacketClientIncreaseBasicStat {
    pub basic_stat_type: BasicStatType,
}

impl TryFrom<&Packet> for PacketClientIncreaseBasicStat {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::IncreaseBasicStat as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let basic_stat_type = match reader.read_u8()? {
            0 => BasicStatType::Strength,
            1 => BasicStatType::Dexterity,
            2 => BasicStatType::Intelligence,
            3 => BasicStatType::Concentration,
            4 => BasicStatType::Charm,
            5 => BasicStatType::Sense,
            _ => return Err(ProtocolError::InvalidPacket),
        };
        Ok(PacketClientIncreaseBasicStat { basic_stat_type })
    }
}

#[derive(Debug)]
pub struct PacketClientPickupItemDrop {
    pub target_entity_id: ClientEntityId,
}

impl TryFrom<&Packet> for PacketClientPickupItemDrop {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::PickupItemDrop as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let target_entity_id = ClientEntityId(reader.read_u16()? as usize);
        Ok(PacketClientPickupItemDrop { target_entity_id })
    }
}

pub struct PacketClientReviveRequest {
    pub revive_request_type: ReviveRequestType,
}

impl TryFrom<&Packet> for PacketClientReviveRequest {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::ReviveRequest as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let revive_request_type = match reader.read_u8()? {
            1 => ReviveRequestType::RevivePosition,
            2 => ReviveRequestType::SavePosition,
            _ => return Err(ProtocolError::InvalidPacket),
        };

        Ok(PacketClientReviveRequest {
            revive_request_type,
        })
    }
}

pub enum PacketClientQuestRequestType {
    DeleteQuest,
    DoTrigger,
}

pub struct PacketClientQuestRequest {
    pub request_type: PacketClientQuestRequestType,
    pub quest_slot: u8,
    pub quest_id: u32,
}

impl TryFrom<&Packet> for PacketClientQuestRequest {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::QuestRequest as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let request_type = match reader.read_u8()? {
            2 => PacketClientQuestRequestType::DeleteQuest,
            3 => PacketClientQuestRequestType::DoTrigger,
            _ => return Err(ProtocolError::InvalidPacket),
        };
        let quest_slot = reader.read_u8()?;
        let quest_id = reader.read_u32()?;

        Ok(PacketClientQuestRequest {
            request_type,
            quest_slot,
            quest_id,
        })
    }
}

#[derive(Debug)]
pub struct PacketClientPersonalStoreListItems {
    pub target_entity_id: ClientEntityId,
}

impl TryFrom<&Packet> for PacketClientPersonalStoreListItems {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::PersonalStoreListItems as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let target_entity_id = ClientEntityId(reader.read_u16()? as usize);
        Ok(PacketClientPersonalStoreListItems { target_entity_id })
    }
}

#[derive(Debug)]
pub struct PacketClientPersonalStoreBuyItem {
    pub store_entity_id: ClientEntityId,
    pub store_slot_index: usize,
    pub buy_item: Item,
}

impl TryFrom<&Packet> for PacketClientPersonalStoreBuyItem {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::PersonalStoreBuyItem as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let store_entity_id = ClientEntityId(reader.read_u16()? as usize);

        // Although the packet supports multiple items, no one uses it
        // so to keep our code simpler we only support single item.
        let _item_count = reader.read_u8()?;

        let store_slot_index = reader.read_u8()? as usize;
        let buy_item = reader
            .read_item_full()?
            .ok_or(ProtocolError::InvalidPacket)?;

        Ok(PacketClientPersonalStoreBuyItem {
            store_entity_id,
            store_slot_index,
            buy_item,
        })
    }
}

#[derive(Debug)]
pub enum PacketClientDropItemFromInventory {
    Item(ItemSlot, u32),
    Money(u32),
}

impl TryFrom<&Packet> for PacketClientDropItemFromInventory {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::DropItemFromInventory as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        // A value of 0 for inventory_index is interpreted to mean dropping money.
        // PacketReader::read_item_slot_u8 returns ProtocolError for value 0 but in this case it is interpreted to mean dropping money.
        let inventory_index = reader.read_item_slot_u8();
        let quantity = reader.read_u32()?;
        match inventory_index {
            Ok(item_slot) => Ok(PacketClientDropItemFromInventory::Item(item_slot, quantity)),
            Err(_) => Ok(PacketClientDropItemFromInventory::Money(quantity)),
        }
    }
}

#[derive(Debug)]
pub struct PacketClientUseItem {
    pub item_slot: ItemSlot,
    pub target_entity_id: Option<ClientEntityId>,
}

impl TryFrom<&Packet> for PacketClientUseItem {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::UseItem as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let item_slot = reader.read_item_slot_u16()?;
        let target_entity_id = reader.read_u16().ok().map(|x| ClientEntityId(x as usize));

        Ok(PacketClientUseItem {
            item_slot,
            target_entity_id,
        })
    }
}

#[derive(Debug)]
pub struct PacketClientCastSkillSelf {
    pub skill_slot: SkillSlot,
}

impl TryFrom<&Packet> for PacketClientCastSkillSelf {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::CastSkillSelf as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let skill_slot = reader.read_skill_slot_u8()?;

        Ok(PacketClientCastSkillSelf { skill_slot })
    }
}

#[derive(Debug)]
pub struct PacketClientCastSkillTargetEntity {
    pub target_entity_id: ClientEntityId,
    pub skill_slot: SkillSlot,
}

impl TryFrom<&Packet> for PacketClientCastSkillTargetEntity {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::CastSkillTargetEntity as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let target_entity_id = ClientEntityId(reader.read_u16()? as usize);
        let skill_slot = reader.read_skill_slot_u8()?;

        Ok(PacketClientCastSkillTargetEntity {
            target_entity_id,
            skill_slot,
        })
    }
}

#[derive(Debug)]
pub struct PacketClientCastSkillTargetPosition {
    pub skill_slot: SkillSlot,
    pub position: Point2<f32>,
}

impl TryFrom<&Packet> for PacketClientCastSkillTargetPosition {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::CastSkillTargetPosition as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let skill_slot = reader.read_skill_slot_u8()?;
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;

        Ok(PacketClientCastSkillTargetPosition {
            skill_slot,
            position: Point2::new(x, y),
        })
    }
}

#[derive(Debug)]
pub struct PacketClientNpcStoreTransactionBuyItem {
    pub tab: u8,
    pub tab_item_index: u8,
    pub quantity: u16,
}

#[derive(Debug)]
pub struct PacketClientNpcStoreTransaction {
    pub npc_entity_id: ClientEntityId,
    pub buy_items: Vec<NpcStoreBuyItem>,
    pub sell_items: Vec<(ItemSlot, usize)>,
}

impl TryFrom<&Packet> for PacketClientNpcStoreTransaction {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::NpcStoreTransaction as u16 {
            return Err(ProtocolError::InvalidPacket);
        }
        let mut reader = PacketReader::from(packet);

        let npc_entity_id = ClientEntityId(reader.read_u16()? as usize);
        let buy_item_count = reader.read_u8()?;
        let sell_item_count = reader.read_u8()?;
        let _economy_time = reader.read_u32()?;
        let mut buy_items = Vec::new();
        let mut sell_items = Vec::new();

        for _ in 0..buy_item_count {
            let tab_index = reader.read_u8()? as usize;
            let item_index = reader.read_u8()? as usize;
            let quantity = reader.read_u16()? as usize;
            buy_items.push(NpcStoreBuyItem {
                tab_index,
                item_index,
                quantity,
            });
        }

        for _ in 0..sell_item_count {
            let item_slot = reader.read_item_slot_u8()?;
            let quantity = reader.read_u16()? as usize;
            sell_items.push((item_slot, quantity));
        }

        Ok(PacketClientNpcStoreTransaction {
            npc_entity_id,
            buy_items,
            sell_items,
        })
    }
}

#[bitfield]
#[derive(Clone, Copy)]
struct ChangeAmmoBits {
    #[skip(setters)]
    ammo_index: B2,
    #[skip(setters)]
    item_slot: B14,
}

#[derive(Debug)]
pub struct PacketClientChangeAmmo {
    pub ammo_index: AmmoIndex,
    pub item_slot: Option<ItemSlot>,
}

impl TryFrom<&Packet> for PacketClientChangeAmmo {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::ChangeAmmo as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let bytes = reader.read_fixed_length_bytes(2)?;
        let change_ammo = ChangeAmmoBits::from_bytes(bytes[0..2].try_into().unwrap());
        let item_slot = decode_item_slot(change_ammo.item_slot() as usize);
        let ammo_index = decode_ammo_index(change_ammo.ammo_index() as usize)
            .ok_or(ProtocolError::InvalidPacket)?;

        Ok(PacketClientChangeAmmo {
            ammo_index,
            item_slot,
        })
    }
}

pub enum PacketClientMoveToggleType {
    Run,
    Sit,
    Drive,
}

pub struct PacketClientMoveToggle {
    pub toggle_type: PacketClientMoveToggleType,
}

impl TryFrom<&Packet> for PacketClientMoveToggle {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::MoveToggle as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let toggle_type = match reader.read_u8()? {
            0 => PacketClientMoveToggleType::Run,
            1 => PacketClientMoveToggleType::Sit,
            2 => PacketClientMoveToggleType::Drive,
            _ => return Err(ProtocolError::InvalidPacket),
        };

        Ok(PacketClientMoveToggle { toggle_type })
    }
}

#[derive(Debug)]
pub struct PacketClientEmote {
    pub motion_id: MotionId,
    pub is_stop: bool,
}

impl TryFrom<&Packet> for PacketClientEmote {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::Emote as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let motion_id = MotionId::new(reader.read_u16()?);
        let is_stop = reader.read_u16()? != 0;

        Ok(PacketClientEmote { motion_id, is_stop })
    }
}

#[derive(Debug)]
pub struct PacketClientWarpGateRequest {
    pub warp_gate_id: WarpGateId,
}

impl TryFrom<&Packet> for PacketClientWarpGateRequest {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::WarpGateRequest as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let warp_gate_id = WarpGateId::new(reader.read_u16()?);
        Ok(PacketClientWarpGateRequest { warp_gate_id })
    }
}

#[derive(Debug)]
pub struct PacketClientPartyRequest {
    pub request: PartyRequest,
}

impl TryFrom<&Packet> for PacketClientPartyRequest {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::PartyRequest as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let request = match reader.read_u8()? {
            0 => PartyRequest::Create(ClientEntityId(reader.read_u16()? as usize)),
            1 => PartyRequest::Invite(ClientEntityId(reader.read_u16()? as usize)),
            2 => PartyRequest::Leave,
            3 => PartyRequest::ChangeOwner(ClientEntityId(reader.read_u16()? as usize)),
            0x81 => PartyRequest::Kick(reader.read_u32()?),
            _ => return Err(ProtocolError::InvalidPacket),
        };
        Ok(PacketClientPartyRequest { request })
    }
}

#[derive(Debug)]
pub struct PacketClientPartyReply {
    pub reply: PartyReply,
}

impl TryFrom<&Packet> for PacketClientPartyReply {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::PartyReply as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let reply = match reader.read_u8()? {
            1 => PartyReply::Busy(ClientEntityId(reader.read_u16()? as usize)),
            2 | 3 => PartyReply::Accept(ClientEntityId(reader.read_u16()? as usize)),
            4 => PartyReply::Reject(ClientEntityId(reader.read_u16()? as usize)),
            _ => return Err(ProtocolError::InvalidPacket),
        };
        Ok(PacketClientPartyReply { reply })
    }
}
