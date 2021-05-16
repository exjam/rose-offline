use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

use super::packet::{Packet, PacketCodec};
use super::ProtocolError;

pub struct Connection<'a> {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
    packet_codec: &'a Box<dyn PacketCodec + Send + Sync>,
}

impl<'a> Connection<'a> {
    pub fn new(socket: TcpStream, packet_codec: &'a Box<dyn PacketCodec + Send + Sync>) -> Self {
        Self {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
            packet_codec,
        }
    }

    pub async fn shutdown(&mut self) {
        let _ = self.stream.shutdown().await;
    }

    pub async fn read_packet(&mut self) -> Result<Packet, ProtocolError> {
        let mut read_length = 6usize;
        let mut have_read_header = false;

        loop {
            while self.buffer.len() < read_length {
                match self.stream.read_buf(&mut self.buffer).await {
                    Ok(_) => {
                        if self.buffer.is_empty() {
                            return Err(ProtocolError::Disconnect);
                        }
                    }
                    Err(_) => {
                        return Err(ProtocolError::Disconnect);
                    }
                }
            }

            if !have_read_header {
                read_length = self.packet_codec.decrypt_client_header(&mut self.buffer);
                if read_length == 0 {
                    return Err(ProtocolError::InvalidPacket);
                }
                have_read_header = true;
            } else if self.packet_codec.decrypt_client_body(&mut self.buffer) {
                // Read packet into size, command, data
                let size = self.buffer.get_u16_le() as usize;
                let command = self.buffer.get_u16_le();
                self.buffer.advance(2);
                let data: Bytes = self.buffer.split_to(size - 6).into();

                // Packets can contain random amount of padding at end
                self.buffer.advance(read_length - size);

                println!("RECV [{:03X}] {:02x?}", command, &data[..]);
                return Ok(Packet { command, data });
            } else {
                return Err(ProtocolError::InvalidPacket);
            }
        }
    }

    pub async fn write_packet(&mut self, packet: Packet) -> Result<(), ProtocolError> {
        println!("SEND [{:03X}] {:02x?}", packet.command, &packet.data[..]);

        let size = packet.data.len() + 6;
        let mut buffer = BytesMut::with_capacity(size);
        buffer.put_u16_le(size as u16);
        buffer.put_u16_le(packet.command);
        buffer.put_u16_le(0);
        buffer.put(packet.data);
        self.packet_codec.encrypt_server(&mut buffer);

        if self.stream.write_all(&buffer).await.is_err() {
            return Err(ProtocolError::Disconnect);
        }

        if self.stream.flush().await.is_err() {
            return Err(ProtocolError::Disconnect);
        }

        Ok(())
    }
}
