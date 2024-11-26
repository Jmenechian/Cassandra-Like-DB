use crate::config::KEY_FILE;
use crate::errors::my_errors::MyError;
use crate::utils::*;
use std::{io::Read, fs::File, io::Write};
use aes::Aes128;
use aes::cipher::{BlockEncrypt, BlockDecrypt, KeyInit, generic_array::GenericArray};
use std::sync::Arc;
use std::sync::RwLock;

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
    if decrypted.len() == 0 {
        return Ok(decrypted);
    }
    // Verifica que el tamaño sea múltiplo de 16
    if decrypted.len() % 16 != 0 {
        return Err(MyError::DecryptionError(format!("Invalid padding 1: decrypted: {:?},  {:?}, len: {:?}", encrypted_body, decrypted.is_empty(), decrypted.len() % 16)));
        //return Err(MyError::DecryptionError("Invalid padding 1".to_string()));
    }

    // Elimina el padding
    let padding = *decrypted.last().unwrap() as usize; // Último byte indica el tamaño del padding
    if padding == 0 || padding > 16 || padding > decrypted.len() {
        return Err(MyError::DecryptionError(format!("Invalid padding 2: padding: {:?}, decrypted: {:?}", padding, decrypted)));
    }

    let unpadded = decrypted[..decrypted.len() - padding].to_vec();

    Ok(unpadded)
}

pub fn read_encrypted_file(file_path: Arc<RwLock<String>>) -> Result<Vec<String>, MyError> {
    let mut rdr_read = get_reader_read(&file_path)?;
    let mut contents = Vec::new();
    rdr_read.read_to_end(&mut contents).map_err(|e| MyError::Error("Error".to_string()))?;
    let mut parts: Vec<Vec<u8>> = contents.split(|&byte| byte == b'\n').map(|s| s.to_vec()).collect::<Vec<_>>().to_vec();
    let mut lines = Vec::new();
    for i in 0..parts.len() {
        let encrypted_line = parts[i].clone();
        let decrypted_line = decrypt_body_aes128(encrypted_line.clone());
        if let Err(MyError::DecryptionError(_)) = &decrypted_line {
            if i + 1 < parts.len() {
                // Concatenamos la línea actual con un salto de línea
                let mut combined_line = Vec::new();
                combined_line.extend_from_slice(&parts[i]);  // Añade la línea actual
                combined_line.push(b'\n');  // Añade un salto de línea
    
                // Añadimos la siguiente línea
                combined_line.extend_from_slice(&parts[i + 1]);
    
                // Actualizamos `parts[i+1]` con la línea combinada
                parts[i+1] = combined_line;
                continue;
            }
        }
        
        let decrypted_string = match decrypted_line {
            Ok(s) => s,
            Err(e) => return Err(MyError::EncryptionError(format!("Error antes de aca: {:?}", e))),
        };
        let decryptes_string = String::from_utf8(decrypted_string).map_err(|e| MyError::EncryptionError(format!("Error: {:?}", e)))?;
        lines.push(decryptes_string);
    }
    Ok(lines)
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