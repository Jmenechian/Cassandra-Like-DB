use crate::config::{self, PORTS_FILE};
use crate::cql::query::Query;
use crate::messagestuff::{
    message::*,
    message_server::gossipmessage::*,
};
use crate::serverstuff::{
    handleconnectionserver::handle_connection_server,
    handleconnectiongossip::handle_connection_gossip,
    handleconnectiongossip::connect_to_random_gossip,
    handleconnectionclient::handle_connection_client,
};
use crate::my_errors::*;
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
    pub ip: String, //temp
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
            ip: format!("127.0.0.1:{}", port),
            puerto: format!("{}", port),
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
        filtered.choose(&mut rng).map(|&n| n)
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
            l_string.parse::<u16>()
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

// Se guarda a TODOS los nodos vecinos
// Tiene guardado el estado de cada nodo
fn connect_to_ports(ports: Vec<u16>, my_port: &String) -> Vec<NeighborNode> {
    let mut known_nodes: Vec<NeighborNode> = Vec::new();

    for &port in &ports {
        if &port.to_string() == my_port {
            continue;
        }
        let status: Status;
        if try_connect(format! {"127.0.0.1:700{}", port}).is_some() {
            status = Status::Connected;
            println!("Conectado exitosamente al nodo en el puerto {}", port);
            // Avisar al nodo que me conecte??
            // Hacerlo por Gossip....
        } else {
            status = Status::Connected; ///////// DEBERIA EMPEZAR DESCONECTADO Y ACTUALIZARSE CON GOSSIP
                                        // status = Status::Disconnected;
            println!("No se pudo conectar al nodo en el puerto {}", port);
        }

        let mut neighbor_node = NeighborNode::new(&port.to_string());
        if status == Status::Connected {
            neighbor_node.connect_node();
        }

        known_nodes.push(neighbor_node);
    }
    known_nodes
}


// Le deberia mandar la query a los nodos correspondientes!
pub fn handle_query(_query: Query) {
    // TO DO
    // println!("Llego la query: {}", query.query);
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
    let t = thread::spawn(move || {
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

pub fn initialize_2(port: String) -> Result<(), MyError> {
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
                ttl: 5 as u8,
            };
            let _ = gossip_msg.send_server_through(&mut s);
            let port = match addr.chars().last() {
                Some(c) => c,
                None => return Err(MyError::Error("No puse obtener el puerto".to_string())),
            };
            let new = current_node
                .add_neighbor(NeighborNode::new(&port.to_string()));
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
        thread::sleep(Duration::from_secs(10));
        match connect_to_random_gossip(ports.clone(), &port, show_g) {
            Some((mut s, _)) => {
                let gossip_msg = GossipMessage {
                    ip: my_ip.clone(),
                    estado: Status::Connected,
                    ttl: 5 as u8,
                };
                let _ = gossip_msg.send_server_through(&mut s);
            }
            None => println!("tengo que ser el primer nodo porque no encontre otro"),
        };
    }
}

// Inicializa el nodo
// Lo conecta al cliente y lo conecta al server-server
// Punto de entrada para toda la logica
pub fn initialize(port: String) -> Result<(), MyError> {
    let mut current_node: Node = Node::new(&port);

    println!("Soy nodo en ip: {}", current_node.server_ip);

    let ports = read_ports_from_file(PORTS_FILE)?;

    let neighbor_nodes: Vec<NeighborNode> = connect_to_ports(ports, &port);
    current_node.add_neighbors(neighbor_nodes);

    let server_ip = current_node.puerto.clone();
    current_node.add_neighbor(NeighborNode::new(
        &server_ip,
    ));

    //let current_node = Arc::new(Mutex::new(current_node)); --> Usamos RwLock en vex de Mutex
    let current_node: Arc<RwLock<Node>> = Arc::new(RwLock::new(current_node)); //node para MAIN THREAD

    ////////////////////////////
    // FALTA HACER GOSSIP ACA //
    ////////////////////////////
    // Seccion cliente-servidor

    let current_node_client = Arc::clone(&current_node); // node para CLIENT THREAD

    let thread_cliente = thread::spawn(move || {
        let client_listener = TcpListener::bind(&current_node_client.read().unwrap().client_ip)
            .expect("Error al conectarse al puerto client_ip");

        for stream in client_listener.incoming() {
            let mut stream = stream.unwrap();

            let node_thread = Arc::clone(&current_node_client);
            let t = thread::spawn(move || {
                // bufreader queda con el ownership de stream --> usamos BufStream para solucionarlo
                let _ = handle_connection_client(node_thread, &mut stream, true);
            });
            let _ = t.join();
        }
    });

    // Seccion servidor-servidor

    let current_node_server = Arc::clone(&current_node); // node para SERVER THREAD

    let thread_servidor = thread::spawn(move || {
        let server_listener = TcpListener::bind(&current_node_server.read().unwrap().server_ip)
            .expect("Error al conectarse al puerto server_ip");

        for stream in server_listener.incoming() {
            let mut stream: TcpStream = stream.unwrap();

            let node_thread = Arc::clone(&current_node_server);
            thread::spawn(move || {
                let _ = handle_connection_server(node_thread.clone(), &mut stream, true);
            });
        }
    });

    thread_cliente.join().unwrap();
    thread_servidor.join().unwrap();
    Ok(())
}
