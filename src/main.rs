use protocol::server::{GameServer, LoginServer, WorldServer};
use tokio::net::TcpListener;

mod game;
mod protocol;

#[tokio::main]
async fn main() {
    let (game_control_tx, game_control_rx) = crossbeam_channel::unbounded();
    std::thread::spawn(move || {
        game::Game::new(game_control_rx).run();
    });

    let mut login_server = LoginServer::new(
        TcpListener::bind("127.0.0.1:29000").await.unwrap(),
        protocol::irose::login_protocol(),
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
        protocol::irose::world_protocol(),
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
        protocol::irose::game_protocol(),
        game_control_tx.clone(),
    )
    .await
    .unwrap();

    game_server.run().await;
}
