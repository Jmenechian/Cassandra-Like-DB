use crate::config::RING_SIZE;
use crate::serverstuff::neighbor_node::NeighborNode;
use fasthash::murmur;
use std::collections::HashMap;

#[derive(Debug, Clone)]
/// El hashing consistente permite la distribución de datos en un clúster,
/// minimizando la reorganización cuando se añaden o eliminan nodos.
//
/// El hashing consistente particiona los datos basándose en la clave de partición.
/// Estructura que guarda los nodos miembros de un anillo y el factor de replicación.
/// Anillo de nodos: Se utiliza un arbol binario que permite encontrar los nodos en el anillo, se guarda:
/// el hash de la Adress (IP:puerto) del nodo normalizada por la cantidad de nodos, y el nodo.
pub struct ConsistentHash<NeighborNode> {
    ring: HashMap<usize, NeighborNode>, // Anillo de nodos con su hash
    rf: usize,                           // Cantidad de réplicas
    total: usize,
}

impl ConsistentHash<NeighborNode> {
    pub fn new(rf: usize) -> Self {
        Self {
            ring: HashMap::new(),
            rf,               // Replication Factor
            total: RING_SIZE, // Cantidad de nodos
        }
    }

    /// Función para calcular el hash de un nodo
    /// Aplicamos el hash a la dirección del nodo y normalizamos este valor en el anillo de nodos.
    /// Esto nos da la posicion del nodo en el anillo.
    pub fn hash(&self, key: &str) -> usize {
        let tocken: usize = murmur::hash32_aligned_with_seed(key, 0).try_into().unwrap_or(0);
        tocken % self.total
    }

    /// Agregar un nodo al anillo
    pub fn add_node(&mut self, node: NeighborNode) {
        let mut hash = self.hash(&node.server_ip);

        while self.ring.contains_key(&hash) {
            if let Some(node_in_ring) = self.ring.get(&hash) {
                if *node_in_ring == node {
                    return;
                }
            }
            hash += 1;
        }

        self.ring.insert(hash, node.clone());
    }

    /// Agregar un nodo al anillo
    pub fn add_nodes(&mut self, nodes: Vec<NeighborNode>) {
        for node in nodes {
            self.add_node(node);
        }
    }


    fn find_closest_greater_index(&self, vec: &Vec<&usize>, target: &usize) -> Option<usize> {
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
    /// Encontrar los nodos responsables de una clave
    /// Clave: string con los valores que se buscan en la query que corresponden a las columnas de partitions, separados por coma.
    /// Ejemplo: en las tablas de Estados de Vuelo: "Aeropuerto,Date"
    //
    /// Para encontrar los nodos propietarios de una clave primero aplicamos un hash a esa clave para generar un token,
    /// luego recorremos el anillo hasta que encontramos tres nodos distintos (si RF=3), momento en el cual hemos encontrado
    /// todas las réplicas de esa clave.
    pub fn get_nodes(&self, key: &str) -> Vec<NeighborNode> {
        //caso borde, 1 o 2 nodos conectados
        let mut nodes: Vec<NeighborNode> = vec![];

        if self.ring.is_empty() {
            println!("Se deben agregar nodos vecinos.");
            return nodes;
        }

        let hash = self.hash(key);
        println!("\n\n\nHash: {}", hash);
        println!("Key: {}\n\n\n", key);
        let mut keys = self.ring.keys().collect::<Vec<&usize>>();
        keys.sort();

        let idx = self.find_closest_greater_index(&keys, &hash).unwrap_or(0);
        let result: Vec<_> = keys[idx..].to_vec();
        println!("\n\n\nRing: {:?}", self.ring);
        println!("Result: {:?}\n\n\n", result);
        for node_idx in result {
            let node = self.ring.get(node_idx).unwrap();
            nodes.push(node.clone());

            if nodes.len() == self.rf {
                break;
            }
        }

        if nodes.len() < self.rf {
            let result: Vec<_> = keys.to_vec();

            for node_idx in result {
                let node = self.ring.get(node_idx).unwrap();

                if nodes.contains(node) {
                    continue;
                }
                nodes.push(node.clone());

                if nodes.len() == self.rf {
                    break;
                }
            }
        }

        nodes
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

        let first = NeighborNode::new("2");
        let second = NeighborNode::new("1");
        let third = NeighborNode::new("4");

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

        let first = NeighborNode::new("8");
        let second = NeighborNode::new("2");
        let third = NeighborNode::new("1");

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

        let first = NeighborNode::new("2");
        let second = NeighborNode::new("1");
        let third = NeighborNode::new("4");

        assert_eq!(nodes_result[0].server_ip, first.server_ip);
        assert_eq!(nodes_result[1].server_ip, second.server_ip);
        assert_eq!(nodes_result[2].server_ip, third.server_ip);
    }
}

