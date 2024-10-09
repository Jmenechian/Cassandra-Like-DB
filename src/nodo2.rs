use chrono::prelude::*;
use rand;
use rand::prelude::*;
use std::env;
use std::f32::consts::E;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::MutexGuard;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::config::{CLIENT_PORT, PORTS_FILE, SERVER_PORT};
use crate::config::STRONG_CONSISTENCY;
use crate::query::Query;
use crate::server_message::ServerMessage;
use crate::server_message::MessageType;

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Connected,
    Disconnected,
}

// Representa a todos los otros nodos del ring
#[derive(Debug, Clone, PartialEq)]
pub struct NeighborNode {
    pub server_ip: String,
    pub status: Status,
    pub time: DateTime<Local>,
    pub primary_key: String, // para saber a quien mandarle la query
}

// Representa al nodo en el que estamos parados
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub ip: String, //temp
    pub server_ip: String, // IP donde nodo se comunica con el servidor
    pub client_ip: String, // IP donde nodo se comunica con el cliente
    pub file_path: String,
    pub nodes_path: String,
    pub neighbors: Vec<NeighborNode>, // todos los otros nodos
    pub replication_nodes: Vec<NeighborNode>, // RF
}

impl Node {
    pub fn new(port: &str, neighbor_nodes: Vec<NeighborNode>) -> Self {
        Self {
            ip: format!("127.0.0.1:{}", port),
            server_ip: format!("127.0.0.{}:{}", port, SERVER_PORT),
            client_ip: format!("127.0.0.{}:{}", port, CLIENT_PORT),
            file_path: "".to_string(),
            nodes_path: "nodes.txt".to_string(),
            neighbors: neighbor_nodes, // todos los nodos que estan en el ring
            replication_nodes: Vec::new(),
        }
    }

    pub fn desarialize(&self, line: &String) -> ServerMessage {
        let message = ServerMessage::new(line.clone()).unwrap();
        message
    }

    // Proceso la query y devuelve la tabla en formato String
    pub fn process_query(&self, query: &Query) -> String {

        ////////////////////////////////////////
        // HACER LA QUERY Y DEVOLVER LA TABLA //
        ////////////////////////////////////////

        // "tabla csv en formato String".to_string()
        "".to_string() // no se para que sirve lo de arriba, puse esto para handlear Received, se puede borrar
    }
}

// Lee los puertos del archivo
fn read_ports_from_file(filename: &str) -> Vec<u16> {
    let file = File::open(filename).expect("No se pudo abrir el archivo de puertos.");
    let reader = BufReader::new(file);
    reader
        .lines()
        .filter_map(|line| line.ok()?.parse().ok())
        .collect()
}

fn try_connect(port: u16) -> Option<TcpStream> {
    match TcpStream::connect(format!("127.0.0.{}:7000", port)) {
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
        if let Some(mut stream) = try_connect(port) {
            status = Status::Connected;
            println!("Conectado exitosamente al nodo en el puerto {}", port);
            // Avisar al nodo que me conecte??
            // Hacerlo por Gossip....
        }
        else {
            status = Status::Connected; ///////// DEBERIA EMPEZAR DESCONECTADO Y ACTUALIZARSE CON GOSSIP
            // status = Status::Disconnected;
            println!("No se pudo conectar al nodo en el puerto {}", port);
        }

        let nodo_vecino = NeighborNode {
            server_ip: format!("127.0.0.{}:7000", port),
            status: status,
            time: Local::now(),
            primary_key: "".to_string(), // falta de alguna forma conseguirlo
        };
        known_nodes.push(nodo_vecino);
    }
    known_nodes
}

// Recibe un mensaje y lo procesa
// Mensajes del tipo ServerMessage
fn handle_connection_server(node: Arc<Mutex<Node>>, stream: TcpStream) {
    let reader = BufReader::new(&stream);
    let mut lines = reader.lines();
    let node = node.lock().unwrap();

    while let Some(Ok(line)) = lines.next() {
        let message = node.desarialize(&line);
        // println!("{:?}", message);
        //message se manda 2 veces, tipo request y tipo received
        match message.message_type {
            MessageType::Request => {
                // Recibe una query CQL 
                let query = Query::new(message.message, node.server_ip.clone());
                let tabla = node.process_query(&query);

                // Mando la tabla al nodo original
                let mut socket: TcpStream = TcpStream::connect(query.sender_node_ip).unwrap(); 
                socket.write(format!("RECEIVED:{}",query.query).as_bytes()); // aviso al nodo que se recibio la query
                socket.write(tabla.as_bytes());
            }
            MessageType::Result => {
                let mut respuestas: Vec<String> = Vec::new();
                respuestas.push(message.message);
                if respuestas.len() == STRONG_CONSISTENCY {
                    // mandarle respuesta al cliente
                    // A que IP se manda?????????????????????????????????????
                    let mut socket = TcpStream::connect(&node.client_ip).unwrap();
                    // habria que chequear que las respuestas son todas iguales
                    socket.write(respuestas[0].as_bytes());
                } else {
                    // estos IPs creo que estan mal!!!!!!!!!!!!!!!
                    let mut socket = TcpStream::connect(&node.client_ip).unwrap();
                    socket.write("HUBO UN ERROR".as_bytes());
                }
            }
            MessageType::Gossip => {
                //hace gossip
                println!("Llego un mensaje de gossip: {}",line);
                // FALTA CAMBIAR EL ESTADO DE NODO VECINO A CONECTADO
                // todo!();
            }
            MessageType::Received => {
                // Todo
                println!("El mensaje fue recibido, {}", message.message);
            }
            MessageType::Test => {
                //// usar para debuggear
                println!("{:?}", message.message_type);
            }
        
        }
    }
}

// Le deberia mandar la query a los nodos correspondientes!
pub fn handle_query(query: Query) {
    // TO DO
    // println!("Llego la query: {}", query.query);
}

//🤝
pub fn handle_connection_client(node: Arc<Mutex<Node>>, stream2: TcpStream, mut stream: TcpStream) {
    let reader = BufReader::new(&stream2);
    let mut lines = reader.lines();
    let node = node.lock().unwrap();

    while let Some(Ok(line)) = lines.next() {
        println!("RECEIVED:{}", line); // query que manda cliente
    
        let res = format!("Mensaje recibido: {}, soy el nodo {}\n", line, node.client_ip).to_string();
        stream.write(res.as_bytes());
        //asi se responde al cliente, guardar el stream para responder despues?

        let query = Query::new(line, node.client_ip.clone()); 

        ///////////////////
        // PARSEAR QUERY //
        ///////////////////

        handle_query(query.clone()); // deberia devolver la query parseada
        send_query_to_nodes(&node.neighbors, query);
    }
}

// Envia la query a los nodos correspondientes
// HAY QUE HACERLO!! ahora se lo esta mandando a todos los nodos
pub fn send_query_to_nodes(neighbors: &Vec<NeighborNode>, query: Query) {
    for neighbor in neighbors {
        if neighbor.server_ip == query.sender_node_ip {
            // ignoramos al que mando la query original
            println!("Nodo original: {}", neighbor.server_ip);
            continue;
        }
        if neighbor.status == Status::Disconnected {
            println!("Nodo desconectado: {}", neighbor.server_ip);
            continue;
        }
        if let Ok(mut socket) = TcpStream::connect(&neighbor.server_ip) {
            socket.write(format!("REQUEST:{}", query.query).as_bytes()).unwrap();
            println!("Enviando mensaje a {}", neighbor.server_ip);
        } else {
            println!("Error conectando a {}", neighbor.server_ip);
        }
    }
}

// Inicializa el nodo
// Lo conecta a los otros nodos que esten abiertos y se queda escuchando
pub fn initialize(port: String) {
    let ports = read_ports_from_file(PORTS_FILE);

    let neighbor_nodes: Vec<NeighborNode> = connect_to_ports(ports, &port);

    let current_node: Node = Node::new(&port, neighbor_nodes);

    println!("Soy nodo en ip: {}", current_node.server_ip);

    let current_node = Arc::new(Mutex::new(current_node)); //node para MAIN THREAD

    ////////////////////////////
    // FALTA HACER GOSSIP ACA //
    ////////////////////////////

    let current_node_client = Arc::clone(&current_node); // node para CLIENT THREAD

    // Single thread cliente-servidor
    // Hay que hacer un thread pool!!!!!!!
    let thread_cliente = thread::spawn(move || {

        let client_listener = TcpListener::bind(&current_node_client.lock().unwrap().client_ip).expect("Error al conectarse al puerto client_ip");

        for stream in client_listener.incoming() {
            let mut stream = stream.unwrap();
            let stream2 = stream.try_clone().unwrap();

            // bufreader queda con el ownership de stream, por ahora clono y uso 2

            handle_connection_client(current_node_client.clone(), stream2, stream);
            
        }
    });

    let current_node_server = Arc::clone(&current_node); // node para SERVER THREAD
    
    // Single thread servidor-servidor
    // Escucha mensajes de otros nodos
    let thread_servidor = thread::spawn(move || {
        let server_listener = TcpListener::bind(&current_node_server.lock().unwrap().server_ip).expect("Error al conectarse al puerto server_ip");

        for stream in server_listener.incoming() {
            let stream: TcpStream = stream.unwrap();
            handle_connection_server(current_node_server.clone(), stream);
        }
    });

    // Handle user input in the main thread
    // Escucha mensajes de terminal
    // loop {
    //     let mut input = String::new();
    //     print!("Enter a query/message: ");
    //     io::stdout().flush().unwrap(); // Make sure the prompt is shown
    //     io::stdin().read_line(&mut input).expect("Failed to read input");

    //     let input = input.trim().to_string();
    //     if input.is_empty() {
    //         continue; // Skip if no input is provided
    //     }

    //     if input == "exit" {
    //         break; // Exit the loop when the user types 'exit'
    //     }

    //     // Broadcast the input to other nodes (for example, as a query)
    //     let query = Query::new(input.clone(), current_node.lock().unwrap().client_ip.clone());
    //     println!("tamos aca");
    //     gossip(&current_node, query);
    // }

    thread_cliente.join().unwrap();
    thread_servidor.join().unwrap();

}