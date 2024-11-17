use crate::config::{
    PORTS_FILE, TABLE_DESTINATION_NAME, TABLE_FLIGHT_SIMULATOR, TABLE_ORIGIN_NAME, TABLE_PATH,
};
use crate::cql::query::Query;
use crate::errors::my_errors::*;
use crate::messagestuff::{message::*, message_server::gossipmessage::*};
use crate::serverstuff::neighbor_node::NeighborNode;
use crate::serverstuff::node_status::Status;
use crate::serverstuff::{
    handleconnectionclient::handle_connection_client,
    handleconnectiongossip::connect_to_random_gossip,
    handleconnectiongossip::handle_connection_gossip,
    handleconnectionserver::handle_connection_server,
};
use rand::seq::SliceRandom;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

// Representa al nodo en el que estamos parados
#[derive(Debug)]
pub struct Node {
    pub number: String,
    pub server_ip: String, // IP donde nodo se pone a escuchar conecciones de servidores
    pub client_ip: String, // IP donde nodo se pone a escuchar conecciones de cliente
    pub gossip_ip: String, // IP donde nodo se pone a escuchar mensajes Gossip
    pub neighbors: Vec<NeighborNode>, // todos los otros nodos
    pub tables: Vec<Arc<RwLock<File>>>, // archivos que representan la tabla abiertos
}

impl Node {
    pub fn new(number: &str) -> Self {
        let tables = Self::open_tables_and_lock(number.to_string());
        Self {
            number: number.to_string(),
            server_ip: format!("127.0.0.1:700{}", number),
            client_ip: format!("127.0.0.1:800{}", number),
            gossip_ip: format!("127.0.0.1:900{}", number),
            neighbors: vec![], // todos los nodos que estan en el ring
            tables,
        }
    }

    fn open_tables_and_lock(number_node: String) -> Vec<Arc<RwLock<File>>> {
        let mut files: Vec<Arc<RwLock<File>>> = vec![];
        for path in Self::get_table_paths(number_node) {
            let file = match OpenOptions::new()
                .append(true)
                .read(true)
                .open(path)
                .map_err(|e| MyError::FileError(e.to_string()))
            {
                Ok(f) => f,
                _ => continue,
            };
            let file = Arc::new(RwLock::new(file));
            files.push(file);
        }
        files
    }

    fn get_table_paths(number_node: String) -> Vec<String> {
        let path_1 = format!("{}/{}_{}.csv", TABLE_PATH, TABLE_ORIGIN_NAME, number_node);
        let path_2 = format!(
            "{}/{}_{}.csv",
            TABLE_PATH, TABLE_DESTINATION_NAME, number_node
        );
        let path_3 = format!(
            "{}/{}_{}.csv",
            TABLE_PATH, TABLE_FLIGHT_SIMULATOR, number_node
        );
        vec![path_1, path_2, path_3]
    }

    pub fn add_neighbors(&mut self, neighbor_nodes: Vec<NeighborNode>) {
        self.neighbors = neighbor_nodes;
    }

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

    // Proceso la query y devuelve la tabla en formato String
    pub fn process_query(&self, query: &mut Query) -> Result<Vec<String>, MyError> {
        let table: Arc<RwLock<File>>;
        if query.get_table() == format!("{}_{}", TABLE_ORIGIN_NAME, self.number) {
            table = self.tables[0].clone();
        } else if query.get_table() == format!("{}_{}", TABLE_DESTINATION_NAME, self.number) {
            table = self.tables[1].clone();
        } else if query.get_table() == format!("{}_{}", TABLE_FLIGHT_SIMULATOR, self.number) {
            table = self.tables[2].clone();
        } else {
            return Err(MyError::InvalidTable("Tabla no valida".to_string()));
        }
        query.execute(table)
    }
}

// Lee los puertos del archivo
fn read_ports_from_file(filename: &str) -> Result<Vec<u16>, MyError> {
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => return Err(MyError::FileError(e.to_string())),
    };
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| {
            let l_string = l.map_err(|e| MyError::FileError(e.to_string()))?;
            l_string
                .parse::<u16>()
                .map_err(|e| MyError::FileError(e.to_string()))
        })
        .collect()
}

pub fn try_connect(ip: String) -> Option<TcpStream> {
    match TcpStream::connect(ip) {
        Ok(stream) => Some(stream),
        Err(_) => None,
    }
}

fn listen<F>(
    ip: String,
    node: Arc<RwLock<Node>>,
    handler: F,
    show: bool,
) -> Result<JoinHandle<Result<(), MyError>>, MyError>
where
    F: Fn(Arc<RwLock<Node>>, &mut TcpStream, bool) -> Result<(), MyError> + Send + 'static + Copy,
{
    let current_node = Arc::clone(&node);
    let t: JoinHandle<Result<(), MyError>> = thread::spawn(move || {
        let listener = match TcpListener::bind(ip) {
            Ok(j) => j,
            Err(_) => {
                return Err(MyError::TcpListenerError(
                    "Error al conectarse al puerto".to_string(),
                ))
            }
        };
        for stream in listener.incoming() {
            let mut s: TcpStream = match stream {
                Ok(s) => s,
                Err(_) => {
                    return Err(MyError::TcpStreamError(
                        "Error al leer TcpStream".to_string(),
                    ))
                }
            };
            let node_thread = Arc::clone(&current_node);
            thread::spawn(move || {
                match handler(node_thread.clone(), &mut s, show) {
                    Ok(_) => {}
                    Err(e) => println!("Error: {:?}", e),
                };
            });
        }
        Ok(())
    });
    Ok(t)
}

pub fn initialize(port: String) -> Result<(), MyError> {
    let show_g: bool = false;
    let show_s: bool = false;
    let show_c: bool = true;

    let mut current_node: Node = Node::new(&port);
    println!("Nodo actual: {:?}", current_node);
    //me agrego a mi mismo a vecinos
    current_node.add_neighbor(NeighborNode::new(&current_node.number));

    //empiezo gossip
    let ports = read_ports_from_file(PORTS_FILE)?;
    let my_ip = current_node.server_ip.clone();
    match connect_to_random_gossip(ports.clone(), &port, show_g) {
        Some((mut s, addr)) => {
            let gossip_msg = GossipMessage {
                ip: my_ip.clone(),
                status: Status::Connected,
                ttl: 5_u8,
            };
            let _ = gossip_msg.send_server_through(&mut s);
            let port = match addr.chars().last() {
                Some(c) => c,
                None => return Err(MyError::Error("No puse obtener el puerto".to_string())),
            };
            let new = current_node.add_neighbor(NeighborNode::new(&port.to_string()));
            if show_g && new {
                println!("G   Mis vecinos son: {:?}", current_node.neighbors);
            }
        }
        None => println!("tengo que ser el primer nodo porque no encontre otro"),
    };

    let server_ip = current_node.server_ip.clone();
    let client_ip = current_node.client_ip.clone();
    let gossip_ip = current_node.gossip_ip.clone();
    let current_node: Arc<RwLock<Node>> = Arc::new(RwLock::new(current_node)); //node para MAIN THREAD

    // Seccion cliente-servidor
    listen(
        client_ip,
        current_node.clone(),
        handle_connection_client,
        show_c,
    )?;

    // Seccion servidor-servidor
    listen(
        server_ip,
        current_node.clone(),
        handle_connection_server,
        show_s,
    )?;

    // Seccion Gossip
    listen(
        gossip_ip,
        current_node.clone(),
        handle_connection_gossip,
        show_g,
    )?;

    loop {
        thread::sleep(Duration::from_secs(2));
        match connect_to_random_gossip(ports.clone(), &port, show_g) {
            Some((mut s, _)) => {
                let gossip_msg = GossipMessage {
                    ip: my_ip.clone(),
                    status: Status::Connected,
                    ttl: 5_u8,
                };
                let _ = gossip_msg.send_server_through(&mut s);
            }
            None => println!("tengo que ser el primer nodo porque no encontre otro"),
        };
    }
}
