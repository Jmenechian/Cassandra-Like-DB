// [1 Byte opcode] [1 byte version] [1 byte consistency level] [4 bytes Length_msg] [msg]  ////// [4 bytes len_ip] [ip_para_responder]

use crate::cql::my_errors::MyError;

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

    //opcode version consistency_level length_msg message len_ip ip
    pub fn to_bytes(&mut self) -> Vec<u8> {
        let mut combined = Vec::new();
        combined.push(self.opcode);
        combined.push(self.version);
        combined.push(self.consistency_level);
        combined.extend(&self.len_msg.to_be_bytes());
        combined.extend(self.message.as_bytes());
        combined
    }

    // 48 bits para ip+puerto
    //

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MyError> {
        if bytes.len() < 8 {
            return Err(MyError::Error("error en from_bytes, len < 8"));
        }
        let opcode = bytes[0];
        let version = bytes[1];
        let consistency_level = bytes[2];
        let len_msg = u32::from_be_bytes([bytes[3], bytes[4], bytes[5], bytes[6]]);
        let message = String::from_utf8(bytes[7..].to_vec())
            .map_err(|_| MyError::Error("error en from_bytes"))?;

        // let mut cursor = Cursor::new(bytes);
        // let opcode = read_u8(&mut cursor)?;
        // let version = read_u8(&mut cursor)?;
        // let consistency_level = read_u8(&mut cursor)?;
        // let len_msg = read_u32(&mut cursor)?;

        // let msg = Self::deserialize_string(&mut cursor, len_msg)?;

        // let mut buf_msg = vec![0u8; len_msg as usize];
        // let _ = cursor.read_exact(&mut buf_msg).map_err(|_| MyError::Error("error en dezerialize"))?;
        // let message = String::from_utf8(buf_msg).map_err(|_| MyError::Error("error en dezerialize"))?;

        Ok(Self {
            version,
            opcode,
            len_msg,
            consistency_level,
            message,
        })
    }

    // fn serialize_string(field: &str) -> Vec<u8> {
    //     let mut field_bytes = Vec::new();
    //     let length = field.len() as u32;

    //     field_bytes.extend(&length.to_be_bytes()); // Write length prefix
    //     field_bytes.extend(field.as_bytes()); // Write string content

    //     field_bytes
    // }

    // fn deserialize_string<R: Read>(reader: &mut R, len_msg: u32) -> Result<String, MyError> {
    //     let mut buf_msg = vec![0u8; len_msg as usize];
    //     let _ = reader
    //         .read_exact(&mut buf_msg)
    //         .map_err(|_| MyError::Error("error en dezerialize"))?;
    //     String::from_utf8(buf_msg).map_err(|_| MyError::Error("error en dezerialize"))

    // let length = read_u32(reader)? as usize;
    // let mut buf = vec![0u8; length];
    // reader.read_exact(&mut buf).map_err(|_| MyError::Error("error en deserlize"))?;
    // String::from_utf8(buf).map_err(|_| MyError::Error("error en deserlize"))
    //}

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

pub fn print_message(mensaje: String) {
    let parts: Vec<&str> = mensaje.split("     ").collect();
    for part in parts {
        println!("{}", part);
    }
}

// fn read_u32<R: Read>(reader: &mut R) -> Result<u32, MyError> {
//     let mut buf = [0u8; 4]; // 4 bytes
//     match reader.read_exact(&mut buf) {
//         Ok(_) => (),
//         Err(_) => return Err(MyError::Error("Failed to read u32")),
//     }
//     Ok(u32::from_be_bytes(buf))
// }

// fn read_u8<R: Read>(reader: &mut R) -> Result<u8, MyError> {
//     let mut buf = [0u8; 1];
//     match reader.read_exact(&mut buf) {
//         Ok(_) => (),
//         Err(_) => return Err(MyError::Error("Failed to read u8")),
//     }
//     Ok(buf[0])
// }

// macro_rules! read_u8 {
//     ($reader:expr) => {{
//         let mut buf = [0u8; 1];
//         match $reader.read_exact(&mut buf) {
//             Ok(_) => (),
//             Err(_) => return Err(MyError::Error("Failed to read u8".to_string())),
//         }
//         Ok(buf[0])
//     }};
// }

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
