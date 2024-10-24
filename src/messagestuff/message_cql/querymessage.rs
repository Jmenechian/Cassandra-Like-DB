use std::error::Error;
use crate::messagestuff::message::*;
use crate::messagestuff::message_cql::cqlenvelope::*;
use crate::messagestuff::message_server::serverpacket::*;




#[derive(Debug, PartialEq)]
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
    fn into_envelope(self) -> Envelope {
        let _body = self.to_bytes();
        Envelope {
            version: 0x05 as u8,
            flags: 0 as u8,
            stream: 0 as u16,
            opcode: CqlOpcode::QueryMessage,
            length: _body.len() as i32,
            body: CqlMessage::Query(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config;
    use super::*;

    #[test]
    fn test_query_message() -> Result<(), Box<dyn Error>> {
        let expected_query: QueryMessage = QueryMessage{query: "Hola".to_string(), consistency: config::ONE};
        let bytes: Vec<u8> = expected_query.to_bytes();
        let (_query, _) = QueryMessage::from_bytes(bytes)?;
        Ok(assert_eq!(_query, expected_query))
    }
}