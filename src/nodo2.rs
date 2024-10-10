use chrono::prelude::*;
use rand;
use rand::prelude::*;
use std::{env, result};
use std::f32::consts::E;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::MutexGuard;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;
use bufstream::BufStream;

use crate::config::{CLIENT_PORT, N_WORKERS, PORTS_FILE, SERVER_PORT};
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
    pub fn new(port: &str) -> Self {
        Self {
            ip: format!("127.0.0.1:{}", port),
            server_ip: format!("127.0.0.1:700{}", port),
            client_ip: format!("127.0.0.1:800{}", port),
            //gossip_ip: puertos 900X,
            file_path: "".to_string(),
            nodes_path: "nodes.txt".to_string(),
            neighbors: vec![], // todos los nodos que estan en el ring
            replication_nodes: Vec::new(),
        }
    }

    pub fn add_neighbor(&mut self, neighbor_nodes: Vec<NeighborNode>) {
        self.neighbors = neighbor_nodes;
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
        
        println!("Procesando tabla...");

        "tabla csv en formato String".to_string()
        // "".to_string() // no se para que sirve lo de arriba, puse esto para handlear Received, se puede borrar
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
            server_ip: format!("127.0.0.1:700{}", port),
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
fn handle_connection_server(node: Arc<Mutex<Node>>, stream: &mut TcpStream) {
    let mut buf = BufStream::new(stream);
    let node = node.lock().unwrap();
    let mut line = String::new();

    while buf.read_line(&mut line).is_ok() {
        let message = node.desarialize(&line);
        match message.message_type {
            MessageType::Request => {
                println!("Llego un mensaje de request: {}, de ip: {}\n",line, message.original_sender_ip);
 
                // Proceso la query
                let query = Query::new(message.message, message.original_sender_ip);
                // let node_clone = node.clone();
                
                // nueva thread para mandar respuesta
                // thread::spawn(move || {
                //     let tabla = node_clone.process_query(&query);

                //     let mut socket = TcpStream::connect(&query.sender_node_ip).unwrap();

                //     // Aviso que llego el mensaje
                //     socket.write(format!("RECEIVED:{}:{}", query.sender_node_ip, line).as_bytes()).expect("[ERROR] no se pudo mandar RECEIVED");
                //     socket.flush().expect("flush no funciono");


                //     // Mando la tabla resultado
                //     socket.write(format!("RESULT:{}:{}", query.sender_node_ip, tabla).as_bytes()).expect("[ERROR] no se pudo mandar tabla");
                //     socket.flush().expect("flush no funciono");
                // });


                // ESTOS WRITES CREO QUE HAY QUE METERLOS EN OTRO THREAD --> No, se hace por el mismo stream.
                let tabla = node.process_query(&query);

                // Aviso que llego el mensaje --- No necesito concetarme de nuevo, se usa el mismo Stream.
                buf.write(format!("RECEIVED: {} : {}\n",query.sender_node_ip, line).as_bytes()).expect("[ERROR] no se pudo mandar RECEIVED"); 
                buf.flush().expect("flush no funciono");

                // Mando la tabla resultado
                buf.write(format!("RESULT: {} : {}\n",query.sender_node_ip, tabla).as_bytes()).expect("[ERROR] no se pudo mandar tabla");
                buf.flush().expect("flush no funciono");
            }
            MessageType::Result => {
                println!("Llego resultado: {}, de nodo: {}", message.message, message.original_sender_ip);
                let mut respuestas: Vec<String> = Vec::new();
                respuestas.push(message.message);
                if respuestas.len() == STRONG_CONSISTENCY {
                    println!("Cumple con Quorum");
                    // mandarle respuesta al cliente
                    // A que IP se manda?????????????????????????????????????
                    //let mut socket = TcpStream::connect(&message.original_sender_ip).unwrap();
                    // habria que chequear que las respuestas son todas iguales
                    buf.write(format!("{}\n",respuestas[0]).as_bytes()).expect("[ERROR] no se pudo mandar tabla a cliente");
                } else {
                    // estos IPs creo que estan mal!!!!!!!!!!!!!!!
                    //let mut socket = TcpStream::connect(&message.original_sender_ip).unwrap();
                    let _ = buf.write("HUBO UN ERROR\n".as_bytes());
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
                println!("El mensaje fue recibido: {}", message.message);
            }
            MessageType::Test => {
                //// usar para debuggear
                //println!("{:?}", message.message);
            }
        }
        line = String::new();
    }
}

// Le deberia mandar la query a los nodos correspondientes!
pub fn handle_query(query: Query) {
    // TO DO
    // println!("Llego la query: {}", query.query);
}

//🤝
// Recibe el mismo stream dos veces (clones), hay que ver de arreglarlo --> Ya no, hago el clone adentro de la funcion.
pub fn handle_connection_client(node: Arc<Mutex<Node>>, stream: TcpStream) {
    // Usamos BufStream para evitar clonar el stream y usarlo en BufReader:
    let mut buf = BufStream::new(&stream);
    let node = node.lock().unwrap();
    let mut line = String::new();

    while buf.read_line(&mut line).is_ok() {
        
        println!("Mensaje de cliente: {}\n", line);
        
        // Le digo al cliente que recibi su query
        let res = format!("Mensaje recibido: {}, soy el nodo {}\n", line, node.client_ip).to_string();
        let _ = buf.write(res.as_bytes());
        let _ = buf.flush();
        //asi se responde al cliente, guardar el stream para responder despues?

        let query = Query::new(line.clone(), node.client_ip.clone()); 

        ///////////////////
        // PARSEAR QUERY //
        ///////////////////

        handle_query(query.clone()); // deberia devolver la query parseada

        // Mando query a los nodos correspondientes
        let all_results = send_query_to_nodes(&node.server_ip, &node.neighbors, query);

        // Le devuelvo el resultado que me devolvieron todos los nodos al cliente 
        // Luego se tiene que implementar Read-repear.
        let res = all_results.join(" - ");
        let _ = buf.write(res.as_bytes());
        let _ = buf.flush();

        line = String::new();
    }
}

// Envia la query a los nodos correspondientes
// HAY QUE HACERLO!! ahora se lo esta mandando a todos los nodos
pub fn send_query_to_nodes(node_server_ip: &str ,neighbors: &Vec<NeighborNode>, query: Query) -> Vec<String> {
    let mut all_response = vec![];
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
        if let Ok(socket) = TcpStream::connect(&neighbor.server_ip) {
            // Mando mensaje y el IP del nodo que se conecto con el cliente y en ese mismo stream leo la respuesta que me envia como resultado de la consulta.
            // - Mando mensaje:
            let mut buf = BufStream::new(socket);
            println!("Enviando mensaje a {}", neighbor.server_ip);
            buf.write(format!("REQUEST:{}:{}\n",node_server_ip, query.query).as_bytes()).unwrap();
            let _ = buf.flush();

            // - Escucho respuesta:
            let mut response = String::new();
            let _ = buf.read_line(&mut response);
            println!("El nodo: {}, nos respondio: {}\n", neighbor.server_ip, response);
            all_response.push(response);
        } //else {
            // println!("Error conectando a {}", neighbor.server_ip);
        //}
    }
    all_response
}

// Inicializa el nodo
// Lo conecta a los otros nodos que esten abiertos y se queda escuchando
pub fn initialize(port: String) {
    let mut current_node: Node = Node::new(&port);

    println!("Soy nodo en ip: {}", current_node.server_ip);

    let ports = read_ports_from_file(PORTS_FILE);

    let neighbor_nodes: Vec<NeighborNode> = connect_to_ports(ports, &port);
    current_node.add_neighbor(neighbor_nodes);

    let current_node = Arc::new(Mutex::new(current_node)); //node para MAIN THREAD

    ////////////////////////////
    // FALTA HACER GOSSIP ACA //
    ////////////////////////////

    let current_node_client = Arc::clone(&current_node); // node para CLIENT THREAD

    // Single thread cliente-servidor
    // Hay que hacer un thread pool!!!!!!! --> Lo agregué: lo usamos para controlar la máxima cantidad de thread que podemos tener.
    
    let pool_client = ThreadPool::new(N_WORKERS);

    let thread_cliente = thread::spawn(move || {

        let client_listener = TcpListener::bind(&current_node_client.lock().unwrap().client_ip).expect("Error al conectarse al puerto client_ip");

        for stream in client_listener.incoming() {
            let stream = stream.unwrap();

            let node_thread = Arc::clone(&current_node_client);
            pool_client.execute(move|| {
                // bufreader queda con el ownership de stream --> usamos BufStream para solucionarlo
                handle_connection_client(node_thread, stream);
            });
        }
    });

    let current_node_server = Arc::clone(&current_node); // node para SERVER THREAD

    let pool_server = ThreadPool::new(N_WORKERS);
    
    // Single thread servidor-servidor
    // Escucha mensajes de otros nodos
    let thread_servidor = thread::spawn(move || {
        let server_listener = TcpListener::bind(&current_node_server.lock().unwrap().server_ip).expect("Error al conectarse al puerto server_ip");

        for stream in server_listener.incoming() {
            let mut stream: TcpStream = stream.unwrap();

            let node_thread = Arc::clone(&current_node_server);
            pool_server.execute(move|| {
                handle_connection_server(node_thread.clone(), &mut stream);
            });
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
