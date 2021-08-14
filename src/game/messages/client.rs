use nalgebra::Point2;
use tokio::sync::oneshot;

use crate::{
    data::{character::CharacterStorage, item::Item, QuestTriggerHash, WorldTicks},
    game::components::{
        AmmoIndex, BasicStatType, BasicStats, CharacterDeleteTime, CharacterInfo, ClientEntityId,
        Equipment, EquipmentIndex, ExperiencePoints, HealthPoints, Hotbar, HotbarSlot, Inventory,
        ItemSlot, Level, ManaPoints, Position, QuestState, SkillList, SkillPoints, SkillSlot,
        Stamina, StatPoints, Team, UnionMembership,
    },
};

#[derive(Debug)]
pub enum ConnectionRequestError {
    Failed,
    InvalidToken,
    InvalidPassword,
}

#[derive(Debug)]
pub struct ConnectionRequestResponse {
    pub packet_sequence_id: u32,
}

#[derive(Debug)]
pub struct ConnectionRequest {
    pub login_token: u32,
    pub password_md5: String,
    pub response_tx: oneshot::Sender<Result<ConnectionRequestResponse, ConnectionRequestError>>,
}

#[derive(Debug)]
pub enum LoginError {
    Failed,
    InvalidAccount,
    InvalidPassword,
}

#[derive(Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password_md5: String,
    pub response_tx: oneshot::Sender<Result<(), LoginError>>,
}

#[derive(Debug)]
pub struct GetWorldServerList {
    pub response_tx: oneshot::Sender<Vec<(u32, String)>>,
}

#[derive(Debug)]
pub enum GetChannelListError {
    InvalidServerId,
}

#[derive(Debug)]
pub struct GetChannelList {
    pub server_id: u32,
    pub response_tx: oneshot::Sender<Result<Vec<(u8, String)>, GetChannelListError>>,
}

#[derive(Debug)]
pub enum JoinServerError {
    InvalidServerId,
    InvalidChannelId,
}

#[derive(Debug)]
pub struct JoinServerResponse {
    pub login_token: u32,
    pub packet_codec_seed: u32,
    pub ip: String,
    pub port: u16,
}

#[derive(Debug)]
pub struct JoinServer {
    pub server_id: u32,
    pub channel_id: u8,
    pub response_tx: oneshot::Sender<Result<JoinServerResponse, JoinServerError>>,
}

#[derive(Clone, Debug)]
pub struct CharacterListItem {
    pub info: CharacterInfo,
    pub level: Level,
    pub delete_time: Option<CharacterDeleteTime>,
    pub equipment: Equipment,
}

impl From<&CharacterStorage> for CharacterListItem {
    fn from(storage: &CharacterStorage) -> CharacterListItem {
        CharacterListItem {
            info: storage.info.clone(),
            delete_time: storage.delete_time.clone(),
            equipment: storage.equipment.clone(),
            level: storage.level.clone(),
        }
    }
}

#[derive(Debug)]
pub struct GetCharacterList {
    pub response_tx: oneshot::Sender<Vec<CharacterListItem>>,
}

#[derive(Debug)]
pub enum CreateCharacterError {
    Failed,
    AlreadyExists,
    InvalidValue,
    NoMoreSlots,
}

#[derive(Debug)]
pub struct CreateCharacter {
    pub gender: u8,
    pub birth_stone: u8,
    pub hair: u8,
    pub face: u8,
    pub name: String,
    pub response_tx: oneshot::Sender<Result<u8, CreateCharacterError>>,
}

#[derive(Debug)]
pub enum DeleteCharacterError {
    Failed,
}

#[derive(Debug)]
pub struct DeleteCharacter {
    pub slot: u8,
    pub name: String,
    pub is_delete: bool,
    pub response_tx: oneshot::Sender<Result<Option<CharacterDeleteTime>, DeleteCharacterError>>,
}

#[derive(Debug)]
pub enum SelectCharacterError {
    Failed,
}

#[derive(Debug)]
pub struct SelectCharacter {
    pub slot: u8,
    pub name: String,
    pub response_tx: oneshot::Sender<Result<JoinServerResponse, SelectCharacterError>>,
}

#[derive(Debug)]
pub struct GameConnectionResponse {
    pub packet_sequence_id: u32,
    pub character_info: CharacterInfo,
    pub position: Position,
    pub equipment: Equipment,
    pub basic_stats: BasicStats,
    pub level: Level,
    pub experience_points: ExperiencePoints,
    pub inventory: Inventory,
    pub skill_list: SkillList,
    pub hotbar: Hotbar,
    pub health_points: HealthPoints,
    pub mana_points: ManaPoints,
    pub stat_points: StatPoints,
    pub skill_points: SkillPoints,
    pub quest_state: QuestState,
    pub union_membership: UnionMembership,
    pub stamina: Stamina,
}

#[derive(Debug)]
pub struct GameConnectionRequest {
    pub login_token: u32,
    pub password_md5: String,
    pub response_tx: oneshot::Sender<Result<GameConnectionResponse, ConnectionRequestError>>,
}

#[derive(Debug)]
pub struct JoinZoneResponse {
    pub entity_id: ClientEntityId,
    pub level: Level,
    pub experience_points: ExperiencePoints,
    pub team: Team,
    pub health_points: HealthPoints,
    pub mana_points: ManaPoints,
    pub world_ticks: WorldTicks,
}

#[derive(Debug)]
pub struct JoinZoneRequest {
    pub response_tx: oneshot::Sender<JoinZoneResponse>,
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
pub enum SetHotbarSlotError {
    InvalidSlot,
}

#[derive(Debug)]
pub struct SetHotbarSlot {
    pub slot_index: usize,
    pub slot: Option<HotbarSlot>,
    pub response_tx: oneshot::Sender<Result<(), SetHotbarSlotError>>,
}

#[derive(Debug)]
pub struct ChangeEquipment {
    pub equipment_index: EquipmentIndex,
    pub item_slot: Option<ItemSlot>,
}

#[derive(Debug)]
pub struct PickupDroppedItem {
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
pub enum ClientMessage {
    ConnectionRequest(ConnectionRequest),
    LoginRequest(LoginRequest),
    GetWorldServerList(GetWorldServerList),
    GetChannelList(GetChannelList),
    JoinServer(JoinServer),
    GetCharacterList(GetCharacterList),
    CreateCharacter(CreateCharacter),
    DeleteCharacter(DeleteCharacter),
    SelectCharacter(SelectCharacter),
    GameConnectionRequest(GameConnectionRequest),
    JoinZoneRequest(JoinZoneRequest),
    Chat(String),
    Move(Move),
    Attack(Attack),
    SetHotbarSlot(SetHotbarSlot),
    ChangeAmmo(AmmoIndex, Option<ItemSlot>),
    ChangeEquipment(ChangeEquipment),
    IncreaseBasicStat(BasicStatType),
    PickupDroppedItem(PickupDroppedItem),
    LogoutRequest(LogoutRequest),
    ReviveRequest(ReviveRequestType),
    QuestTrigger(QuestTriggerHash),
    QuestDelete(QuestDelete),
    PersonalStoreListItems(ClientEntityId),
    PersonalStoreBuyItem(PersonalStoreBuyItem),
    DropItem(ItemSlot, usize),
    UseItem(ItemSlot, Option<ClientEntityId>),
    CastSkillSelf(SkillSlot),
    CastSkillTargetEntity(SkillSlot, ClientEntityId),
    CastSkillTargetPosition(SkillSlot, Point2<f32>),
    NpcStoreTransaction(NpcStoreTransaction),
}
