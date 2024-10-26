use crate::config;
use crate::messagestuff::{
    message::*, message_cql::errormessage::ErrorMessage, message_cql::querymessage::QueryMessage,
    message_cql::resultmessage::ResultMessage,
};
use std::{error::Error, io::Read, net::TcpStream};

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum CqlOpcode {
    QueryMessage = config::QUERY,
    ErrorMessage = config::ERROR,
    ResultMessage = config::RESULT,
}

impl CqlOpcode {
    pub fn from_u8(value: u8) -> Result<Self, Box<dyn Error>> {
        match value {
            config::QUERY => Ok(CqlOpcode::QueryMessage),
            config::ERROR => Ok(CqlOpcode::ErrorMessage),
            config::RESULT => Ok(CqlOpcode::ResultMessage),
            _ => Err(Box::from("Invalid Opcode")),
        }
    }
}

#[derive(Debug)]
pub enum CqlMessage {
    Query(QueryMessage),
    Result(ResultMessage),
    Error(ErrorMessage),
}

#[derive(Debug)]
pub struct Envelope {
    pub version: u8, // 0x05 for Requests. 0x85 for Response
    pub flags: u8,
    pub stream: u16,
    pub opcode: CqlOpcode,
    pub length: i32,
    pub body: CqlMessage,
}

impl Envelope {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&byte_to_byte(self.version));
        bytes.extend_from_slice(&byte_to_byte(self.flags));
        bytes.extend_from_slice(&short_to_bytes(self.stream));
        bytes.extend_from_slice(&byte_to_byte(self.opcode as u8));
        bytes.extend_from_slice(&int_to_bytes(self.length));

        match &self.body {
            CqlMessage::Query(msg) => bytes.extend_from_slice(&msg.to_bytes()),
            CqlMessage::Result(msg) => bytes.extend_from_slice(&msg.to_bytes()),
            CqlMessage::Error(msg) => bytes.extend_from_slice(&msg.to_bytes()),
        }

        bytes
    }

    pub fn from_stream(stream: &mut TcpStream) -> Result<Self, Box<dyn Error>> {
        let _ = stream.set_nonblocking(false);
        let mut header_buffer = [0u8; 9];
        if let Err(_) = stream.read_exact(&mut header_buffer) {
            return Err(Box::from(
                "Error al leer el <header> del envelope desde Stream",
            ));
        }
        let _version = header_buffer[0];
        let _flags = header_buffer[1];
        let _stream_id = u16::from_be_bytes([header_buffer[2], header_buffer[3]]);
        let _opcode = CqlOpcode::from_u8(header_buffer[4])?;
        let _length = i32::from_be_bytes([
            header_buffer[5],
            header_buffer[6],
            header_buffer[7],
            header_buffer[8],
        ]);

        let mut _body_buffer = vec![0u8; _length as usize];
        if let Err(_) = stream.read_exact(&mut _body_buffer) {
            return Err(Box::from("Error al leer el <body> desde Stream"));
        }
        let _body = match _opcode {
            CqlOpcode::ErrorMessage => {
                let (error, _) = ErrorMessage::from_bytes(_body_buffer)?;
                CqlMessage::Error(error)
            }
            CqlOpcode::QueryMessage => {
                let (query, _) = QueryMessage::from_bytes(_body_buffer)?;
                CqlMessage::Query(query)
            }
            CqlOpcode::ResultMessage => {
                let (result, _) = ResultMessage::from_bytes(_body_buffer)?;
                CqlMessage::Result(result)
            }
        };
        Ok(Envelope {
            version: _version,
            flags: _flags,
            stream: _stream_id,
            opcode: _opcode,
            length: _length,
            body: _body,
        })
    }
}

pub trait IntoEnvelope: std::fmt::Debug {
    fn into_envelope(self) -> Envelope;
}
