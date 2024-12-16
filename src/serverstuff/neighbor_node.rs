// Representa a cada uno de los otros nodos del ring
// El nodo actual (tipo Nodo) tiene una lista de estos nodos vecinos (tipo NeighborNode)

use crate::{serverstuff::node_status::Status, utils::date_is_not_recent};
use chrono::{DateTime, Local, Utc};
use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Clone, PartialEq)]
pub struct NeighborNode {
    pub server_socket: SocketAddr, // uso su ip para saber a quien mandarle la query --> Consistent Hashing
    pub gossip_socket: SocketAddr,
    pub ip: IpAddr,
    pub status: Status,
    pub time: DateTime<Local>,
    pub last_connected: DateTime<Utc>,
}

impl NeighborNode {
    pub fn new(ip: IpAddr) -> Self {
        Self {
            server_socket: SocketAddr::new(ip, 7000 as u16),
            gossip_socket: SocketAddr::new(ip, 9000 as u16),
            ip: ip,
            status: Status::Connected,
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
