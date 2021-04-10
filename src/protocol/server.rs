use std::sync::Arc;

use super::Protocol;
use super::{Connection, ProtocolError};
use crate::game::messages::{control::ControlMessage, client::ClientMessage, server::ServerMessage};
use tokio::{net::TcpListener, sync::oneshot};

impl From<oneshot::error::RecvError> for ProtocolError {
    fn from(_: oneshot::error::RecvError) -> ProtocolError {
        ProtocolError::IscError
    }
}

impl From<crossbeam_channel::SendError<ClientMessage>> for ProtocolError {
    fn from(_: crossbeam_channel::SendError<ClientMessage>) -> ProtocolError {
        ProtocolError::IscError
    }
}

impl From<crossbeam_channel::SendError<ControlMessage>> for ProtocolError {
    fn from(_: crossbeam_channel::SendError<ControlMessage>) -> ProtocolError {
        ProtocolError::IscError
    }
}

async fn run_connection(
    connection: &mut Connection,
    protocol: Arc<Protocol>,
    control_message_tx: crossbeam_channel::Sender<ControlMessage>,
) -> Result<(), ProtocolError> {
    let (recv_message_tx, recv_message_rx) = crossbeam_channel::unbounded();
    let (send_message_tx, mut send_message_rx) =
        tokio::sync::mpsc::unbounded_channel::<ServerMessage>();

    let (response_tx, response_rx) = oneshot::channel();
    control_message_tx.send(ControlMessage::AddClient {
        client_type: protocol.client_type,
        send_message_tx: send_message_tx,
        recv_message_rx: recv_message_rx,
        response_tx: response_tx,
    })?;

    let entity = response_rx.await?;

    loop {
        tokio::select! {
            packet = connection.read_packet() => {
                match packet {
                    Ok(packet) => {
                        recv_message_tx.send(protocol.packet_decoder.decode(&packet)?)?;
                    },
                    Err(error) => {
                        return Err(error);
                    }
                }
            },
            Some(message) = send_message_rx.recv() => {
                connection.write_packet(protocol.packet_encoder.encode(&message)).await?;
            }
        };
    }
}

pub async fn run_server(
    listener: TcpListener,
    protocol: Arc<Protocol>,
    control_message_tx: crossbeam_channel::Sender<ControlMessage>,
) {
    loop {
        tokio::select! {
            _ = async {
                loop {
                    let (socket, _) = listener.accept().await.unwrap();
                    let protocol = protocol.clone();
                    let control_message_tx = control_message_tx.clone();
                    tokio::spawn(async move {
                        let mut connection = Connection::new(socket, protocol.clone());
                        if let Err(err) = run_connection(&mut connection, protocol, control_message_tx).await {
                            println!("Connection error: {:?}", err);
                        }
                        connection.shutdown().await;
                    });
                }
            } => {},
        };
    }
}
