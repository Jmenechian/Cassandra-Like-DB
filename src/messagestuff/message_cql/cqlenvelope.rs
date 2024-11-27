// Aca esta la logica de la mensajeria entre Cliente y Servidor
// Cumple con el protocolo de comunicacion nativo de Cassandra.
//
// - 'Envelope`: Contiene al mensaje y a toda su metadata
// - `CqlOpcode`: Enumeracion de los tipos de mensajes
// - `CqlMessage`: Contiene al mensaje en si

use crate::config;
use crate::errors::my_errors::MyError;
use crate::messagestuff::{
    message::*, 
    message_cql::errormessage::ErrorMessage, 
    message_cql::querymessage::QueryMessage,
    message_cql::resultmessage::ResultMessage,
    message_cql::optionmessage::OptionMessage,
    message_cql::readymessage::ReadyMessage,
    message_cql::startupmessage::StartupMessage,
    encrypt::*,
};
use std::{io::Read, net::TcpStream};

use super::supportedmessage::SupportedMessage;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum CqlOpcode {
    QueryMessage = config::QUERY,
    ErrorMessage = config::ERROR,
    ResultMessage = config::RESULT,
    OptionMessage = config::OPTIONS,
    SupportedMessage = config::SUPPORTED,
    StartupMessage = config::STARTUP,
    ReadyMessage = config::READY,
}

impl CqlOpcode {
    pub fn from_u8(value: u8) -> Result<Self, MyError> {
        match value {
            config::QUERY => Ok(CqlOpcode::QueryMessage),
            config::ERROR => Ok(CqlOpcode::ErrorMessage),
            config::RESULT => Ok(CqlOpcode::ResultMessage),
            config::OPTIONS => Ok(CqlOpcode::OptionMessage),
            config::SUPPORTED => Ok(CqlOpcode::SupportedMessage),
            config::STARTUP => Ok(CqlOpcode::StartupMessage),
            config::READY => Ok(CqlOpcode::ReadyMessage),
            _ => Err(MyError::CqlMessageError(format!("Invalid Opcode: {:?}", value))),
        }
    }
}

#[derive(Debug)]
pub enum CqlMessage {
    Query(QueryMessage),
    Result(ResultMessage),
    Error(ErrorMessage),
    Option(OptionMessage),
    Supported(SupportedMessage),
    Startup(StartupMessage),
    Ready(ReadyMessage),
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
        let mut final_body =  match &self.body {
            CqlMessage::Query(msg) => msg.to_bytes(),
            CqlMessage::Result(msg) => msg.to_bytes(),
            CqlMessage::Error(msg) => msg.to_bytes(),
            CqlMessage::Option(msg) => msg.to_bytes(),
            CqlMessage::Supported(msg )  => msg.to_bytes(),
            CqlMessage::Startup(msg) => msg.to_bytes(),
            CqlMessage::Ready(msg) => msg.to_bytes(),
        };
        let final_length ;
        if false {
            final_body = match encrypt_body_aes128(final_body.clone()) {
                Ok(encrypted_body) => encrypted_body,
                Err(_) => final_body,
            };
            final_length = final_body.len() as i32;
        } else {
            final_length = self.length;
        }
        
        
        bytes.extend_from_slice(&int_to_bytes(final_length));
        bytes.extend_from_slice(&final_body);

        bytes
    }

    pub fn from_stream(stream: &mut TcpStream) -> Result<Self, MyError> {
        let _ = stream.set_nonblocking(false);
        let mut header_buffer = [0u8; 9];
        if let Err(e) = stream.read_exact(&mut header_buffer) {
            let _error = format!(
                "Error al leer el <header> del envelope desde Stream: {:?}",
                e
            );
            return Err(MyError::TcpStreamError(_error));
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
        if let Err(e) = stream.read_exact(&mut _body_buffer) {
            let _error = format!("Error al leer el <body> desde Stream: {:?}", e);
            return Err(MyError::TcpStreamError(_error));
        }

        let decrypted_body = decrypt_body_aes128(_body_buffer)?;
        let body_length =decrypted_body.len() as i32;

        let _body = match _opcode {
            CqlOpcode::ErrorMessage => {
                let (error, _) = ErrorMessage::from_bytes(decrypted_body)?;
                CqlMessage::Error(error)
            }
            CqlOpcode::QueryMessage => {
                let (query, _) = QueryMessage::from_bytes(decrypted_body)?;
                CqlMessage::Query(query)
            }
            CqlOpcode::ResultMessage => {
                let (result, _) = ResultMessage::from_bytes(decrypted_body)?;
                CqlMessage::Result(result)
            }
            CqlOpcode::OptionMessage => {
                let (option, _) = OptionMessage::from_bytes(decrypted_body)?;
                CqlMessage::Option(option)
            }
            CqlOpcode::SupportedMessage => {
                let(supported, _) = SupportedMessage::from_bytes(decrypted_body)?;
                CqlMessage::Supported(supported)
            }
            CqlOpcode::StartupMessage => {
                let(startup, _) = StartupMessage::from_bytes(decrypted_body)?;
                CqlMessage::Startup(startup)
            }
            CqlOpcode::ReadyMessage => {
                let(ready, _) = ReadyMessage::from_bytes(decrypted_body)?;
                CqlMessage::Ready(ready)
            }
        };
        Ok(Envelope {
            version: _version,
            flags: _flags,
            stream: _stream_id,
            opcode: _opcode,
            length: body_length,
            body: _body,
        })
    }
}

pub trait IntoEnvelope: std::fmt::Debug {
    fn into_envelope(self) -> Envelope;
}
