// Funciones auxiliares para el manejo de archivos y strings

use crate::my_errors::MyError;
use std::{
    fs::File, fs::OpenOptions, io::BufRead, io::BufReader, io::Seek, io::SeekFrom, io::Write,
    iter::Peekable, fs::remove_file, io::Read
};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, generic_array::GenericArray},
    Aes256Gcm, Nonce, Key
};
use base64::{engine::general_purpose, Engine};

// Returns the full quote inside ' 'S
pub fn get_full_quote(parts: &[&str], index: usize) -> String {
    let mut name = String::new();

    for part in parts.iter().skip(index) {
        name.push_str(&part.replace("'", "").replace(",", ""));
        if part.ends_with("'") {
            break; // Exit the loop when you find a closing quote
        }
        name.push(' ');
    }
    name
}

// Returns the full quote inside ' ' while advancing iterator
pub fn get_full_quote_iter<'a, I>(parts: &mut Peekable<I>) -> String
where
    I: Iterator<Item = &'a &'a str>,
{
    let mut name = String::new();

    for part in parts.by_ref() {
        let p = *part;
        let p_cleaned = p.replace(",", "").replace("(", "").replace(")", "");
        if p_cleaned.ends_with("'") {
            name.push_str(&p_cleaned.replace("'", ""));
            break;
        }
        name.push_str(&p_cleaned.replace("'", ""));
        name.push(' ');
    }
    name
}

// Gets the complete file path, returns it as String
pub fn get_file_path(path: &str, file_name: &str) -> String {
    let file_path = format!("{}/{}.csv", path, file_name);
    file_path
}

// Open file in given path, returns an iterable reader
pub fn get_reader(path: &str, table_name: &str) -> Result<BufReader<File>, MyError> {
    let file_path = get_file_path(path, table_name);
    println!("Se abrirá el archivo en FILE PATH: {}", file_path);
    let file = File::open(file_path).map_err(|e| MyError::FileError(e.to_string()))?;
    let rdr = BufReader::new(file);
    Ok(rdr)
}

// Receives an iterable into the CSV file
// Returns the CSV file headers
pub fn get_file_headers<R: std::io::BufRead>(rdr: &mut R) -> Result<Vec<String>, MyError> {
    let mut headers_line = String::new();
    rdr.read_line(&mut headers_line)
        .map_err(|e| MyError::FileError(e.to_string()))?;

    let headers: Vec<String> = headers_line
        .trim()
        .split(',')
        .map(|s| s.to_string())
        .collect();

    let mut cleaned_headers: Vec<String> = Vec::new();
    for header in headers {
        cleaned_headers.push(header.replace("'", ""));
    }
    Ok(cleaned_headers)
}

// Writes rows to file in given path
pub fn write_rows(path: &str, table_name: &str, records: &Vec<Vec<String>>) -> Result<(), MyError> {
    let file_path = get_file_path(path, table_name);
    let mut file = File::create(file_path).map_err(|e| MyError::FileError(e.to_string()))?;

    for record in records {
        let row = record.join(",") + "\n";
        file.write_all(row.as_bytes())
            .map_err(|e| MyError::FileError(e.to_string()))?;
    }
    Ok(())
}

// Appends row to file in given path
// Expects file to end with a newline
pub fn append_row(path: &str, table_name: &str, records: &[String]) -> Result<(), MyError> {
    let file_path = get_file_path(path, table_name);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)
        .map_err(|e| MyError::FileError(e.to_string()))?;

    writeln!(file, "{}", records.join(",")).map_err(|e| MyError::FileError(e.to_string()))?;
    Ok(())
}

// Apppends row to already opened file
// Expects file to end with a newline
pub fn append_row_with_file(file: &mut File, row: &[String]) -> Result<(), MyError> {
    file.seek(SeekFrom::End(0))
        .map_err(|e| MyError::FileError(e.to_string()))?;
    writeln!(file, "{}", row.join(",")).map_err(|e| MyError::FileError(e.to_string()))?;
    Ok(())
}

// Creates a temporary file
// Returns the opened File and it's name
pub fn create_temporary_file(path: &str, original_table: &str) -> Result<(File, String), MyError> {
    let temp_file_name = format!("{}_temp", original_table);
    let file_path = get_file_path(path, &temp_file_name);
    let _ = File::create(&file_path).map_err(|e| MyError::FileError(e.to_string()))?;

    let temp_file = File::options()
        .append(true)
        .open(file_path)
        .map_err(|e| MyError::FileError(e.to_string()))?;
    Ok((temp_file, temp_file_name))
}

pub fn delete_file(file_name: &str, path: &str) -> Result<(), MyError> {
    let file_path = get_file_path(path, file_name);
    std::fs::remove_file(file_path).map_err(|e| MyError::FileError(e.to_string()))?;
    Ok(())
}

// Writes the temporary file content in the original file
pub fn write_temporary_to_original(
    path: &str,
    temporary_file: &str,
    original_table: &str,
) -> Result<(), MyError> {
    let full_path: String = get_file_path(path, original_table);
    let _ = File::create(full_path).map_err(|e| MyError::FileError(e.to_string()))?; // empty oriignal file

    let rdr = get_reader(path, temporary_file)?;
    let lines = rdr.lines();

    // Go through temp file and write in original
    for line in lines {
        let line = line.map_err(|e| MyError::FileError(e.to_string()))?;
        let record: Vec<String> = line.trim().split(",").map(|s| s.to_string()).collect();
        append_row(path, original_table, &record).map_err(|e| MyError::FileError(e.to_string()))?;
    }
    Ok(())
}

/// -------------------- ///
/// Encriptacion AT-REST ///
/// -------------------- ///

/// Encriptar la data con la clave AES
pub fn encrypt_data(data: &str, key: &Key<Aes256Gcm>) -> Result<Vec<u8>, MyError> {
    println!("En encrypt_data");
    let cipher = Aes256Gcm::new(key);

    // Genera nonce (12 bytes)
    let nonce = Aes256Gcm::generate_nonce(&mut rand::thread_rng()); // Usar OsRng si se necesita mayor seguridad
    let ciphertext = cipher
        .encrypt(&nonce, data.as_bytes())
        .map_err(|x| MyError::EncryptionError(x.to_string()))?;

    // Concatenar nonce y ciphertext
    let mut encrypted_data = nonce.to_vec();
    encrypted_data.extend(ciphertext);
    Ok(encrypted_data)
}

/// Desencriptar la data con la clave AES
pub fn decrypt_data(encrypted_data: &[u8], key: &Key<Aes256Gcm>) -> Result<String, MyError> {
    println!("En decrypt_data");
    let cipher = Aes256Gcm::new(key);

    // Separar el nonce (primeros 12 bytes) y el ciphertext
    if encrypted_data.len() < 12 {
        return Err(MyError::DecryptionError("Datos insuficientes para descifrar".to_string()));
    }
    let (nonce, ciphertext) = encrypted_data.split_at(12);
    println!("Nonce: {:x?}", nonce);
    println!("Ciphertext: {:x?}", ciphertext);  

    let nonce = Nonce::from_slice(nonce);
    let decrypted_data = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|x| MyError::DecryptionError("Error durante el desencriptado".to_string()))?;
    let data: String = String::from_utf8(decrypted_data).map_err(|e| MyError::DecryptionError(format!("Datos no válidos UTF-8: {:?}",e)))?;
    Ok(data)
}

/// Recibe una fila en la forma Vec<String> y la encripta
pub fn encrypt_row(row: &[String], key: &Key<Aes256Gcm>) -> Result<Vec<u8>, MyError> {
    println!("En encrypt_row");
    let data = row.join(",");
    encrypt_data(&data, key)
}

/// Recibe una fila encriptada, la desencripta, y la devuelve como Vec<String>
pub fn decrypt_row(row: &[u8], key: &Key<Aes256Gcm>) -> Result<Vec<String>, MyError> {
    println!("En decrypt_row");
    let decoded_data = general_purpose::STANDARD
        .decode(row)
        .map_err(|e| MyError::DecryptionError(e.to_string()))?;


    let decrypted_data = decrypt_data(&decoded_data, key)?;
    Ok(decrypted_data.split(",").map(|s| s.to_string()).collect())
}

/// Recibe una fila encriptada y la agrega al final del archivo
pub fn append_encrypted_row(path: &str, table_name: &str, records: &[u8]) -> Result<(), MyError>{
    println!("Appendeamos fila encriptada");
    let file_path = get_file_path(path, table_name);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)
        .map_err(|e| MyError::FileError(e.to_string()))?;

    let encoded_data = general_purpose::STANDARD.encode(records);
    file.write_all(encoded_data.as_bytes()).map_err(|e| MyError::FileError(e.to_string()))?;
    file.write_all(b"\n").map_err(|e| MyError::FileError(e.to_string()))?; 
    Ok(())
}

pub fn open_encrypted_file(path: &str, table_name: &str, records: &mut Vec<u8>) -> Result<(), MyError> {
    println!("Abrimos archivo encriptado");
    let file_path = get_file_path(path, table_name);
    let mut file = File::open(&file_path).map_err(|e| MyError::FileError(e.to_string()))?;
    file.read_to_end(records).map_err(|e| MyError::FileError(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::config::TABLE_PATH;

    #[test]
    pub fn test_encrypt_decrypt() {
        let key = Aes256Gcm::generate_key(OsRng);
        let original_data = "Hola mundo";

        let encrypted_data: Vec<u8> = encrypt_data(original_data, &key).unwrap();
        let decrypted_data: String = decrypt_data(&encrypted_data, &key).unwrap();

        assert_eq!(original_data, decrypted_data);
    }

    #[test]
    pub fn test_encrypt_decrypt_row() {
        let key = Aes256Gcm::generate_key(OsRng);
        let original_row = vec!["Hola mundo".to_string(), "Esto es un test".to_string()];

        let encrypted_row: Vec<u8> = encrypt_row(&original_row, &key).unwrap();
        let decrypted_row: Vec<String> = decrypt_row(&encrypted_row, &key).unwrap();

        assert_eq!(original_row, decrypted_row);
    }

    #[test]
    fn test_append_encrypted_rows() {
        let encryption_key: Key<Aes256Gcm> = Aes256Gcm::generate_key(OsRng);

        // Datos a encriptar
        let fila1 = vec!["valor1".to_string(), "valor2".to_string(),"valor3".to_string()];
        let fila2 = vec!["valor4".to_string(), "valor5".to_string(),"valor6".to_string()];

        // Encriptar los datos
        let encrypted_row1 = encrypt_row(&fila1, &encryption_key).unwrap();
        let encrypted_row2 = encrypt_row(&fila2, &encryption_key).unwrap();

        // Crear archivo temporario
        let temp_table = "test_table_encrypt_1";
        let file_path = format!("{}/{}.csv", TABLE_PATH, temp_table);

        let _ = append_encrypted_row(TABLE_PATH, &temp_table, &encrypted_row1);
        let _ = append_encrypted_row(TABLE_PATH, &temp_table, &encrypted_row2);

        let mut file = File::open(&file_path).unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();

        // Separar las filas cifradas por saltos de línea
        let parts: Vec<&[u8]> = contents.split(|&byte| byte == b'\n').collect();

        // Validar desencriptación
        let decrypted_row1 = decrypt_row(parts[0], &encryption_key).unwrap();
        let decrypted_row2 = decrypt_row(parts[1], &encryption_key).unwrap();
        assert_eq!(fila1, decrypted_row1);
        assert_eq!(fila2, decrypted_row2);

        let _ = remove_file(&file_path);   
    }

    #[test]
    fn test_encrypted_csv() {
        let encryption_key: Key<Aes256Gcm> = Aes256Gcm::generate_key(OsRng);

        // Datos a encriptar
        let headers: Vec<String> = vec!["columna1".to_string(), "columna2".to_string(), "columna3".to_string()];
        let fila1: Vec<String> = vec!["valor1".to_string(), "valor2".to_string(),"valor3".to_string()];
        let fila2: Vec<String> = vec!["valor4".to_string(), "valor5".to_string(),"valor6".to_string()];

        // Encriptar los datos
        let encrypted_headers = encrypt_row(&headers, &encryption_key).unwrap();
        let encrypted_row1 = encrypt_row(&fila1, &encryption_key).unwrap();
        let encrypted_row2 = encrypt_row(&fila2, &encryption_key).unwrap();

        // Crear archivo temporario
        let temp_table = "test_table_encrypt_2";
        let file_path = format!("{}/{}.csv", TABLE_PATH, temp_table);

        let _ = append_encrypted_row(TABLE_PATH, &temp_table,&encrypted_headers);
        let _ = append_encrypted_row(TABLE_PATH, &temp_table,&encrypted_row1);
        let _ = append_encrypted_row(TABLE_PATH, &temp_table,&encrypted_row2);

        // Leer el archivo y comparar
        let mut file = File::open(&file_path).unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();
        let parts: Vec<&[u8]> = contents.split(|&byte| byte == b'\n').collect();
        assert_eq!(4, parts.len()); // linea vacia al final

        let decrypted_header = decrypt_row(parts[0], &encryption_key).unwrap();
        let decrypted_row1 = decrypt_row(parts[1], &encryption_key).unwrap();
        let decrypted_row2 = decrypt_row(parts[2], &encryption_key).unwrap();
        assert_eq!(headers, decrypted_header);
        assert_eq!(fila1, decrypted_row1);
        assert_eq!(fila2, decrypted_row2);
        let _ = remove_file(&file_path);
    }
}