use crate::cql::utils::*;
use std::error::Error;
use std::io::BufRead;

#[derive(Debug, Eq, PartialEq)]
pub struct Select {
    pub columns: Vec<String>,
    pub table: String,
    pub condition_tokens: String,
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
            condition_tokens: "".to_string(),
        }
    }

    pub fn parse_select(&mut self, query: &str) -> Result<(), Box<dyn Error>> {
        let mut parts: Vec<&str> = query.split_whitespace().collect();

        if parts.len() < 4 || parts[0] != "SELECT" || parts[1] == "FROM" {
            return Err(Box::from("Select query not valid"));
        }

        if let Some(from_index) = parts.iter().position(|&x| x == "FROM") {
            self.parse_columns(&parts[1..from_index]);
        } else {
            return Err(Box::from("Select query not valid"));
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
            self.table = parts[index].to_string()
        }
    }

    // sets WHERE condition
    // returns error if condition query is not valid
    fn parse_where_condition(&mut self, _parts: &mut [&str]) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn execute_command(&mut self, _path: &str) -> Result<String, Box<dyn Error>> {
        println!("self table: {}", self.table);
        let rdr = get_reader(
            "/home/loli/Documents/Taller/Grupal/aerolineas_rusticas/src/tablas",
            &self.table,
        )?;

        let mut tabla: String = String::new();
        for line in rdr.lines() {
            let line = line?;
            tabla.push_str(&line);
        }
        Ok(tabla)
    }
    // let headers = get_file_headers(&mut rdr)?;

    // let expression = self.get_where_expression()?;
    // let mut rdr =
    //     get_reader(path, &self.table)?;
    // let headers = get_file_headers(&mut rdr)?;

    // if self.columns[0] != "*" {
    //     self.check_columns_are_valid(&headers)?;
    // }

    // let temporary_file =
    //     create_temporary_file(path, &self.table)?;

    // // get rows that fulfill WHERE condition
    // let mut records: Vec<Vec<String>> = Vec::new();
    // let lines = rdr.lines();
    // for line in lines {
    //     let line = line?;
    //     let row_array: Vec<String> = line.trim().split(",").map(|s| s.to_string()).collect();

    //     if fulfill_condition(&headers, &row_array, &expression)? {
    //         if !self.has_orderby() {
    //             append_row(path, &temporary_file, &row_array)
    //         }
    //         records.push(row_array);
    //     }
    // }

    // if self.has_orderby() {
    //     self.order_rows_by_orderby(&headers, &mut records)?;
    //     write_rows(path, &temporary_file, &records)
    // }

    // self.print_rows(&temporary_file, path, headers)?;
    // delete_file(&temporary_file, path)?;
    // Ok(())
}
