use super::token_map::TokenMap;
use crate::serverstuff::{neighbor_node::NeighborNode, node_status::Status};

use std::{collections::HashMap, vec};

#[derive(Debug, Clone)]
/// El hashing consistente permite la distribución de datos en un clúster,
/// minimizando la reorganización cuando se añaden o eliminan nodos.
//
/// El hashing consistente particiona los datos basándose en la clave de partición.
///
/// Esta estructura que guarda los nodos miembros de un anillo y el factor de replicación.
/// Además, cuenta con un Token Map que mapea todas las posiciones que ocupan los vnodes de un nodo físico
///
/// Anillo de nodos: Se guardan todos los vnodes de todos los nodos de la red.
///
/// Los nodos virtuales nos aseguramos tener carga de consultas iguales entre los nodos físicos.
/// Cada vnode es una asignacíon de múltiples tokens en el anillo de tokens a cada nodo físico.
///
/// Al permitir que un solo nodo físico ocupe múltiples posiciones en el anillo, podemos hacer que
/// los clústeres pequeños parezcan más grandes y, por lo tanto, incluso con la adición de un solo
/// nodo físico podemos hacer que parezca que agregamos muchos más nodos, tomando de hecho muchos
/// fragmentos de datos más pequeños de más vecinos del anillo cuando agregamos incluso un solo nodo.
///
/// Cuando se agrega un nuevo nodo, este acepta cantidades aproximadamente iguales de datos de otros
/// nodos del anillo, lo que da como resultado una distribución equitativa de datos en todo el clúster.
///
/// Si un nodo deja de estar disponible, la carga de consultas (especialmente la carga de consultas que
/// reconoce tokens) se distribuye de manera uniforme entre muchos otros nodos.
/// Además al desconectarse un nodo, éste mantiene su posición en el ring, de esta forma nos da pie a usar
/// Hinted Handoff.
pub struct ConsistentHash<NeighborNode> {
    pub ring: HashMap<usize, NeighborNode>, // Anillo de nodos con su hash
    pub rf: usize,                          // Cantidad de réplicas
    pub token_map: TokenMap,
}

impl ConsistentHash<NeighborNode> {
    /// Se crea el ring vacio.
    pub fn new(rf: usize) -> Self {
        Self {
            ring: HashMap::new(),         // Todos los vnodes.
            rf,                           // Replication Factor
            token_map: TokenMap::new(rf), // Posiciones de los vnodes.
        }
    }

    /// Se agrega un nodo físico al anillo como tantos vitual nodes nos indica RF.
    pub fn add_node(&mut self, node: NeighborNode) {
        let tokens = self.token_map.get_tokens(&node.server_ip);

        for hash in tokens {
            if self.ring.contains_key(&hash) {
                continue;
            }

            self.ring.insert(hash, node.clone());
        }
    }

    /// Asignar los nodos al anillo.
    pub fn add_nodes(&mut self, nodes: Vec<NeighborNode>) {
        for node in nodes {
            self.add_node(node);
        }
    }

    /// Buscamos el nodo más cercano a una posición del anillo.
    pub fn find_closest_greater_index(&self, vec: &[&usize], target: &usize) -> Option<usize> {
        // Realizamos la búsqueda binaria
        let result = vec.binary_search(&target);

        match result {
            Ok(index) => Some(index), // Si encontramos el valor exacto, devolvemos el índice
            Err(index) => {
                if index < vec.len() {
                    Some(index) // El índice del siguiente valor mayor
                } else {
                    None // Si el índice es fuera de los límites del vector
                }
            }
        }
    }
    /// Encontrar los nodos responsables de una clave.
    /// Clave: string con los valores que se buscan en la query que corresponden a las columnas de partitions, separados por coma.
    /// Ejemplo: en las tablas de Estados de Vuelo: "Aeropuerto,Date"
    //
    /// Para encontrar los nodos propietarios de una clave primero aplicamos un hash a esa clave para generar un token,
    /// luego recorremos el anillo hasta que encontramos tres nodos distintos (si RF=3), momento en el cual hemos encontrado
    /// todas las réplicas de esa clave.
    pub fn get_nodes(&self, key: &str) -> Vec<NeighborNode> {
        let mut nodes: Vec<NeighborNode> = vec![];

        if self.ring.is_empty() {
            return nodes;
        }

        let hash = self.token_map.hash(key, None);
        println!("\n\n\nHash: {}", hash);
        println!("Key: {}\n\n\n", key);
        let keys = self.get_keys_in_order();

        let idx = self.find_closest_greater_index(&keys, &hash).unwrap_or(0);
        let result: Vec<_> = keys[idx..].to_vec();
        // println!("\n\n\nRing: {:?}", self.ring);
        println!("Result: {:?}\n\n\n", result);

        let mut connected_nodes = 0;

        self.get_nodes_in_partition(result, &mut connected_nodes, &mut nodes);

        if connected_nodes < self.rf {
            let result: Vec<_> = keys.to_vec();
            self.get_nodes_in_partition(result, &mut connected_nodes, &mut nodes)
        }

        nodes
    }

    fn get_nodes_in_partition(
        &self,
        result: Vec<&usize>,
        connected_nodes: &mut usize,
        nodes: &mut Vec<NeighborNode>,
    ) {
        for node_idx in result {
            let node = self.ring.get(node_idx).unwrap();

            if nodes.contains(node) {
                continue;
            }

            nodes.push(node.clone());

            if node.status == Status::Connected {
                *connected_nodes += 1;
            }

            if *connected_nodes == self.rf {
                break;
            }
        }
    }

    /// Obtenemos las posiciones usadas por los vnodes en el ring, de forma ordenada.
    pub fn get_keys_in_order(&self) -> Vec<&usize> {
        let mut keys = self.ring.keys().collect::<Vec<&usize>>();
        keys.sort();
        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_return_nodes_from_key_with_consistent_hash() {
        let key = "AEP,12-12-24".to_string();

        let rf = 3;

        let node_1 = NeighborNode::new("1");
        let node_2 = NeighborNode::new("2");
        let node_3 = NeighborNode::new("3");
        let node_4 = NeighborNode::new("4");
        let node_5 = NeighborNode::new("5");
        let node_6 = NeighborNode::new("6");
        let node_7 = NeighborNode::new("7");
        let node_8 = NeighborNode::new("8");

        let nodes: Vec<NeighborNode> = vec![
            node_1, node_2, node_3, node_4, node_5, node_6, node_7, node_8,
        ];

        let mut hashing = ConsistentHash::new(rf);
        hashing.add_nodes(nodes);

        let nodes_result = hashing.get_nodes(&key);

        let first = NeighborNode::new("4");
        let second = NeighborNode::new("8");
        let third = NeighborNode::new("6");

        assert_eq!(nodes_result[0].server_ip, first.server_ip);
        assert_eq!(nodes_result[1].server_ip, second.server_ip);
        assert_eq!(nodes_result[2].server_ip, third.server_ip);
    }

    #[test]
    pub fn test_return_nodes_from_key_with_consistent_hash_2() {
        let key = "MDZ,12-12-24".to_string();

        let rf = 3;

        let node_1 = NeighborNode::new("1");
        let node_2 = NeighborNode::new("2");
        let node_3 = NeighborNode::new("3");
        let node_4 = NeighborNode::new("4");
        let node_5 = NeighborNode::new("5");
        let node_6 = NeighborNode::new("6");
        let node_7 = NeighborNode::new("7");
        let node_8 = NeighborNode::new("8");

        let nodes: Vec<NeighborNode> = vec![
            node_1, node_2, node_3, node_4, node_5, node_6, node_7, node_8,
        ];

        let mut hashing = ConsistentHash::new(rf);
        hashing.add_nodes(nodes);

        let nodes_result = hashing.get_nodes(&key);

        let first = NeighborNode::new("2");
        let second = NeighborNode::new("4");
        let third = NeighborNode::new("5");

        assert_eq!(nodes_result[0].server_ip, first.server_ip);
        assert_eq!(nodes_result[1].server_ip, second.server_ip);
        assert_eq!(nodes_result[2].server_ip, third.server_ip);
    }

    #[test]
    pub fn test_return_nodes_from_key_with_consistent_hash_3() {
        let key = "ZRH,12-12-24".to_string();

        let rf = 3;

        let node_1 = NeighborNode::new("1");
        let node_2 = NeighborNode::new("2");
        let node_3 = NeighborNode::new("3");
        let node_4 = NeighborNode::new("4");
        let node_5 = NeighborNode::new("5");
        let node_6 = NeighborNode::new("6");
        let node_7 = NeighborNode::new("7");
        let node_8 = NeighborNode::new("8");

        let nodes: Vec<NeighborNode> = vec![
            node_1, node_2, node_3, node_4, node_5, node_6, node_7, node_8,
        ];

        let mut hashing = ConsistentHash::new(rf);
        hashing.add_nodes(nodes);

        let nodes_result = hashing.get_nodes(&key);
        println!("{:?}", nodes_result);

        let first = NeighborNode::new("2");
        let second = NeighborNode::new("6");
        let third = NeighborNode::new("3");

        assert_eq!(nodes_result[0].server_ip, first.server_ip);
        assert_eq!(nodes_result[1].server_ip, second.server_ip);
        assert_eq!(nodes_result[2].server_ip, third.server_ip);
    }
}
