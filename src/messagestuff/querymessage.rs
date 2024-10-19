use std::{error::Error, io::Read, net::TcpStream};

use crate::messagestuff::message2::Serialize;

use super::message2::*;

#[derive(Debug)]
pub struct QueryMessage {
    pub query: String,
    pub consistency: u16,
}

impl Serialize for QueryMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&longstring_to_bytes(self.query.clone()));
        bytes.extend_from_slice(&short_to_bytes(self.consistency));
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), Box<dyn Error>> {
        let (_query, resto) = longstring_from_bytes(bytes)?;
        let (_consistency, resto) = short_from_bytes(resto)?;
        Ok((QueryMessage{query: _query, consistency: _consistency}, resto))
    }
}

impl IntoEnvelope for QueryMessage {
    fn into_envelope(self, stream: u16) -> Envelope {
        let _body = self.to_bytes();
        Envelope {
            version: 0x05 as u8,
            flags: 0 as u8,
            stream,
            opcode: Opcode::QueryMessage,
            length: _body.len() as i32,
            body: _body
        }
    }
}