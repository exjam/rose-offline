mod data;
mod game;
mod irose;
mod protocol;

use std::{path::Path, sync::Arc};
use tokio::net::TcpListener;

use crate::{
    data::formats::VfsIndex,
    protocol::server::{GameServer, LoginServer, WorldServer},
};

#[tokio::main]
async fn main() {
    let (game_control_tx, game_control_rx) = crossbeam_channel::unbounded();
    let vfs_index = VfsIndex::load(&Path::new("data.idx")).expect("Failed reading data.idx");
    let skill_database = Arc::new(
        irose::data::get_skill_database(&vfs_index).expect("Failed to load skill database"),
    );
    let item_database =
        Arc::new(irose::data::get_item_database(&vfs_index).expect("Failed to load item database"));
    let npc_database =
        Arc::new(irose::data::get_npc_database(&vfs_index).expect("Failed to load npc database"));
    let zone_database =
        Arc::new(irose::data::get_zone_database(&vfs_index).expect("Failed to load zone database"));
    let character_creator = irose::data::get_character_creator(&vfs_index, &skill_database)
        .expect("Failed to get character creator");
    let ability_value_calculator =
        irose::data::get_ability_value_calculator(item_database.clone(), skill_database.clone())
            .expect("Failed to get ability value calculator");

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
