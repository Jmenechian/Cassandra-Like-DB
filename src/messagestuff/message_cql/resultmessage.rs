use std::error::Error;
use crate::config;
use crate::messagestuff::message::*;
use crate::messagestuff::message_cql::{
    cqlenvelope::*,
    resultrowsmessage::ResultRowsMessage, 
    resultvoidmessage::ResultVoidMessage,
};

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(i32)]
pub enum ResultKind {
    Void = config::VOID,
    Rows = config::ROWS, 
}

impl ResultKind {
    pub fn from_i32(value: i32) -> Result<Self, Box<dyn Error>> {
        match value {
            config::VOID => Ok(ResultKind::Void),
            config::ROWS => Ok(ResultKind::Rows),
            _ => Err(Box::from("Invalid Result Kind"))
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ResultBody {
    Void(ResultVoidMessage),
    Rows(ResultRowsMessage),
}

#[derive(Debug, PartialEq)]
pub struct ResultMessage {
    pub kind: ResultKind,
    pub body: ResultBody,
}

impl Serialize for ResultMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(& int_to_bytes(self.kind as i32));
        match &self.body {
            ResultBody::Void(_) => {}
            ResultBody::Rows(rows) => {
                bytes.extend_from_slice(&rows.to_bytes());
            }
        }
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), Box<dyn Error>> {
        let (value, resto) = int_from_bytes(bytes)?;
        let _kind = ResultKind::from_i32(value)?;
        let (_body, resto) = match _kind {
            ResultKind::Void => (ResultBody::Void(ResultVoidMessage), resto),
            ResultKind::Rows => {
                let (rows, resto) = ResultRowsMessage::from_bytes(resto)?;
                (ResultBody::Rows(rows), resto)
            }
        };

        Ok((ResultMessage{kind: _kind, body: _body}, resto))
    }
}

impl IntoEnvelope for ResultMessage {
    fn into_envelope(self) -> Envelope {
        let _body = self.to_bytes();
        Envelope {
            version: 0x85 as u8,
            flags: 0 as u8,
            stream: 0 as u16,
            opcode: CqlOpcode::ResultMessage,
            length: _body.len() as i32,
            body: CqlMessage::Result(self)
        }
    }
}

pub trait IntoResultMessage: std::fmt::Debug {
    fn into_result_message(self) -> ResultMessage;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_rows_message() -> Result<(), Box<dyn Error>> {
        let expected_result: ResultMessage = ResultMessage{
            kind: ResultKind::Rows, 
            body: ResultBody::Rows(ResultRowsMessage{
                rows_count: 1 as i32, 
                rows_content: vec!["hola".to_string()],
            })
        };
        let bytes: Vec<u8> = expected_result.to_bytes();
        let (_result, _) = ResultMessage::from_bytes(bytes)?;
        Ok(assert_eq!(_result, expected_result))
    }

    #[test]
    fn test_result_rows_message_2() -> Result<(), Box<dyn Error>> {
        let expected_result: ResultMessage = ResultMessage{
            kind: ResultKind::Rows, 
            body: ResultBody::Rows(ResultRowsMessage{
                rows_count: 2 as i32, 
                rows_content: vec!["hola".to_string(), "chau".to_string()],
            })
        };
        let bytes: Vec<u8> = expected_result.to_bytes();
        let (_result, _) = ResultMessage::from_bytes(bytes)?;
        Ok(assert_eq!(_result, expected_result))
    }

    #[test]
    fn test_result_rows_message_3() -> Result<(), Box<dyn Error>> {
        let expected_result: ResultMessage = ResultMessage{
            kind: ResultKind::Rows, 
            body: ResultBody::Rows(ResultRowsMessage{
                rows_count: 2 as i32, 
                rows_content: vec!["hola".to_string(), "chau".to_string()],
            })
        };
        let mut bytes: Vec<u8> = expected_result.to_bytes();
        bytes.pop();
        if let Err(e) = ResultMessage::from_bytes(bytes) {
            let error = e.to_string();
            return Ok(assert_eq!(error, "No hay suficientes bytes prar leer el <String>"));
        }
        Ok(())
    }
    #[test]
    fn test_result_rows_message_4() -> Result<(), Box<dyn Error>> {
        let expected_result: ResultMessage = ResultMessage{
            kind: ResultKind::Rows, 
            body: ResultBody::Rows(ResultRowsMessage{
                rows_count: 3 as i32, 
                rows_content: vec!["hola".to_string(), "chau".to_string()],
            })
        };
        let mut bytes: Vec<u8> = expected_result.to_bytes();
        bytes.pop();
        if let Err(e) = ResultMessage::from_bytes(bytes) {
            let error = e.to_string();
            return Ok(assert_eq!(error, "No hay suficientes bytes prar leer el <String>"));
        }
        Ok(())
    }

    #[test]
    fn test_result_void_message() -> Result<(), Box<dyn Error>> {
        let expected_result: ResultMessage = ResultMessage{kind: ResultKind::Void, body: ResultBody::Void(ResultVoidMessage)};
        let bytes: Vec<u8> = expected_result.to_bytes();
        let (_result, _) = ResultMessage::from_bytes(bytes)?;
        Ok(assert_eq!(_result, expected_result))
    }
}