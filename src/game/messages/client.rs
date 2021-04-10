pub enum ClientMessage {
    ConnectionRequest {
        unique_id: u32,
        password_md5: String,
    }
}
