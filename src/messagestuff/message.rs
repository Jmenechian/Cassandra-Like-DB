// Struct que se usa para representar un mensaje entre el cliente y un nodo del servidor
// Estos mensajes tienen la siguiente entructura:
//
// [1 byte: opcode] [1 byte: version] [1 byte: consistency lvl] [4 bytes: len msg] [left over bytes: msg]
//
// Cada mensaje se serializa y deserializa a bytes para poder ser enviado por la red

use crate::my_errors::MyError;
use crate::config::{QUERY, RESULT, REQUEST, RESPONSE, QUORUM};

#[derive(Debug)]
pub struct Message {
    pub version: u8,
    pub opcode: u8,
    pub len_msg: u32,
    pub consistency_level: u8,
    pub message: String,
}

impl Message {
    pub fn new(
        message: String,
        version: u8,
        opcode: u8,
        len_msg: u32,
        consistency_level: u8,
    ) -> Self {
        Self {
            version,
            opcode,
            len_msg,
            consistency_level,
            message,
        }
    }

    // Devuelve el mensaje serializado en bytes
    pub fn to_bytes(&mut self) -> Vec<u8> {
        let mut combined = Vec::new();
        combined.push(self.opcode);
        combined.push(self.version);
        combined.push(self.consistency_level);
        combined.extend(&self.len_msg.to_be_bytes());
        combined.extend(self.message.as_bytes());
        combined
    }

    // Devuelve un mensaje a partir de un vector de bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MyError> {
        if bytes.len() < 8 {
            return Err(MyError::Error("error en from_bytes, len < 8".to_string()));
        }
        let opcode = bytes[0];
        let version = bytes[1];
        let consistency_level = bytes[2];
        let len_msg = u32::from_be_bytes([bytes[3], bytes[4], bytes[5], bytes[6]]);
        let message = String::from_utf8(bytes[7..].to_vec())
            .map_err(|_| MyError::Error("error en from_bytes".to_string()))?;

        Ok(Self {
            version,
            opcode,
            len_msg,
            consistency_level,
            message,
        })
    }

    pub fn create_message_as_client(input: String) -> Message {
        let ip = "127.0.0.1:45450".to_string();
        let lenght = (ip.len() + input.len()) as u32;
        Message::new(input, REQUEST, QUERY, lenght, QUORUM)
    }

    pub fn create_message_as_server(input: String, ip: String) -> Message {
        let lenght = (ip.len() + input.len()) as u32;
        //saca el ip.len() fijo
        Message::new(input, RESPONSE, RESULT, lenght, QUORUM)
    }
}


// OPCODE
// 0x00    ERROR
// 0x01    STARTUP
// 0x02    READY
// 0x03    AUTHENTICATE
// 0x05    OPTIONS
// 0x06    SUPPORTED
// 0x07    QUERY
// 0x08    RESULT
// 0x09    PREPARE
// 0x0A    EXECUTE
// 0x0B    REGISTER
// 0x0C    EVENT
// 0x0D    BATCH
// 0x0E    AUTH_CHALLENGE
// 0x0F    AUTH_RESPONSE
// 0x10    AUTH_SUCCESS

// [consistency]  A consistency level specification. This is a [short]
// representing a consistency level with the following
// correspondance:
//   0x0000    ANY
//   0x0001    ONE
//   0x0002    TWO
//   0x0003    THREE
//   0x0004    QUORUM
//   0x0005    ALL
//   0x0006    LOCAL_QUORUM
//   0x0007    EACH_QUORUM
//   0x0008    SERIAL
//   0x0009    LOCAL_SERIAL
//   0x000A    LOCAL_ONE
