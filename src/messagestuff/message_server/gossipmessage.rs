use crate::messagestuff::message::*;
use crate::messagestuff::message_server::serverpacket::*;
use crate::my_errors::MyError;
use crate::nodo::Status;

#[derive(Debug, PartialEq, Clone)]
pub struct GossipMessage {
    pub ip: String,
    pub estado: Status,
    pub ttl: u8,
}

impl GossipMessage {
    pub fn decrease_ttl(&mut self) -> Option<bool> {
        if self.ttl == 0 {
            return Some(false);
        }
        self.ttl -= 1;
        Some(self.ttl != 0)
    }
}

impl Serialize for GossipMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&string_to_bytes(self.ip.clone()));
        bytes.extend_from_slice(&byte_to_byte(self.estado as u8));
        bytes.extend_from_slice(&byte_to_byte(self.ttl));
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), MyError>
    where
        Self: Sized,
    {
        let (_ip, resto) = string_from_bytes(bytes)?;
        let (value, resto) = byte_from_bytes(resto)?;
        let _status = Status::from_u8(value)?;
        let (_ttl, resto) = byte_from_bytes(resto)?;
        Ok((
            GossipMessage {
                ip: _ip,
                estado: _status,
                ttl: _ttl,
            },
            resto,
        ))
    }
}

impl IntoPacket for GossipMessage {
    fn into_packet(self) -> Packet {
        let _body = self.to_bytes();
        Packet {
            opcode: ServerOpcode::GossipMessage,
            length: _body.len() as i32,
            body: ServerMessage::Gossip(self),
        }
    }
}
