// Funciones auxiliares para el manejo de archivos y strings

use crate::errors::my_errors::MyError;
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    iter::Peekable,
    sync::{Arc, RwLock},
};

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

// Open file in mode read, returns an iterable reader
pub fn get_reader_read(table: &Arc<RwLock<File>>) -> Result<BufReader<File>, MyError> {
    let table_file = table
        .read()
        .map_err(|e| MyError::FileError(e.to_string()))?;
    Ok(BufReader::new(
        table_file
            .try_clone()
            .map_err(|e| MyError::FileError(e.to_string()))?,
    ))
}

// Open file in mode write, returns an iterable reader
pub fn get_file_to_write(table: &Arc<RwLock<File>>) -> Result<File, MyError> {
    println!("en get, table: {:?}", table);
    let table_file = table
        .write()
        .map_err(|e| MyError::FileError(e.to_string()))?;
    println!("table_file: {:?}", table_file);
    Ok(
        table_file
            .try_clone()
            .map_err(|e| MyError::FileError(e.to_string()))?,
    )
}

// Receives an iterable into the CSV file
// Returns the CSV file headers
pub fn get_file_headers<R: std::io::BufRead>(rdr: &mut R) -> Result<Vec<String>, MyError> {
    let mut headers_line = String::new();
    rdr.read_line(&mut headers_line)
        .map_err(|e| MyError::FileError(e.to_string()))?;
    println!("hace read, headers_line: {:?}", headers_line);

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
pub fn append_row(file: &mut File, records: &[String]) -> Result<(), MyError> {
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
    mut original_table: File,
) -> Result<(), MyError> {

    let rdr = get_reader(path, temporary_file)?;
    let lines = rdr.lines();

    // Go through temp file and write in original
    for line in lines {
        let line = line.map_err(|e| MyError::FileError(e.to_string()))?;
        let record: Vec<String> = line.trim().split(",").map(|s| s.to_string()).collect();

        append_row(&mut original_table, &record).map_err(|e| MyError::FileError(e.to_string()))?;
    }
    Ok(())
}
