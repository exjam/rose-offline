use std::sync::Arc;

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
    let skill_database = Arc::new(irose::data::get_skill_database(&VFS_INDEX).unwrap());
    let item_database = Arc::new(irose::data::get_item_database(&VFS_INDEX).unwrap());
    let npc_database = Arc::new(irose::data::get_npc_database(&VFS_INDEX).unwrap());
    let zone_database = Arc::new(irose::data::get_zone_database(&VFS_INDEX).unwrap());
    let character_creator =
        irose::data::get_character_creator(&VFS_INDEX, &skill_database).unwrap();
    let ability_value_calculator =
        irose::data::get_ability_value_calculator(item_database.clone(), skill_database.clone())
            .unwrap();

    std::thread::spawn(move || {
        game::Game::new(game_control_rx).run(
            character_creator,
            ability_value_calculator,
            item_database,
            npc_database,
            skill_database,
            zone_database,
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
