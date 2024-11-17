use crate::errors::my_errors::MyError;
use crate::messagestuff::message::*;
use crate::messagestuff::message_server::{resultservermessage::*, serverpacket::*};

#[derive(Debug, PartialEq)]
pub struct ResultRowsServerMessage {
    pub table: String,
    pub rows_count: i32,
    pub rows_content: Vec<String>,
}

impl Serialize for ResultRowsServerMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&string_to_bytes(self.table.clone()));
        bytes.extend_from_slice(&int_to_bytes(self.rows_count));
        for row in &self.rows_content {
            bytes.extend_from_slice(&string_to_bytes(row.clone()));
        }
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), MyError> {
        let (_table, resto) = string_from_bytes(bytes)?;
        let (_rows_count, resto) = int_from_bytes(resto)?;
        let mut _bytes = resto;
        let mut _rows_content: Vec<String> = Vec::new();
        for _ in 0.._rows_count {
            let (_row, resto) = string_from_bytes(_bytes)?;
            _rows_content.push(_row);
            _bytes = resto;
        }
        Ok((
            ResultRowsServerMessage {
                table: _table,
                rows_count: _rows_count,
                rows_content: _rows_content,
            },
            _bytes,
        ))
    }
}

impl IntoResultServerMessage for ResultRowsServerMessage {
    fn into_result_server_message(self) -> ResultServerMessage {
        ResultServerMessage {
            kind: ResultServerKind::Rows,
            body: ResultServerBody::Rows(self),
        }
    }
}

impl IntoPacket for ResultRowsServerMessage {
    fn into_packet(self) -> Packet {
        let result = self.into_result_server_message();
        result.into_packet()
    }
}
