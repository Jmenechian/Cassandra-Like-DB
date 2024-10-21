use crate::my_errors::MyError;
use std::{fs::File, fs::OpenOptions, io::BufRead, io::BufReader, io::Write, iter::Peekable};

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
    println!("Se abrirá el archivo en FILE PATH: {}", file_path);
    file_path
}

// Open file in given path, returns an iterable reader
pub fn get_reader(path: &str, table_name: &str) -> Result<BufReader<File>, MyError> {
    let file_path = get_file_path(path, table_name);
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

// Writes row to file in given path
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
// Expects file to end with with a newline
pub fn append_row(path: &str, table_name: &str, records: &[String]) -> Result<(), MyError> {
    let file_path = get_file_path(path, table_name);

    let mut file = OpenOptions::new()
        .append(true)
        .open(file_path)
        .map_err(|e| MyError::FileError(e.to_string()))?;

    writeln!(file, "{}", records.join(",")).map_err(|e| MyError::FileError(e.to_string()))?;
    Ok(())
}

pub fn create_temporary_file(path: &str, original_table: &str) -> Result<String, MyError> {
    let mut temporary_table: String = String::from(original_table);
    temporary_table.push_str("_temp");

    let file_path = get_file_path(path, &temporary_table);
    let _ = File::create(file_path).map_err(|e| MyError::FileError(e.to_string()))?;
    Ok(temporary_table)
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
