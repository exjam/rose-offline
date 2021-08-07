// We must globally allow dead_code because of modular-bitfield..
#![allow(dead_code)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

mod data;
mod game;
mod irose;
mod protocol;

use log::debug;
use simplelog::*;
use std::time::Instant;
use tokio::net::TcpListener;

use crate::protocol::server::{GameServer, LoginServer, WorldServer};

#[tokio::main]
async fn main() {
    TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Stdout,
        ColorChoice::Auto,
    )
    .expect("Failed to initialise logging");

    let started_load = Instant::now();
    let game_data = irose::get_game_data();
    debug!("Time take to read game data {:?}", started_load.elapsed());

    let (game_control_tx, game_control_rx) = crossbeam_channel::unbounded();
    std::thread::spawn(move || {
        game::GameWorld::new(game_control_rx).run(game_data);
    });

    let mut login_server = LoginServer::new(
        TcpListener::bind("127.0.0.1:29000").await.unwrap(),
        irose::login_protocol(),
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
        irose::world_protocol(),
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
        irose::game_protocol(),
        game_control_tx.clone(),
    )
    .await
    .unwrap();

    game_server.run().await;
}
