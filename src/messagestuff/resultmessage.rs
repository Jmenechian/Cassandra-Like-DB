use std::{error::Error, io::Read, net::TcpStream};

use crate::messagestuff::message2::Serialize;
use crate::config;
use super::message2::*;

#[derive(Copy, Clone, Debug)]
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

#[derive(Debug)]
pub enum ResultBody {
    Void(ResultVoidMessage),
    Rows(ResultRowsMessage),
}

#[derive(Debug)]
pub struct ResultVoidMessage;



#[derive(Debug)]
pub struct ResultRowsMessage {
    pub rows_count: i32,
    pub rows_content: Vec<String>,
}

impl Serialize for ResultRowsMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&int_to_bytes(self.rows_count));
        for row in &self.rows_content {
            bytes.extend_from_slice(&string_to_bytes(row.clone()));
        }
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), Box<dyn Error>> {
        let (_rows_count, resto) = int_from_bytes(bytes)?;
        let mut _bytes = resto;
        let mut _rows_content: Vec<String> = Vec::new();
        for _ in 0.._rows_count {
            let (_row, resto) = string_from_bytes(_bytes)?;
            _rows_content.push(_row);
            _bytes = resto;
        }
        Ok((ResultRowsMessage{rows_count: _rows_count, rows_content: _rows_content}, _bytes))
    }
}

#[derive(Debug)]
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
    fn into_envelope(self, stream: u16) -> Envelope {
        let _body = self.to_bytes();
        Envelope {
            version: 0x85 as u8,
            flags: 0 as u8,
            stream,
            opcode: Opcode::ResultMessage,
            length: _body.len() as i32,
            body: _body
        }
    }
}