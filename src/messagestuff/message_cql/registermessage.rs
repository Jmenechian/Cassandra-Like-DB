use crate::errors::my_errors::MyError;
use crate::messagestuff::message::*;
use crate::messagestuff::message_cql::cqlenvelope::*;
use crate::config::*;

#[derive(Debug, PartialEq)]
pub struct RegisterMessage {
    pub stringlist: Vec<String>
}

impl Serialize for RegisterMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&stringlist_to_bytes(self.stringlist.clone()));
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), MyError> {
        let (_stringlist, resto) = stringlist_from_bytes(bytes)?;

        Ok((
            RegisterMessage {
                stringlist: _stringlist,
            },
            resto,
        ))
    }
}

impl IntoEnvelope for RegisterMessage {
    fn into_envelope(self, cql_stream: u16) -> Envelope {
        let _body = self.to_bytes();
        Envelope {
            version: REQUEST,
            flags: 0_u8,      //0 as u8,
            stream: cql_stream,    //0 as u16,
            opcode: CqlOpcode::RegisterMessage,
            length: _body.len() as i32,
            body: CqlMessage::Register(self),
        }
    }
}