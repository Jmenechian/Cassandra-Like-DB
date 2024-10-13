#[derive(Debug, Clone)]
pub struct Query {
    pub query: String,
    pub sender_node_ip: String,
    // pub client_ip: String, // de alguna forma hay que conseguir el IP del cliente que mando la request
}

impl Query {
    pub fn new(query: String, sender_node_ip: String) -> Query {
        Query {
            query,
            sender_node_ip,
        }
    }

    // Debe devolver de la query los valores de los campos de las columnas que pertenecen a la partición de esa tabla.
    pub fn get_key_representing_partition(&self) -> String {
        self.query.clone()
    }
}
