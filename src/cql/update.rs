// Representa las consultas del tipo UPDATE
// Aca se parsea la consulta y se ejecuta
use crate::config::TABLE_PATH;
use crate::cql::filter::Filter;
use crate::cql::where_clause::WhereClause;
use crate::errors::my_errors::MyError;
use crate::utils::*;
use crate::config::ENCRIPT_DATA;
use crate::messagestuff::encrypt::*;
use std::collections::HashMap;
use std::io::BufRead;
use chrono::Utc;
use std::sync::{Arc, RwLock};

#[derive(Debug, Eq, PartialEq)]
pub struct Update {
    pub table: String,
    pub set: HashMap<String, String>,
    pub whereclause: WhereClause,
    pub num_node: String,
}

impl Default for Update {
    fn default() -> Self {
        Self::new()
    }
}

impl Update {
    pub fn new() -> Update {
        Update {
            table: "".to_string(),
            set: HashMap::new(),
            whereclause: WhereClause::True,
            num_node: "0".to_string(),
        }
    }

    // Parser query for later execution
    pub fn parse_update(&mut self, query: &str, address_node: &str) -> Result<(), MyError> {
        let port = match address_node.chars().last() {
            Some(c) => c,
            None => return Err(MyError::Error("No puse obtener el puerto".to_string())),
        };
        self.num_node = port.to_string();

        let parts: Vec<&str> = query.split_whitespace().collect();

        if parts.len() < 6 || parts[0] != "UPDATE" || parts[1] == "SET" {
            return Err(MyError::InvalidSyntax("UPDATE query not valid".to_string()));
        }

        let name = if parts[1].starts_with("'") {
            get_full_quote(&parts, 1)
        } else {
            parts[1].to_string()
        };

        self.table = format!("{}_{}", name, self.num_node);

        if let Some(set_index) = parts.iter().position(|&part| part == "SET") {
            self.get_set_values(&parts[set_index + 1..])?;
        } else {
            return Err(MyError::InvalidSyntax("UPDATE query not valid".to_string()));
        }

        if let Some(where_index) = parts.iter().position(|&part| part == "WHERE") {
            let (_, right) = parts.split_at(where_index);
            let condition_query = right[1..].join(" ");
            let where_clause = WhereClause::new(condition_query)?;
            self.whereclause = where_clause;
        }
        Ok(())
    }

    // Saves SET values in self.set
    fn get_set_values(&mut self, parts: &[&str]) -> Result<(), MyError> {
        let cleaned_parts = self.get_cleaned_parts(parts);
        let chunks_iter = cleaned_parts.chunks(3);
        for chunk in chunks_iter {
            if chunk[0] == "WHERE" {
                return Ok(());
            }
            self._get_set_values(chunk)?;
        }
        self.set.insert("timestamp".to_string(), Utc::now().to_string());
        Ok(())
    }

    fn _get_set_values(&mut self, chunk: &[String]) -> Result<(), MyError> {
        if chunk.len() < 3 || chunk[1] != "=" {
            return Err(MyError::InvalidSyntax("Invalid SET syntax".to_string()));
        }

        let col = chunk[0].replace(",", "").replace("(", "").replace(")", "");
        let val = chunk[2].replace(",", "").replace("(", "").replace(")", "");
        self.set.insert(col, val);
        Ok(())
    }

    // Cleans the parts in the vector
    // Joins elements inside ' '
    fn get_cleaned_parts(&self, parts: &[&str]) -> Vec<String> {
        let mut cleaned_parts: Vec<String> = Vec::new();

        let mut parts_iter = parts.iter().peekable();
        while let Some(part) = parts_iter.next() {
            let cleaned_part = part.replace(",", "").replace("(", "").replace(")", "");

            if cleaned_part == "WHERE" {
                break;
            }
            if !cleaned_part.starts_with("'") {
                cleaned_parts.push(cleaned_part);
                continue;
            }
            if cleaned_part.ends_with("'") {
                cleaned_parts.push(cleaned_part.replace("'", ""));
            } else {
                let mut quoted_name = cleaned_part.replace("'", "");
                quoted_name.push(' ');
                quoted_name.push_str(get_full_quote_iter(&mut parts_iter).as_str());
                cleaned_parts.push(quoted_name);
            }
        }
        cleaned_parts
    }

    // Checks if SET columns are found in the file
    fn check_columns_are_valid(&self, file_headers: &[String]) -> Result<(), MyError> {
        for header in self.set.keys() {
            if !file_headers.contains(header) {
                return Err(MyError::InvalidColumn(format!(
                    "Column `{}` not valid",
                    &header
                )));
            }
        }
        Ok(())
    }

    // Executes the query

    pub fn execute_command(
        &mut self,
        path_table: Arc<RwLock<String>>,
    ) -> Result<Vec<String>, MyError> {
        let my_path: Arc<RwLock<String>> = path_table.clone();

        let mut rdr_read = get_reader_read(&my_path)?;
        let headers = get_file_headers(&mut rdr_read)?;
        self.check_columns_are_valid(&headers)?;

        let file_open = get_file_to_write(&path_table)?;
        let (mut temp_file, temp_file_name) = create_temporary_file(TABLE_PATH, &self.table)?;
        append_row_with_file(&mut temp_file, &headers)?;
        let filter = Filter::new(headers.clone(), self.whereclause.clone())?;
        for line in rdr_read.lines() {
            let line = line.map_err(|e| MyError::Error(e.to_string()))?;
            let final_line: String = if ENCRIPT_DATA {
                let decrypted_line = decrypt_body_aes128(line.as_bytes().to_vec())?;
                match  String::from_utf8(decrypted_line) {
                    Ok(s) => s,
                    Err(e) => return Err(MyError::DecryptionError(e.to_string())),
                }
            } else {
                line
            };
            
            let mut row: Vec<String> = final_line
                .split(",")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            if !filter.filter(&row) {
                append_row_with_file(&mut temp_file, &row)?;
                continue;
            }

            let mut updated_row: Vec<String> = Vec::new();

            // Modify the row based on the SET clause
            for (header_index, value) in row.iter_mut().enumerate() {
                if let Some(header_name) = headers.get(header_index) {
                    if let Some(new_value) = self.set.get(header_name) {
                        updated_row.push(new_value.to_string());
                    } else {
                        updated_row.push(value.to_string());
                    }
                }
            }
            append_row_with_file(&mut temp_file, &updated_row)?;
        }

        write_temporary_to_original(TABLE_PATH, &temp_file_name, file_open)?;
        delete_file(&temp_file_name, TABLE_PATH)?;
        Ok(vec!["Se updateo la tabla".to_string()])
    }

    pub fn get_key(&self) -> String {
        println!("Where = {:?}", self.whereclause);
        let aeropuerto = if let Some(aeropuerto) = self.whereclause.find_value_for_key("airport") {
            aeropuerto
        } else {
            "".to_string() // algun error
        };
        let date = if let Some(date) = self.whereclause.find_value_for_key("date") {
            date
        } else {
            "".to_string()
        };
        let res = format!("{},{}", aeropuerto, date);
        res
    }
}
