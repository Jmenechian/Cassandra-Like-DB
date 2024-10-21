use std::error::Error;
use crate::messagestuff::message::*;
use crate::messagestuff::message_server::serverpacket::*;

#[derive(Debug, PartialEq)]
pub struct GossipMessage {
    pub ip: String,
}

impl Serialize for GossipMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&longstring_to_bytes(self.ip.clone()));
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), Box<dyn Error>> where Self: Sized {
        let (_ip, resto) = string_from_bytes(bytes)?;
        Ok((GossipMessage{ip: _ip}, resto))
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