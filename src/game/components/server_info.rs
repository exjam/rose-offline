use bevy_ecs::prelude::Component;

#[derive(Component)]
pub struct ServerInfo {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub packet_codec_seed: u32,
}
