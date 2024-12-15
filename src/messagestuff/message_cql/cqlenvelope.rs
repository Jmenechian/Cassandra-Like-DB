// Aca esta la logica de la mensajeria entre Cliente y Servidor
// Cumple con el protocolo de comunicacion nativo de Cassandra.
//
// - 'Envelope`: Contiene al mensaje y a toda su metadata
// - `CqlOpcode`: Enumeracion de los tipos de mensajes
// - `CqlMessage`: Contiene al mensaje en si

use crate::config;
use crate::encryption::encrypt::{decrypt_body_aes128, encrypt_body_aes128};
use crate::errors::my_errors::MyError;
use crate::loggerstuff::logger::Logger;
use crate::messagestuff::{
    message::*, message_cql::errormessage::ErrorMessage, message_cql::querymessage::QueryMessage,
    message_cql::resultmessage::ResultMessage,
};
use std::sync::{Arc, RwLock};
use std::{io::Read, net::TcpStream};

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum CqlOpcode {
    QueryMessage = config::QUERY,
    ErrorMessage = config::ERROR,
    ResultMessage = config::RESULT,
}

impl CqlOpcode {
    pub fn from_u8(value: u8) -> Result<Self, MyError> {
        match value {
            config::QUERY => Ok(CqlOpcode::QueryMessage),
            config::ERROR => Ok(CqlOpcode::ErrorMessage),
            config::RESULT => Ok(CqlOpcode::ResultMessage),
            _ => Err(MyError::CqlMessageError("Invalid Opcode".to_string())),
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
    pub fn to_bytes(&self, stream: &mut TcpStream, logger: Option<Arc<RwLock<Logger>>>) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&byte_to_byte(self.version));
        bytes.extend_from_slice(&byte_to_byte(self.flags));
        bytes.extend_from_slice(&short_to_bytes(self.stream));
        bytes.extend_from_slice(&byte_to_byte(self.opcode as u8));
        //bytes.extend_from_slice(&int_to_bytes(self.length));

        let body_bytes = match &self.body {
            CqlMessage::Query(msg) => msg.to_bytes(),
            CqlMessage::Result(msg) => msg.to_bytes(),
            CqlMessage::Error(msg) => msg.to_bytes(),
        };
        let final_body = match encrypt_body_aes128(body_bytes.clone()) {
            Ok(encrypted_body) => encrypted_body,
            Err(_) => body_bytes,
        };
        let final_length = final_body.len() as i32;
        bytes.extend_from_slice(&int_to_bytes(final_length));
        bytes.extend_from_slice(&final_body);

        if let Some(logger) = logger {
            let _ = Envelope::write_log(stream, logger, self);
        }
        bytes
    }

    pub fn from_stream(
        stream: &mut TcpStream,
        logger: Option<Arc<RwLock<Logger>>>,
    ) -> Result<Self, MyError> {
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
        let body_length = decrypted_body.len() as i32;

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
        };

        let env = Envelope {
            version: _version,
            flags: _flags,
            stream: _stream_id,
            opcode: _opcode,
            length: body_length,
            body: _body,
        };

        if let Some(logger) = logger {
            Envelope::write_log(stream, logger, &env)?;
        }
        Ok(env)
    }

    fn write_log(
        stream: &mut TcpStream,
        logger: Arc<RwLock<Logger>>,
        env: &Envelope,
    ) -> Result<(), MyError> {
        let mut logger_guard = match logger.write() {
            Ok(l) => l,
            Err(e) => {
                let error = format!("Error al leer el logger: {:?}", e);
                return Err(MyError::LockingError(error));
            }
        };

        let ip = match stream.peer_addr() {
            Ok(addr) => format!("{:?}", addr),
            Err(_) => "".to_string(),
        };

        match &env.body {
            CqlMessage::Query(msg) => {
                let message = format!("[CLIENT] [RECEIVED] [{}] Query: {}", ip, msg.query.trim());
                logger_guard.write_log(message.as_str())?;
            }
            CqlMessage::Result(msg) => {
                let message = format!("[CLIENT] [SENT]     [{}] Result from query: {}", ip, msg);
                logger_guard.write_log(message.as_str())?;
            }
            CqlMessage::Error(msg) => {
                let message = format!(
                    "[CLIENT] [SENT]     [{}] Error from server: {}",
                    ip, msg.error
                );
                logger_guard.write_log(message.as_str())?;
            }
        }
        Ok(())
    }
}

pub trait IntoEnvelope: std::fmt::Debug {
    fn into_envelope(self) -> Envelope;
}
