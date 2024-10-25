use crate::messagestuff::message_cql::cqlenvelope::IntoEnvelope;
use crate::messagestuff::message_server::serverpacket::IntoPacket;
use std::{
    error::Error,
    io::Write,
    net::TcpStream,
};
pub trait Serialize: std::fmt::Debug {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), Box<dyn Error>>
    where
        Self: Sized;
}

pub trait Sendable: std::fmt::Debug + IntoEnvelope {
    fn send_through(self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>>
    where
        Self: Sized,
    {
        let env = self.into_envelope();
        let bytes = env.to_bytes();
        if stream.write_all(&bytes).is_err() {
            return Err(Box::from("Error al escribir en el stream"));
        };
        if stream.flush().is_err() {
            return Err(Box::from("Error al vecial es stream"));
        }
        Ok(())
    }

    fn sent_to(self, addres: String) -> Result<(), Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut stream = match TcpStream::connect(addres) {
            Ok(s) => s,
            Err(_) => return Err(Box::from("Error al conectarse con servidor")),
        };
        self.send_through(&mut stream)
    }
}

impl<T: IntoEnvelope> Sendable for T {}

pub trait ServerSendable: std::fmt::Debug + IntoPacket {
    fn send_server_through(self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>>
    where
        Self: Sized,
    {
        let packet = self.into_packet();
        let bytes = packet.to_bytes();
        if stream.write_all(&bytes).is_err() {
            return Err(Box::from("Error al escribir en el stream"));
        };
        if stream.flush().is_err() {
            return Err(Box::from("Error al vecial es stream"));
        }
        Ok(())
    }

    fn sent_to(self, addres: String) -> Result<(), Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut stream = match TcpStream::connect(addres) {
            Ok(s) => s,
            Err(_) => return Err(Box::from("Error al conectarse con servidor")),
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

pub fn byte_from_bytes(bytes: Vec<u8>) -> Result<(u8, Vec<u8>), Box<dyn Error>> {
    if bytes.is_empty() {
        return Err(Box::from("No hay suficientes bytes para leer un <byte>"));
    }
    let value = u8::from_be_bytes([bytes[0]]);
    Ok((value, bytes[1..].to_vec()))
}

//[short]: 2 bytes unsigned integer
pub fn short_to_bytes(s: u16) -> Vec<u8> {
    s.to_be_bytes().to_vec()
}

pub fn short_from_bytes(bytes: Vec<u8>) -> Result<(u16, Vec<u8>), Box<dyn Error>> {
    if bytes.len() < 2 {
        return Err(Box::from("No hay suficientes bytes para leer un <short>"));
    }
    let value = u16::from_be_bytes([bytes[0], bytes[1]]);
    Ok((value, bytes[2..].to_vec()))
}

//[int]: 4 bytes integer
pub fn int_to_bytes(i: i32) -> Vec<u8> {
    i.to_be_bytes().to_vec()
}

pub fn int_from_bytes(bytes: Vec<u8>) -> Result<(i32, Vec<u8>), Box<dyn Error>> {
    if bytes.len() < 4 {
        return Err(Box::from("No hay suficientes bytes para leer un <int>"));
    }
    let value = i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    Ok((value, bytes[4..].to_vec()))
}

//[long]: 8 bytes integer
pub fn long_to_bytes(l: i64) -> Vec<u8> {
    l.to_be_bytes().to_vec()
}

pub fn long_from_bytes(bytes: Vec<u8>) -> Result<(i64, Vec<u8>), Box<dyn Error>> {
    if bytes.len() < 8 {
        return Err(Box::from("No hay suficientes bytes para leer un <long>"));
    }
    let value = i64::from_be_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    Ok((value, bytes[8..].to_vec()))
}

//[sring]: [short] n, followed by n bytes representing an UFT-8 string
pub fn string_to_bytes(s: String) -> Vec<u8> {
    let mut bytes = Vec::new();
    let length = s.len() as u16;
    bytes.extend_from_slice(&short_to_bytes(length));
    bytes.extend_from_slice(s.as_bytes());
    bytes
}

pub fn string_from_bytes(bytes: Vec<u8>) -> Result<(String, Vec<u8>), Box<dyn Error>> {
    if bytes.len() < 2 {
        return Err(Box::from(
            "No hay suficientes bytes para leer la longitud del <String>",
        ));
    }
    let (length, resto) = short_from_bytes(bytes)?;
    if resto.len() < length as usize {
        return Err(Box::from("No hay suficientes bytes prar leer el <String>"));
    }
    let string_buffer = &resto[..length as usize];
    let _string = match String::from_utf8(string_buffer.to_vec()) {
        Ok(string) => string,
        Err(_) => return Err(Box::from("Error al pasar utf8 a <String>")),
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

pub fn longstring_from_bytes(bytes: Vec<u8>) -> Result<(String, Vec<u8>), Box<dyn Error>> {
    if bytes.len() < 4 {
        return Err(Box::from(
            "No hay suficientes bytes para leer la longitud del <String>",
        ));
    }
    let (length, resto) = int_from_bytes(bytes)?;
    if resto.len() < length as usize {
        return Err(Box::from("No hay suficientes bytes para leer el <String>"));
    }
    let string_buffer = &resto[..length as usize];
    let _string = match String::from_utf8(string_buffer.to_vec()) {
        Ok(string) => string,
        Err(_) => return Err(Box::from("Error al pasar utf8 a <string>")),
    };
    Ok((_string, resto[length as usize..].to_vec()))
}
