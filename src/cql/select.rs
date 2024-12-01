use crate::config::ENCRIPT_DATA;
use crate::cql::filter::Filter;
use crate::cql::where_clause::WhereClause;
use crate::encryption::encrypt::decrypt_body_aes128;
use crate::errors::my_errors::MyError;
use crate::utils::*;
use std::io::BufRead;
use std::sync::{Arc, RwLock};

#[derive(Debug, Eq, PartialEq)]
/// Representa las consultas del tipo SELECT
/// Se parsea la consulta y se ejecuta
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

    /// Se parsea la query.
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
        let whitespaces: Vec<&str> = x.split_whitespace().collect();
        let index = whitespaces.len() + 2; // skip SELECT, FROM y columnas

        self.parse_table(&parts, index);

        self.parse_where_condition(&mut parts)?;

        Ok(())
    }

    /// Setea lista de columnas.
    /// Columnas entre comillas, quedan entre comillas.
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

    /// Setea el nombre de la tabla con el numero de nodo.
    /// Si el nombre de la tabla tiene comillas, se las saca.
    fn parse_table(&mut self, parts: &[&str], index: usize) {
        if parts[index].contains("'") {
            self.table = get_full_quote(parts, index);
        } else {
            self.table = parts[index].to_string();
        }
        self.table = format!("{}_{}", self.table, self.num_node);
        println!("SELECT en tabla: {}", self.table);
    }

    /// Setea condicion WHERE.
    /// Devuelve error si la condicion no es valida.
    fn parse_where_condition(&mut self, parts: &mut [&str]) -> Result<(), MyError> {
        if let Some(where_index) = parts.iter().position(|&x| x == "WHERE") {
            let (_, right) = parts.split_at(where_index);
            let condition_query = right[1..].join(" ");
            let where_clause = WhereClause::new(condition_query)?;
            self.whereclause = where_clause;
        }
        Ok(())
    }

    /// Se ejecuta la query sobre la tabla.
    pub fn execute_command(
        &mut self,
        path_table: Arc<RwLock<String>>,
    ) -> Result<Vec<String>, MyError> {
        let my_path: Arc<RwLock<String>> = path_table.clone();
        let mut rdr_read = get_reader_read(&my_path)?;
        let headers = get_file_headers(&mut rdr_read)?;
        let filter = Filter::new(headers.clone(), self.whereclause.clone())?;
        let mut tabla = Vec::new();
        for line in rdr_read.lines() {
            let line = line.map_err(|e| MyError::FileError(e.to_string()))?;
            let final_line: String = if ENCRIPT_DATA {
                let decrypted_line = decrypt_body_aes128(line.as_bytes().to_vec())?;
                match String::from_utf8(decrypted_line) {
                    Ok(s) => s,
                    Err(e) => return Err(MyError::DecryptionError(e.to_string())),
                }
            } else {
                line
            };
            let row: Vec<String> = final_line
                .split(",")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            if filter.filter(&row) {
                tabla.push(final_line);
            }
        }

        Ok(tabla)
    }

    /// Se devuelven los valores que representa las pasticiones de las tablas.
    /// Es decir, el nombre del aeropuerto y la fecha, y se la devuleve como un string de la forma: "airport,date".
    pub fn get_key(&self) -> String {
        println!("Where = {:?}", self.whereclause);
        let aeropuerto = if let Some(aeropuerto) = self.whereclause.find_value_for_key("airport") {
            aeropuerto
        } else {
            "".to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_select_valid_query() {
        let mut select = Select::new();
        let query = "SELECT id, name FROM 'users' WHERE age > 30";
        let result = select.parse_select(query, "node_1");

        assert!(result.is_ok());
        assert_eq!(select.columns, vec!["id".to_string(), "name".to_string()]);
        assert_eq!(select.table, "users_1".to_string());
        assert_eq!(select.num_node, "1".to_string());
    }

    #[test]
    fn test_parse_select_invalid_query() {
        let mut s1 = Select::new();
        let q1 = "SELECT FROM 'users' WHERE age > 30";
        let r1 = s1.parse_select(q1, "node_1");
        assert!(matches!(r1, Err(MyError::InvalidSyntax(_))));

        let mut s2 = Select::new();
        let q2 = "SELECT id, name WHERE age > 30";
        let r2 = s2.parse_select(q2, "node_1");

        assert!(matches!(r2, Err(MyError::InvalidSyntax(_))));
    }

    #[test]
    fn test_parse_table() {
        let mut select = Select::new();
        let query = "SELECT id, name FROM 'users'";
        let parts = query.split_whitespace().collect::<Vec<&str>>();
        select.parse_table(&parts, 4);
        assert_eq!(select.num_node, "0".to_string());
        assert_eq!(select.table, "users_0".to_string());
    }

    #[test]
    fn test_parse_columns() {
        let mut select = Select::new();
        let parts = vec!["'columna", "1',", "'columna2',", "'columna", "3'"];
        select.parse_columns(&parts);
        assert_eq!(
            select.columns,
            vec![
                "'columna 1'".to_string(),
                "'columna2'".to_string(),
                "'columna 3'".to_string()
            ]
        );
    }
}
