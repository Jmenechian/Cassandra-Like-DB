use crate::config;
use crate::messagestuff::message::*;
use crate::messagestuff::message_server::{
    resultrowsevermsg::*, resultvoidservermsg::*, serverpacket::*,
};
use crate::my_errors::MyError;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(i32)]
pub enum ResultServerKind {
    Void = config::VOID,
    Rows = config::ROWS,
}

impl ResultServerKind {
    pub fn from_i32(value: i32) -> Result<Self, MyError> {
        match value {
            config::VOID => Ok(ResultServerKind::Void),
            config::ROWS => Ok(ResultServerKind::Rows),
            _ => Err(MyError::ServerMessageError(
                "Invalid Result Kind".to_string(),
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ResultServerBody {
    Void(ResultVoidServerMessage),
    Rows(ResultRowsServerMessage),
}

#[derive(Debug, PartialEq)]
pub struct ResultServerMessage {
    pub kind: ResultServerKind,
    pub body: ResultServerBody,
}

impl Serialize for ResultServerMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&int_to_bytes(self.kind as i32));
        match &self.body {
            ResultServerBody::Void(_) => {}
            ResultServerBody::Rows(rows) => {
                bytes.extend_from_slice(&rows.to_bytes());
            }
        }
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), MyError> {
        let (value, resto) = int_from_bytes(bytes)?;
        let _kind = ResultServerKind::from_i32(value)?;
        let (_body, resto) = match _kind {
            ResultServerKind::Void => (ResultServerBody::Void(ResultVoidServerMessage), resto),
            ResultServerKind::Rows => {
                let (rows, resto) = ResultRowsServerMessage::from_bytes(resto)?;
                (ResultServerBody::Rows(rows), resto)
            }
        };

        Ok((
            ResultServerMessage {
                kind: _kind,
                body: _body,
            },
            resto,
        ))
    }
}

pub trait IntoResultServerMessage: std::fmt::Debug {
    fn into_result_server_message(self) -> ResultServerMessage;
}

impl IntoPacket for ResultServerMessage {
    fn into_packet(self) -> Packet {
        let _body = self.to_bytes();
        Packet {
            opcode: ServerOpcode::ResultServerMessage,
            length: _body.len() as i32,
            body: ServerMessage::Result(self),
        }
    }
}
