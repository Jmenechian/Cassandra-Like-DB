use std::io::Cursor;
use std::io::Read;

#[derive(Debug)]
pub struct Message {
    // No se como agregar un campo sin tener que rearmar el serialize
    // pub version: u8,
    // pub stream_id: u32, //no se si entra para nosotros
    // pub opcode: u8, //puede ser util
    // lenght: u32,
    // consistency_level: u8, // -> notations tiene todo esto o solo usamos este?
    pub message: String,
    pub ip: String, //esta hardcodeado 1000 porque igual no lo usamos por ahora
} //faltan mas

impl Message {
    pub fn new(message: String) -> Self {
        Self {
            message,
            ip: "127.0.0.1:45450".to_string(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut combined = Vec::new();

        combined.extend(self.serialize_field(&self.message));
        combined.extend(self.serialize_field(&self.ip));

        combined
    }

    fn serialize_field(&self, field: &str) -> Vec<u8> {
        let length = field.len() as u32;
        let mut field_bytes = Vec::new();

        field_bytes.extend(&length.to_be_bytes());
        field_bytes.extend(field.as_bytes());

        field_bytes
    }

    fn deserialize_field<R: Read>(reader: &mut R) -> String {
        let mut length_bytes = [0u8; 4];
        reader
            .read_exact(&mut length_bytes)
            .expect("Failed to read length");

        let length = u32::from_be_bytes(length_bytes) as usize;
        println!("Reading field of length: {}", length); // Debug log

        if length > 10_000 {
            panic!("Field length too large: {}", length);
        }

        let mut field_bytes = vec![0u8; length];
        reader
            .read_exact(&mut field_bytes)
            .expect("Failed to read field");

        match String::from_utf8(field_bytes) {
            Ok(field) => field,
            Err(e) => {
                eprintln!("Invalid UTF-8 sequence: {}", e);
                "<Invalid UTF-8>".to_string()
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut cursor = Cursor::new(bytes);
        let message = Self::deserialize_field(&mut cursor);
        let ip = Self::deserialize_field(&mut cursor);

        Self { message, ip }
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
