use std::sync::{Arc, RwLock};

use crate::cql::create_table::CreateTable;
use crate::cql::delete::Delete;
use crate::cql::insert::Insert;
use crate::cql::select::Select;
use crate::cql::update::Update;
use crate::errors::my_errors::MyError;

#[derive(Debug, Eq, PartialEq)]
/// Enum de los tipos de queries.
pub enum QType {
    None,
    Select(Select),
    Update(Update),
    Insert(Insert),
    Delete(Delete),
    CreateTable(CreateTable),
}

#[derive(Debug)]
/// Estructura que representa una consulta CQL.
pub struct Query {
    pub query: String, //SELECT * FROM table WHERE id = 1
    pub sender_node: String,
    pub sender_node_ip: String,
    pub qtype: QType,
}

impl Query {
    pub fn new(query: String, sender_node: String, sender_node_ip: String) -> Query {
        Query {
            query,
            sender_node,
            sender_node_ip,
            qtype: QType::None,
        }
    }

    /// Se ejecuta la query sobre la tabla indicada.
    pub fn execute(&mut self, table: Arc<RwLock<String>>) -> Result<Vec<String>, MyError> {
        self.execute_command(table)
    }

    fn get_type(&mut self) {
        let q_upper = self.query.to_ascii_uppercase();

        if q_upper.contains("SELECT") {
            self.set_select();
        } else if q_upper.contains("UPDATE") {
            self.set_update();
        } else if q_upper.contains("INSERT") {
            self.set_insert();
        } else if q_upper.contains("DELETE") {
            self.set_delete();
        } else if q_upper.contains("CREATE") {
            self.set_createtable();
        } else {
            self.set_none();
        }
    }

    pub fn get_values(&mut self) -> String {
        self.get_type();

        match &mut self.qtype {
            QType::Select(select) => {
                select.get_key();
                "SELECT".to_string()
            }
            QType::Insert(insert) => insert.get_values(),
            _ => "Error".to_string(),
        }
    }

    pub fn set_none(&mut self) {
        self.qtype = QType::None;
    }

    pub fn set_select(&mut self) {
        self.qtype = QType::Select(Select::new());
    }

    pub fn set_update(&mut self) {
        self.qtype = QType::Update(Update::new());
    }

    pub fn set_insert(&mut self) {
        self.qtype = QType::Insert(Insert::new());
    }

    pub fn set_delete(&mut self) {
        self.qtype = QType::Delete(Delete::new());
    }

    pub fn set_createtable(&mut self) {
        self.qtype = QType::CreateTable(CreateTable::new());
    }

    /// Delegamos el paseo de la query a cada tipo de query.
    pub fn parse_query(&mut self) -> Result<(), MyError> {
        self.get_type();

        match &mut self.qtype {
            QType::None => Err(MyError::Error("No query to parse".to_string())),
            QType::Select(select) => select.parse_select(&self.query, &self.sender_node_ip),
            QType::Update(update) => update.parse_update(&self.query, &self.sender_node_ip),
            QType::Insert(insert) => insert.parse_insert(&self.query, &self.sender_node_ip),
            QType::Delete(delete) => delete.parse_delete(&self.query, &self.sender_node_ip),
            QType::CreateTable(create_table) => {
                create_table.parse_create_table(&self.query, &self.sender_node)
            }
        }
    }

    /// Se delega la ejecición de la query a cada tipo de estructura de query.
    pub fn execute_command(&mut self, table: Arc<RwLock<String>>) -> Result<Vec<String>, MyError> {
        // println!("\n - Query: {:?} \n", self.qtype);
        match &mut self.qtype {
            QType::None => Err(MyError::Error("No query to execute".to_string())),
            QType::Select(select) => select.execute_command(table),
            QType::Update(update) => update.execute_command(table),
            QType::Insert(insert) => insert.execute_command(table),
            QType::Delete(delete) => delete.execute_command(table),
            QType::CreateTable(create_table) => create_table.execute_command(),
        }
    }

    /// Debe devolver de la query los valores de los campos de las columnas que pertenecen a la partición de esa tabla.
    pub fn get_key_representing_partition(&mut self, address_node: &str) -> String {
        //devolveme el nombre del aeropuerto
        self.get_type();
        match &mut self.qtype {
            QType::Select(select) => {
                let _ = select.parse_select(self.query.as_str(), address_node);
                select.get_key()
            }
            QType::Insert(insert) => {
                let _ = insert.parse_insert(self.query.as_str(), address_node);
                insert.get_key()
            }
            QType::Delete(delete) => {
                let _ = delete.parse_delete(self.query.as_str(), address_node);
                delete.get_key()
            }
            QType::Update(update) => {
                let _ = update.parse_update(self.query.as_str(), address_node);
                update.get_key()
            }
            _ => "".to_string(),
        }
    }

    /// Devuelve la tabla en la que se aplica la query.
    pub fn get_table(&self) -> String {
        let table: String = match &self.qtype {
            QType::None => "".to_string(),
            QType::Select(select) => select.table.clone(),
            QType::Update(update) => update.table.clone(),
            QType::Insert(insert) => insert.table.clone(),
            QType::Delete(delete) => delete.table.clone(),
            QType::CreateTable(create_table) => create_table.table_name.clone(),
        };
        table.trim().to_string()
    }
}
