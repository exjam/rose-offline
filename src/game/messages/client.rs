use std::time::Duration;

use tokio::sync::oneshot;

use crate::game::{
    components::{CharacterDeleteTime, CharacterInfo, CharacterList, Equipment, Level},
    data::character::CharacterStorage,
};

pub enum ConnectionRequestError {
    Failed,
    InvalidId,
    InvalidPassword,
}

pub struct ConnectionRequestResponse {
    pub packet_sequence_id: u32,
}

pub struct ConnectionRequest {
    pub login_token: Option<(u32, String)>,
    pub response_tx: oneshot::Sender<Result<ConnectionRequestResponse, ConnectionRequestError>>,
}

pub enum LoginError {
    Failed,
    InvalidAccount,
    InvalidPassword,
}

pub struct LoginRequest {
    pub username: String,
    pub password_md5: String,
    pub response_tx: oneshot::Sender<Result<(), LoginError>>,
}

pub struct GetWorldServerList {
    pub response_tx: oneshot::Sender<Vec<(u32, String)>>,
}

pub enum GetChannelListError {
    InvalidServerId,
}

pub struct GetChannelList {
    pub server_id: u32,
    pub response_tx: oneshot::Sender<Result<Vec<(u8, String)>, GetChannelListError>>,
}

pub enum JoinServerError {
    InvalidServerId,
    InvalidChannelId,
}

pub struct JoinServerResponse {
    pub login_token: u32,
    pub packet_codec_seed: u32,
    pub ip: String,
    pub port: u16,
}

pub struct JoinServer {
    pub server_id: u32,
    pub channel_id: u8,
    pub response_tx: oneshot::Sender<Result<JoinServerResponse, JoinServerError>>,
}

#[derive(Clone)]
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

pub struct GetCharacterList {
    pub response_tx: oneshot::Sender<Vec<CharacterListItem>>,
}

pub enum CreateCharacterError {
    Failed,
    AlreadyExists,
    InvalidValue,
    NoMoreSlots,
}

pub struct CreateCharacter {
    pub gender: u8,
    pub birth_stone: u8,
    pub hair: u8,
    pub face: u8,
    pub name: String,
    pub response_tx: oneshot::Sender<Result<u8, CreateCharacterError>>,
}

pub enum DeleteCharacterError {
    Failed,
}

pub struct DeleteCharacter {
    pub slot: u8,
    pub name: String,
    pub is_delete: bool,
    pub response_tx: oneshot::Sender<Result<Option<CharacterDeleteTime>, DeleteCharacterError>>,
}

pub enum SelectCharacterError {
    Failed,
}

pub struct SelectCharacter {
    pub slot: u8,
    pub name: String,
    pub response_tx: oneshot::Sender<Result<JoinServerResponse, SelectCharacterError>>,
}

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
}
