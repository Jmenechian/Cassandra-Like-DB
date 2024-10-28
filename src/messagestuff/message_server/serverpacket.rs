// Los nodos del servidor se hablan entre si utlizando Packets:
//   - Packet cumple con el protocolo de Cassandra.
//   - Packet guarda por dentro el tipo de OpCode y el mensaje de tipo ServerMessage.
//   - ServerMessage es un enum que contiene los diferentes tipos de mensajes que se pueden enviar.
//   - Para enviar el packet y leerlo por stream estan las funciones to_bytes() y from_stream().

use crate::config;
use crate::messagestuff::{
    message::*, message_server::errorservermessage::ErrorServerMessage,
    message_server::gossipmessage::GossipMessage,
    message_server::queryservermessage::QueryServerMessage,
    message_server::resultservermessage::ResultServerMessage,
};
use crate::my_errors::MyError;
use std::{io::Read, net::TcpStream};

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum ServerOpcode {
    GossipMessage = config::GOSSIP,
    QueryServerMessage = config::QUERY,
    ErrorServerMessage = config::ERROR,
    ResultServerMessage = config::RESULT,
}

impl ServerOpcode {
    pub fn from_u8(value: u8) -> Result<Self, MyError> {
        match value {
            config::GOSSIP => Ok(ServerOpcode::GossipMessage),
            config::QUERY => Ok(ServerOpcode::QueryServerMessage),
            config::ERROR => Ok(ServerOpcode::ErrorServerMessage),
            config::RESULT => Ok(ServerOpcode::ResultServerMessage),
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
}

#[derive(Debug)]
pub struct Packet {
    pub opcode: ServerOpcode,
    pub length: i32,
    pub body: ServerMessage,
}

impl Packet {
    // Para serializar el mensaje
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&byte_to_byte(self.opcode as u8));
        bytes.extend_from_slice(&int_to_bytes(self.length));

        match &self.body {
            ServerMessage::Gossip(msg) => bytes.extend_from_slice(&msg.to_bytes()),
            ServerMessage::ErrorServer(msg) => bytes.extend_from_slice(&msg.to_bytes()),
            ServerMessage::Query(msg) => bytes.extend_from_slice(&msg.to_bytes()),
            ServerMessage::Result(msg) => bytes.extend_from_slice(&msg.to_bytes()),
        }
        bytes
    }

    // Para deseriarlizar el mensaje
    pub fn from_stream(stream: &mut TcpStream) -> Result<Self, MyError> {
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
        let _body = match _opcode {
            ServerOpcode::GossipMessage => {
                let (gossip, _) = GossipMessage::from_bytes(body_buffer)?;
                ServerMessage::Gossip(gossip)
            }
            ServerOpcode::ErrorServerMessage => {
                let (errorserver, _) = ErrorServerMessage::from_bytes(body_buffer)?;
                ServerMessage::ErrorServer(errorserver)
            }
            ServerOpcode::QueryServerMessage => {
                let (query, _) = QueryServerMessage::from_bytes(body_buffer)?;
                ServerMessage::Query(query)
            }
            ServerOpcode::ResultServerMessage => {
                let (result, _) = ResultServerMessage::from_bytes(body_buffer)?;
                ServerMessage::Result(result)
            }
        };
        Ok(Packet {
            opcode: _opcode,
            length: _length,
            body: _body,
        })
    }
}

pub trait IntoPacket: std::fmt::Debug {
    fn into_packet(self) -> Packet;
}
