use crate::my_errors::MyError;
use crate::utils::*;
use std::io::BufRead;

use crate::config::TABLE_PATH;
use crate::cql::filter::Filter;
use crate::cql::where_clause::WhereClause;

#[derive(Debug, Eq, PartialEq)]
pub struct Select {
    pub columns: Vec<String>,
    pub table: String,
    pub whereclause: WhereClause,
    pub num_node: String,
}

impl Default for Select {
    fn default() -> Self {
        Self::new()
    }
}

impl Select {
    pub fn new() -> Select {
        Select {
            columns: Vec::new(),
            table: "".to_string(),
            whereclause: WhereClause::True,
            num_node: "0".to_string(),
        }
    }

    pub fn parse_select(&mut self, query: &str, address_node: &str) -> Result<(), MyError> {
        let port = match address_node.chars().last() {
            Some(c) => c,
            None => return Err(MyError::Error("No puse obtener el puerto".to_string())),
        };
        self.num_node = port.to_string();

        let mut parts: Vec<&str> = query.split_whitespace().collect();
        if parts.len() < 4 || parts[0] != "SELECT" || parts[1] == "FROM" {
            return Err(MyError::InvalidSyntax("Select query not valid".to_string()));
        }

        if let Some(from_index) = parts.iter().position(|&x| x == "FROM") {
            self.parse_columns(&parts[1..from_index]);
        } else {
            return Err(MyError::InvalidSyntax("Select query not valid".to_string()));
        }

        let x = self.columns.join(" ");
        let whitespaces: Vec<&str> = x.split_whitespace().collect(); // makes sure to count columns with whistespaces inside
        let index = whitespaces.len() + 2; // skip SELECT, FROM and columns

        self.parse_table(&parts, index);

        self.parse_where_condition(&mut parts)?;

        Ok(())
    }

    // sets list of columns
    // columns between quotes stay between quotes for the Tokenizer
    fn parse_columns(&mut self, parts: &[&str]) {
        let joined_parts = parts.join(" ");
        let comma_separated: Vec<&str> = joined_parts.split(",").collect();
        let mut columns: Vec<String> = Vec::new();

        for column in comma_separated {
            if column == "FROM" {
                break;
            }
            columns.push(
                column
                    .trim()
                    .replace("(", "")
                    .replace(")", "")
                    .replace(",", "")
                    .to_string(),
            );
        }
        self.columns = columns
    }

    // sets table name
    // if table name between quotes, removes quotes
    fn parse_table(&mut self, parts: &[&str], index: usize) {
        if parts[index].contains("'") {
            self.table = get_full_quote(parts, index);
        } else {
            self.table = parts[index].to_string();
        }
        self.table = format!("{}_{}", self.table, self.num_node);
        println!("SELECT en tabla: {}", self.table);
    }

    // sets WHERE condition
    // returns error if condition query is not valid
    fn parse_where_condition(&mut self, parts: &mut [&str]) -> Result<(), MyError> {
        if let Some(where_index) = parts.iter().position(|&x| x == "WHERE") {
            let (_, right) = parts.split_at(where_index);
            let condition_query = right[1..].join(" ");
            let where_clause = WhereClause::new(condition_query)?;
            self.whereclause = where_clause;
        }
        Ok(())
    }

    pub fn execute_command(&mut self) -> Result<Vec<String>, MyError> {
        //cambiarlo a vec string
        let mut rdr = get_reader(TABLE_PATH, &self.table)?;
        let col_names = get_file_headers(&mut rdr)?;

        let mut tabla: Vec<String> = Vec::new();
        let filter = Filter::new(col_names.clone(), self.whereclause.clone())?;

        for line in rdr.lines() {
            let line = line.map_err(|e| MyError::FileError(e.to_string()))?;
            let split_line = line.split(",").collect::<Vec<&str>>();
            if filter.filter(split_line.into_iter().map(|s| s.to_string()).collect()) {
                tabla.push(line);
            }
        }
        Ok(tabla)
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
