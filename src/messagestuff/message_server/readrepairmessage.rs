use crate::messagestuff::message::*;
use crate::messagestuff::message_server::serverpacket::*;
use crate::my_errors::MyError;

#[derive(Debug, PartialEq)]
pub struct ReadRepairMessage {
    pub table: String,
    pub rows_to_keep: Vec<String>,
}

impl Serialize for ReadRepairMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&longstring_to_bytes(self.table.clone()));
        bytes.extend_from_slice(&int_to_bytes(self.rows_to_keep.len() as i32));
        for row in &self.rows_to_keep {
            bytes.extend_from_slice(&longstring_to_bytes(row.to_string()));
        }
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), MyError> {
        let (_table, resto) = longstring_from_bytes(bytes)?;
        let (_int, resto) = int_from_bytes(resto)?;
        let mut _rows_to_keep = Vec::new();
        let mut _bytes = resto;
        for _ in 0.._int {
            let (_longstring, resto) = longstring_from_bytes(_bytes)?;
            _rows_to_keep.push(_longstring);
            _bytes = resto;
        }
        Ok((
            ReadRepairMessage {
                rows_to_keep: _rows_to_keep,
                table: _table,
            },
            _bytes,
        ))
    }
}

impl IntoPacket for ReadRepairMessage {
    fn into_packet(self) -> Packet {
        let _body = self.to_bytes();
        Packet {
            opcode: ServerOpcode::ReadRepairMessage,
            length: _body.len() as i32,
            body: ServerMessage::ReadRepair(self),
        }
    }
}
