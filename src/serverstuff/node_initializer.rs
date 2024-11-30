use crate::config::PORTS_FILE;
use crate::errors::my_errors::MyError;
use crate::messagestuff::{message::*, message_server::gossipmessage::*};
use crate::serverstuff::neighbor_node::NeighborNode;
use crate::serverstuff::node::Node;
use crate::serverstuff::node_status::Status;
use crate::serverstuff::{
    handleconnectionclient::handle_connection_client,
    handleconnectiongossip::connect_to_random_gossip,
    handleconnectiongossip::handle_connection_gossip,
    handleconnectionserver::handle_connection_server,
};

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

/// Funcion de inicializacíon, donde se crea el nodo indicado y se escucha y responde por los puertos correspondientes.
pub fn node_initialize(port: String) -> Result<(), MyError> {
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
        None => println!("Tengo que ser el primer nodo porque no encontre otro"),
    };

    let server_ip = current_node.server_ip.clone();
    let client_ip = current_node.client_ip.clone();
    let gossip_ip = current_node.gossip_ip.clone();
    let current_node: Arc<RwLock<Node>> = Arc::new(RwLock::new(current_node));

    // Seccion cliente-servidor
    listen(client_ip, &current_node, handle_connection_client, show_c)?;

    // Seccion servidor-servidor
    listen(server_ip, &current_node, handle_connection_server, show_s)?;

    // Seccion Gossip
    listen(gossip_ip, &current_node, handle_connection_gossip, show_g)?;

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
            None => println!("No encuentre otro nodo."),
        };
    }
}

/// Lee los puertos del archivo
fn read_ports_from_file(filename: &str) -> Result<Vec<u16>, MyError> {
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => return Err(MyError::FileError(format!("Error 1 {:?}, en archivo: {:?}, dir: {:?}",e, filename, std::env::current_dir()))),
    };
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| {
            let l_string = l.map_err(|e| MyError::FileError(format!("Error 2 {:?}",e)))?;
            l_string
                .parse::<u16>()
                .map_err(|e| MyError::FileError(format!("Error 3 {:?}",e)))
        })
        .collect()
}

/// Nos conectamos al adress indicado.
pub fn try_connect(ip: String) -> Option<TcpStream> {
    match TcpStream::connect(ip) {
        Ok(stream) => Some(stream),
        Err(_) => None,
    }
}

fn listen<F>(
    ip: String,
    node: &Arc<RwLock<Node>>,
    handler: F,
    show: bool,
) -> Result<JoinHandle<Result<(), MyError>>, MyError>
where
    F: Fn(Arc<RwLock<Node>>, &mut TcpStream, bool) -> Result<(), MyError> + Send + 'static + Copy,
{
    let current_node = Arc::clone(node);

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
                match handler(node_thread, &mut s, show) {
                    Ok(_) => {}
                    Err(e) => println!("Error: {:?}", e),
                };
            });
        }
        Ok(())
    });
    Ok(t)
}
