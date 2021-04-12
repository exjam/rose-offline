use tokio::sync::oneshot;

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

pub enum ClientMessage {
    ConnectionRequest(ConnectionRequest),
    LoginRequest(LoginRequest),
    GetWorldServerList(GetWorldServerList),
    GetChannelList(GetChannelList),
    JoinServer(JoinServer),
}
