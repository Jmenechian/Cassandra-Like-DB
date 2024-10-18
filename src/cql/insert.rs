use crate::cql::utils::*;
use std::collections::HashMap;
use std::error::Error;
use chrono::{DateTime, Utc};
#[derive(Debug, Eq, PartialEq)]
pub struct Insert {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<String>,
}

impl Default for Insert {
    fn default() -> Self {
        Self::new()
    }
}

impl Insert {
    pub fn new() -> Insert {
        Insert {
            table: "".to_string(),
            columns: Vec::new(),
            values: Vec::new(),
        }
    }

    // Parses the query for later execution
    pub fn parse_insert(&mut self, query: &str) -> Result<(), Box<dyn Error>> {
        let parts: Vec<&str> = query.split_whitespace().collect();

        if parts.len() < 4 || parts[0] != "INSERT" || parts[1] != "INTO" {
            return Err(Box::from("INSERT query not valid".to_string()));
        }

        if !parts.contains(&"VALUES") {
            return Err(Box::from("INSERT query not valid".to_string()));
        }

        let mut skip_index = 3;
        if parts[2].contains("'") {
            let name = get_full_quote(&parts, 2);
            let name_v: Vec<&str> = name.split_whitespace().collect();
            skip_index += name_v.len();
            self.table = name;
        } else {
            self.table = parts[2].to_string();
        }

        let mut headers: Vec<String> = self.extract_values(&parts, skip_index);
        // headers.push("timestamp".to_string());

        skip_index += headers
            .iter()
            .map(|h| h.split_whitespace().count())
            .sum::<usize>()
            + 1;

        let values: Vec<String> = self.extract_values(&parts, skip_index);

        self.columns = headers;
        self.values = values;
        Ok(())
    }

    // Extracts columns names and values to insert
    fn extract_values(&self, parts: &[&str], skip_index: usize) -> Vec<String> {
        let mut values: Vec<String> = Vec::new();
        let mut parts_iter = parts.iter().skip(skip_index).peekable();

        while let Some(value) = parts_iter.next() {
            let cleaned_value = value.replace(",", "").replace("(", "").replace(")", "");

            if cleaned_value == "VALUES" {
                break;
            }

            let mut name: String = String::new();
            if cleaned_value.contains("'") {
                if !cleaned_value.ends_with("'") {
                    name.push_str(&cleaned_value.replace("'", ""));
                    name.push(' ');
                    name.push_str(get_full_quote_iter(&mut parts_iter).as_str());
                } else {
                    name.push_str(&cleaned_value.replace("'", ""));
                }
                values.push(name);
            } else {
                values.push(cleaned_value);
            }
        }
        values
    }

    // Returns columns and values mapped into a HashMap
    fn get_mapped_values(&self) -> Result<HashMap<&str, &str>, Box<dyn Error>> {
        let mut map: HashMap<&str, &str> = HashMap::new();
        for (key, value) in self.columns.iter().zip(self.values.iter()) {
            if map.keys().any(|k| k == &key.as_str()) {
                return Err(Box::from("Columns are repeated".to_string()));
            }
            map.insert(key.as_str(), value.as_str());
        }
        Ok(map)
    }

    // No se si es necesaria
    fn check_columns_are_valid(&self, file_headers: &[String]) -> Result<(), Box<dyn Error>> {
        for header in &self.columns {
            if !file_headers.contains(header) {
                return Err(Box::from("Column not valid"));
            }
        }
        if self.columns.len() != self.values.len() {
            return Err(Box::from(
                "Different number of values and columns".to_string(),
            ));
        }

        if self.columns.len() > file_headers.len() {
            return Err(Box::from("Columns are repeated".to_string()));
        }
        Ok(())
    }

    pub fn execute_command(& mut self, _path: &str) -> Result<String, Box<dyn Error>> {
        println!("En execute_command de INSERT");
        let path = "src/tablas";
        let mut rdr = get_reader(path, &self.table)?;
        let mut headers = get_file_headers(&mut rdr)?;
        // self.values.push(Utc::now().to_string());
        // INSERT INTO VALUES (val1, val2)
        // values are inserted in the same order as the file headers
        if self.columns.is_empty() {
            if self.values.len() == headers.len() {
                append_row(path, &self.table, &self.values)?;
                return Ok("linea fue agregada".to_string());
            }
            return Err(Box::from("Invalid INSERT syntax".to_string()));
        }

        self.check_columns_are_valid(&headers)?;

        let map: HashMap<&str, &str> = self.get_mapped_values()?;

        let mut new_record: Vec<String> = Vec::new();
        for header in headers {
            if let Some(value) = map.get(header.as_str()) {
                new_record.push(value.to_string());
            } else {
                new_record.push(String::new());
            }
        }
        new_record.pop(); // sacas el timestamp vacio porque nunca lo llenas
        new_record.push(Utc::now().to_string()); // metes lo mas nuevo
        append_row(path, &self.table, &new_record)?;
        Ok("linea fue agregada".to_string())
    }

    pub fn get_key(&self) -> String { //suponiendo que siempre se manda bien
        println!("values = {:?}", self.values);
        let res = format!("{}{}", self.values[0], self.values[1]);
        res
    }
}
