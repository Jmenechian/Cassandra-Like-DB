// Si hay algun tipo de error se manda este mensaje por Paquete

use crate::errors::my_errors::MyError;
use crate::messagestuff::message::*;
use crate::messagestuff::message_server::serverpacket::*;

#[derive(Debug, PartialEq)]
pub struct ErrorServerMessage {
    pub error: String,
    pub code: i32,
}

impl Serialize for ErrorServerMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&int_to_bytes(self.code));
        bytes.extend_from_slice(&string_to_bytes(self.error.clone()));
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), MyError> {
        let (_code, resto) = int_from_bytes(bytes)?;
        let (_error, resto) = string_from_bytes(resto)?;

        Ok((
            ErrorServerMessage {
                error: _error,
                code: _code,
            },
            resto,
        ))
    }
}

impl IntoPacket for ErrorServerMessage {
    fn into_packet(self) -> Packet {
        let _body = self.to_bytes();
        Packet {
            opcode: ServerOpcode::ErrorServerMessage,
            length: _body.len() as i32,
            body: ServerMessage::ErrorServer(self),
        }
    }
}
