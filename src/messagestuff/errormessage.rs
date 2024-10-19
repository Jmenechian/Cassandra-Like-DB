use std::{error::Error, io::Read, net::TcpStream};

use crate::messagestuff::message2::Serialize;

use super::message2::*;

#[derive(Debug)]
pub struct ErrorMessage {
    error: String,
    code: i32,
}

impl Serialize for ErrorMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&int_to_bytes(self.code));
        bytes.extend_from_slice(&string_to_bytes(self.error.clone()));
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), Box<dyn Error>> {
        let (_code, resto) = int_from_bytes(bytes)?;
        let (_error, resto) = string_from_bytes(resto)?;

        Ok((ErrorMessage{error: _error, code: _code}, resto))
    }
}

impl IntoEnvelope for ErrorMessage {
    fn into_envelope(self, stream: u16) -> Envelope {
        let _body = self.to_bytes();
        Envelope {
            version: 0x85 as u8,
            flags: 0 as u8,
            stream,
            opcode: Opcode::ErrorMessage,
            length: _body.len() as i32,
            body: _body
        }
    }
}