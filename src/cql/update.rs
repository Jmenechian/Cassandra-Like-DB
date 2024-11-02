// Representa las consultas del tipo UPDATE
// Aca se parsea la consulta y se ejecuta
use crate::config::TABLE_PATH;
use crate::cql::filter::Filter;
use crate::cql::where_clause::WhereClause;
use crate::my_errors::MyError;
use crate::utils::*;
use std::collections::HashMap;
use std::io::BufRead;

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
    pub fn execute_command(&mut self) -> Result<Vec<String>, MyError> {
        let mut rdr = get_reader(TABLE_PATH, &self.table)?;
        let col_names = get_file_headers(&mut rdr)?;
        self.check_columns_are_valid(&col_names)?;

        let (mut temp_file, temp_file_name) = create_temporary_file(TABLE_PATH, &self.table)?;
        append_row_with_file(&mut temp_file, &col_names)?;

        // Save old rows and updated rows
        let filter = Filter::new(col_names.clone(), self.whereclause.clone())?;
        let lines = rdr.lines();
        for line in lines {
            let line = line.map_err(|e| MyError::Error(e.to_string()))?;
            let mut row: Vec<String> = line
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
                if let Some(header_name) = col_names.get(header_index) {
                    if let Some(new_value) = self.set.get(header_name) {
                        updated_row.push(new_value.to_string());
                    } else {
                        updated_row.push(value.to_string());
                    }
                }
            }
            append_row_with_file(&mut temp_file, &updated_row)?;
        }

        write_temporary_to_original(TABLE_PATH, &temp_file_name, &self.table)?;
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

// UPDATE origin SET airport = AAAAA, date = 02-12-02, destination = BBBBBBB WHERE airport = MEX AND date = 24-12-24 AND destination = AEP
// SELECT * FROM origin WHERE airport = MEX AND date = 24-12-24 AND destination = AEP
// INSERT INTO origin (airport, destination, date, flight, status) VALUES (MEX, AEP, 24-12-24, 123, scheduled)
