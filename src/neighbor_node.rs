// Representa a cada uno de los otros nodos del ring
// El nodo actual (tipo Nodo) tiene una lista de estos nodos vecinos (tipo NeighborNode)

use crate::nodo::Status;
use chrono::{DateTime, Local};

#[derive(Debug, Clone, PartialEq)]
pub struct NeighborNode {
    pub server_ip: String, // uso su ip para saber a quien mandarle la query --> Consistent Hashing
    pub status: Status,
    pub time: DateTime<Local>,
}

impl NeighborNode {
    pub fn new(port: &str) -> Self {
        Self {
            server_ip: format!("127.0.0.1:700{}", port),
            status: Status::Connected, // Deberia empezar desconectado -> cambiar con Gossip
            time: Local::now(),
        }
    }

    pub fn connect_node(&mut self) {
        self.status = Status::Connected;
    }

    pub fn disconnect_node(&mut self) {
        self.status = Status::Disconnected;
    }
}
