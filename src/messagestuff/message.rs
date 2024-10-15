use std::io::{self, Write};
use std::net::TcpStream;
use std::io::{Cursor, Read};

const REQUEST: u8 = 0x03;
const QUORUM: u8 = 0x0004;
const QUERY: u8 = 0x07;

const RESPONSE: u8 = 0x83;
const RESULT: u8 = 0x08;
// const RESPONSE: u8 = 0x83;
    
#[derive(Debug)]
pub struct Message { 
    pub version: u8, 
    pub opcode: u8,
    pub length: u32, 
    pub consistency_level: u8,
    pub message: String,
    pub ip: String 
} 

impl Message {
    pub fn new(message: String, ip: String, version: u8, opcode: u8, length: u32, consistency_level: u8) -> Self {
        Self {
            version,
            opcode,
            length,
            consistency_level,
            message,
            ip,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut combined = Vec::new();
        combined.push(self.version);
        combined.push(self.opcode);
        combined.extend(&self.length.to_be_bytes());
        combined.push(self.consistency_level);

        combined.extend(Self::serialize_string(&self.ip));
        combined.extend(Self::serialize_string(&self.message));
        combined
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut cursor = Cursor::new(bytes);
        let version = read_u8(&mut cursor);
        let opcode = read_u8(&mut cursor);
        let length = read_u32(&mut cursor);
        let consistency_level = read_u8(&mut cursor);
        
        let ip = Self::deserialize_string(&mut cursor);
        let message = Self::deserialize_string(&mut cursor);

        Self {
            version,
            opcode,
            length,
            consistency_level,
            message,
            ip,
        }
    }

    fn serialize_string(field: &str) -> Vec<u8> {
        let mut field_bytes = Vec::new();
        let length = field.len() as u32;

        field_bytes.extend(&length.to_be_bytes()); // Write length prefix
        field_bytes.extend(field.as_bytes());      // Write string content

        field_bytes
    }

    fn deserialize_string<R: Read>(reader: &mut R) -> String {
        let length = read_u32(reader) as usize;
        let mut buf = vec![0u8; length];
        reader.read_exact(&mut buf).expect("Failed to read string");
        String::from_utf8(buf).expect("Invalid UTF-8 sequence")
    }

    pub fn create_message_as_client(input: String) -> Message {
        let ip = "127.0.0.1:45450".to_string();
        let lenght = (ip.len() + input.len()) as u32;
        Message::new(
            input, 
            ip,
            REQUEST,
            QUERY,
            lenght,
            QUORUM,
        )
    }

    pub fn create_message_as_server(input: String, ip: String) -> Message {
        let lenght = (ip.len() + input.len()) as u32;
//saca el ip.len() fijo
        Message::new(
            input, 
            ip,
            RESPONSE,
            RESULT,
            lenght,
            QUORUM,
        )
    }

}

fn read_u32<R: Read>(reader: &mut R) -> u32 {
    let mut buf = [0u8; 4]; // 4 bytes
    reader.read_exact(&mut buf).expect("Failed to read u32");
    u32::from_be_bytes(buf)
}

fn read_u8<R: Read>(reader: &mut R) -> u8 {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf).expect("Failed to read u8");
    buf[0]
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