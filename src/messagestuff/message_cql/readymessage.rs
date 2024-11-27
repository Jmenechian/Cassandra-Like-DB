use crate::errors::my_errors::MyError;
use crate::messagestuff::message::*;
use crate::messagestuff::message_cql::cqlenvelope::*;
use crate::config::*;

#[derive(Debug, PartialEq)]
pub struct ReadyMessage {
    
}

impl Serialize for ReadyMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), MyError> {
        Ok((
            ReadyMessage {
        
            },
            Vec::<u8>::new(),
        ))
    }
}

impl IntoEnvelope for ReadyMessage {
    fn into_envelope(self) -> Envelope {
        let _body = self.to_bytes();
        Envelope {
            version: RESPONSE,
            flags: 0_u8,      //0 as u8,
            stream: 0_u16,    //0 as u16,
            opcode: CqlOpcode::ReadyMessage,
            length: _body.len() as i32,
            body: CqlMessage::Ready(self),
        }
    }
}
