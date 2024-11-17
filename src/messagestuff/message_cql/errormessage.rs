use crate::errors::my_errors::MyError;
use crate::messagestuff::message::*;
use crate::messagestuff::message_cql::cqlenvelope::*;

#[derive(Debug, PartialEq)]
pub struct ErrorMessage {
    pub error: String,
    pub code: i32,
}

impl Serialize for ErrorMessage {
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
            ErrorMessage {
                error: _error,
                code: _code,
            },
            resto,
        ))
    }
}

impl IntoEnvelope for ErrorMessage {
    fn into_envelope(self) -> Envelope {
        let _body = self.to_bytes();
        Envelope {
            version: 0x85_u8, //0x85 as u8,
            flags: 0_u8,      //0 as u8,
            stream: 0_u16,    //0 as u16,
            opcode: CqlOpcode::ErrorMessage,
            length: _body.len() as i32,
            body: CqlMessage::Error(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_message() -> Result<(), MyError> {
        let expected_error: ErrorMessage = ErrorMessage {
            error: "Hola".to_string(),
            code: 0 as i32,
        };
        let bytes: Vec<u8> = expected_error.to_bytes();
        let (_error, _) = ErrorMessage::from_bytes(bytes)?;
        Ok(assert_eq!(_error, expected_error))
    }
}
