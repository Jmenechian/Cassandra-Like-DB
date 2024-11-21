// Representa las consultas del tipo DELETE
// Aca se parsea la consulta y se ejecuta
use crate::config::TABLE_PATH;
use crate::cql::{filter::Filter, where_clause::WhereClause};
use crate::errors::my_errors::MyError;
use crate::utils::*;
use std::io::BufRead;
use std::sync::{Arc, RwLock};

#[derive(Debug, Eq, PartialEq)]
pub struct Delete {
    pub table: String,
    pub whereclause: WhereClause,
    pub num_node: String,
}

impl Default for Delete {
    fn default() -> Self {
        Self::new()
    }
}

impl Delete {
    pub fn new() -> Delete {
        Delete {
            table: "".to_string(),
            whereclause: WhereClause::True,
            num_node: "0".to_string(),
        }
    }

    // Parsea la consulta para ejecutar mas tarde
    pub fn parse_delete(&mut self, query: &str, address_node: &str) -> Result<(), MyError> {
        let port = match address_node.chars().last() {
            Some(c) => c,
            None => return Err(MyError::Error("No puse obtener el puerto".to_string())),
        };
        self.num_node = port.to_string();

        let parts: Vec<&str> = query.split_whitespace().collect();

        if parts.len() < 3 || parts[0] != "DELETE" || parts[1] != "FROM" {
            return Err(MyError::InvalidSyntax("DELETE query not valid".to_string()));
        }

        let name: String = if parts[2].contains("'") {
            get_full_quote(&parts, 2)
        } else {
            parts[2].to_string()
        };

        self.table = format!("{}_{}", name, self.num_node);
        println!("DELETE en tabla: {}", self.table);

        if let Some(where_index) = parts.iter().position(|&x| x == "WHERE") {
            let (_, right) = parts.split_at(where_index);
            let condition_query = right[1..].join(" ");
            let where_clause = WhereClause::new(condition_query)?;
            self.whereclause = where_clause;
        }
        Ok(())
    }

    // Ejecuta la consulta
    pub fn execute_command(
        &mut self,
        path_table: Arc<RwLock<String>>,
    ) -> Result<Vec<String>, MyError> {
        let mut rdr = get_reader_read(&path_table)?;
        let col_names = get_file_headers(&mut rdr)?;
        let mut file_open = get_file_to_write(&path_table)?;

        // Vacia archivo completo si no hay condicion
        if self.whereclause == WhereClause::True {
            append_row(&mut file_open, &col_names)?;
            return Ok(vec!["Se elimino la tabla completa".to_string()]);
        }

        let (mut temp_file, temp_file_name) = create_temporary_file(TABLE_PATH, &self.table)?;
        append_row_with_file(&mut temp_file, &col_names)?;

        let filter = Filter::new(col_names.clone(), self.whereclause.clone())?;

        for line in rdr.lines() {
            let line = line.map_err(|e| MyError::FileError(e.to_string()))?;
            let row: Vec<String> = line
                .split(",")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            if !filter.filter(&row) {
                append_row_with_file(&mut temp_file, &row)?;
            }
        }

        write_temporary_to_original(TABLE_PATH, &temp_file_name, file_open)?;
        delete_file(&temp_file_name, TABLE_PATH)?;
        Ok(vec!["Se eliminaron las filas".to_string()])
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

// DELETE FROM origin ----> POR AHORA NO BORRA NADA
// DELETE FROM origin WHERE airport = MEX AND date = 24-12-24 AND destination = AEP
// SELECT * FROM origin WHERE airport = MEX AND date = 24-12-24 AND destination = AEP

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_delte_valid_query() {
        let mut delete = Delete::new();
        let query = "DELETE FROM 'users' WHERE age > 30";
        let result = delete.parse_delete(query, "node_1");

        assert!(result.is_ok());
        assert_eq!(delete.table, "users_1".to_string());
        assert_eq!(delete.num_node, "1".to_string());
    }

    #[test]
    fn test_parse_select_invalid_query() {
        let mut d1 = Delete::new();
        let q1 = "DELETE origin WHERE age > 30";
        let r1 = d1.parse_delete(q1, "node_1");
        assert!(matches!(r1, Err(MyError::InvalidSyntax(_))));

        let mut d2 = Delete::new();
        let q2 = "DELETE origin FROM WHERE age > 30";
        let r2 = d2.parse_delete(q2, "node_1");

        assert!(matches!(r2, Err(MyError::InvalidSyntax(_))));
    }
}
