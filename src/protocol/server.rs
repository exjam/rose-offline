use bevy_ecs::prelude::Entity;
use lazy_static::__Deref;
use log::info;
use std::sync::Arc;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::oneshot,
};

use crate::{
    game::messages::{control::ControlMessage, server::ServerMessage},
    protocol::{Client, Connection, Protocol},
};

async fn run_connection(
    stream: TcpStream,
    protocol: &Protocol,
    control_message_tx: crossbeam_channel::Sender<ControlMessage>,
) -> Result<(), anyhow::Error> {
    let (client_message_tx, client_message_rx) = crossbeam_channel::unbounded();
    let (server_message_tx, server_message_rx) =
        tokio::sync::mpsc::unbounded_channel::<ServerMessage>();
    let (response_tx, response_rx) = oneshot::channel();

    control_message_tx.send(ControlMessage::AddClient {
        client_type: protocol.client_type,
        server_message_tx,
        client_message_rx,
        response_tx,
    })?;

    let entity = response_rx.await?;
    let mut client = Client {
        entity,
        connection: Connection::new(stream, protocol.packet_codec.deref()),
        client_message_tx,
        server_message_rx,
    };
    let result = (protocol.create_server)().run_client(&mut client).await;

    control_message_tx
        .send(ControlMessage::RemoveClient {
            client_type: protocol.client_type,
            entity: client.entity,
        })
        .ok();
    client.connection.shutdown().await;
    result
}

pub struct LoginServer {
    listener: TcpListener,
    protocol: Arc<Protocol>,
    control_message_tx: crossbeam_channel::Sender<ControlMessage>,
}

impl LoginServer {
    pub async fn new(
        listener: TcpListener,
        protocol: Arc<Protocol>,
        control_message_tx: crossbeam_channel::Sender<ControlMessage>,
    ) -> Result<LoginServer, anyhow::Error> {
        Ok(LoginServer {
            listener,
            protocol,
            control_message_tx,
        })
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                _ = async {
                    loop {
                        let (socket, _) = self.listener.accept().await.unwrap();
                        let protocol = self.protocol.clone();
                        let control_message_tx = self.control_message_tx.clone();
                        tokio::spawn(async move {
                            if let Ok(addr) = socket.peer_addr() {
                                info!("Login Server new connection from: {:?}", addr);
                            }
                            if let Err(err) = run_connection(socket, protocol.deref(), control_message_tx).await {
                                info!("Login Server connection error: {:?}", err);
                            }
                        });
                    }
                } => {},
            };
        }
    }
}

pub struct WorldServer {
    entity: Entity,

    listener: TcpListener,
    protocol: Arc<Protocol>,
    control_message_tx: crossbeam_channel::Sender<ControlMessage>,
}

impl WorldServer {
    pub async fn new(
        name: String,
        listener: TcpListener,
        protocol: Arc<Protocol>,
        control_message_tx: crossbeam_channel::Sender<ControlMessage>,
    ) -> Result<WorldServer, anyhow::Error> {
        let (response_tx, response_rx) = oneshot::channel();
        let local_addr = listener.local_addr().unwrap();
        control_message_tx.send(ControlMessage::AddWorldServer {
            name,
            ip: local_addr.ip().to_string(),
            port: local_addr.port(),
            packet_codec_seed: protocol.packet_codec.get_seed(),
            response_tx,
        })?;
        let entity = response_rx.await?;

        Ok(WorldServer {
            entity,
            listener,
            protocol,
            control_message_tx,
        })
    }

    pub fn get_entity(&self) -> Entity {
        self.entity
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                _ = async {
                    loop {
                        let (socket, _) = self.listener.accept().await.unwrap();
                        let protocol = self.protocol.clone();
                        let control_message_tx = self.control_message_tx.clone();
                        tokio::spawn(async move {
                            if let Ok(addr) = socket.peer_addr() {
                                info!("World Server new connection from: {:?}", addr);
                            }
                            if let Err(err) = run_connection(socket, protocol.deref(), control_message_tx).await {
                                info!("World Server connection error: {:?}", err);
                            }
                        });
                    }
                } => {},
            };
        }

        // TODO: Allow server to exit gracefully
        #[allow(unreachable_code)]
        {
            self.control_message_tx
                .send(ControlMessage::RemoveServer {
                    entity: self.entity,
                })
                .ok();
        }
    }
}

pub struct GameServer {
    entity: Entity,

    listener: TcpListener,
    protocol: Arc<Protocol>,
    control_message_tx: crossbeam_channel::Sender<ControlMessage>,
}

impl GameServer {
    pub async fn new(
        name: String,
        world_server: Entity,
        listener: TcpListener,
        protocol: Arc<Protocol>,
        control_message_tx: crossbeam_channel::Sender<ControlMessage>,
    ) -> Result<GameServer, anyhow::Error> {
        let (response_tx, response_rx) = oneshot::channel();
        let local_addr = listener.local_addr().unwrap();
        control_message_tx.send(ControlMessage::AddGameServer {
            name,
            world_server,
            ip: local_addr.ip().to_string(),
            port: local_addr.port(),
            packet_codec_seed: protocol.packet_codec.get_seed(),
            response_tx,
        })?;
        let entity = response_rx.await?;

        Ok(GameServer {
            entity,
            listener,
            protocol,
            control_message_tx,
        })
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                _ = async {
                    loop {
                        let (socket, _) = self.listener.accept().await.unwrap();
                        let protocol = self.protocol.clone();
                        let control_message_tx = self.control_message_tx.clone();
                        tokio::spawn(async move {
                            if let Ok(addr) = socket.peer_addr() {
                                info!("Game Server connection from: {:?}", addr);
                            }
                            if let Err(err) = run_connection(socket, protocol.deref(), control_message_tx).await {
                                info!("Game Server connection error: {:?}", err);
                            }
                        });
                    }
                } => {},
            };
        }

        // TODO: Allow server to exit gracefully
        #[allow(unreachable_code)]
        {
            self.control_message_tx
                .send(ControlMessage::RemoveServer {
                    entity: self.entity,
                })
                .ok();
        }
    }
}
