use std::collections::HashMap;

use fasthash::murmur;

use crate::config::{RING_SIZE, SEED_0, SEED_1, SEED_2};

#[derive(Debug, Clone)]
/// Representa un Token Map de Cassandra.
/// Dado el valor del token (con SEED_0) de un nodo obtenemos todas las posiciones en las que se encuantran sus vnodes.
/// En Cassandra cada token introducido agrega hasta 2 * (RF - 1) nuevas posiciones de vnodes. Por lo que nosotros decidimos
/// agregar RF cantidad de vnodes por cada nodo físico.
pub struct TokenMap {
    map: HashMap<usize, Vec<usize>>, // Anillo de nodos con su hash
    rf: usize,                       // Cantidad de réplicas
    total_buckets: usize,
}

impl TokenMap {
    /// Obtenermos un nuevo token map sin valores asignados.
    pub fn new(rf: usize) -> Self {
        Self {
            map: HashMap::new(),
            rf,
            total_buckets: RING_SIZE,
        }
    }

    /// Función para calcular el hash de un nodo o de los valores de una partition.
    /// Aplicamos el hash a la dirección del nodo y normalizamos este valor en el anillo de vnodes.
    /// Esto nos da una de las posicion del nodo en el anillo.
    /// Si recibe como seed None, usa SEED_0. El cual es el valor utilizado para obtener el token key del map.
    /// Para el caso de una key de una partition obtenemos el valor al nodo mas proximo en el sentido de las agujas del reloj.
    pub fn hash(&self, key: &str, seed: Option<u32>) -> usize {
        let seed_value = match seed {
            Some(seed) => seed,
            None => SEED_0,
        };

        let tocken: usize = murmur::hash32_aligned_with_seed(key, seed_value)
            .try_into()
            .unwrap_or(0);
        tocken % self.total_buckets
    }

    /// Obtenemos todas las posiciones de los vnodes dado el adress de un nodo.
    pub fn get_tokens(&mut self, value: &str) -> Vec<usize> {
        let token = self.hash(value, None);

        if let Some(tokens) = self.map.get(&token) {
            return tokens.to_vec();
        }

        self.assign_tokens(value)
    }

    fn assign_tokens(&mut self, value: &str) -> Vec<usize> {
        let mut num_replicas = 0;
        let mut tokens = vec![];
        while num_replicas < self.rf {
            let seed = match num_replicas {
                0 => SEED_0,
                1 => SEED_1,
                2 => SEED_2,
                _ => SEED_2 - (SEED_1 * num_replicas as u32),
            };

            let token = self.hash(value, Some(seed));
            tokens.push(token);

            num_replicas += 1;
        }

        self.map.insert(tokens[0], tokens.clone());

        tokens
    }
}
