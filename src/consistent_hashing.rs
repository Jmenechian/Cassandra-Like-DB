// El hashing consistente permite la distribución de datos en un clúster,
// minimizando la reorganización cuando se añaden o eliminan nodos.
//
// El hashing consistente particiona los datos basándose en la clave de partición.

use fasthash::murmur;
use std::collections::BTreeMap;

use crate::NeighborNode;

#[derive(Debug, Clone)]
// Estructura que guarda los nodos miembros de un anillo y el factor de replicación.
// Anillo de nodos: Se utiliza un arbol binario que permite encontrar los nodos en el anillo, se guarda:
// el hash de la Adress (IP:puerto) del nodo normalizada por la cantidad de nodos, y el nodo.
pub struct ConsistentHash<NeighborNode> {
    ring: BTreeMap<usize, NeighborNode>, // Anillo de nodos con su hash
    rf: usize,                           // Cantidad de réplicas
    total: usize,
}

impl ConsistentHash<NeighborNode> {
    pub fn new(rf: usize) -> Self {
        Self {
            ring: BTreeMap::new(),
            rf,       // Replication Factor
            total: 0, // Cantidad de nodos
        }
    }

    // Función para calcular el hash de un nodo
    // Aplicamos el hash a la dirección del nodo y normalizamos este valor en el anillo de nodos.
    // Esto nos da la posicion del nodo en el anillo.
    pub fn hash(&self, key: &str) -> usize {
        let tocken: usize = murmur::hash32(key).try_into().unwrap();
        tocken % self.total
    }

    // Agregar un nodo al anillo
    pub fn add_node(&mut self, node: NeighborNode) {
        self.total += 1;
        let mut hash = self.hash(&node.server_ip);
        while self.ring.contains_key(&hash) {
            hash += 1;
        }
        self.ring.insert(hash, node.clone());
    }

    // Agregar un nodo al anillo
    pub fn add_nodes(&mut self, nodes: Vec<NeighborNode>) {
        self.total += nodes.len();
        for node in nodes {
            let mut hash = self.hash(&node.server_ip);
            while self.ring.contains_key(&hash) {
                hash += 1;
            }
            self.ring.insert(hash, node.clone());
        }
    }

    // Eliminar un nodo del anillo
    pub fn remove_node(&mut self, node: &NeighborNode) {
        for i in 0..self.rf {
            let hash = self.hash(&format!("{}-{}", self.hash(&node.server_ip), i));
            self.ring.remove(&hash);
        }
    }

    // Encontrar los nodos responsables de una clave
    // Clave: string con los valores que se buscan en la query que corresponden a las columnas de partitions, separados por coma.
    // Ejemplo: en las tablas de Estados de Vuelo: "Aeropuerto,Date"
    //
    // Para encontrar los nodos propietarios de una clave primero aplicamos un hash a esa clave para generar un token,
    // luego recorremos el anillo hasta que encontramos tres nodos distintos (si RF=3), momento en el cual hemos encontrado
    // todas las réplicas de esa clave.
    pub fn get_nodes(&self, key: &str) -> Vec<NeighborNode> {
        let mut nodes: Vec<NeighborNode> = vec![];

        if self.ring.is_empty() {
            println!("Se deben agregar nodos vecinos.");
            return nodes;
        }

        let hash = self.hash(key);

        let result = self.ring.range(hash..);

        for node in result {
            nodes.push(node.1.clone());

            if nodes.len() == self.rf {
                break;
            }
        }

        if nodes.len() < 3 {
            let result = self.ring.range(0..);

            for node in result {
                nodes.push(node.1.clone());

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
    pub fn view_nodes_by_key() {
        // Función para obtener los nodos en los que se debería guardar a mano un vuelo a partir de su key (valores de las particiones).
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

        println!("\n \t - Key: {} - Nodes: {:?}", key, nodes_result);
    }

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
