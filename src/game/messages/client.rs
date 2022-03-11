use nalgebra::Point2;

use rose_data::{
    AmmoIndex, EquipmentIndex, Item, MotionId, QuestTriggerHash, VehiclePartIndex, WarpGateId,
};
use rose_game_common::components::CharacterGender;

use crate::game::components::{
    BasicStatType, CharacterUniqueId, ClientEntityId, HotbarSlot, ItemSlot, SkillSlot,
};

#[derive(Debug)]
pub struct ConnectionRequest {
    pub login_token: u32,
    pub password_md5: String,
}

#[derive(Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password_md5: String,
}

#[derive(Debug)]
pub struct GetChannelList {
    pub server_id: usize,
}

#[derive(Debug)]
pub struct JoinServer {
    pub server_id: u32,
    pub channel_id: u8,
}

#[derive(Debug)]
pub struct CreateCharacter {
    pub gender: CharacterGender,
    pub birth_stone: u8,
    pub hair: u8,
    pub face: u8,
    pub name: String,
}

#[derive(Debug)]
pub struct DeleteCharacter {
    pub slot: u8,
    pub name: String,
    pub is_delete: bool,
}

#[derive(Debug)]
pub struct SelectCharacter {
    pub slot: u8,
    pub name: String,
}

#[derive(Debug)]
pub struct GameConnectionRequest {
    pub login_token: u32,
    pub password_md5: String,
}

#[derive(Debug)]
pub struct Move {
    pub target_entity_id: Option<ClientEntityId>,
    pub x: f32,
    pub y: f32,
    pub z: u16,
}

#[derive(Debug)]
pub struct Attack {
    pub target_entity_id: ClientEntityId,
}

#[derive(Debug)]
pub struct SetHotbarSlot {
    pub slot_index: usize,
    pub slot: Option<HotbarSlot>,
}

#[derive(Debug)]
pub struct ChangeEquipment {
    pub equipment_index: EquipmentIndex,
    pub item_slot: Option<ItemSlot>,
}

#[derive(Debug)]
pub struct PickupItemDrop {
    pub target_entity_id: ClientEntityId,
}

#[derive(Debug)]
pub enum LogoutRequest {
    Logout,
    ReturnToCharacterSelect,
}

#[derive(Debug)]
pub enum ReviveRequestType {
    RevivePosition,
    SavePosition,
}

#[derive(Debug)]
pub struct QuestDelete {
    pub slot: usize,
    pub quest_id: usize,
}

#[derive(Debug)]
pub struct PersonalStoreBuyItem {
    pub store_entity_id: ClientEntityId,
    pub store_slot_index: usize,
    pub buy_item: Item,
}

#[derive(Debug)]
pub struct NpcStoreBuyItem {
    pub tab_index: usize,
    pub item_index: usize,
    pub quantity: usize,
}

#[derive(Debug)]
pub struct NpcStoreTransaction {
    pub npc_entity_id: ClientEntityId,
    pub buy_items: Vec<NpcStoreBuyItem>,
    pub sell_items: Vec<(ItemSlot, usize)>,
}

#[derive(Debug)]
pub enum PartyRequest {
    Create(ClientEntityId),
    Invite(ClientEntityId),
    Leave,
    ChangeOwner(ClientEntityId),
    Kick(CharacterUniqueId),
}

#[derive(Debug)]
pub enum PartyReply {
    Busy(ClientEntityId),
    Accept(ClientEntityId),
    Reject(ClientEntityId),
}

#[derive(Debug)]
pub enum ClientMessage {
    ConnectionRequest(ConnectionRequest),
    LoginRequest(LoginRequest),
    GetChannelList(GetChannelList),
    JoinServer(JoinServer),
    GetCharacterList,
    CreateCharacter(CreateCharacter),
    DeleteCharacter(DeleteCharacter),
    SelectCharacter(SelectCharacter),
    GameConnectionRequest(GameConnectionRequest),
    JoinZoneRequest,
    Chat(String),
    Move(Move),
    Attack(Attack),
    SetHotbarSlot(SetHotbarSlot),
    ChangeAmmo(AmmoIndex, Option<ItemSlot>),
    ChangeEquipment(ChangeEquipment),
    ChangeVehiclePart(VehiclePartIndex, Option<ItemSlot>),
    IncreaseBasicStat(BasicStatType),
    PickupItemDrop(PickupItemDrop),
    LogoutRequest(LogoutRequest),
    ReviveRequest(ReviveRequestType),
    SetReviveZone,
    QuestTrigger(QuestTriggerHash),
    QuestDelete(QuestDelete),
    PersonalStoreListItems(ClientEntityId),
    PersonalStoreBuyItem(PersonalStoreBuyItem),
    DropItem(ItemSlot, usize),
    DropMoney(usize),
    UseItem(ItemSlot, Option<ClientEntityId>),
    LevelUpSkill(SkillSlot),
    CastSkillSelf(SkillSlot),
    CastSkillTargetEntity(SkillSlot, ClientEntityId),
    CastSkillTargetPosition(SkillSlot, Point2<f32>),
    NpcStoreTransaction(NpcStoreTransaction),
    RunToggle,
    SitToggle,
    DriveToggle,
    UseEmote(MotionId, bool),
    WarpGateRequest(WarpGateId),
    PartyRequest(PartyRequest),
    PartyReply(PartyReply),
}
