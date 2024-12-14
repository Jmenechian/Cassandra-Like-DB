use crate::{
    config::{RF, TABLE_PATH},
    errors::my_errors::MyError,
    messagestuff::{
        message::ServerSendable,
        message_server::{queryservermessage::QueryServerMessage, serverpacket::Packet},
    },
    partitioning::{
        consistent_hashing::ConsistentHash,
        queries::{get_create_query, get_insert_query, get_partition_delete_query},
    },
    serverstuff::{neighbor_node::NeighborNode, node::Node},
    utils::{get_file_headers, get_reader},
    Query,
};

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    net::TcpStream,
    vec,
};

#[derive(Debug, Clone)]
/// Estructura encargada de agregar un nuevo nodo conectado a la red.
/// Se le asigna posiciones a sus vnodes en el ring.
/// Se le pasa la informacion de las tablas que le corresponde y se borra de los nodos que ya no deberían tenerla.
pub struct NewNodeAgregator {
    new_node: NeighborNode,
    node: Node,
    hashing: ConsistentHash<NeighborNode>,
    tables: Vec<String>,
}

impl NewNodeAgregator {
    /// Se agrega el nodo al ring.
    pub fn new(new_node: NeighborNode, node: Node) -> Self {
        let mut hashing = ConsistentHash::new(RF);
        hashing.add_nodes(node.neighbors.to_vec());
        let tables = vec![
            "origin".to_string(),
            "destination".to_string(),
            "flights_in_progress".to_string(),
        ];
        Self {
            new_node,
            node,
            hashing,
            tables,
        }
    }

    /// Se indica al nuevo nodo que cree las tablas que le faltan.
    /// El nodo que identidicó que se conecto un nuevo nodo, borra de sus tablas las lineas que le corresponden
    /// al nuevo nodo, y le pasa a este los INSERTs que debe hacer para tener la info que le corresponde.
    pub fn new_node_in_ring(&mut self) {
        let mut lines_to_insert: HashMap<String, Vec<Vec<String>>> = HashMap::new();

        // Vemos las posiciones de los vnodes del new_node:
        let tokens = self.hashing.token_map.get_tokens(&self.new_node.server_ip);

        // Vemos vnodes siguientes a estas posiciones:
        let keys = self.hashing.get_keys_in_order();
        for position_new_node in tokens {
            let next_pos = position_new_node + 1;
            let next_node_idx = self
                .hashing
                .find_closest_greater_index(&keys, &next_pos)
                .unwrap_or(0);
            if let Some(next_node) = self.hashing.ring.get(&next_node_idx) {
                // Si soy ese nodo le mando INSERT de las lineas que le correspondes (de las 3 tablas que usamos):
                if next_node.server_ip == self.node.server_ip {
                    // Veo si una linea le corresponde viendo los hash de aeroport,date:
                    let hashmap_lines = self.get_lines_to_insert();
                    for table in hashmap_lines.keys() {
                        if let Some(new_lines) = hashmap_lines.get(table) {
                            // Usamos entry() para modificar el valor del HashMap:
                            lines_to_insert
                                .entry(table.to_string())
                                .or_default() // Si la clave no existe, se inicializa con un vector vacío
                                .extend(new_lines.clone()); // Concatenamos el nuevo vector
                        }
                    }
                }
            }
        }

        self.send_info(lines_to_insert);
    }

    fn get_lines_to_insert(&self) -> HashMap<String, Vec<Vec<String>>> {
        let mut lines_to_insert: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        for table in self.tables.clone() {
            if let Ok(lines) = self.get_all_lines(&table) {
                if let Ok(lines) = self.filter_lines(lines, &table) {
                    lines_to_insert.insert(table.to_string(), lines);
                }
            }
        }

        lines_to_insert
    }

    fn get_all_lines(&self, table: &str) -> Result<Vec<Vec<String>>, MyError> {
        let rdr_read = self.get_reader(table)?;

        let mut tabla = Vec::new();
        for line in rdr_read.lines() {
            let line = line.map_err(|e| MyError::FileError(e.to_string()))?;

            let row: Vec<String> = line
                .split(",")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            tabla.push(row);
        }
        Ok(tabla)
    }

    fn get_reader(&self, table: &str) -> Result<BufReader<File>, MyError> {
        let table = format!("{}_{}", table, self.node.number);
        let mut rdr_read = get_reader(TABLE_PATH, &table)?;
        let _ = get_file_headers(&mut rdr_read)?;
        Ok(rdr_read)
    }

    fn filter_lines(
        &self,
        lines: Vec<Vec<String>>,
        table: &str,
    ) -> Result<Vec<Vec<String>>, MyError> {
        let mut result: Vec<Vec<String>> = vec![];
        let headers = self.get_headers(table)?;
        let mut node_maintains_info = false;
        for line in lines {
            let key = self.get_key_representing_partition(headers.clone(), line.clone())?;
            let nodes_in_partition = self.hashing.get_nodes(&key);
            for node_part in nodes_in_partition {
                if node_part.server_ip == *self.new_node.server_ip {
                    // El nuevo nodo debería tener la info:
                    result.push(line.clone());
                }
                if node_part.server_ip == self.node.server_ip {
                    node_maintains_info = true;
                }
            }

            if !node_maintains_info {
                // El nodo anterior no debe mantener la info:
                let partition: Vec<&str> = key.split(",").collect();
                let query = get_partition_delete_query(table, partition[0], partition[1]);
                let mut query =
                    Query::new(query, self.node.number.clone(), self.node.client_ip.clone());
                self.node.process_query(&mut query)?;
            }
        }

        Ok(result)
    }

    fn get_headers(&self, table: &str) -> Result<Vec<String>, MyError> {
        let table = format!("{}_{}", table, self.node.number);
        let mut rdr_read = get_reader(TABLE_PATH, &table)?;
        let headers = get_file_headers(&mut rdr_read)?;
        Ok(headers)
    }

    fn get_key_representing_partition(
        &self,
        headers: Vec<String>,
        line: Vec<String>,
    ) -> Result<String, MyError> {
        if let (Some(idx_1), Some(idx_2)) = (
            headers.iter().position(|c| *c == "airport"),
            headers.iter().position(|c| *c == "date"),
        ) {
            let key = format!("{},{}", line[idx_1], line[idx_2]);
            return Ok(key);
        }
        Err(MyError::InvalidColumn(
            "No se encontro la key en la tabla.".to_string(),
        ))
    }

    fn send_info(&self, lines: HashMap<String, Vec<Vec<String>>>) {
        for table in lines.keys() {
            self.create_table(table);
            let lines = match lines.get(table) {
                Some(lines) => lines,
                None => continue,
            };
            let headers = match self.get_headers(table) {
                Ok(h) => h,
                _ => continue,
            };
            for line in lines {
                println!("\n ----- line: {:?}", line);
                let query = get_insert_query(table, headers.clone(), line.clone());
                println!(" ----- query: {}\n", query);
                if let Ok(mut socket) = TcpStream::connect(&self.new_node.server_ip) {
                    let msg = QueryServerMessage {
                        query,
                        consistency: 0,
                    };
                    let _ = msg.send_server_through(&mut socket);

                    let _ = Packet::from_stream(&mut socket);
                }
            }
        }
    }

    fn create_table(&self, table: &str) {
        let query = get_create_query(table);
        println!(" ----- query: {}\n", query);
        if let Ok(mut socket) = TcpStream::connect(self.new_node.server_ip.clone()) {
            let msg = QueryServerMessage {
                query,
                consistency: 0,
            };
            let _ = msg.send_server_through(&mut socket);

            let _ = Packet::from_stream(&mut socket);
        }
    }
}
