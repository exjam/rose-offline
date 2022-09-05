use bevy::math::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        BasicStatType, CharacterGender, CharacterUniqueId, HotbarSlot, ItemSlot, SkillSlot,
    },
    data::Password,
    messages::{ClientEntityId, PartyItemSharing, PartyRejectInviteReason, PartyXpSharing},
};
use rose_data::{
    AmmoIndex, EquipmentIndex, Item, MotionId, QuestTriggerHash, VehiclePartIndex, WarpGateId,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionRequest {
    pub login_token: u32,
    pub password: Password,
}

impl Default for ConnectionRequest {
    fn default() -> Self {
        Self {
            login_token: 0,
            password: Password::Plaintext(String::new()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: Password,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetChannelList {
    pub server_id: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinServer {
    pub server_id: usize,
    pub channel_id: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateCharacter {
    pub gender: CharacterGender,
    pub hair: i32,
    pub face: i32,
    pub name: String,
    pub start_point: i32,

    // irose
    pub birth_stone: i32,

    // narose667
    pub hair_color: i32,
    pub weapon_type: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeleteCharacter {
    pub slot: u8,
    pub name: String,
    pub is_delete: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectCharacter {
    pub slot: u8,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameConnectionRequest {
    pub login_token: u32,
    pub password: Password,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Move {
    pub target_entity_id: Option<ClientEntityId>,
    pub x: f32,
    pub y: f32,
    pub z: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attack {
    pub target_entity_id: ClientEntityId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetHotbarSlot {
    pub slot_index: usize,
    pub slot: Option<HotbarSlot>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeEquipment {
    pub equipment_index: EquipmentIndex,
    pub item_slot: Option<ItemSlot>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LogoutRequest {
    Logout,
    ReturnToCharacterSelect,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReviveRequestType {
    RevivePosition,
    SavePosition,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuestDelete {
    pub slot: usize,
    pub quest_id: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonalStoreBuyItem {
    pub store_entity_id: ClientEntityId,
    pub store_slot_index: usize,
    pub buy_item: Item,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpcStoreBuyItem {
    pub tab_index: usize,
    pub item_index: usize,
    pub quantity: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpcStoreTransaction {
    pub npc_entity_id: ClientEntityId,
    pub buy_items: Vec<NpcStoreBuyItem>,
    pub sell_items: Vec<(ItemSlot, usize)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    MoveCollision(Vec3),
    Attack(Attack),
    SetHotbarSlot(SetHotbarSlot),
    ChangeAmmo(AmmoIndex, Option<ItemSlot>),
    ChangeEquipment(ChangeEquipment),
    ChangeVehiclePart(VehiclePartIndex, Option<ItemSlot>),
    IncreaseBasicStat(BasicStatType),
    PickupItemDrop(ClientEntityId),
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
    CastSkillTargetPosition(SkillSlot, Vec2),
    NpcStoreTransaction(NpcStoreTransaction),
    RunToggle,
    SitToggle,
    DriveToggle,
    UseEmote(MotionId, bool),
    WarpGateRequest(WarpGateId),
    PartyCreate(ClientEntityId),
    PartyInvite(ClientEntityId),
    PartyLeave,
    PartyChangeOwner(ClientEntityId),
    PartyKick(CharacterUniqueId),
    PartyAcceptCreateInvite(ClientEntityId),
    PartyAcceptJoinInvite(ClientEntityId),
    PartyRejectInvite(PartyRejectInviteReason, ClientEntityId),
    PartyUpdateRules(PartyItemSharing, PartyXpSharing),
    CraftInsertGem {
        equipment_index: EquipmentIndex,
        item_slot: ItemSlot,
    },
    CraftSkillDisassemble {
        skill_slot: SkillSlot,
        item_slot: ItemSlot,
    },
    CraftNpcDisassemble {
        npc_entity_id: ClientEntityId,
        item_slot: ItemSlot,
    },
    CraftSkillUpgradeItem {
        skill_slot: SkillSlot,
        item_slot: ItemSlot,
        ingredients: [ItemSlot; 3],
    },
    CraftNpcUpgradeItem {
        npc_entity_id: ClientEntityId,
        item_slot: ItemSlot,
        ingredients: [ItemSlot; 3],
    },
    BankOpen,
    BankDepositItem {
        item_slot: ItemSlot,
        item: Item,
        is_premium: bool,
    },
    BankWithdrawItem {
        bank_slot: usize,
        item: Item,
        is_premium: bool,
    },
}
