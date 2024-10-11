#[derive(Debug, Clone)]
pub struct Query {
    pub query: String,
    pub sender_node_ip: String, 
    // pub client_ip: String, // de alguna forma hay que conseguir el IP del cliente que mando la request
}

impl Query {
    pub fn new(query: String, sender_node_ip: String) -> Query {
        Query { query, sender_node_ip }
    }
}