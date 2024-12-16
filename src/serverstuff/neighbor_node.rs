// Representa a cada uno de los otros nodos del ring
// El nodo actual (tipo Nodo) tiene una lista de estos nodos vecinos (tipo NeighborNode)

use crate::{serverstuff::node_status::Status, utils::date_is_not_recent};
use chrono::{DateTime, Local, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct NeighborNode {
    pub server_ip: String, // uso su ip para saber a quien mandarle la query --> Consistent Hashing
    pub gossip_ip: String,
    pub number: String,
    pub status: Status,
    pub time: DateTime<Local>,
    pub last_connected: DateTime<Utc>,
}

impl NeighborNode {
    pub fn new(port: &str) -> Self {
        Self {
            server_ip: format!("127.0.0.1:700{}", port),
            gossip_ip: format!("127.0.0.1:900{}", port),
            number: port.to_string(),
            status: Status::Connected, // Deberia empezar desconectado?
            time: Local::now(),
            last_connected: Utc::now()
        }
    }

    pub fn connect_node(&mut self) {
        self.last_connected = Utc::now();
        self.status = Status::Connected;
    }

    pub fn disconnect_node(&mut self) {
        self.status = Status::Disconnected;
        if !date_is_not_recent(self.last_connected) {
            self.disengaged_node();
        }
    }

    pub fn disengaged_node(&mut self) {
        self.status = Status::Disengaged;
    }
}
