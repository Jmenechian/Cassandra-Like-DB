use crate::errors::my_errors::MyError;
use crate::messagestuff::message::*;
use crate::messagestuff::message_cql::cqlenvelope::*;
use crate::config::*;

#[derive(Debug, PartialEq)]
pub struct SupportedMessage {
    pub multimap: Vec<(String, Vec<String>)>
}

impl Serialize for SupportedMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&stringmultimap_to_bytes(self.multimap.clone()));
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), MyError> {
        let (_multimap, resto) = stringmultimap_from_bytes(bytes)?;

        Ok((
            SupportedMessage {
                multimap: _multimap,
            },
            resto,
        ))
    }
}

impl IntoEnvelope for SupportedMessage {
    fn into_envelope(self, cql_stream: u16) -> Envelope {
        let _body = self.to_bytes();
        Envelope {
            version: RESPONSE,
            flags: 0_u8,      //0 as u8,
            stream: cql_stream,    //0 as u16,
            opcode: CqlOpcode::SupportedMessage,
            length: _body.len() as i32,
            body: CqlMessage::Supported(self),
        }
    }
}