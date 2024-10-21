use std::{error::Error, io::Read, net::TcpStream};
use crate::config;
use crate::messagestuff::{
    message::*,
    message_cql::querymessage::QueryMessage,
    message_cql::resultmessage::ResultMessage,
    message_cql::resultrowsmessage::ResultRowsMessage,
    message_cql::resultvoidmessage::ResultVoidMessage,
    message_server::gossipmessage::GossipMessage,
    message_server::errorservermessage::ErrorServerMessage,
};

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum ServerOpcode {
    GossipMessage = config::GOSSIP,
    QueryMessage = config::QUERY,
    ErrorServerMessage = config::ERROR,
    ResultMessage = config::RESULT,
}

impl ServerOpcode {
    pub fn from_u8(value: u8) -> Result<Self, Box<dyn Error>> {
        match value {
            config::GOSSIP => Ok(ServerOpcode::GossipMessage),
            config::QUERY => Ok(ServerOpcode::QueryMessage),
            config::ERROR => Ok(ServerOpcode::ErrorServerMessage),
            config::RESULT => Ok(ServerOpcode::ResultMessage),
            _ => Err(Box::from("Invalid Opcode")),
        }
    }
}

#[derive(Debug)]
pub enum ServerMessage {
    Gossip(GossipMessage),
    Query(QueryMessage),
    Result(ResultMessage),
    ErrorServer(ErrorServerMessage),
}

impl IntoPacket for ServerMessage {
    fn into_packet(self) -> Packet {
        self.into_packet()
    }
}

#[derive(Debug)]
pub struct Packet {
    pub opcode: ServerOpcode,
    pub length: i32,
    pub body: ServerMessage,
}

impl Packet {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(& byte_to_byte(self.opcode as u8));
        bytes.extend_from_slice(& int_to_bytes(self.length));

        match &self.body {
            ServerMessage::Gossip(msg) => bytes.extend_from_slice(& msg.to_bytes()),
            ServerMessage::ErrorServer(msg) => bytes.extend_from_slice(& msg.to_bytes()),
            ServerMessage::Query(msg) => bytes.extend_from_slice(& msg.to_bytes()),
            ServerMessage::Result(msg) => bytes.extend_from_slice(& msg.to_bytes()),
        }
        bytes
    }

    pub fn from_stream(stream: &mut TcpStream) -> Result<Self, Box<dyn Error>> {
        let mut header_buffer = [0u8; 5];
        if let Err(_) = stream.read_exact(&mut header_buffer) {
            return Err(Box::from("Error al leerl el <header> del packet desde Stream"));
        };
        let _opcode = ServerOpcode::from_u8(header_buffer[0])?;
        let _length = i32::from_be_bytes([header_buffer[1],header_buffer[2],header_buffer[3],header_buffer[4]]);

        let mut body_buffer = vec![0u8; _length as usize];
        if let Err(_) = stream.read_exact(&mut body_buffer) {
            return Err(Box::from("Error al leer el <body> desde Stream"))
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
            ServerOpcode::QueryMessage => {
                let (query, _) = QueryMessage::from_bytes(body_buffer)?;
                ServerMessage::Query(query)
            }
            ServerOpcode::ResultMessage => {
                let (result, _) = ResultMessage::from_bytes(body_buffer)?;
                ServerMessage::Result(result)
            }
        };
        Ok(Packet{opcode: _opcode, length: _length, body: _body})
    }
}

pub trait IntoPacket: std::fmt::Debug {
    fn into_packet(self) -> Packet;
}