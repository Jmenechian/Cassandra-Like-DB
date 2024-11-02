// Representa las consultas del tipo CREATE TABLE
// Aca se parsea la consulta y se ejecuta
// Asumimos :
//          - Palabras claves CQL en mayusculas
//          - No se pueden crear tablas con nombres que contengan espacios
//          - No se pueden crear tablas con nombres de columnas que contengan espacios
//          - PRIMARY KEY se setea al final, despues de todas las columnas
//          - Nodo que habla con cliente crea las 8 tablas, hay un solo archivo de metadata
//          - Tira error si tabla ya existe
//
// Ejemplo:
// CREATE TABLE application_logs (
//     id                    INT,
//     app_name              VARCHAR,
//     hostname              VARCHAR,
//     log_datetime          TIMESTAMP,
//     env                   VARCHAR,
//     log_level             VARCHAR,
//     log_message           TEXT,
//     PRIMARY KEY ((app_name,env), hostname, log_datetime)
//   )
//   WITH CLUSTERING ORDER BY (hostname ASC, log_datetime DESC);

use crate::config::{RING_SIZE, TABLE_PATH};
use crate::my_errors::MyError;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

#[derive(Debug, PartialEq, Eq)]
pub struct CreateTable {
    pub if_not_exists: bool, // si contiene [IF NOT EXISTS] en la query
    pub table_name: String,
    pub column_type: HashMap<String, String>, // nombre de columna, tipo de columna
    pub columns: Vec<String>,
    pub partition_keys: Vec<String>, // {"partition": ["app_name", "env"], "clustering": ["hostname", "log_datetime"]}
    pub clustering_keys: Vec<String>,
    pub clustering_orderby: HashMap<String, String>,
}

impl Default for CreateTable {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateTable {
    pub fn new() -> CreateTable {
        CreateTable {
            if_not_exists: false,
            table_name: "".to_string(),
            column_type: HashMap::new(),
            columns: Vec::new(),
            partition_keys: Vec::new(),
            clustering_keys: Vec::new(),
            clustering_orderby: HashMap::new(),
        }
    }

    pub fn parse_create_table(&mut self, query: &str) -> Result<(), MyError> {
        let parts: Vec<&str> = query.split_whitespace().collect();
        if parts.len() < 7 || parts[0] != "CREATE" || parts[1] != "TABLE" {
            return Err(MyError::InvalidSyntax(
                "Create table query not valid".to_string(),
            ));
        }

        self.if_not_exists = query.contains("IF NOT EXISTS");

        // por ahora no acepta nombre de tablas con espacios
        // por ahora no acepta nombres de columnas con espacios
        if self.if_not_exists {
            self.table_name = parts[5].to_string().replace("'", "");
            self.column_type = self.set_columns(parts[6..].join(" "));
        } else {
            self.table_name = parts[2].to_string().replace("'", "");
            self.column_type = self.set_columns(parts[3..].join(" "));
        }

        // PRIMARY KEY
        if let Some(pkey_index) = parts.iter().position(|&x| x == "KEY") {
            (self.partition_keys, self.clustering_keys) =
                self.set_primary_key(parts[pkey_index + 1..].join(" "));
        }

        // WITH CLUSTERING ORDER BY
        if let Some(pkey_index) = parts.iter().position(|&x| x == "BY") {
            self.clustering_orderby =
                self.set_cluster_orderby(parts[pkey_index + 1..].join(" "))?;
        }

        Ok(())
    }

    fn set_columns(&mut self, columns_str: String) -> HashMap<String, String> {
        let parts = columns_str.split_whitespace().collect::<Vec<&str>>();

        let mut column_type: HashMap<String, String> = HashMap::new();

        for chunk in parts.chunks(2) {
            if chunk[0].contains("PRIMARY") || chunk[1].contains(")") {
                break;
            }

            let col_name = chunk[0].trim().replace("(", "");
            let col_type = chunk[1].trim().replace(")", "").replace(",", "");

            // Guardo nombre columna y su tipo
            self.columns.push(col_name.to_string());
            column_type.insert(col_name, col_type);
        }
        column_type
    }

    fn set_primary_key(&self, primary_key_str: String) -> (Vec<String>, Vec<String>) {
        //  WITH CLUSTERING ORDER BY
        let mut parts: Vec<&str> = primary_key_str.split_whitespace().collect();
        if let Some(ob_index) = parts.iter().position(|&x| x.contains("WITH")) {
            parts = parts[..ob_index].to_vec();
        }

        let cleaned_str = parts.join(" ");
        parts = cleaned_str.split(",").collect();

        let mut partition_keys: Vec<String> = Vec::new();
        let mut clustering_keys: Vec<String> = Vec::new();

        if parts[0].starts_with("((") {
            //estamos en composite
            let mut in_partition = true;

            for part in parts {
                println!("part: {}", part);
                if in_partition {
                    let key = part
                        .trim()
                        .replace("(", "")
                        .replace(")", "")
                        .replace(",", "");
                    partition_keys.push(key);
                    if part.ends_with(")") {
                        in_partition = false;
                        continue;
                    }
                } else {
                    let key = part
                        .trim()
                        .replace("(", "")
                        .replace(")", "")
                        .replace(",", "");
                    clustering_keys.push(key);
                }
                if part.contains("))") {
                    break;
                }
            }
        } else {
            //estamos en simple
            let keys = parts.join(",").replace("(", "").replace(")", "");
            partition_keys = keys.split(",").map(|x| x.trim().to_string()).collect();
        }
        (partition_keys, clustering_keys)
    }

    fn set_cluster_orderby(
        &self,
        clustring_str: String,
    ) -> Result<HashMap<String, String>, MyError> {
        let mut orderby: HashMap<String, String> = HashMap::new();

        let parts: Vec<&str> = clustring_str.split(",").collect();

        for part in parts {
            let part = part
                .trim()
                .replace("(", "")
                .replace(")", "")
                .replace(",", "");
            let p: Vec<&str> = part.split_whitespace().collect();

            if p.len() != 2 {
                return Err(MyError::InvalidSyntax(
                    "Clustering order by query not valid".to_string(),
                ));
            }

            let (key, val) = (p[0].to_string(), p[1].to_string());
            orderby.insert(key, val);
        }
        Ok(orderby)
    }

    fn get_display_columns(&self) -> String {
        let mut format = String::new();
        for col in &self.columns {
            format.push_str(&format!("{} {},", col, self.column_type.get(col).unwrap()));
        }
        format
    }

    // Se ejecuta la query
    pub fn execute_command(&mut self) -> Result<Vec<String>, MyError> {
        for i in 1..=RING_SIZE {
            // Creo nueva tabla por nodo
            let path = format!("{}/{}_{}.csv", TABLE_PATH, self.table_name, i);
            let mut f = File::create_new(path).map_err(|e| MyError::FileError(e.to_string()))?;

            println!("columns:{:?}", self.columns);
            f.write_all(self.columns.join(",").as_bytes())
                .map_err(|e| MyError::FileError(e.to_string()))?;
            writeln!(f,"").map_err(|e| MyError::FileError(e.to_string()))?;
        }

        // Creo un solo archivo de metadata
        let path_metadata = format!("{}/{}_{}.csv", TABLE_PATH, self.table_name, "metadata");
        let mut f_metadata =
            File::create_new(path_metadata).map_err(|e| MyError::FileError(e.to_string()))?;

        let display_orderby: String = self
            .clustering_orderby
            .iter()
            .map(|(k, v)| format!("{} {},", k, v))
            .collect::<Vec<String>>()
            .join(",");

        let display_columns = self.get_display_columns();

        f_metadata
            .write_all(
                format!(
                    "partition: {}\nclusters: {}\nclustering_orderby: {}\ncolumns: {}",
                    self.partition_keys.join(","),
                    self.clustering_keys.join(","),
                    display_orderby,
                    display_columns
                )
                .as_bytes(),
            )
            .map_err(|e| MyError::FileError(e.to_string()))?;

        Ok(vec!["tabla creada".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parse_table_name() {
        let mut create_table = CreateTable::new();
        let query = "CREATE TABLE application_logs (id INT, PRIMARY KEY (id)) WITH CLUSTERING ORDER BY (id);".to_string();
        let _ = create_table.parse_create_table(&query);
        assert_eq!(create_table.table_name, "application_logs".to_string());
    }

    #[test]
    pub fn test_set_columns() {
        let mut create_table = CreateTable::new();
        let columns_str =
            "(id INT, app_name VARCHAR, hostname VARCHAR, PRIMARY KEY (id))".to_string();

        let mut expected_map = HashMap::new();
        expected_map.insert("id".to_string(), "INT".to_string());
        expected_map.insert("app_name".to_string(), "VARCHAR".to_string());
        expected_map.insert("hostname".to_string(), "VARCHAR".to_string());

        let columns = create_table.set_columns(columns_str);
        assert_eq!(columns, expected_map);
    }

    #[test]
    pub fn test_set_primary_key_simple() {
        let create_table = CreateTable::new();
        let primary_key_str = "(id)) WITH CLUSTERING ORDER BY (hostname ASC) ".to_string();
        let expected_partition_keys = vec!["id".to_string()];
        let expected_clustering_keys: Vec<String> = vec![];

        let (partition_keys, clustering_keys) = create_table.set_primary_key(primary_key_str);
        assert_eq!(partition_keys, expected_partition_keys);
        assert_eq!(clustering_keys, expected_clustering_keys);
    }

    #[test]
    pub fn test_set_primary_key_composite() {
        let create_table = CreateTable::new();
        let primary_key_str = "((app_name,env), hostname, log_datetime))".to_string();
        let expected_partition_keys: Vec<String> = vec!["app_name".to_string(), "env".to_string()];
        let expected_clustering_keys: Vec<String> =
            vec!["hostname".to_string(), "log_datetime".to_string()];

        let (partition_keys, clustering_keys) = create_table.set_primary_key(primary_key_str);
        assert_eq!(partition_keys, expected_partition_keys);
        assert_eq!(clustering_keys, expected_clustering_keys);
    }

    #[test]
    pub fn test_set_cluster_orderby() {
        let create_table = CreateTable::new();
        let clustering_str = "(id ASC, name DESC)".to_string();
        let mut expected_map = HashMap::new();
        expected_map.insert("id".to_string(), "ASC".to_string());
        expected_map.insert("name".to_string(), "DESC".to_string());

        match create_table.set_cluster_orderby(clustering_str) {
            Ok(orderby) => assert_eq!(orderby, expected_map),
            Err(_) => assert!(false),
        }
    }

    #[test]
    pub fn parse_create_table() {
        let mut create_table = CreateTable::new();
        let query = "CREATE TABLE application_logs (id INT, app_name VARCHAR, hostname VARCHAR, log_datetime TIMESTAMP, PRIMARY KEY ((app_name,env), hostname, log_datetime)) WITH CLUSTERING ORDER BY (hostname ASC, log_datetime DESC)";
        let _ = create_table.parse_create_table(query);

        let mut expected_columns = HashMap::new();
        expected_columns.insert("id".to_string(), "INT".to_string());
        expected_columns.insert("app_name".to_string(), "VARCHAR".to_string());
        expected_columns.insert("hostname".to_string(), "VARCHAR".to_string());
        expected_columns.insert("log_datetime".to_string(), "TIMESTAMP".to_string());

        let expected_partition_keys = vec!["app_name".to_string(), "env".to_string()];
        let expected_clustering_keys = vec!["hostname".to_string(), "log_datetime".to_string()];

        let mut expected_orderby = HashMap::new();
        expected_orderby.insert("hostname".to_string(), "ASC".to_string());
        expected_orderby.insert("log_datetime".to_string(), "DESC".to_string());

        assert_eq!(create_table.table_name, "application_logs".to_string());
        assert_eq!(create_table.if_not_exists, false);
        assert_eq!(create_table.column_type, expected_columns);
        assert_eq!(
            create_table.columns,
            vec![
                "id".to_string(),
                "app_name".to_string(),
                "hostname".to_string(),
                "log_datetime".to_string()
            ]
        );
        assert_eq!(create_table.partition_keys, expected_partition_keys);
        assert_eq!(create_table.clustering_keys, expected_clustering_keys);
        assert_eq!(create_table.clustering_orderby, expected_orderby);
    }
}

// CREATE TABLE application_logs (id INT, app_name VARCHAR, hostname VARCHAR, log_datetime TIMESTAMP, PRIMARY KEY ((app_name,env), hostname, log_datetime)) WITH CLUSTERING ORDER BY (hostname ASC, log_datetime DESC)
