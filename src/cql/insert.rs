// Representa a las consultas del tipo INSERT
// Aca se parsea la consulta y se ejecuta

use crate::config::TABLE_PATH;
use crate::my_errors::MyError;
use crate::utils::*;
use chrono::Utc;
use std::collections::HashMap;
#[derive(Debug, Eq, PartialEq)]
pub struct Insert {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<String>,
    pub num_node: String,
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
            num_node: "0".to_string(),
        }
    }

    // Parses the query for later execution
    pub fn parse_insert(&mut self, query: &str, address_node: &str) -> Result<(), MyError> {
        self.num_node = address_node.chars().last().unwrap().to_string();

        let parts: Vec<&str> = query.split_whitespace().collect();

        if parts.len() < 4 || parts[0] != "INSERT" || parts[1] != "INTO" {
            return Err(MyError::InvalidSyntax("INSERT query not valid".to_string()));
        }

        if !parts.contains(&"VALUES") {
            return Err(MyError::InvalidSyntax("INSERT query not valid".to_string()));
        }

        let mut skip_index = 3;
        if parts[2].contains("'") {
            let name = get_full_quote(&parts, 2);
            let name_v: Vec<&str> = name.split_whitespace().collect();
            println!("nameee {:?}", name_v);
            skip_index += name_v.len() - 1;
            self.table = name;
        } else {
            self.table = parts[2].to_string();
        }
        self.table = format!("{}_{}", self.table, self.num_node);
        println!("INSERT en tabla: {}", self.table);

        let headers: Vec<String> = self.extract_values(&parts, skip_index);
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

    // Extrae los nombres de las columns y los valores a setear
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

    // Devuelve un HashMap de las columnas y sus valores
    fn get_mapped_values(&self) -> Result<HashMap<&str, &str>, MyError> {
        let mut map: HashMap<&str, &str> = HashMap::new();
        for (key, value) in self.columns.iter().zip(self.values.iter()) {
            if map.keys().any(|k| k == &key.as_str()) {
                return Err(MyError::InvalidColumn("Column repeated".to_string()));
            }
            map.insert(key.as_str(), value.as_str());
        }
        Ok(map)
    }

    // Chequea que las columnas sean validas
    fn check_columns_are_valid(&self, file_headers: &[String]) -> Result<(), MyError> {
        for header in &self.columns {
            if !file_headers.contains(header) {
                return Err(MyError::InvalidColumn("Column not valid".to_string()));
            }
        }
        if self.columns.len() != self.values.len() {
            return Err(MyError::InvalidSyntax(
                "Different number of values and columns".to_string(),
            ));
        }

        if self.columns.len() > file_headers.len() {
            return Err(MyError::InvalidColumn("Columns are repeated".to_string()));
        }
        Ok(())
    }

    // Ejecuta la consulta INSERT
    pub fn execute_command(&self) -> Result<String, MyError> {
        println!("En execute_command de INSERT");
        let mut rdr = get_reader(TABLE_PATH, &self.table)?;
        let headers = get_file_headers(&mut rdr)?;
        // self.values.push(Utc::now().to_string());
        // INSERT INTO VALUES (val1, val2)
        // values are inserted in the same order as the file headers
        if self.columns.is_empty() {
            if self.values.len() == headers.len() {
                append_row(TABLE_PATH, &self.table, &self.values)?;
                return Ok("linea fue agregada".to_string());
            }
            return Err(MyError::InvalidSyntax("Invalid INSERT syntax".to_string()));
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
        append_row(TABLE_PATH, &self.table, &new_record)?;
        Ok("linea fue agregada".to_string())
    }

    pub fn get_key(&self) -> String {
        //suponiendo que siempre se manda bien
        println!("values = {:?}", self.values);
        let res = format!("{},{}", self.values[0], self.values[1]);
        res
    }

    pub fn get_values(&mut self) -> String {
        self.values.push(Utc::now().to_string());
        self.values.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::my_errors::MyError;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_parse_insert_valid_query() {
        let mut insert = Insert::new();
        let query = "INSERT INTO 'users' (id, name, age) VALUES (1, 'Sofia Lopez', 30)";
        let result = insert.parse_insert(query, "node_1");

        assert!(result.is_ok());
        assert_eq!(insert.table, "users_1");
        assert_eq!(
            insert.columns,
            vec!["id".to_string(), "name".to_string(), "age".to_string()]
        );
        assert_eq!(
            insert.values,
            vec!["1".to_string(), "Sofia Lopez".to_string(), "30".to_string()]
        );
    }

    // #[test]
    // fn test_parse_insert_invalid_syntax() {
    //     let mut insert = Insert::new();
    //     let query = "INSERT INTO users id, name VALUES (1, 'Alice')";
    //     let result = insert.parse_insert(query, "node_1");

    //     assert!(matches!(result, Err(MyError::InvalidSyntax(_))));
    // }

    #[test]
    fn test_parse_insert_missing_values() {
        let mut insert = Insert::new();
        let query = "INSERT INTO 'users' (id, name, age)";
        let result = insert.parse_insert(query, "node_1");

        assert!(matches!(result, Err(MyError::InvalidSyntax(_))));
    }

    #[test]
    fn test_get_mapped_values() {
        let mut insert = Insert::new();
        insert.columns = vec!["id".to_string(), "name".to_string()];
        insert.values = vec!["1".to_string(), "Alice".to_string()];
        let result = insert.get_mapped_values();

        let mut expected = HashMap::new();
        expected.insert("id", "1");
        expected.insert("name", "Alice");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_get_mapped_values_repeated_columns() {
        let mut insert = Insert::new();
        insert.columns = vec!["id".to_string(), "id".to_string()];
        insert.values = vec!["1".to_string(), "2".to_string()];
        let result = insert.get_mapped_values();

        assert!(matches!(result, Err(MyError::InvalidColumn(_))));
    }

    #[test]
    fn test_check_columns_are_valid() {
        let mut insert = Insert::new();
        insert.columns = vec!["id".to_string(), "name".to_string()];
        insert.values = vec!["1".to_string(), "Sofia".to_string()];
        let file_headers = vec!["id".to_string(), "name".to_string(), "age".to_string()];

        let result = insert.check_columns_are_valid(&file_headers);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_columns_are_invalid() {
        let mut insert = Insert::new();
        insert.columns = vec!["invalid_col".to_string()];
        insert.values = vec!["valid_value".to_string()];
        let file_headers = vec!["id".to_string(), "name".to_string()];

        let result = insert.check_columns_are_valid(&file_headers);
        assert!(matches!(result, Err(MyError::InvalidColumn(_))));
    }

    #[test]
    fn test_execute_command() {
        // Create a mock CSV file
        let file_path = format!("{}/test_table_1.csv", TABLE_PATH);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "id,name,age").unwrap();
        writeln!(file, "1,Alice,30").unwrap();

        // Create and parse a valid INSERT query
        let mut insert = Insert::new();
        insert
            .parse_insert(
                "INSERT INTO 'test_table' (id, name, age) VALUES (2, 'Bob', 25)",
                "node_1",
            )
            .unwrap();

        // Execute the command
        let result = insert.execute_command().unwrap();
        assert_eq!(result, "linea fue agregada");

        // Clean up
        std::fs::remove_file(file_path).unwrap();
    }

    // #[test]
    // fn test_execute_command_invalid_syntax() {
    //     let mut insert = Insert::new();
    //     insert.columns = vec!["id".to_string(), "name".to_string()];
    //     insert.values = vec!["1".to_string()]; // Missing values
    //     let result = insert.execute_command();

    //     assert!(matches!(result, Err(MyError::InvalidSyntax(_))));
    // }
}
