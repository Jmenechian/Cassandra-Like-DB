use fasthash::murmur;
use std::collections::BTreeMap;

use crate::NeighborNode;

#[derive(Debug, Clone)]
// Estructura que guarda los nodos miembros de un anillo y el factor de replicación.
// Anillo de nodos: Se utiliza un arbol binario que permite encontrar los nodos en el anillo, se guarda: 
// el hash de la Adress (IP:puerto) del nodo normalizada por la cantidad de nodos, y el nodo.
pub struct ConsistentHash<NeighborNode> {
    ring: BTreeMap<usize, NeighborNode>, // Anillo de nodos con su hash
    rf: usize,           // Cantidad de réplicas 
    total: usize,
}

impl ConsistentHash<NeighborNode> {
    pub fn new(rf: usize) -> Self {
        Self {
            ring: BTreeMap::new(),
            rf, // Replication Factor
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
        self.total = self.total + 1;
        for i in 0..self.rf {
            let hash = self.hash(&format!("{}-{}", self.hash(&node.server_ip), i));
            self.ring.insert(hash, node.clone());
        }
    }

    // Agregar un nodo al anillo
    pub fn add_nodes(&mut self, nodes: Vec<NeighborNode>) {
        self.total = self.total + nodes.len();
        for node in nodes {
            //for i in 0..self.rf {
                //let hash = self.hash(&format!("{}-{}", self.hash(&node.server_ip), i));
                let hash = self.hash(&format!("{}", self.hash(&node.server_ip)));
                self.ring.insert(hash, node.clone());
            //}
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

        let first_node = self.hash(key);
        let last_node = first_node + self.rf;

        for i in first_node..last_node {
            nodes.push(self.ring[&i].clone());
        }

        nodes
    }
}
