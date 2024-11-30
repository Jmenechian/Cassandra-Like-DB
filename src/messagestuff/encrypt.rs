use crate::config::KEY_FILE;
use crate::errors::my_errors::MyError;
use std::{io::Read, fs::File};
use aes::Aes128;
use aes::cipher::{BlockEncrypt, BlockDecrypt, KeyInit, generic_array::GenericArray};

pub fn load_key_from_file(filename: &str) -> Result<[u8;16], MyError> {
    let mut file = match File::open(filename) {
        Ok(f) => f,
        Err(_) => return Err(MyError::FileError("Error".to_string())),
    };

    let mut key = [0u8; 16];
    if let Err(_) = file.read_exact(&mut key) {
        return Err(MyError::FileError("Error".to_string()));
    }
    Ok(key)
}

pub fn encrypt_body_aes128(body: Vec<u8>) -> Result<Vec<u8>, MyError> {
    let key = load_key_from_file(KEY_FILE)?;
    let cipher = Aes128::new(GenericArray::from_slice(&key));
    let mut body_padded = body.to_vec();
    let padding = 16 - (body.len() % 16);
    body_padded.extend(vec![padding as u8; padding]);
    let mut encrypted = Vec::new();
    for chunk in body_padded.chunks_exact(16) {
        let mut block = GenericArray::clone_from_slice(chunk);
        cipher.encrypt_block(&mut block);
        encrypted.extend_from_slice(&block);
    }
    Ok(encode(encrypted))
}

pub fn encode(bytes: Vec<u8>) -> Vec<u8> {
    const CUSTOM_ENCODING: &[u8] = b"0123456789ABCDEF";
    let mut encoded = Vec::new();
    for byte in bytes {
        let index1 = (byte >> 4) as usize;
        let index2 = (byte & 0x0F) as usize;
        encoded.push(CUSTOM_ENCODING[index1]);
        encoded.push(CUSTOM_ENCODING[index2]);
    }
    encoded
}

pub fn decode(bytes: Vec<u8>) -> Result<Vec<u8>, MyError> {
    let bytes_to_decode: Vec<u8>;
    if bytes.len() % 2 != 0 {
        let last = match bytes.last() {
            Some(c) => c,
            None => return Err(MyError::DecryptionError("Error".to_string())),
        };
        if *last == b'\n' {
            bytes_to_decode = bytes[..bytes.len()-1].to_vec();
        } else {
            return Err(MyError::DecryptionError(format!("Datos codificados no validos: {:?}, last: {:?}", bytes.len(), bytes.last().unwrap())))
        }
    } else {
        bytes_to_decode = bytes;
    }
    
    const CUSTOM_ENCODING: &[u8] = b"0123456789ABCDEF";
    let mut decoding_map = [0u8; 256];
    for (i, &c) in CUSTOM_ENCODING.iter().enumerate() {
        decoding_map[c as usize] = i as u8;
    }
    let mut decoded = Vec::new();
    for chunk in bytes_to_decode.chunks(2) {
        let index1 = decoding_map[chunk[0] as usize];
        let index2 = decoding_map[chunk[1] as usize];
        decoded.push((index1 << 4) | index2);
    }
    Ok(decoded)
}

pub fn decrypt_body_aes128(encrypted_body: Vec<u8>) -> Result<Vec<u8>, MyError> {
    let decoded_body = decode(encrypted_body.clone())?;
    let key = load_key_from_file(KEY_FILE)?;
    let cipher = Aes128::new(GenericArray::from_slice(&key));
    let mut decrypted = Vec::new();
    for chunck in decoded_body.chunks_exact(16) {
        let mut block = GenericArray::clone_from_slice(chunck);
        cipher.decrypt_block(&mut block);
        decrypted.extend_from_slice(&block);
    }
    if decrypted.is_empty() {
        return Ok(decrypted);
    }
    if decrypted.len() % 16 != 0 {
        return Err(MyError::DecryptionError(format!("Invalid padding 1: decrypted: {:?},  {:?}, len: {:?}", encrypted_body, decrypted.is_empty(), decrypted.len() % 16)));
    }

    let padding = *decrypted.last().unwrap() as usize;
    if padding == 0 || padding > 16 || padding > decrypted.len() {
        return Err(MyError::DecryptionError(format!("Invalid padding 2: padding: {:?}, decrypted: {:?}", padding, decrypted)));
    }

    let unpadded = decrypted[..decrypted.len() - padding].to_vec();

    Ok(unpadded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messagestuff::message::*;
    use crate::messagestuff::message_cql::querymessage::*;
    #[test]
    fn test_encrypt_string() -> Result<(), MyError> {
        let string = "Hola".to_string();
        println!("Texto: {:?}", string);
        println!("Texto como u8: {:?}", string.as_bytes());
        let string_vec = string_to_bytes(string.clone());
        println!("string_vec: {:?}", string_vec);
        let string_encrypted = encrypt_body_aes128(string_vec)?;
        println!("Encriptado: {:?}", string_encrypted);
        let string_decrypted = decrypt_body_aes128(string_encrypted)?;
        println!("Desencriptado: {:?}", string_decrypted);
        let (string_from_vec, _) = string_from_bytes(string_decrypted)?;
        println!("Texto Desencriptado: {:?}", string_from_vec);

        println!("El '/n' es: {:?}", b' ');
        assert_ne!(string, string_from_vec);
        Ok(())
    }

    #[test]
    fn test_encrypt_message() -> Result<(), MyError> {
        let message = QueryMessage{query: "SELECT FROM origin WHERE airport = AEP".to_string(),consistency: 8 as u16};
        let message_vec = message.to_bytes();
        let message_encrypt = encrypt_body_aes128(message_vec)?;
        let message_decrypt = decrypt_body_aes128(message_encrypt)?;
        let (message_from_vec, _) = QueryMessage::from_bytes(message_decrypt)?;
        assert_eq!(message, message_from_vec);
        Ok(())
    }
}