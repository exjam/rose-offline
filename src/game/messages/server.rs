
pub enum ConnectionResult {
    Ok,
    InvalidId,
    InvalidPassword,
}

pub enum ServerMessage {
    ConnectionReply {
        status: ConnectionResult,
        packet_sequence_id: u32,
    }
}
