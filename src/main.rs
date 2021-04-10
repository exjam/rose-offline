use protocol::server::run_server;
use tokio::net::TcpListener;

mod game;
mod protocol;

#[tokio::main]
async fn main() {
    let world_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let game_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    // protocol::server::Server::new(login_listener, codec, )
    let (game_control_tx, game_control_rx) = crossbeam_channel::unbounded();
    std::thread::spawn(move || {
        game::Game::new(game_control_rx).run();
    });

    let login_game_control_tx = game_control_tx.clone();
    tokio::spawn(async move {
        run_server(
            TcpListener::bind("127.0.0.1:29000").await.unwrap(),
            protocol::irose::login_protocol(),
            login_game_control_tx,
        )
        .await;
    });

    let world_game_control_tx = game_control_tx.clone();
    tokio::spawn(async move {
        run_server(
            TcpListener::bind("127.0.0.1:0").await.unwrap(),
            protocol::irose::world_protocol(),
            world_game_control_tx,
        )
        .await;
    });

    let game_game_control_tx = game_control_tx.clone();
    run_server(
        TcpListener::bind("127.0.0.1:0").await.unwrap(),
        protocol::irose::game_protocol(),
        game_game_control_tx,
    )
    .await;
}
