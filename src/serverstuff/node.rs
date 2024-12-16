use crate::config::{
    TABLE_DESTINATION_NAME, TABLE_FLIGHT_SIMULATOR, TABLE_ORIGIN_NAME, TABLE_PATH,
};
use crate::cql::query::Query;
use crate::errors::my_errors::*;
use crate::loggerstuff::logger::Logger;
use crate::serverstuff::neighbor_node::NeighborNode;
use crate::serverstuff::node_status::Status;
use crate::utils::get_rustic_airline_path_tables;

use chrono::{DateTime, Utc};
use rand::seq::SliceRandom;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Estructura que eepresenta al nodo en el que estamos parados.
#[derive(Debug, Clone)]
pub struct Node {
    pub ip: IpAddr,
    pub server_socket: SocketAddr, // IP donde nodo se pone a escuchar conexiones de servidores
    pub client_socket: SocketAddr, // IP donde nodo se pone a escuchar conexiones de cliente
    pub gossip_socket: SocketAddr, // IP donde nodo se pone a escuchar mensajes Gossip
    pub neighbors: Vec<NeighborNode>, // todos los otros nodos
    pub path_table_origin: Arc<RwLock<String>>, // path donde se encuntra el archivo que representan la tabla
    pub path_table_destination: Arc<RwLock<String>>,
    pub path_table_simulator: Arc<RwLock<String>>,
    pub logger: Arc<RwLock<Logger>>, // logger para guardar los logs
    pub connection_date: DateTime<Utc>,
}

impl Node {
    /// Obtenemos los paths de las tablas que se crearon a traves de la query CREATED.
    pub fn tables_created(&self) -> Vec<String> {
        let paths = get_rustic_airline_path_tables(self.ip.to_string());
        let mut tables = vec![];
        for path in paths {
            if Path::new(&path).exists() {
                tables.push(path);
            }
        }
        tables
    }
    /// Se crea el nodo.
    pub fn new(ip: IpAddr) -> Result<Self, MyError> {
        let logger = match Logger::new(ip.to_string()) {
            Ok(logger) => Arc::new(RwLock::new(logger)),
            Err(_) => return Err(MyError::LoggerError("Error al crear logger".to_string())),
        };
        Ok(Self {
            ip,
            server_socket: SocketAddr::new(ip, 7000_u16),
            client_socket: SocketAddr::new(ip, 8000_u16),
            gossip_socket: SocketAddr::new(ip, 9000_u16),
            neighbors: vec![], // todos los nodos que estan en el ring
            path_table_origin: Self::lock_path_table(&ip.to_string(), "origin"),
            path_table_destination: Self::lock_path_table(&ip.to_string(), "destination"),
            path_table_simulator: Self::lock_path_table(&ip.to_string(), "simulator"),
            connection_date: Utc::now(),
            logger,
        })
    }

    fn lock_path_table(ip: &str, table: &str) -> Arc<RwLock<String>> {
        let path = Self::get_table_path(ip, table);
        Arc::new(RwLock::new(path))
    }

    fn get_table_path(ip: &str, table: &str) -> String {
        if table == "origin" {
            format!("{}/{}_{}.csv", TABLE_PATH, TABLE_ORIGIN_NAME, ip)
        } else if table == "destination" {
            format!("{}/{}_{}.csv", TABLE_PATH, TABLE_DESTINATION_NAME, ip)
        } else {
            format!("{}/{}_{}.csv", TABLE_PATH, TABLE_FLIGHT_SIMULATOR, ip)
        }
    }

    /// Se agregan los nodos del anillo de nodos.
    pub fn add_neighbors(&mut self, neighbor_nodes: Vec<NeighborNode>) {
        self.neighbors = neighbor_nodes;
    }

    /// Se agrega un nodo en particular a los ya existentes.
    pub fn add_neighbor(&mut self, new_neighbor_node: NeighborNode) -> u8 {
        if let Some(pos) = self
            .neighbors
            .iter()
            .position(|n| n.ip == new_neighbor_node.ip)
        {
            if self.neighbors[pos].status != new_neighbor_node.status {
                //ya existias
                if self.neighbors[pos].status != Status::Disengaged
                    || new_neighbor_node.status != Status::Disconnected
                {
                    self.neighbors[pos] = new_neighbor_node.clone();
                    if self.neighbors[pos].status == Status::Disengaged
                        && new_neighbor_node.status == Status::Connected
                    {
                        self.neighbors[pos] = new_neighbor_node;
                        2
                    } else {
                        1
                    }
                } else {
                    0
                }
            } else if new_neighbor_node.status == Status::Disconnected {
                self.neighbors[pos].disconnect_node();
                0
            } else {
                0
            }
        } else {
            self.neighbors.push(new_neighbor_node); //nuevo
            2
        }
    }

    pub fn get_random_neighbor(&self) -> Option<&NeighborNode> {
        let mut rng = rand::thread_rng();

        let filtered: Vec<&NeighborNode> = self
            .neighbors
            .iter()
            .filter(|n| n.ip != self.ip && n.status != Status::Disengaged)
            .collect();
        filtered.choose(&mut rng).copied()
    }

    pub fn disconnect_neighbor(&mut self, ip: IpAddr) {
        if let Some(n) = self.neighbors.iter_mut().find(|n| n.ip == ip) {
            n.disconnect_node();
        }
    }

    pub fn connect_neighbor(&mut self, ip: IpAddr) {
        if let Some(n) = self.neighbors.iter_mut().find(|n| n.ip == ip) {
            n.connect_node();
        }
    }

    /// Proceso la query y devuelve la tabla en formato String.
    pub fn process_query(&self, query: &mut Query) -> Result<Vec<String>, MyError> {
        query.parse_query()?;
        let path_table: Arc<RwLock<String>>;

        if [
            TABLE_ORIGIN_NAME.to_string(),
            format!("{}_{}", TABLE_ORIGIN_NAME, self.ip),
        ]
        .contains(&query.get_table())
        {
            path_table = self.path_table_origin.clone();
        } else if [
            TABLE_DESTINATION_NAME.to_string(),
            format!("{}_{}", TABLE_DESTINATION_NAME, self.ip),
        ]
        .contains(&query.get_table())
        {
            path_table = self.path_table_destination.clone();
        } else if [
            TABLE_FLIGHT_SIMULATOR.to_string(),
            format!("{}_{}", TABLE_FLIGHT_SIMULATOR, self.ip),
        ]
        .contains(&query.get_table())
        {
            path_table = self.path_table_simulator.clone();
        } else {
            return Err(MyError::InvalidTable("Tabla no valida".to_string()));
        }
        query.execute(path_table)
    }
}
