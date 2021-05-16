use protocol::server::{GameServer, LoginServer, WorldServer};
use tokio::net::TcpListener;

mod data;
use data::VFS_INDEX;

mod game;
mod irose;
mod protocol;

#[tokio::main]
async fn main() {
    let (game_control_tx, game_control_rx) = crossbeam_channel::unbounded();
    let skill_database = irose::data::get_skill_database(&VFS_INDEX).unwrap();

    std::thread::spawn(move || {
        game::Game::new(game_control_rx).run(
            irose::data::get_character_creator(&VFS_INDEX, &skill_database).unwrap(),
            irose::data::get_item_database(&VFS_INDEX).unwrap(),
            irose::data::get_npc_database(&VFS_INDEX).unwrap(),
            skill_database,
            irose::data::get_zone_database(&VFS_INDEX).unwrap(),
        );
    });

    let mut login_server = LoginServer::new(
        TcpListener::bind("127.0.0.1:29000").await.unwrap(),
        irose::protocol::login_protocol(),
        game_control_tx.clone(),
    )
    .await
    .unwrap();
    tokio::spawn(async move {
        login_server.run().await;
    });

    let mut world_server = WorldServer::new(
        String::from("_WorldServer"),
        TcpListener::bind("127.0.0.1:0").await.unwrap(),
        irose::protocol::world_protocol(),
        game_control_tx.clone(),
    )
    .await
    .unwrap();
    let world_server_entity = world_server.get_entity();
    tokio::spawn(async move {
        world_server.run().await;
    });

    let mut game_server = GameServer::new(
        String::from("GameServer"),
        world_server_entity,
        TcpListener::bind("127.0.0.1:0").await.unwrap(),
        irose::protocol::game_protocol(),
        game_control_tx.clone(),
    )
    .await
    .unwrap();

    game_server.run().await;
}
