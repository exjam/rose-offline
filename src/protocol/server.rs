use std::sync::Arc;

use super::Client;
use super::Protocol;
use super::{Connection, ProtocolError};
use crate::game::messages::control::ControlMessage;
use crate::game::messages::server::ServerMessage;
use lazy_static::__Deref;
use legion::Entity;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

async fn run_connection(
    stream: TcpStream,
    protocol: &Protocol,
    control_message_tx: crossbeam_channel::Sender<ControlMessage>,
) -> Result<(), ProtocolError> {
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
        connection: Connection::new(stream, &protocol.packet_codec),
        client_message_tx,
        server_message_rx,
    };
    let result = (protocol.create_client)().run_client(&mut client).await;

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
    ) -> Result<LoginServer, ProtocolError> {
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
                                println!("[LOGIN] New connection from: {:?}", addr);
                            }
                            if let Err(err) = run_connection(socket, protocol.deref(), control_message_tx).await {
                                println!("[LOGIN] Connection error: {:?}", err);
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
    ) -> Result<WorldServer, ProtocolError> {
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
                                println!("[WORLD] New connection from: {:?}", addr);
                            }
                            if let Err(err) = run_connection(socket, protocol.deref(), control_message_tx).await {
                                println!("[WORLD] Connection error: {:?}", err);
                            }
                        });
                    }
                } => {},
            };
        }

        self.control_message_tx
            .send(ControlMessage::RemoveServer {
                entity: self.entity,
            })
            .ok();
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
    ) -> Result<GameServer, ProtocolError> {
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
                                println!("[ GAME] New connection from: {:?}", addr);
                            }
                            if let Err(err) = run_connection(socket, protocol.deref(), control_message_tx).await {
                                println!("[ GAME] Connection error: {:?}", err);
                            }
                        });
                    }
                } => {},
            };
        }

        self.control_message_tx
            .send(ControlMessage::RemoveServer {
                entity: self.entity,
            })
            .ok();
    }
}
