use crate::messagestuff::message::*;
use crate::messagestuff::message_server::serverpacket::*;
use crate::my_errors::MyError;

#[derive(Debug, PartialEq)]
pub struct QueryServerMessage {
    pub query: String,
    pub consistency: u16,
}

impl Serialize for QueryServerMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&longstring_to_bytes(self.query.clone()));
        bytes.extend_from_slice(&short_to_bytes(self.consistency));
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), MyError> {
        let (_query, resto) = longstring_from_bytes(bytes)?;
        let (_consistency, resto) = short_from_bytes(resto)?;
        Ok((
            QueryServerMessage {
                query: _query,
                consistency: _consistency,
            },
            resto,
        ))
    }
}

impl IntoPacket for QueryServerMessage {
    fn into_packet(self) -> Packet {
        let _body = self.to_bytes();
        Packet {
            opcode: ServerOpcode::QueryServerMessage,
            length: _body.len() as i32,
            body: ServerMessage::Query(self),
        }
    }
}
