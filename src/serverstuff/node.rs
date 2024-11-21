use crate::config::{
    TABLE_DESTINATION_NAME, TABLE_FLIGHT_SIMULATOR, TABLE_ORIGIN_NAME, TABLE_PATH,
};
use crate::cql::query::Query;
use crate::errors::my_errors::*;
use crate::serverstuff::neighbor_node::NeighborNode;

use rand::seq::SliceRandom;
use std::sync::{Arc, RwLock};

/// Estructura que eepresenta al nodo en el que estamos parados.
#[derive(Debug, Clone)]
pub struct Node {
    pub number: String,
    pub server_ip: String, // IP donde nodo se pone a escuchar conecciones de servidores
    pub client_ip: String, // IP donde nodo se pone a escuchar conecciones de cliente
    pub gossip_ip: String, // IP donde nodo se pone a escuchar mensajes Gossip
    pub neighbors: Vec<NeighborNode>, // todos los otros nodos
    pub path_table_origin: Arc<RwLock<String>>, // path donde se encuntra el archivo que representan la tabla
    pub path_table_destination: Arc<RwLock<String>>,
    pub path_table_simulator: Arc<RwLock<String>>,
}

impl Node {
    /// Se crea el nodo.
    pub fn new(number: &str) -> Self {
        Self {
            number: number.to_string(),
            server_ip: format!("127.0.0.1:700{}", number),
            client_ip: format!("127.0.0.1:800{}", number),
            gossip_ip: format!("127.0.0.1:900{}", number),
            neighbors: vec![], // todos los nodos que estan en el ring
            path_table_origin: Self::lock_path_table(number, "origin"),
            path_table_destination: Self::lock_path_table(number, "destination"),
            path_table_simulator: Self::lock_path_table(number, "simulator"),
        }
    }

    fn lock_path_table(number_node: &str, table: &str) -> Arc<RwLock<String>> {
        let path = Self::get_table_path(number_node, table);
        Arc::new(RwLock::new(path))
    }

    fn get_table_path(number_node: &str, table: &str) -> String {
        if table == "origin" {
            format!("{}/{}_{}.csv", TABLE_PATH, TABLE_ORIGIN_NAME, number_node)
        } else if table == "destination" {
            format!(
                "{}/{}_{}.csv",
                TABLE_PATH, TABLE_DESTINATION_NAME, number_node
            )
        } else {
            format!(
                "{}/{}_{}.csv",
                TABLE_PATH, TABLE_FLIGHT_SIMULATOR, number_node
            )
        }
    }

    /// Se agregan los nodos del anillo de nodos.
    pub fn add_neighbors(&mut self, neighbor_nodes: Vec<NeighborNode>) {
        self.neighbors = neighbor_nodes;
    }

    /// Se agrega un nodo en particular a los ya existentes.
    pub fn add_neighbor(&mut self, neighbor_node: NeighborNode) -> bool {
        if let Some(pos) = self
            .neighbors
            .iter()
            .position(|n| n.puerto == neighbor_node.puerto)
        {
            if self.neighbors[pos].time < neighbor_node.time {
                self.neighbors[pos] = neighbor_node;
                false
            } else {
                false
            }
        } else {
            self.neighbors.push(neighbor_node);
            true
        }
    }

    pub fn get_random_neighbor(&self, my_port: String) -> Option<&NeighborNode> {
        let mut rng = rand::thread_rng();

        let filtered: Vec<&NeighborNode> = self
            .neighbors
            .iter()
            .filter(|n| n.puerto != my_port)
            .collect();
        filtered.choose(&mut rng).copied()
    }

    /// Proceso la query y devuelve la tabla en formato String.
    pub fn process_query(&self, query: &mut Query) -> Result<Vec<String>, MyError> {
        query.parse_query()?;

        let path_table: Arc<RwLock<String>>;

        if [
            TABLE_ORIGIN_NAME.to_string(),
            format!("{}_{}", TABLE_ORIGIN_NAME, self.number),
        ]
        .contains(&query.get_table())
        {
            path_table = self.path_table_origin.clone();
        } else if [
            TABLE_DESTINATION_NAME.to_string(),
            format!("{}_{}", TABLE_DESTINATION_NAME, self.number),
        ]
        .contains(&query.get_table())
        {
            path_table = self.path_table_destination.clone();
        } else if [
            TABLE_FLIGHT_SIMULATOR.to_string(),
            format!("{}_{}", TABLE_FLIGHT_SIMULATOR, self.number),
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
