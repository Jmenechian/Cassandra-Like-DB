use bufstream::BufStream;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::thread;
use threadpool::ThreadPool;

use crate::config::{N_WORKERS, PORTS_FILE};

use crate::config::RF;
use crate::cql::query::Query;
use crate::messagestuff::message::Message;
use crate::my_errors::*;
use crate::server_message::MessageType;
use crate::server_message::ServerMessage;
use crate::{ConsistentHash, NeighborNode};

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Connected,
    Disconnected,
}

// Representa al nodo en el que estamos parados
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub ip: String,        //temp
    pub server_ip: String, // IP donde nodo se comunica con el servidor
    pub client_ip: String, // IP donde nodo se comunica con el cliente
    pub file_path: String,
    pub nodes_path: String,
    pub neighbors: Vec<NeighborNode>, // todos los otros nodos
}

impl Node {
    pub fn new(port: &str) -> Self {
        Self {
            ip: format!("127.0.0.1:{}", port),
            server_ip: format!("127.0.0.1:700{}", port),
            client_ip: format!("127.0.0.1:800{}", port),
            //gossip_ip: puertos 900X,
            file_path: "".to_string(),
            nodes_path: "nodes.txt".to_string(),
            neighbors: vec![], // todos los nodos que estan en el ring
        }
    }

    pub fn add_neighbors(&mut self, neighbor_nodes: Vec<NeighborNode>) {
        self.neighbors = neighbor_nodes;
    }

    pub fn add_neighbor(&mut self, neighbor_node: NeighborNode) {
        self.neighbors.push(neighbor_node);
    }

    pub fn desarialize(&self, line: &str) -> Result<ServerMessage, MyError> {
        ServerMessage::new(line.to_owned())
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

fn try_connect(port: u16) -> Option<TcpStream> {
    match TcpStream::connect(format!("127.0.0.1:700{}", port)) {
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
        if try_connect(port).is_some() {
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

// Recibe un mensaje y lo procesa
// Mensajes del tipo ServerMessage
fn handle_connection_server(node: Arc<RwLock<Node>>, stream: TcpStream) -> Result<(), MyError> {
    let mut buf = BufStream::new(stream);
    let node = node.read().unwrap();
    let mut line = String::new();

    while buf.read_line(&mut line).is_ok() {
        if line.is_empty() {
            continue;
        }

        let message = match node.desarialize(&line) {
            Ok(message) => message,
            Err(e) => {
                println!("Error al deserializar: {:?}", e); //////////////////////////// Romper todo o sguir?
                continue;
            }
        };

        match message.message_type {
            MessageType::Request => {
                println!(
                    "Llego un mensaje de request: {}, de ip: {}\n",
                    line.trim(),
                    message.original_sender_ip.trim()
                );

                // Proceso la query
                let mut query = Query::new(message.message, message.original_sender_ip);

                let tabla = match node.process_query(&mut query) {
                    Ok(tabla) => tabla,
                    Err(e) => {
                        // return Err(e);
                        println!("Error al procesar la query: {:?}", e); //////////// IGNORAR QUERY INCORRECTA O DEVOLVER ERROR?
                        "tabla vacia de ERROR".to_string()
                    }
                };
                println!("tabla: {}", tabla);

                // Aviso que llego el mensaje --- No necesito concetarme de nuevo, se usa el mismo Stream.
                buf.write_all(format!("RECEIVED:{}:{}\n", query.sender_node_ip, line).as_bytes())
                    .map_err(|e| MyError::SendingMessageError(e.to_string()))?;
                buf.flush().expect("flush no funciono");

                // Mando la tabla resultado
                buf.write_all(format!("RESULT:{}:{}\n", node.server_ip, tabla).as_bytes())
                    .map_err(|e| MyError::SendingMessageError(e.to_string()))?;
                buf.flush().expect("flush no funciono");
            }
            MessageType::Gossip => {
                //hace gossip
                println!("Llego un mensaje de gossip: {}", line);
                // FALTA CAMBIAR EL ESTADO DE NODO VECINO A CONECTADO
                // todo!();
            }
            MessageType::Test => {
                //// usar para debuggear
                println!("testeo: {:?}", message.message);
            }
            _ => println!("{:?}", message.message_type),
        }
        line = String::new();
    }
    Ok(())
}

pub fn handle_connection_client(node: Arc<RwLock<Node>>, stream: TcpStream) -> Result<(), MyError> {
    // Usamos BufStream para evitar clonar el stream y usarlo en BufReader:
    let mut buf = BufStream::new(&stream);
    let node = node.read().unwrap();
    let mut line = String::new();

    while buf.read_line(&mut line).is_ok() {
        if line.is_empty() {
            line = String::new();
            continue;
        }
        let message = match Message::from_bytes(line.as_bytes()) {
            Ok(message) => message,
            Err(_) => {
                println!("Error al parsear el mensaje");
                continue;
            }
        };

        println!("Mensaje de cliente: {:?}\n", message);

        // en un futuro pasarlo a "otro nodo"
        let consulta = message.message;

        let mut query = Query::new(consulta, node.client_ip.clone());

        // let values = query.get_values(); //values lo guarda 1 sola vez
        //si es insert -> creas las lineas aca y las devolves
        //otros nodos agarran y appendean esas lineas en sus archivos

        // println!("Query: {:?}", query);

        // Mando query a los nodos correspondientes
        let all_results = match send_query_to_nodes(&node, &mut query) {
            Ok(results) => results,
            Err(e) => {
                println!("Error llamando send_query_to_nodes: {:?}", e);
                vec![]
            }
        };

        // Le devuelvo el resultado que me devolvieron todos los nodos al cliente
        // let res = process_results(all_results); //devuelve solo una respuesta, para testear sacar esto porque no anda todavia
        //consistency level / read-repair

        let message: Message = Message::create_message_as_server(all_results[0].clone(), node.client_ip.clone()); // Create the message
        println!("{:?}", message);

        let _ = buf.write_all(message.message.as_bytes()); //todo esto en una macro?
        let _ = buf.write_all(b"\n");
        let _ = buf.flush();

        line = String::new();
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

pub fn look_newer(results: &[String]) -> (&str, usize) { //falta testear en process results
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
    query: &mut Query,
) -> Result<Vec<String>, MyError> {
    let mut all_response: Vec<String> = vec![];
    let mut hash = ConsistentHash::new(RF);

    hash.add_nodes(node.neighbors.to_vec());
    let key = query.get_key_representing_partition(&node.server_ip);
    println!("La key es: {}", key);
    println!("La query es: {:?}", query);
    let nodes_with_result = hash.get_nodes(&key);

    println!(
        "Los nodos que tienen la info son: {:?}\n",
        nodes_with_result
    );

    for neighbor in nodes_with_result {
        // El nodo original porque puede tener el resultado de la query:
        if neighbor.server_ip == node.server_ip {
            println!(
                "- Buscando el resultado en el nodo original: {}",
                neighbor.server_ip
            );
            match node.process_query(query) {
                Ok(result) => {
                    all_response.push(result);
                }
                Err(e) => {
                    println!("Error al procesar la query: {:?}", e);
                    all_response.push("Error al procesar la query".to_string());
                    //////////// que hacemos aca?
                }
            }
        }

        //query::new te tira la fila a llenar

        if neighbor.status == Status::Disconnected {
            println!("- Nodo desconectado: {}", neighbor.server_ip);
            continue;
        }
        if let Ok(socket) = TcpStream::connect(&neighbor.server_ip) {
            // Mando mensaje y el IP del nodo que se conecto con el cliente y en ese mismo stream leo la respuesta que me envia como resultado de la consulta.
            // - Mando mensaje:
            let mut buf = BufStream::new(socket);
            println!("- Enviando mensaje a {}", neighbor.server_ip);
            buf.write_all(format!("REQUEST:{}:{}\n", neighbor.server_ip, query.query).as_bytes())
                .map_err(|e| MyError::SendingMessageError(e.to_string()))?;
            let _ = buf.flush();
            println!("se envio? el mensaje a {}", neighbor.server_ip);

            // - Escucho respuesta:
            let mut response = String::new();
            while buf.read_line(&mut response).is_ok() {
                if response.trim().is_empty() {
                    response = String::new();
                    continue;
                }
                let message: ServerMessage = match ServerMessage::new(response.clone()) {
                    Ok(s) => s,
                    Err(e) => {
                        println!("Error: {}", e);
                        return Ok(vec![]);
                    }
                };
                match message.message_type {
                    MessageType::Result => {
                        println!(
                            "Llego resultado: {}, de nodo: {}",
                            message.message, message.original_sender_ip
                        );
                        all_response.push(message.message);
                        break;
                    }
                    MessageType::Received => {
                        println!("El mensaje fue recibido: {}", message.message);
                        response = String::new();
                    }
                    _ => println!("Error: {}", response),
                }
                // if consistency_level == ONE && all_response.len() == 1 {
                //     process_results(all_response)
                // }
                

            }
            // if consistency_level == QUORUM && all_response.len() >= 2 {
            //     process_results(all_response)
            // } else {
            //     return "Error".to_string()
            // }
        } //else {
          // println!("Error conectando a {}", neighbor.server_ip);
          //}
    }
    Ok(all_response)
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
    let current_node = Arc::new(RwLock::new(current_node)); //node para MAIN THREAD

    ////////////////////////////
    // FALTA HACER GOSSIP ACA //
    ////////////////////////////

    // Seccion cliente-servidor

    let current_node_client = Arc::clone(&current_node); // node para CLIENT THREAD

    let pool_client = ThreadPool::new(N_WORKERS);

    let thread_cliente = thread::spawn(move || {
        let client_listener = TcpListener::bind(&current_node_client.read().unwrap().client_ip)
            .expect("Error al conectarse al puerto client_ip");

        for stream in client_listener.incoming() {
            let stream = stream.unwrap();

            let node_thread = Arc::clone(&current_node_client);
            pool_client.execute(move || {
                // bufreader queda con el ownership de stream --> usamos BufStream para solucionarlo
                match handle_connection_client(node_thread, stream) {
                    Ok(_) => println!("Conexion exitosa"),
                    Err(e) => {
                        println!("HUBO ERROR: {:?}", e);
                        panic!("Error en handle_connection_client");
                    }
                }
            });
        }
        pool_client.join();
    });

    // Seccion servidor-servidor

    let current_node_server = Arc::clone(&current_node); // node para SERVER THREAD

    let pool_server = ThreadPool::new(N_WORKERS);

    let thread_servidor = thread::spawn(move || {
        let server_listener = TcpListener::bind(&current_node_server.read().unwrap().server_ip)
            .expect("Error al conectarse al puerto server_ip");

        for stream in server_listener.incoming() {
            let stream: TcpStream = stream.unwrap();

            let node_thread = Arc::clone(&current_node_server);
            pool_server.execute(
                move || match handle_connection_server(node_thread, stream) {
                    Ok(_) => println!("Conexion exitosa"),
                    Err(e) => {
                        println!("HUBO ERROR: {:?}", e);
                        panic!("Error en handle_connection_server");
                    }
                },
            );
        }
        pool_server.join();
    });

    join_thread(thread_cliente)?;
    join_thread(thread_servidor)?;
    Ok(())
}
