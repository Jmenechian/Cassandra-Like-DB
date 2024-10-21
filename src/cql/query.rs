use crate::cql::delete::Delete;
use crate::cql::insert::Insert;
use crate::cql::select::Select;
use crate::cql::update::Update;
use std::error::Error;

#[derive(Debug, Eq, PartialEq)]
pub enum QType {
    None,
    Select(Select),
    Update(Update),
    Insert(Insert),
    Delete(Delete),
}

#[derive(Debug)]
pub struct Query {
    pub query: String, //SELECT * FROM table WHERE id = 1
    pub sender_node_ip: String,
    pub qtype: QType,
    // pub client_ip: String, // de alguna forma hay que conseguir el IP del cliente que mando la request
}

impl Query {
    pub fn new(query: String, sender_node_ip: String) -> Query {
        Query {
            query,
            sender_node_ip,
            qtype: QType::None,
        }
    }

    pub fn execute(&mut self, table: &str) -> String {
        println!("En execute Query");
        self.get_type();
        let _ = self.parse_query();
        match self.execute_command(table) {
            Ok(result) => result,
            Err(e) => e.to_string(),
        }
    }

    fn get_type(&mut self) {
        let q_upper = self.query.to_ascii_uppercase();
        println!("query: {:?}", self.query);
        println!("q_upper: {:?}", q_upper);
        if q_upper.contains("SELECT") {
            self.set_select();
        } else if q_upper.contains("UPDATE") {
            self.set_update();
        } else if q_upper.contains("INSERT") {
            self.set_insert();
        } else {
            self.set_delete();
        }
    }

    pub fn get_values(&mut self) -> String {
        self.get_type();

        match &mut self.qtype {
            QType::Select(select) => { //nada
                select.get_key(); //sacar -> esta para clippy
                "SELECT".to_string()
            }
            QType::Insert(insert) => {
                insert.get_values()
            }
            _ => "Error".to_string(),
        }
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

    // Delegates the parsing
    pub fn parse_query(&mut self) -> Result<(), Box<dyn Error>> {
        match &mut self.qtype {
            QType::None => Err(Box::from("No query to parse".to_string())),
            QType::Select(select) => select.parse_select(&self.query),
            // QType::Update(update) => update.parse_update(query)?,
            QType::Insert(insert) => insert.parse_insert(&self.query),
            // QType::Delete(delete) => delete.parse_delete(query)?,
            _ => Ok(()),
        }
    }

    // Delegates the command execution
    pub fn execute_command(&mut self, path: &str) -> Result<String, Box<dyn Error>> {
        println!("En execute_command GRAL: {:?}", self.qtype);
        match &mut self.qtype {
            QType::None => Err(Box::from("No query to execute".to_string())),
            QType::Select(select) => select.execute_command(path),
            // QType::Update(update) => update.execute_command(path)?,
            QType::Insert(insert) => insert.execute_command(path),
            // QType::Delete(delete) => delete.execute_command(path)?,
            _ => Ok("AAAAAAAAAAAAAAAA".to_string()),
        }
    }

    // Debe devolver de la query los valores de los campos de las columnas que pertenecen a la partición de esa tabla.
    pub fn get_key_representing_partition(&mut self) -> String {
        //devolveme el nombre del aeropuerto
        // self.query.clone()
        self.get_type();
        println!("type: {:?}", self.qtype);
        match &mut self.qtype {
            QType::Select(select) => {
                let _ = select.parse_select(self.query.as_str());
                select.get_key()
            }
            QType::Insert(insert) => {
                let _ = insert.parse_insert(self.query.as_str());
                insert.get_key()
            }
            _ => "BBBBBBBBBBBB".to_string(),
        }
    }

    
}
//INSERT: Suponiendo que no se puede insertar filas incompletas:
//te traers self.values[0] y self.values[1]

//SELECT: aeropuerto = ...
// date = ...
//pueden estar por separado
//pueden no estar ninguno en el where?
