// We must globally allow dead_code because of modular-bitfield..
#![allow(dead_code)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::needless_option_as_deref)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

mod data;
mod game;
mod irose;
mod protocol;

use std::{path::Path, time::Instant};

use clap::{App, Arg};
use log::debug;
use simplelog::*;
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

    let app = App::new("rose-offline")
        .arg(
            Arg::new("data-idx")
                .long("data-idx")
                .help("Path to data.idx")
                .takes_value(true)
                .default_value("data.idx"),
        )
        .arg(
            Arg::new("ip")
                .long("ip")
                .help("Listen IP used for login, world, game servers")
                .takes_value(true)
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::new("login-port")
                .long("login-port")
                .help("Port for login server")
                .takes_value(true)
                .default_value("29000"),
        )
        .arg(
            Arg::new("world-port")
                .long("world-port")
                .help("Port for world server")
                .takes_value(true)
                .default_value("29100"),
        )
        .arg(
            Arg::new("game-port")
                .long("game-port")
                .help("Port for login server")
                .takes_value(true)
                .default_value("29200"),
        );
    let matches = app.get_matches();
    let listen_ip = matches.value_of("ip").unwrap();
    let login_port = matches.value_of("login-port").unwrap();
    let world_port = matches.value_of("world-port").unwrap();
    let game_port = matches.value_of("game-port").unwrap();
    let data_idx_path = Path::new(matches.value_of("data-idx").unwrap());

    let started_load = Instant::now();
    let game_data = irose::get_game_data(data_idx_path);
    debug!("Time take to read game data {:?}", started_load.elapsed());

    let (game_control_tx, game_control_rx) = crossbeam_channel::unbounded();
    std::thread::spawn(move || {
        game::GameWorld::new(game_control_rx).run(game_data);
    });

    let mut login_server = LoginServer::new(
        TcpListener::bind(format!("{}:{}", listen_ip, login_port))
            .await
            .unwrap(),
        irose::login_protocol(),
        game_control_tx.clone(),
    )
    .await
    .unwrap();

    let mut world_server = WorldServer::new(
        String::from("_WorldServer"),
        TcpListener::bind(format!("{}:{}", listen_ip, world_port))
            .await
            .unwrap(),
        irose::world_protocol(),
        game_control_tx.clone(),
    )
    .await
    .unwrap();

    let mut game_server = GameServer::new(
        String::from("GameServer"),
        world_server.get_entity(),
        TcpListener::bind(format!("{}:{}", listen_ip, game_port))
            .await
            .unwrap(),
        irose::game_protocol(),
        game_control_tx.clone(),
    )
    .await
    .unwrap();

    tokio::spawn(async move {
        game_server.run().await;
    });

    tokio::spawn(async move {
        world_server.run().await;
    });

    login_server.run().await;
}
