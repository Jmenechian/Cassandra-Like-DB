use crate::errors::my_errors::MyError;
use crate::messagestuff::message::*;
use crate::messagestuff::message_cql::cqlenvelope::*;
use crate::config::*;

#[derive(Debug, PartialEq)]
pub struct StartupMessage {
    pub map: Vec<(String, String)>
}

impl Serialize for StartupMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&stringmap_to_bytes(self.map.clone()));
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), MyError> {
        let (_map, resto) = stringmap_from_bytes(bytes)?;

        Ok((
            StartupMessage {
                map: _map,
            },
            resto,
        ))
    }
}

impl IntoEnvelope for StartupMessage {
    fn into_envelope(self, cql_stream: u16) -> Envelope {
        let _body = self.to_bytes();
        Envelope {
            version: REQUEST,
            flags: 0_u8,      //0 as u8,
            stream: cql_stream,    //0 as u16,
            opcode: CqlOpcode::StartupMessage,
            length: _body.len() as i32,
            body: CqlMessage::Startup(self),
        }
    }
}