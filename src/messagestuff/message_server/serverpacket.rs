// Los nodos del servidor se hablan entre si utlizando Packets:
//   - Packet cumple con el protocolo de Cassandra.
//   - Packet guarda por dentro el tipo de OpCode y el mensaje de tipo ServerMessage.
//   - ServerMessage es un enum que contiene los diferentes tipos de mensajes que se pueden enviar.
//   - Para enviar el packet y leerlo por stream estan las funciones to_bytes() y from_stream().

use crate::config;
use crate::config::SHOW_GOSSIP_LOGGER;
use crate::encryption::encrypt::{decrypt_body_aes128, encrypt_body_aes128};
use crate::errors::my_errors::MyError;
use crate::loggerstuff::logger::Logger;
use crate::messagestuff::{
    message::*, message_server::errorservermessage::ErrorServerMessage,
    message_server::gossipmessage::GossipMessage,
    message_server::queryservermessage::QueryServerMessage,
    message_server::readrepairmessage::ReadRepairMessage,
    message_server::resultservermessage::ResultServerMessage,
};
use std::sync::{Arc, RwLock};
use std::{io::Read, net::TcpStream};

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum ServerOpcode {
    GossipMessage = config::GOSSIP,
    QueryServerMessage = config::QUERY,
    ErrorServerMessage = config::ERROR,
    ResultServerMessage = config::RESULT,
    ReadRepairMessage = config::READ_REPAIR,
}

impl ServerOpcode {
    pub fn from_u8(value: u8) -> Result<Self, MyError> {
        match value {
            config::GOSSIP => Ok(ServerOpcode::GossipMessage),
            config::QUERY => Ok(ServerOpcode::QueryServerMessage),
            config::ERROR => Ok(ServerOpcode::ErrorServerMessage),
            config::RESULT => Ok(ServerOpcode::ResultServerMessage),
            config::READ_REPAIR => Ok(ServerOpcode::ReadRepairMessage),
            _ => Err(MyError::Error("Invalid Opcode".to_string())),
        }
    }
}

#[derive(Debug)]
pub enum ServerMessage {
    Gossip(GossipMessage),
    Query(QueryServerMessage),
    Result(ResultServerMessage),
    ErrorServer(ErrorServerMessage),
    ReadRepair(ReadRepairMessage),
}

#[derive(Debug)]
pub struct Packet {
    pub opcode: ServerOpcode,
    pub length: i32,
    pub body: ServerMessage,
}

impl Packet {
    /// Para serializar el mensaje
    pub fn to_bytes(&self, stream: &mut TcpStream, logger: Arc<RwLock<Logger>>) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&byte_to_byte(self.opcode as u8));

        let body_bytes = match &self.body {
            ServerMessage::Gossip(msg) => msg.to_bytes(),
            ServerMessage::ErrorServer(msg) => msg.to_bytes(),
            ServerMessage::Query(msg) => msg.to_bytes(),
            ServerMessage::Result(msg) => msg.to_bytes(),
            ServerMessage::ReadRepair(msg) => msg.to_bytes(),
        };

        let final_body = match encrypt_body_aes128(body_bytes.clone()) {
            Ok(encrypted_body) => encrypted_body,
            Err(_) => body_bytes,
        };
        let final_length = final_body.len() as i32;
        bytes.extend_from_slice(&int_to_bytes(final_length));
        bytes.extend_from_slice(&final_body);

        let _ = Packet::write_log(stream, logger, self, "[SENT]    ".to_string());
        bytes
    }

    /// Para deseriarlizar el mensaje
    pub fn from_stream(
        stream: &mut TcpStream,
        logger: Arc<RwLock<Logger>>,
    ) -> Result<Self, MyError> {
        let _ = stream.set_nonblocking(false);
        let mut header_buffer = [0u8; 5];
        if let Err(e) = stream.read_exact(&mut header_buffer) {
            return Err(MyError::ServerMessageError(format!(
                "Error al leer el <header> del packet desde Stream{:?}",
                e
            )));
        };
        let _opcode = ServerOpcode::from_u8(header_buffer[0])?;
        let _length = i32::from_be_bytes([
            header_buffer[1],
            header_buffer[2],
            header_buffer[3],
            header_buffer[4],
        ]);

        let mut body_buffer = vec![0u8; _length as usize];
        if stream.read_exact(&mut body_buffer).is_err() {
            return Err(MyError::ServerMessageError(
                "Error al leer el <body> desde Stream".to_string(),
            ));
        }

        let decrypted_body = decrypt_body_aes128(body_buffer)?;
        let body_length = decrypted_body.len() as i32;

        let _body = match _opcode {
            ServerOpcode::GossipMessage => {
                let (gossip, _) = GossipMessage::from_bytes(decrypted_body)?;
                ServerMessage::Gossip(gossip)
            }
            ServerOpcode::ErrorServerMessage => {
                let (errorserver, _) = ErrorServerMessage::from_bytes(decrypted_body)?;
                ServerMessage::ErrorServer(errorserver)
            }
            ServerOpcode::QueryServerMessage => {
                let (query, _) = QueryServerMessage::from_bytes(decrypted_body)?;
                ServerMessage::Query(query)
            }
            ServerOpcode::ResultServerMessage => {
                let (result, _) = ResultServerMessage::from_bytes(decrypted_body)?;
                ServerMessage::Result(result)
            }
            ServerOpcode::ReadRepairMessage => {
                let (readrepair, _) = ReadRepairMessage::from_bytes(decrypted_body)?;
                ServerMessage::ReadRepair(readrepair)
            }
        };
        let packet = Packet {
            opcode: _opcode,
            length: body_length,
            body: _body,
        };

        Packet::write_log(stream, logger, &packet, "[RECEIVED]".to_string())?;
        Ok(packet)
    }

    fn write_log(
        stream: &mut TcpStream,
        logger: Arc<RwLock<Logger>>,
        packet: &Packet,
        msg_type: String,
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

        match &packet.body {
            ServerMessage::Gossip(msg) => {
                if SHOW_GOSSIP_LOGGER {
                    let message = format!(
                        "[SERVER] {} [{}] Gossip protocol. Node at: {} with status: {:?}",
                        msg_type, ip, msg.ip, msg.status
                    );
                    logger_guard.write_log(message.as_str())?;
                }
            }
            ServerMessage::Query(msg) => {
                let message = format!("[SERVER] {} [{}] Query: {}", msg_type, ip, msg.query.trim());
                logger_guard.write_log(message.as_str())?;
            }
            ServerMessage::Result(msg) => {
                let message = format!("[SERVER] {} [{}] Result: {}", msg_type, ip, msg);
                logger_guard.write_log(message.as_str())?;
            }
            ServerMessage::ErrorServer(msg) => {
                let message = format!("[SERVER] {} [{}] Error: {}", msg_type, ip, msg.error);
                logger_guard.write_log(message.as_str())?;
            }
            ServerMessage::ReadRepair(msg) => {
                let message = format!(
                    "[SERVER] {} [{}] Read Repair protocol. Rows to keep: {:?}",
                    msg_type, ip, msg.rows_to_keep
                );
                logger_guard.write_log(message.as_str())?;
            }
        };
        Ok(())
    }
}

pub trait IntoPacket: std::fmt::Debug {
    fn into_packet(self) -> Packet;
}
