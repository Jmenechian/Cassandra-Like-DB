use crate::{
    config::*,
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
    utils::{get_file_headers, get_reader_path},
    Query,
};

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    net::TcpStream,
    vec,
};

#[derive(Debug)]
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
        let tables = node.tables_created();
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
        for table in self.tables.clone() {
            if table.contains("origin") {
                lines_to_insert.insert("origin".to_string(), vec![]);
            } else if table.contains("destination") {
                lines_to_insert.insert("destination".to_string(), vec![]);
            } else if table.contains("flights_in_progress") {
                lines_to_insert.insert("flights_in_progress".to_string(), vec![]);
            }
        }

        if lines_to_insert.is_empty() {
            return;
        }

        // Vemos las posiciones de los vnodes del new_node:
        let tokens = self.hashing.token_map.get_tokens(&self.new_node.ip.to_string());
        // Vemos vnodes siguientes a estas posiciones:
        let keys = self.hashing.get_keys_in_order();
        for position_new_node in tokens {
            let next_pos = position_new_node + 1;
            let next_node_idx = self
                .hashing
                .find_closest_greater_index(&keys, &next_pos)
                .unwrap_or(0);
            if let Some(next_node) = self.hashing.ring.get(keys[next_node_idx]) {
                // Si soy ese nodo le mando INSERT de las lineas que le correspondes (de las 3 tablas que usamos):
                if next_node.ip == self.node.ip {
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
                .replace(" ", "-")
                .split(",")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            tabla.push(row);
        }
        Ok(tabla)
    }

    fn get_reader(&self, table: &str) -> Result<BufReader<File>, MyError> {
        let mut rdr_read = get_reader_path(table)?;
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

        for line in lines {
            let key = self.get_key_representing_partition(headers.clone(), line.clone())?;
            let nodes_in_partition = self.hashing.get_nodes(&key);
            let mut ips = vec![];
            for node_part in nodes_in_partition {
                ips.push(node_part.ip);
            }
            if ips.contains(&self.new_node.ip) && !ips.contains(&self.node.ip) {
                result.push(line.clone());

                // El nodo anterior no debe mantener la info:
                let partition: Vec<&str> = key.split(",").collect();
                let query = get_partition_delete_query(table, partition[0], partition[1]);
                let mut query =
                    Query::new(query, self.node.ip.to_string());
                self.node.process_query(&mut query)?;
            }
        }

        Ok(result)
    }

    fn get_headers(&self, table: &str) -> Result<Vec<String>, MyError> {
        let mut rdr_read = get_reader_path(table)?;
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
            let lines_table = match lines.get(table) {
                Some(lines_result) => lines_result,
                None => continue,
            };
            let headers = match self.get_headers(table) {
                Ok(h) => h,
                _ => continue,
            };
            for line in lines_table {
                let query = get_insert_query(table, &mut headers.clone(), &mut line.clone());
                if let Ok(mut socket) = TcpStream::connect(&self.new_node.server_socket) {
                    let msg = QueryServerMessage {
                        query,
                        consistency: 0,
                    };
                    let _ = msg.send_server_through(&mut socket, self.node.logger.clone());

                    let _ = Packet::from_stream(&mut socket, self.node.logger.clone());
                }
            }
        }
    }

    fn create_table(&self, table: &str) {
        let query = get_create_query(table);
        if let Ok(mut socket) = TcpStream::connect(self.new_node.server_socket.clone()) {
            let msg = QueryServerMessage {
                query,
                consistency: 0,
            };
            let _ = msg.send_server_through(&mut socket, self.node.logger.clone());

            let _ = Packet::from_stream(&mut socket, self.node.logger.clone());
        }
    }
}
/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test() {
        let key = "DEL,25-12-24".to_string();
        let rf = 3;

        let node_1 = NeighborNode::new("1");
        let node_2 = NeighborNode::new("2");
        let node_3 = NeighborNode::new("3");
        let node_4 = NeighborNode::new("4");
        let node_5 = NeighborNode::new("5"); //3,1,2 AEP,25-12-24
        let node_6 = NeighborNode::new("6"); //6,1,2 AEP,25-12-24
        let node_7 = NeighborNode::new("7");
        let node_8 = NeighborNode::new("8"); //2,6,3 DEL,25-12-24
                                             //let node_9 = NeighborNode::new("9"); //2,9,6 DEL,25-12-24

        let nodes: Vec<NeighborNode> = vec![
            node_1, node_2, node_3, node_4, node_5, node_6, node_7, node_8, //, node_9
        ];

        let mut hashing = ConsistentHash::new(rf);
        hashing.add_nodes(nodes.clone());

        let nodes_result_before = hashing.get_nodes(&key);
        println!("{:?}", nodes_result_before);
    }
}
*/