use crate::config::{self, PORTS_FILE};
use crate::cql::query::Query;
use crate::messagestuff::{message::*, message_server::gossipmessage::*};
use crate::my_errors::*;
use crate::serverstuff::{
    handleconnectionclient::handle_connection_client,
    handleconnectiongossip::connect_to_random_gossip,
    handleconnectiongossip::handle_connection_gossip,
    handleconnectionserver::handle_connection_server,
};
use crate::NeighborNode;
use rand::seq::SliceRandom;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Copy)]
#[repr(u8)]
pub enum Status {
    Connected = config::CONNECTED,
    Disconnected = config::DISCONNECTED,
}

impl Status {
    pub fn from_u8(value: u8) -> Result<Self, MyError> {
        match value {
            config::CONNECTED => Ok(Status::Connected),
            config::DISCONNECTED => Ok(Status::Disconnected),
            _ => Err(MyError::Error("Invalid Status".to_string())),
        }
    }
}

// Representa al nodo en el que estamos parados
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub puerto: String,
    pub server_ip: String, // IP donde nodo se pone a escuchar conecciones de servidores
    pub client_ip: String, // IP donde nodo se pone a escuchar conecciones de cliente
    pub gossip_ip: String, // IP donde nodo se pone a escuchar mensajes Gossip
    pub file_path: String,
    pub nodes_path: String,
    pub neighbors: Vec<NeighborNode>, // todos los otros nodos
}

impl Node {
    pub fn new(port: &str) -> Self {
        Self {
            puerto: port.to_string(),
            server_ip: format!("127.0.0.1:700{}", port),
            client_ip: format!("127.0.0.1:800{}", port),
            gossip_ip: format!("127.0.0.1:900{}", port),
            file_path: "".to_string(),
            nodes_path: "nodes.txt".to_string(),
            neighbors: vec![], // todos los nodos que estan en el ring
        }
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
            let existing = &self.neighbors[pos];
            if existing.status != neighbor_node.status {
                self.neighbors[pos] = neighbor_node;
                return true;
            }
            if existing.time < neighbor_node.time {
                self.neighbors[pos] = neighbor_node;
                return false;
            }
            false
        } else {
            self.neighbors.push(neighbor_node);
            true
        }
    }

    pub fn get_random_neighbor(&self) -> Option<&NeighborNode> {
        let mut rng = rand::thread_rng();

        let filtered: Vec<&NeighborNode> = self
            .neighbors
            .iter()
            .filter(|n| n.puerto != self.puerto && n.status == Status::Connected)
            .collect();
        filtered.choose(&mut rng).copied()
    }

    pub fn disconnect_neighbor(&mut self, ip: String) {
        if let Some(c) = ip.chars().last() {
            let port = c.to_string();
            if let Some(n) = self.neighbors.iter_mut().find(|n| n.puerto == port) {
                n.disconnect_node();
            }
        }
    }

    pub fn connect_neighbor(&mut self, ip: String) {
        if let Some(c) = ip.chars().last() {
            let port = c.to_string();
            if let Some(n) = self.neighbors.iter_mut().find(|n| n.puerto == port) {
                n.connect_node();
            }
        }
    }

    // Proceso la query y devuelve la tabla en formato String
    pub fn process_query(&self, query: &mut Query) -> Result<Vec<String>, MyError> {
        query.execute()
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
    //threads_joins: &mut Vec<JoinHandle<()>>,
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
            //threads_joins.push(t);
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
    current_node.add_neighbor(NeighborNode::new(&current_node.puerto));

    //empiezo gossip
    let ports = read_ports_from_file(PORTS_FILE)?;
    let my_ip = current_node.server_ip.clone();
    match connect_to_random_gossip(ports.clone(), &port, show_g) {
        Some((mut s, addr)) => {
            let gossip_msg = GossipMessage {
                ip: my_ip.clone(),
                estado: Status::Connected,
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

    //let mut threads_joins: Vec<JoinHandle<()>> = Vec::new();

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
                    estado: Status::Connected,
                    ttl: 5_u8,
                };
                let _ = gossip_msg.send_server_through(&mut s);
            }
            None => println!("tengo que ser el primer nodo porque no encontre otro"),
        };
        //joinear(&mut threads_joins);
    }
}
/*
fn joinear (threads_joins: &mut Vec<JoinHandle<()>>) {
    for j in threads_joins.into_iter() {
        if j.is_finished() {
            j.join().unwrap();
        }
    }
}*/
