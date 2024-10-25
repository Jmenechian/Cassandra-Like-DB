use chrono::prelude::*;
use bufstream::BufStream;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use threadpool::ThreadPool;
use rand::prelude::IteratorRandom;
use crate::config::{self, N_WORKERS, PORTS_FILE};
use rand::seq::SliceRandom;
use crate::config::RF;
use crate::cql::query::Query;
use crate::messagestuff::{
    message::*,
    message_cql::{
        cqlenvelope::*,
        querymessage::*,
        errormessage::*,
        resultmessage::*,
        resultrowsmessage::*,
        resultvoidmessage::*,
    },
    message_server::{
        serverpacket::Packet,
        serverpacket::ServerMessage,
        queryservermessage::QueryServerMessage,
        resultservermessage::*,
        resultvoidservermsg::*,
        resultrowsevermsg::*,
        gossipmessage::*,
    },
};
use crate::my_errors::*;
use crate::{ConsistentHash, NeighborNode};

#[derive(Debug, Clone, PartialEq, Copy)]
#[repr(u8)]
pub enum Status {
    Connected = config::CONNECTED,
    Disconnected = config::DISCONNECTED,
}

impl Status {
    pub fn from_u8(value: u8) -> Result<Self,MyError> {
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
    pub ip: String,        //temp
    pub puerto: String,
    pub server_ip: String, // IP donde nodo se pone a escuchar conecciones de servidores
    pub client_ip: String, // IP donde nodo se pone a escuchar conecciones de cliente
    pub gossip_ip: String,  // IP donde nodo se pone a escuchar mensajes Gossip
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
        if let Some(pos) = self.neighbors.iter().position(|n| n.puerto == neighbor_node.puerto) {
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
        
        let filtered: Vec<&NeighborNode> = self.neighbors.iter().filter(|n| n.puerto != my_port).collect();
        filtered.choose(&mut rng).map(|&n| n)
    }

    // Proceso la query y devuelve la tabla en formato String
    pub fn process_query(&self, query: &mut Query) -> Result<String, MyError> {
        query.execute()
    }
}

// Lee los puertos del archivo
fn read_ports_from_file(filename: &str) -> Vec<u16> {
    let file = File::open(filename)
        .map_err(|e| MyError::FileError(e.to_string()))
        .unwrap();

    let reader = BufReader::new(file);
    reader
        .lines()
        .filter_map(|line| line.ok()?.parse().ok())
        .collect()
}

fn try_connect(ip: String) -> Option<TcpStream> {
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
        if try_connect(format!{"127.0.0.1:700{}", port}).is_some() {
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

fn handle_connection_server(node:Arc<RwLock<Node>>, stream: &mut TcpStream, show: bool) -> Result<(), MyError> {
    loop {
        match Packet::from_stream(stream) {
            Ok(packet) => {
                if show{println!("S   El packet recibido es: {:?}", packet);}
                match packet.body {
                    ServerMessage::Query(query) => {
                        let node = match node.read() {
                            Ok(n) => n,
                            Err(e) => {
                                if show{println!("S   Error al tratar de usar el nodo: {:?}", e);}
                                return Ok(());
                            }
                        };
                        let mut query = Query::new(query.query.clone(), node.server_ip.clone());
                        let table = match query.execute() {
                            Ok(s) => {
                                println!("{}", s);
                                s
                            },
                            Err(_) => "".to_string(),
                        };
                        println!("RESULTADO, TABLA: {}", table);
                        let msg = ResultRowsServerMessage{rows_count: 1, rows_content: vec![table]};
                        let _ = msg.send_server_through(stream);
                    }
                    _ => {
                        if show{println!("S   Error entre server y servers");}
                    }
                }
            }
            Err(e) => {
                if show{println!("S   Error al leer el packet: {:?}", e);}
                break;
            }
        }
    }
    Ok(())
}

// Le deberia mandar la query a los nodos correspondientes!
pub fn handle_query(_query: Query) {
    // TO DO
    // println!("Llego la query: {}", query.query);
}

pub fn handle_connection_client (node: Arc<RwLock<Node>>, stream: & mut TcpStream, show: bool) -> Result<(), MyError> {
    let node = match node.read() {
        Ok(n) => n,
        Err(e) => {
            if show{println!("C   Error al tratar de usar el nodo: {:?}", e);}
            return Ok(());
        }
    };
    
    loop {
        match Envelope::from_stream(stream) {
            Ok(env) => {
                if show{println!("C   El env recibido es: {:?}", env);}
                match env.body {
                    CqlMessage::Query(querymessage) => {
                        let mut query = Query::new(querymessage.query.clone(), node.client_ip.clone());
                        let table = match send_query_to_nodes(&node, querymessage.query.clone(), show) {
                            Ok(r) => r,
                            Err(e) => {
                                let _error = "Error: algo fallo en send_query_to_node".to_string();
                                let msg = ErrorMessage{error: _error, code: 0 as i32};
                                if let Err(e) = msg.send_through(stream) {
                                    if show{println!("C   Error an enviarle un ErrorMessage al cliente: {:?}", e);}
                                }
                                return Err(e)
                            }
                        };
                        if table.len() == 0 {
                            let void_msg = ResultVoidMessage;
                            if let Err(e) = void_msg.send_through(stream) {
                                if show{println!("C   Error: {:?}", e);}
                            }
                        } else {
                            let content: Vec<String> = table[0].clone().trim_end().split("     ").map(|s| s.to_string()).collect();
                            let rows_msg = ResultRowsMessage{rows_count: content.len() as i32, rows_content:content};
                            if let Err(e) = rows_msg.send_through(stream) {
                                if show{println!("C   Error: {:?}", e);}
                            }
                        }
                    }
                    _ => {
                        let _error = "Error: Solo me puede llegar un QueryMessage".to_string();
                        let msg = ErrorMessage{error: _error, code: 0 as i32};
                        if let Err(e) = msg.send_through(stream) {
                            if show{println!("C   Error an enviarle un ErrorMessage al cliente: {:?}", e);}
                        }
                    }
                }
            }
            Err(e) => {
                if show{println!("C    Error al leer el env: {:?}", e);}
                break;
            }
        }
    }
    Ok(())
}

//mapa
//hago click en AEP con fecha
//aviones volando -> threads
//hago click en NY

// fn process_results(results: &Vec<String>) -> String {
//     println!("pr ===> {:?}", results);

//     match results.len() {
//         1 => return results[0].clone(),
//         2 => {
//             let (a, b) = (&results[0], &results[1]);
//             if a == b {
//                 return a.clone() //no hay que hacer nada mas creo
//             } else {
//                 // let newer = look_newer(results);
//                 // read_repair(newer, ) //update a todos los nodos
//                 // return newer
//             }
//             }
//         3 => {
//             let (a, b, c) = (&results[0], &results[1], &results[2]);
//             // return look_newer(vec![a.clone(), b.clone(), c.clone()]);
//         }
//         _ => return "error".to_string(), //rompio todo
//     }
//     results[0].clone()
// }

pub fn look_newer(results: &[String]) -> (&str, usize) {
    //falta testear en process results
    let mut newer = "";
    let mut index = 0;
    for (i, result) in results.iter().enumerate() {
        let filas: Vec<&str> = result.split("     ").collect::<Vec<&str>>();
        for timestamp in filas.iter().filter(|p| p.contains("UTC")) {
            if timestamp > &newer {
                newer = timestamp;
                index = i;
            }
        }
    }
    (newer, index)
}
// Envia la query a los nodos correspondientes
// HAY QUE HACERLO!! ahora se lo esta mandando a todos los nodos
pub fn send_query_to_nodes(
    node: &RwLockReadGuard<'_, Node>,
    _query: String,
    show: bool,
) -> Result<Vec<String>, MyError> {
    let mut all_response: Vec<String> = vec![];
    let mut hash = ConsistentHash::new(RF);
    let mut query = Query::new(_query.clone(), node.client_ip.clone());
    hash.add_nodes(node.neighbors.to_vec());
    let key = query.get_key_representing_partition(&node.server_ip);
    println!("La key es: {}", key);
    println!("La query es: {:?}", query);
    let nodes_with_result = hash.get_nodes(&key);

    if show{println!(
        "
        C   vecinos: {:?}
        C   Los nodos que tienen la info son: {:?}\n",
        node.neighbors, nodes_with_result
    );}

    for neighbor in nodes_with_result {
        // El nodo original porque puede tener el resultado de la query:
        if neighbor.server_ip == node.server_ip {
            if show{println!(
                "C   - Buscando el resultado en el nodo original: {}",
                neighbor.server_ip
            );}
            match node.process_query(&mut query) {
                Ok(result) => {
                    all_response.push(result);
                }
                Err(e) => {
                    if show{println!("C   Error al procesar la query: {:?}", e);}
                    all_response.push("Error al procesar la query".to_string());
                    //////////// que hacemos aca?
                }
            }
            continue;
        } 
        else if neighbor.status == Status::Disconnected {
            if show{println!("C   - Nodo desconectado: {}", neighbor.server_ip);}
            continue;
        }
        else if let Ok(mut socket) = TcpStream::connect(&neighbor.server_ip) {
            // Mando mensaje y el IP del nodo que se conecto con el cliente y en ese mismo stream leo la respuesta que me envia como resultado de la consulta.
            // - Mando mensaje:
            let msg = QueryServerMessage { query: _query.clone(), consistency: 0 };
            let _ = msg.send_server_through(&mut socket);
            loop {
                match Packet::from_stream(& mut socket) {
                    Ok(packet) => {
                        if show{println!("C   El packet recibido es: {:?}", packet);}
                        match packet.body {
                            ServerMessage::Result(resultmessage) => {
                                match resultmessage.body {
                                    ResultServerBody::Rows(r) => {
                                        all_response.push(r.rows_content[0].clone());
                                    },
                                    ResultServerBody::Void(_)=> {}
                                }
                                break;
                            }
                            _ => {
                                if show{println!("C   Error entre server y servers");}
                            }
                        }
                    }
                    Err(e) => {
                        if show{println!("C   Error al leer el paket: {:?}", e);}
                    }
                }
            }
        } 
    }
    Ok(all_response)
}

fn handle_connection_gossip(node:Arc<RwLock<Node>>, stream: &mut TcpStream, show: bool) -> Result<(), MyError> {
    if show{println!("G   Entro al handle_connection_gossip");}
    match Packet::from_stream(stream) {
        Ok(packet) => {
            if show{println!("G   El packet recibido es: {:?}", packet);}
            match packet.body {
                ServerMessage::Gossip(mut gossip) => {
                    let ip: String = gossip.ip.clone();
                    let mut node = match node.write() {
                        Ok(n) => n,
                        Err(e) => {
                            if show {println!("G   Error al tratar de usar el nodo: {:?}", e);}
                            return Err(MyError::LockingError("No puedo usar el nodo".to_string()));
                        }
                    };
                    
                    if let Some(true) = gossip.decrease_ttl() {
                        if let Some(n) = node.get_random_neighbor(node.puerto.clone()) {
                            if let Some(mut stream) = try_connect(n.gossip_ip.clone()) {
                                //thread::sleep(Duration::from_secs(1));
                                if let Err(e) = gossip.send_server_through(&mut stream) {
                                    if show{println!("G   Error al evialr mensaje gossip: {:?}", e);}
                                }
                            } else {
                                if show{println!("G   Error al conectarse al vecino");}
                            }
                        }
                    }
                    let new = node.add_neighbor(NeighborNode::new(
                        &ip.chars().last().unwrap().to_string(),
                    ));
                    if show && new {println!("G   Mis vecinos son: {:?}", node.neighbors);}
                }
                _ => {
                    if show{println!("G   Error entre gossip y gossip");}
                }
            }
        }
        Err(e) => {
            let error = format!("G   Error al leer el packet: {:?}", e);
            if show{println!("{}", error);}
            return Err(MyError::Error(error))
        }
    }
    Ok(())
}

fn connect_to_random_gossip(ports: Vec<u16>, my_port: &String, show: bool) -> Option<(TcpStream, String)> {
    let mut shuffeld_ports = ports.clone();
    shuffeld_ports.shuffle(&mut rand::thread_rng());
    if show{println!("MEZCLO y me queda: {:?}, siendo y el: {:?}", shuffeld_ports, my_port);}
    for &port in &shuffeld_ports {
        if &port.to_string() == my_port {continue};
        let addr = format!("127.0.0.1:900{}", port);
        if let Some (mut stream) = try_connect(addr.clone()) {
            if show{println!("El primero que encontre fue el: {:?}", addr);}
            return Some((stream, addr));
        }
    }
    None
}

fn listen<F> (ip: String, node: Arc<RwLock<Node>>, handler: F, show: bool) 
-> Result<JoinHandle<Result<(), MyError>>, MyError> 
where F: Fn(Arc<RwLock<Node>>, &mut TcpStream, bool) -> Result<(), MyError> + Send + 'static + Copy  {
    let current_node = Arc::clone(&node);
    let t = thread::spawn(move || {
        let listener = match  TcpListener::bind(ip) {
            Ok(j) => j,
            Err(_) => return Err(MyError::TcpListenerError("Error al conectarse al puerto".to_string())),
        };
        for stream in listener.incoming() {
            let mut s: TcpStream = match stream {
                Ok(s) => s,
                Err(_) => return Err(MyError::TcpStreamError("Error al leer TcpStream".to_string())),
            };
            let node_thread = Arc::clone(&current_node);
            thread::spawn(move || {
                match handler(node_thread.clone(), &mut s, show) {
                    Ok(_) => {},
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
    current_node.add_neighbor(NeighborNode::new(
        &current_node.puerto,
    ));

    //empiezo gossip
    let ports = read_ports_from_file(PORTS_FILE);
    let my_ip = current_node.server_ip.clone();
    match connect_to_random_gossip(ports.clone(), &port, show_g) {
        Some((mut s, addr)) => {
            let gossip_msg = GossipMessage{ip: my_ip.clone(), estado: Status::Connected, ttl: 5 as u8};
            let _ = gossip_msg.send_server_through(&mut s);
            let new = current_node.add_neighbor(NeighborNode::new(
                &addr.chars().last().unwrap().to_string(),
            ));
            if show_g && new {println!("G   Mis vecinos son: {:?}", current_node.neighbors);}
        },
        None => println!("tengo que ser el primer nodo porque no encontre otro"),
    };
    
    let current_node: Arc<RwLock<Node>> = Arc::new(RwLock::new(current_node)); //node para MAIN THREAD
    // Seccion cliente-servidor
    listen(current_node.read().unwrap().client_ip.clone(), current_node.clone(), handle_connection_client, show_c)?;

    // Seccion servidor-servidor
    listen(current_node.read().unwrap().server_ip.clone(), current_node.clone(), handle_connection_server, show_s)?;

    // Seccion Gossip
    listen(current_node.read().unwrap().gossip_ip.clone(), current_node.clone(), handle_connection_gossip, show_g)?;


    loop {
        thread::sleep(Duration::from_secs(10));
        match connect_to_random_gossip(ports.clone(), &port, show_g) {
            Some((mut s, _)) => {
                let gossip_msg = GossipMessage{ip: my_ip.clone(), estado: Status::Connected, ttl: 5 as u8};
                let _ = gossip_msg.send_server_through(&mut s);
            },
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

    let ports = read_ports_from_file(PORTS_FILE);

    let neighbor_nodes: Vec<NeighborNode> = connect_to_ports(ports, &port);
    current_node.add_neighbors(neighbor_nodes);

    let server_ip = current_node.server_ip.clone();
    current_node.add_neighbor(NeighborNode::new(
        &server_ip.chars().last().unwrap().to_string(),
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
            let t = thread::spawn(move ||  {
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
