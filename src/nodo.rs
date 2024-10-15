use bufstream::BufStream;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use threadpool::ThreadPool;

use crate::config::{N_WORKERS, PORTS_FILE};

use crate::config::RF;
use crate::config::STRONG_CONSISTENCY;
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
    pub table: String,                // tabla del nodo
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
            table: format!("tablas/tabla{}.csv", { port }).to_string(),
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
    pub fn process_query(&self, query: &Query) -> String {
        ////////////////////////////////////////
        // HACER LA QUERY Y DEVOLVER LA TABLA //
        ////////////////////////////////////////

        println!("Procesando tabla para query {}....", query.query);

        "tabla csv en formato String".to_string()
        // "".to_string() // no se para que sirve lo de arriba, puse esto para handlear Received, se puede borrar
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
fn handle_connection_server(node: Arc<Mutex<Node>>, stream: &mut TcpStream) -> Result<(), MyError> {
    let mut buf = BufStream::new(stream);
    println!("Antes de pedir el lock en handle connection server");
    let node = node
        .lock()
        .map_err(|e| MyError::LockingError(e.to_string()))?;
    let mut line = String::new();
    println!("Despues de pedir el lock en handle connection server");
    println!("este es mi ip: {}", node.server_ip);

    while buf.read_line(&mut line).is_ok() {
        if line.is_empty() {
            continue;
        }
        println!("leyendo la puta linea: {}", line);

        let message = match node.desarialize(&line) {
            Ok(message) => message,
            Err(e) => {
                println!("Error al deserializar: {:?}", e); ////////// Romper todo o sguir?
                continue;
            }
        };

        match message.message_type {
            MessageType::Request => {
                println!(
                    "Llego un mensaje de request: {}, de ip: {}\n",
                    line, message.original_sender_ip
                );

                // Proceso la query
                let query = Query::new(message.message, message.original_sender_ip);

                let tabla = node.process_query(&query);

                // Aviso que llego el mensaje --- No necesito concetarme de nuevo, se usa el mismo Stream.
                buf.write_all(
                    format!("RECEIVED: {} : {}\n", query.sender_node_ip, line).as_bytes(),
                )
                .expect("[ERROR] no se pudo mandar RECEIVED");
                buf.flush().expect("flush no funciono");

                // Mando la tabla resultado
                buf.write_all(format!("RESULT: {} : {}\n", node.server_ip, tabla).as_bytes())
                    .expect("[ERROR] no se pudo mandar tabla");
                buf.flush().expect("flush no funciono");
            }
            MessageType::Result => {
                println!(
                    "Llego resultado: {}, de nodo: {}",
                    message.message, message.original_sender_ip
                );
                let mut respuestas: Vec<String> = Vec::new();
                respuestas.push(message.message);
                if respuestas.len() == STRONG_CONSISTENCY {
                    println!("Cumple con Quorum");
                    // mandarle respuesta al cliente
                    // FALTA READ-REPAIR
                    buf
                        .write_all(format!("{}\n", respuestas[0]).as_bytes())
                        .map_err(|e| MyError::SendingMessageError(e.to_string()))?;
                } else {
                    buf
                        .write_all("HUBO UN ERROR\n".as_bytes())
                        .map_err(|e| MyError::SendingMessageError(e.to_string()))?;
                }
            }
            MessageType::Gossip => {
                //hace gossip
                println!("Llego un mensaje de gossip: {}", line);
                // FALTA CAMBIAR EL ESTADO DE NODO VECINO A CONECTADO
                // todo!();
            }
            MessageType::Received => {
                // Todo
                println!("El mensaje fue recibido: {}", message.message);
            }
            MessageType::Test => {
                //// usar para debuggear
                println!("testeo: {:?}", message.message);
            }
        }
        line = String::new();
    }
    Ok(())
}

// Le deberia mandar la query a los nodos correspondientes!
pub fn handle_query(_query: Query) {
    // TO DO
    // println!("Llego la query: {}", query.query);
}

//🤝
// Recibe el mismo stream dos veces (clones), hay que ver de arreglarlo --> Ya no, hago el clone adentro de la funcion.
pub fn handle_connection_client(node: Arc<Mutex<Node>>, stream: TcpStream) -> Result<(), MyError> {
    // Usamos BufStream para evitar clonar el stream y usarlo en BufReader:
    let mut buf = BufStream::new(&stream);
    let mut node = node
        .lock()
        .map_err(|e| MyError::LockingError(e.to_string()))?;
    let mut line = String::new();

    while buf.read_line(&mut line).is_ok() {
        let message = Message::from_bytes(line.as_bytes());
        println!("Mensaje de cliente: {:?}\n", message);

        // en un futuro pasarlo a "otro nodo"
        let consulta = message.message;
        let mut query = Query::new(consulta, node.client_ip.clone());
        let tabla = query.execute(&node.table.clone());
        println!("Tabla: {}", tabla);

        let ans_msg = Message::new(tabla);
        let _ = buf.write_all(&ans_msg.to_bytes());
        let _ = buf.write_all(b"\n"); //sino no se manda hasta que cierra
        let _ = buf.flush();

        // let ans_msg = Message::new("Processing...".to_string());
        // let _ = buf.write(&ans_msg.to_bytes());
        // buf.write_all(b"\n"); //sino no se manda hasta que cierra
        // let _ = buf.flush();

        /////////////////////////////////////////
        // LO QUE SIGUE ABAJO DEBE ROMPER TODO //
        /////////////////////////////////////////

        ///////////////////
        // PARSEAR QUERY //
        ///////////////////

        // handle_query(query); // deberia devolver la query parseada

        // Mando query a los nodos correspondientes
        let all_results = send_query_to_nodes(&mut node, query)?;

        // Le devuelvo el resultado que me devolvieron todos los nodos al cliente
        // Luego se tiene que implementar Read-repear.
        let res = all_results.join(" - ") + "\n";
        let _ = buf.write_all(res.as_bytes());
        let _ = buf.flush();

        line = String::new();
    }
    Ok(())
}

// Envia la query a los nodos correspondientes
// HAY QUE HACERLO!! ahora se lo esta mandando a todos los nodos
pub fn send_query_to_nodes(
    node: &mut MutexGuard<'_, Node>,
    query: Query,
) -> Result<Vec<String>, MyError> {
    let mut all_response = vec![];
    let mut hash = ConsistentHash::new(RF);
    let server_ip = node.server_ip.clone();
    node.add_neighbor(NeighborNode::new(
        &server_ip.chars().last().unwrap().to_string(),
    ));
    hash.add_nodes(node.neighbors.to_vec());
    let key = query.get_key_representing_partition();
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
            let _ = node.process_query(&query);
            let response = format!(
                "RECEIVED:{}:REQUEST:{}:{}",
                node.server_ip, neighbor.server_ip, key
            );
            all_response.push(response);
            continue;
        }

        if neighbor.status == Status::Disconnected {
            println!("- Nodo desconectado: {}", neighbor.server_ip);
            continue;
        }
        if let Ok(socket) = TcpStream::connect(&neighbor.server_ip) {
            // Mando mensaje y el IP del nodo que se conecto con el cliente y en ese mismo stream leo la respuesta que me envia como resultado de la consulta.
            // - Mando mensaje:
            let mut buf = BufStream::new(socket);
            println!("- Enviando mensaje a {}", neighbor.server_ip);
            buf.write_all(format!("REQUEST:{}:{}\n", neighbor.server_ip, key).as_bytes())
                .map_err(|e| MyError::SendingMessageError(e.to_string()))?;
            let _ = buf.flush();
            println!("se envio? el mensaje a {}", neighbor.server_ip);

            // - Escucho respuesta:
            let mut response = String::new();
            let _ = buf.read_line(&mut response);
            println!(
                "El nodo: {}, nos respondio: {}\n",
                neighbor.server_ip, response
            );
            all_response.push(response);
        } //else {
          // println!("Error conectando a {}", neighbor.server_ip);
          //}
    }
    Ok(all_response)
}

// Inicializa el nodo
// Lo conecta a los otros nodos que esten abiertos y se queda escuchando
pub fn initialize(port: String) -> Result<(), MyError> {
    let mut current_node: Node = Node::new(&port);

    println!("Soy nodo en ip: {}", current_node.server_ip);

    let ports = read_ports_from_file(PORTS_FILE);

    let neighbor_nodes: Vec<NeighborNode> = connect_to_ports(ports, &port);
    current_node.add_neighbors(neighbor_nodes);

    let current_node = Arc::new(Mutex::new(current_node)); //node para MAIN THREAD

    ////////////////////////////
    // FALTA HACER GOSSIP ACA //
    ////////////////////////////

    // Seccion cliente-servidor

    let current_node_client = Arc::clone(&current_node); // node para CLIENT THREAD

    let pool_client = ThreadPool::new(N_WORKERS);

    let thread_cliente = thread::spawn(move || {
        // let client_listener = TcpListener::bind(&current_node_client.lock().map_err(|e| MyError::LockingError(e.to_string()))?.server_ip)
        //     .map_err(|_| MyError::TcpListenerError("Error al conectarse al puerto client_ip".to_string()))?;

        let client_listener = TcpListener::bind(&current_node_client.lock().unwrap().client_ip)
            .expect("Error al conectarse al puerto client_ip");

        for stream in client_listener.incoming() {
            let stream = stream.unwrap();

            let node_thread = Arc::clone(&current_node_client);
            pool_client.execute(move || {
                // bufreader queda con el ownership de stream --> usamos BufStream para solucionarlo
                let _ = handle_connection_client(node_thread, stream); ///////////// VER ALGUNA FORMA DE PASAR EL ERROR
            });
        }
        pool_client.join();
    });

    // Seccion servidor-servidor

    let current_node_server = Arc::clone(&current_node); // node para SERVER THREAD

    let pool_server = ThreadPool::new(N_WORKERS);

    let thread_servidor = thread::spawn(move || {
        // let server_listener = TcpListener::bind(&current_node_server.lock().map_err(|e| MyError::LockingError(e.to_string()))?.server_ip)
        //     .map_err(|_| MyError::TcpListenerError("Error al conectarse al puerto server_ip".to_string()))?;

        let server_listener = TcpListener::bind(&current_node_server.lock().unwrap().server_ip)
            .expect("Error al conectarse al puerto server_ip");

        for stream in server_listener.incoming() {
            let mut stream: TcpStream = stream.unwrap();

            let node_thread = Arc::clone(&current_node_server);
            pool_server.execute(move || {
                let _ = handle_connection_server(node_thread.clone(), &mut stream); ///////////// VER ALGUNA FORMA DE PASAR EL ERROR
            });
        }
        pool_server.join();
    });

    join_thread(thread_cliente)?;
    join_thread(thread_servidor)?;
    Ok(())
}
