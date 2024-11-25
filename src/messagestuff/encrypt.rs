use crate::config::KEY_FILE;
use crate::errors::my_errors::MyError;
use std::{io::Read, fs::File, io::Write};
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
    Ok(encrypted)
}

pub fn decrypt_body_aes128(encrypted_body: Vec<u8>) -> Result<Vec<u8>, MyError> {
    let key = load_key_from_file(KEY_FILE)?;
    let cipher = Aes128::new(GenericArray::from_slice(&key));
    let mut decrypted = Vec::new();
    for chunck in encrypted_body.chunks_exact(16) {
        let mut block = GenericArray::clone_from_slice(chunck);
        cipher.decrypt_block(&mut block);
        decrypted.extend_from_slice(&block);
    }
    Ok(decrypted)
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::messagestuff::message::*;
    use crate::messagestuff::message_cql::querymessage::*;
    #[test]
    fn test_encrypt_string() -> Result<(), MyError> {
        let string = "Hola".to_string();
        let string_vec = string_to_bytes(string.clone());
        let string_encrypted = encrypt_body_aes128(string_vec)?;
        let string_decrypted = decrypt_body_aes128(string_encrypted)?;
        let (string_from_vec, _) = string_from_bytes(string_decrypted)?;
        assert_eq!(string, string_from_vec);
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