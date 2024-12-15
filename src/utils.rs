// Funciones auxiliares para el manejo de archivos y strings

use chrono::{DateTime, Duration, Utc};

use crate::config::{
    DIFFERENCE_DATE_IS_RECENT, TABLE_DESTINATION_NAME, TABLE_FLIGHT_SIMULATOR, TABLE_ORIGIN_NAME,
    TABLE_PATH,
};
use crate::encryption::encrypt::*;
use crate::{config::ENCRIPT_DATA, errors::my_errors::MyError};
use std::collections::VecDeque;
use std::fs::metadata;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    iter::Peekable,
    sync::{Arc, RwLock},
};

/// Returns the full quote inside ' 'S
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

/// Returns the full quote inside ' ' while advancing iterator
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

/// Gets the complete file path, returns it as String
pub fn get_file_path(path: &str, file_name: &str) -> String {
    let file_path = format!("{}/{}.csv", path, file_name);
    file_path
}

/// Open file in given path, returns an iterable reader
pub fn get_reader(path: &str, table_name: &str) -> Result<BufReader<File>, MyError> {
    let file_path = get_file_path(path, table_name);
    let file = File::open(file_path).map_err(|e| MyError::FileError(e.to_string()))?;
    let rdr = BufReader::new(file);
    Ok(rdr)
}

/// Open file in given path, returns an iterable reader
pub fn get_reader_path(file_path: &str) -> Result<BufReader<File>, MyError> {
    let file = File::open(file_path).map_err(|e| MyError::FileError(e.to_string()))?;
    let rdr = BufReader::new(file);
    Ok(rdr)
}

/// Open file in mode read, returns an iterable reader
pub fn get_reader_read(path: &Arc<RwLock<String>>) -> Result<BufReader<File>, MyError> {
    let path_table_file = path.read().map_err(|e| MyError::FileError(e.to_string()))?;
    let table_file = open_file(path_table_file.clone())?;
    Ok(BufReader::new(
        table_file
            .try_clone()
            .map_err(|e| MyError::FileError(e.to_string()))?,
    ))
}

/// Open file in mode write, returns an iterable reader
pub fn get_file_to_write(path: &Arc<RwLock<String>>) -> Result<File, MyError> {
    let path_table_file = path
        .write()
        .map_err(|e| MyError::FileError(e.to_string()))?;
    let table_file = open_file(path_table_file.clone())?;
    table_file
        .try_clone()
        .map_err(|e| MyError::FileError(e.to_string()))
}

/// Open file
pub fn open_file(path: String) -> Result<File, MyError> {
    let file = OpenOptions::new()
        .append(true)
        .read(true)
        .open(path)
        .map_err(|e| MyError::FileError(e.to_string()))?;
    Ok(file)
}

/// Receives an iterable into the CSV file
/// Returns the CSV file headers
pub fn get_file_headers<R: std::io::BufRead>(rdr: &mut R) -> Result<Vec<String>, MyError> {
    let mut headers_line = String::new();
    rdr.read_line(&mut headers_line)
        .map_err(|_| MyError::FileError("Error n get_File_headers".to_string()))?;
    let final_headers: String = if ENCRIPT_DATA {
        let decrypted_headers = decrypt_body_aes128(headers_line.as_bytes().to_vec())?;
        match String::from_utf8(decrypted_headers) {
            Ok(s) => s,
            Err(e) => return Err(MyError::DecryptionError(e.to_string())),
        }
    } else {
        headers_line
    };

    let headers: Vec<String> = final_headers
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

/// Writes rows to file in given path
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

/// Appends row to file
/// Expects file to end with a newline
pub fn append_row(file: &mut File, records: &[String]) -> Result<(), MyError> {
    writeln!(file, "{}", records.join(",")).map_err(|e| MyError::FileError(e.to_string()))?;
    Ok(())
}

/// Apppends row to already opened file
/// Expects file to end with a newline
pub fn append_row_with_file(file: &mut File, row: &[String]) -> Result<(), MyError> {
    file.seek(SeekFrom::End(0))
        .map_err(|e| MyError::FileError(e.to_string()))?;
    writeln!(file, "{}", row.join(",")).map_err(|e| MyError::FileError(e.to_string()))?;
    Ok(())
}

/// Creates a temporary file
/// Returns the opened File and it's name
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

/// Se elimina el archivo.
pub fn delete_file(file_name: &str, path: &str) -> Result<(), MyError> {
    let file_path = get_file_path(path, file_name);
    std::fs::remove_file(file_path).map_err(|e| MyError::FileError(e.to_string()))?;
    Ok(())
}

/// Writes the temporary file content in the original file
pub fn write_temporary_to_original(
    path: &str,
    temporary_file: &str,
    mut original_table: File,
) -> Result<(), MyError> {
    empty_table(&original_table)?;

    let rdr = get_reader(path, temporary_file)?;
    let lines = rdr.lines();

    for line in lines {
        let line = line.map_err(|e| MyError::FileError(e.to_string()))?;
        let record: Vec<String> = line.trim().split(",").map(|s| s.to_string()).collect();
        if ENCRIPT_DATA {
            let encrypted_line: Vec<u8> =
                encrypt_body_aes128(record.join(",").as_bytes().to_vec())?;
            original_table
                .write_all(&encrypted_line)
                .map_err(|e| MyError::FileError(e.to_string()))?;
            writeln!(original_table).map_err(|e| MyError::FileError(e.to_string()))?;
        } else {
            append_row(&mut original_table, &record)
                .map_err(|e| MyError::FileError(e.to_string()))?;
        }
    }
    Ok(())
}

fn empty_table(table: &File) -> Result<(), MyError> {
    table
        .set_len(0)
        .map_err(|e| MyError::FileError(e.to_string()))?;
    Ok(())
}

/// Se devuelve la primer linea de un archivo y la eliminamos de este.
pub fn get_and_remove_first_line(path: String) -> Result<Vec<String>, MyError> {
    let file = open_file(path.clone())?;
    let rdr = BufReader::new(file);

    let mut lines: VecDeque<String> = rdr
        .lines()
        .collect::<Result<_, _>>()
        .map_err(|e| MyError::FileError(e.to_string()))?;

    if lines.is_empty() {
        return Ok(vec![]);
    }

    if let Some(first_line) = lines.pop_front() {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|e| MyError::FileError(e.to_string()))?;

        for line in lines {
            writeln!(file, "{}", line).map_err(|e| MyError::FileError(e.to_string()))?;
        }

        return Ok(split_with_last_comma(first_line));
    }

    Ok(vec![])
}

fn split_with_last_comma(line: String) -> Vec<String> {
    let mut parts = line.rsplitn(2, ',');

    // rsplitn(2, ',') divide el string en dos partes, tomando la última coma como separador
    let last = parts.next().unwrap_or_default();
    let first = parts.next().unwrap_or_default(); // Todo lo anterior al último elemento

    vec![first.to_string(), last.to_string()]
}

/// Indica si un archivo esta vacio.
pub fn is_file_empty(path: &str) -> bool {
    match metadata(path) {
        Ok(metadata) => metadata.len() == 0,
        Err(_) => false,
    }
}

/// Se devuelve los paths de las tablas que se usan en el proyecto Rustic Airline.
pub fn get_rustic_airline_path_tables(node_number: String) -> Vec<String> {
    let mut tables = vec![];
    tables.push(format!(
        "{}/{}_{}.csv",
        TABLE_PATH, TABLE_ORIGIN_NAME, node_number
    ));
    tables.push(format!(
        "{}/{}_{}",
        TABLE_PATH, TABLE_ORIGIN_NAME, node_number
    ));
    tables.push(format!(
        "{}/{}_{}",
        TABLE_PATH, TABLE_DESTINATION_NAME, node_number
    ));
    tables.push(format!(
        "{}/{}_{}",
        TABLE_PATH, TABLE_FLIGHT_SIMULATOR, node_number
    ));
    tables
}

/// Indica si una fecha fue reciente.
pub fn date_is_recent(date: DateTime<Utc>) -> bool {
    let now = Utc::now();
    let difference = now.signed_duration_since(date);
    difference < Duration::seconds(DIFFERENCE_DATE_IS_RECENT) // Si la diferencia es menor a 5 minutos, es reciente
}
