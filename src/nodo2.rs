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

static GOSSIP: i64 = 3;

#[derive(Debug, Clone, PartialEq)]
pub enum Estado {
    Activo,
    Levantando,
    Caído,
    Cayendo,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Nodo {
    pub puerto: u16,
    pub estado: Estado,
    pub time: DateTime<Local>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Mensaje {
    pub contenido: String,
}

fn read_ports_from_file(filename: &str) -> Vec<u16> {
    let file = File::open(filename).expect("No se pudo abrir el archivo de puertos.");
    let reader = BufReader::new(file);
    reader
        .lines()
        .filter_map(|line| line.ok()?.parse().ok())
        .collect()
}

fn serialize_nodo(nodo: &Nodo) -> String {
    format!(
        "Nodo:{}:{}:{}",
        nodo.puerto,
        match nodo.estado {
            Estado::Activo => "Activo",
            Estado::Levantando => "Levantando",
            Estado::Caído => "Caído",
            Estado::Cayendo => "Cayendo",
        },
        nodo.time.to_rfc3339(),
    )
}

fn deserialize_nodo(data: &str) -> Option<Nodo> {
    let mut parts: Vec<&str> = data.split(':').collect();

    if parts.len() >= 4 && parts[0] == "Nodo" {
        let puerto = parts[1].parse::<u16>().ok()?;
        let estado = match parts[2] {
            "Activo" => Estado::Activo,
            "Levantando" => Estado::Levantando,
            "Caído" => Estado::Caído,
            "Cayendo" => Estado::Cayendo,
            _ => return None,
        };
        let time = parts.split_off(3);
        let time = time.join(":");
        
        let time = DateTime::parse_from_rfc3339(&time).ok()?.into();
        Some(Nodo {
            puerto,
            estado,
            time,
        })
    } else {
        None
    }
}

fn serialize_mensaje(mensaje: &Mensaje) -> String {
    format!("Mensaje:{}", mensaje.contenido)
}

fn deserialize_mensaje(data: &str) -> Option<Mensaje> {
    if data.starts_with("Mensaje:") {
        Some(Mensaje {
            contenido: data["Mensaje:".len()..].to_string(),
        })
    } else {
        None
    }
}

fn try_connect(port: u16) -> Option<TcpStream> {
    match TcpStream::connect(format!("127.0.0.1:{}", port)) {
        Ok(stream) => Some(stream),
        Err(_) => None,
    }
}

fn handle_user_input(known_nodes: Arc<Mutex<Vec<u16>>>, current_port: u16) {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(msg) = line {
            let message = Mensaje { contenido: msg };
            let serialized_msg = serialize_mensaje(&message);

            let mut nodes_to_remove = vec![];
            {
                let nodes = known_nodes.lock().unwrap();
                for &node_port in nodes.iter() {
                    if let Ok(mut stream) = TcpStream::connect(format!("127.0.0.1:{}", node_port)) {
                        if stream.write_all(serialized_msg.as_bytes()).is_err() {
                            println!("No se pudo enviar mensaje a {}", node_port);
                        }
                    } else {
                        println!("No se pudo conectar al nodo en el puerto {}", node_port);
                        nodes_to_remove.push(node_port);
                    }
                }
            }

            if !nodes_to_remove.is_empty() {
                let mut nodes = known_nodes.lock().unwrap();
                for port in nodes_to_remove {
                    if let Some(pos) = nodes.iter().position(|&x| x == port) {
                        nodes.remove(pos);
                    }

                    println!(
                        "El nodo {} se eliminó de la lista de nodos conocidos.",
                        port
                    );
                    gossip2(
                        &nodes,
                        serialize_nodo(&Nodo {
                            puerto: port,
                            estado: Estado::Caído,
                            time: Local::now(),
                        }),
                    );
                }
            }
        }
    }
}

fn gossip2(nodes: &MutexGuard<'_, Vec<u16>>, serialized_nodo: String) {
    {
        let mut rng = rand::thread_rng();
        let mut nodos_shuffle = nodes.to_vec();
        nodos_shuffle.shuffle(&mut rng);

        for &node_port in nodos_shuffle.iter().take(3) {
            if let Some(mut stream) = try_connect(node_port) {
                if stream.write_all(serialized_nodo.as_bytes()).is_err() {
                    println!(
                        "Error enviando mensaje de estado inicial al nodo en el puerto {}",
                        node_port
                    );
                }
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, known_nodes: Arc<Mutex<Vec<u16>>>) {
    let mut buffer = [0; 512];
    while match stream.read(&mut buffer) {
        Ok(size) if size > 0 => {
            let data = String::from_utf8_lossy(&buffer[..size]);
            if let Some(nodo) = deserialize_nodo(&data) {
                println!("Nodo conectado: {:?}", nodo);

                let mut nodes = known_nodes.lock().unwrap();
                if !nodes.contains(&nodo.puerto) {
                    if nodo.estado != Estado::Caído {
                        nodes.push(nodo.puerto);
                        println!(
                            "Nodo {} agregado a la lista de nodos conocidos.",
                            nodo.puerto
                        );
                        gossip2(&nodes, serialize_nodo(&nodo));
                    }
                }
            } else if let Some(msg) = deserialize_mensaje(&data) {
                println!("Mensaje recibido: {:?}", msg);
            } else {
                println!("Mensaje desconocido: {}", data);
            }
            true
        }
        _ => false,
    } {}
}

fn start_node_listener(port: u16, known_nodes: Arc<Mutex<Vec<u16>>>) {
    let listener =
        TcpListener::bind(format!("127.0.0.1:{}", port)).expect("No se pudo enlazar al puerto.");
    println!("Nodo escuchando en el puerto: {}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let addr = stream.peer_addr().unwrap();
                println!("Nueva conexión desde {}", addr);

                let known_nodes = Arc::clone(&known_nodes);

                thread::spawn(move || {
                    handle_connection(stream, known_nodes);
                });
            }
            Err(e) => {
                println!("Error al aceptar conexión: {}", e);
            }
        }
    }
}

fn connect_to_ports(ports: Vec<u16>, known_nodes: &mut Vec<u16>) -> Option<u16> {
    let mut port_to_listen = None;

    for &port in &ports {
        if let Some(mut stream) = try_connect(port) {
            known_nodes.push(port);
            println!("Conectado exitosamente al nodo en el puerto {}", port);

            let nodo_actual = Nodo {
                puerto: port,
                estado: Estado::Levantando,
                time: Local::now(),
            };
            let serialized_nodo = serialize_nodo(&nodo_actual);
            if stream.write_all(serialized_nodo.as_bytes()).is_err() {
                println!("Error enviando mensaje al nodo en el puerto {}", port);
            }
        } else if port_to_listen.is_none() {
            port_to_listen = Some(port);
        }
    }
    return port_to_listen;
}

fn gossip(known_nodes: Arc<Mutex<Vec<u16>>>, serialized_nodo: String) {
    {
        let mut nodes = known_nodes.lock().unwrap();
        let mut rng = rand::thread_rng();
        let mut nodos_shuffle = nodes.to_vec();
        nodos_shuffle.shuffle(&mut rng);

        for &node_port in nodos_shuffle.iter().take(1) {
            if let Some(mut stream) = try_connect(node_port) {
                if stream.write_all(serialized_nodo.as_bytes()).is_err() {
                    println!(
                        "Error enviando mensaje de estado inicial al nodo en el puerto {}",
                        node_port
                    );
                }
            }
        }
    }
}

pub fn initialize(ports_file: &String) {
    let ports = read_ports_from_file(ports_file);
    let mut known_nodes = Vec::new();

    let mut port_to_listen = connect_to_ports(ports, &mut known_nodes);

    let port_to_listen = port_to_listen.expect("No se encontró un puerto libre para escuchar.");

    let known_nodes: Arc<Mutex<Vec<u16>>> = Arc::new(Mutex::new(known_nodes));

    let known_nodes_clone = Arc::clone(&known_nodes);
    thread::spawn(move || {
        start_node_listener(port_to_listen, known_nodes_clone);
    });

    let nodo_actual = Nodo {
        puerto: port_to_listen,
        estado: Estado::Activo,
        time: Local::now(),
    };
    let serialized_nodo = serialize_nodo(&nodo_actual);
    gossip(Arc::clone(&known_nodes), serialized_nodo);

    println!(
        "Nodo iniciado y escuchando en el puerto: {}",
        port_to_listen
    );
    println!("Nodos conocidos: {:?}", *known_nodes.lock().unwrap());

    handle_user_input(Arc::clone(&known_nodes), port_to_listen);
}
