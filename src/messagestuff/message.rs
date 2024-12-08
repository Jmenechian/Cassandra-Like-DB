// Coleccion de traits y funciones que usan los otros modulos de mensajeria.

use crate::messagestuff::message_cql::cqlenvelope::IntoEnvelope;
use crate::messagestuff::message_server::serverpacket::IntoPacket;
use crate::my_errors::MyError;
use std::{io::Write, net::TcpStream};
pub trait Serialize: std::fmt::Debug {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), MyError>
    where
        Self: Sized;
}

pub trait Sendable: std::fmt::Debug + IntoEnvelope {
    fn send_through(self, stream: &mut TcpStream) -> Result<(), MyError>
    where
        Self: Sized,
    {
        let env = self.into_envelope();
        let bytes = env.to_bytes();
        if stream.write_all(&bytes).is_err() {
            return Err(MyError::TcpStreamError(
                "Error al escribir en el stream".to_string(),
            ));
        };
        if stream.flush().is_err() {
            return Err(MyError::TcpStreamError(
                "Error al vecial es stream".to_string(),
            ));
        }
        Ok(())
    }

    fn sent_to(self, addres: String) -> Result<(), MyError>
    where
        Self: Sized,
    {
        let mut stream = match TcpStream::connect(addres) {
            Ok(s) => s,
            Err(_) => {
                return Err(MyError::TcpStreamError(
                    "Error al conectarse con servidor".to_string(),
                ))
            }
        };
        self.send_through(&mut stream)
    }
}

impl<T: IntoEnvelope> Sendable for T {}

pub trait ServerSendable: std::fmt::Debug + IntoPacket {
    fn send_server_through(self, stream: &mut TcpStream) -> Result<(), MyError>
    where
        Self: Sized,
    {
        let packet = self.into_packet();
        let bytes = packet.to_bytes();
        if stream.write_all(&bytes).is_err() {
            return Err(MyError::TcpStreamError(
                "Error al escribir en el stream".to_string(),
            ));
        };
        if stream.flush().is_err() {
            return Err(MyError::TcpStreamError(
                "Error al vaciar el stream".to_string(),
            ));
        }
        Ok(())
    }

    fn sent_to(self, addres: String) -> Result<(), MyError>
    where
        Self: Sized,
    {
        let mut stream = match TcpStream::connect(addres) {
            Ok(s) => s,
            Err(_) => {
                return Err(MyError::TcpStreamError(
                    "Error al conectarse con servidor".to_string(),
                ))
            }
        };
        self.send_server_through(&mut stream)
    }
}

impl<T: IntoPacket> ServerSendable for T {}

//Las funciones to_bytes() y from_bytes() de los diferentes tipos de datos
//[byte]: 1 byte unsigned integer
pub fn byte_to_byte(s: u8) -> Vec<u8> {
    s.to_be_bytes().to_vec()
}

pub fn byte_from_bytes(bytes: Vec<u8>) -> Result<(u8, Vec<u8>), MyError> {
    if bytes.is_empty() {
        return Err(MyError::TcpStreamError(
            "No hay suficientes bytes para leer un <byte>".to_string(),
        ));
    }
    let value = u8::from_be_bytes([bytes[0]]);
    Ok((value, bytes[1..].to_vec()))
}

//[short]: 2 bytes unsigned integer
pub fn short_to_bytes(s: u16) -> Vec<u8> {
    s.to_be_bytes().to_vec()
}

pub fn short_from_bytes(bytes: Vec<u8>) -> Result<(u16, Vec<u8>), MyError> {
    if bytes.len() < 2 {
        return Err(MyError::TcpStreamError(
            "No hay suficientes bytes para leer un <short>".to_string(),
        ));
    }
    let value = u16::from_be_bytes([bytes[0], bytes[1]]);
    Ok((value, bytes[2..].to_vec()))
}

//[int]: 4 bytes integer
pub fn int_to_bytes(i: i32) -> Vec<u8> {
    i.to_be_bytes().to_vec()
}

pub fn int_from_bytes(bytes: Vec<u8>) -> Result<(i32, Vec<u8>), MyError> {
    if bytes.len() < 4 {
        return Err(MyError::TcpStreamError(
            "No hay suficientes bytes para leer un <int>".to_string(),
        ));
    }
    let value = i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    Ok((value, bytes[4..].to_vec()))
}

//[long]: 8 bytes integer
pub fn long_to_bytes(l: i64) -> Vec<u8> {
    l.to_be_bytes().to_vec()
}

pub fn long_from_bytes(bytes: Vec<u8>) -> Result<(i64, Vec<u8>), MyError> {
    if bytes.len() < 8 {
        return Err(MyError::TcpStreamError(
            "No hay suficientes bytes para leer un <long>".to_string(),
        ));
    }
    let value = i64::from_be_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    Ok((value, bytes[8..].to_vec()))
}

//[string]: [short] n, followed by n bytes representing an UFT-8 string
pub fn string_to_bytes(s: String) -> Vec<u8> {
    let mut bytes = Vec::new();
    let length = s.len() as u16;
    bytes.extend_from_slice(&short_to_bytes(length));
    bytes.extend_from_slice(s.as_bytes());
    bytes
}

pub fn string_from_bytes(bytes: Vec<u8>) -> Result<(String, Vec<u8>), MyError> {
    if bytes.len() < 2 {
        return Err(MyError::TcpStreamError(
            "No hay suficientes bytes para leer la longitud del <String>".to_string(),
        ));
    }
    let (length, resto) = short_from_bytes(bytes)?;
    if resto.len() < length as usize {
        return Err(MyError::TcpStreamError(
            "No hay suficientes bytes prar leer el <String>".to_string(),
        ));
    }
    let string_buffer = &resto[..length as usize];
    let _string = match String::from_utf8(string_buffer.to_vec()) {
        Ok(string) => string,
        Err(_) => {
            return Err(MyError::TcpStreamError(
                "Error al pasar utf8 a <String>".to_string(),
            ))
        }
    };
    Ok((_string, resto[length as usize..].to_vec()))
}

//[long string]: [int] n, followed by n bytes representing an UTF-8 string
pub fn longstring_to_bytes(s: String) -> Vec<u8> {
    let mut bytes = Vec::new();
    let length = s.len() as i32;
    bytes.extend_from_slice(&int_to_bytes(length));
    bytes.extend_from_slice(s.as_bytes());
    bytes
}

pub fn longstring_from_bytes(bytes: Vec<u8>) -> Result<(String, Vec<u8>), MyError> {
    if bytes.len() < 4 {
        return Err(MyError::TcpStreamError(
            "No hay suficientes bytes para leer la longitud del <String>".to_string(),
        ));
    }
    let (length, resto) = int_from_bytes(bytes)?;
    if resto.len() < length as usize {
        return Err(MyError::TcpStreamError(
            "No hay suficientes bytes para leer el <String>".to_string(),
        ));
    }
    let string_buffer = &resto[..length as usize];
    let _string = match String::from_utf8(string_buffer.to_vec()) {
        Ok(string) => string,
        Err(_) => {
            return Err(MyError::TcpStreamError(
                "Error al pasar utf8 a <string>".to_string(),
            ))
        }
    };
    Ok((_string, resto[length as usize..].to_vec()))
}
