use bytes::{Buf, BufMut, Bytes, BytesMut};
use log::trace;
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use crate::{Packet, PacketCodec};

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("connection lost")]
    ConnectionLost,

    #[error("failed to decrypt packet header")]
    DecryptHeaderFailed,

    #[error("failed to decrypt packet body")]
    DecryptBodyFailed,
}

pub struct Connection<'a> {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
    packet_codec: &'a (dyn PacketCodec + Send + Sync),
}

impl<'a> Connection<'a> {
    pub fn new(socket: TcpStream, packet_codec: &'a (dyn PacketCodec + Send + Sync)) -> Self {
        Self {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
            packet_codec,
        }
    }

    pub async fn shutdown(&mut self) {
        let _ = self.stream.shutdown().await;
    }

    pub async fn read_packet(&mut self) -> Result<Packet, anyhow::Error> {
        let mut read_length = 6usize;
        let mut have_read_header = false;

        loop {
            while self.buffer.len() < read_length {
                match self.stream.read_buf(&mut self.buffer).await {
                    Ok(_) => {
                        if self.buffer.is_empty() {
                            return Err(ConnectionError::ConnectionLost.into());
                        }
                    }
                    Err(_) => {
                        return Err(ConnectionError::ConnectionLost.into());
                    }
                }
            }

            if !have_read_header {
                read_length = self.packet_codec.decrypt_packet_header(&mut self.buffer);
                if read_length == 0 {
                    return Err(ConnectionError::DecryptHeaderFailed.into());
                }
                have_read_header = true;
            } else if self.packet_codec.decrypt_packet_body(&mut self.buffer) {
                // Read packet into size, command, data
                let size = self.buffer.get_u16_le() as usize;
                let command = self.buffer.get_u16_le();

                if size < 6 {
                    return Err(ConnectionError::DecryptBodyFailed.into());
                }

                self.buffer.advance(2);
                let data: Bytes = self.buffer.split_to(size - 6).into();

                // Packets can contain random amount of padding at end
                self.buffer.advance(read_length - size);

                trace!(target: "packets", "RECV [{:03X}] {:02x?}", command, &data[..]);
                return Ok(Packet { command, data });
            } else {
                return Err(ConnectionError::DecryptBodyFailed.into());
            }
        }
    }

    pub async fn write_packet(&mut self, packet: Packet) -> Result<(), anyhow::Error> {
        trace!(target: "packets", "SEND [{:03X}] {:02x?}", packet.command, &packet.data[..]);

        let size = packet.data.len() + 6;
        let mut buffer = BytesMut::with_capacity(size);
        buffer.put_u16_le(size as u16);
        buffer.put_u16_le(packet.command);
        buffer.put_u16_le(0);
        buffer.put(packet.data);
        self.packet_codec.encrypt_packet(&mut buffer);

        self.stream
            .write_all(&buffer)
            .await
            .map_err(|_| ConnectionError::ConnectionLost)?;

        self.stream
            .flush()
            .await
            .map_err(|_| ConnectionError::ConnectionLost)?;

        Ok(())
    }
}
