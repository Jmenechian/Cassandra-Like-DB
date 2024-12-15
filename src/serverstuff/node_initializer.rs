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
use std::net::{IpAddr, TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

/// Funcion de inicializacíon, donde se crea el nodo indicado y se escucha y responde por los puertos correspondientes.
pub fn node_initialize(ip: IpAddr, neighbor: Option<IpAddr>) -> Result<(), MyError> {
    let show_g: bool = false;
    let show_s: bool = false;
    let show_c: bool = true;

    let mut current_node: Node = Node::new(ip);
    println!("Nodo actual: {:?}", current_node);
    //me agrego a mi mismo a vecinos
    current_node.add_neighbor(NeighborNode::new(current_node.ip.clone()));

    let my_ip = current_node.ip.clone();    
    
    let server_socket = current_node.server_socket.clone();
    let client_socket = current_node.client_socket.clone();
    let gossip_socket = current_node.gossip_socket.clone();
    let current_node: Arc<RwLock<Node>> = Arc::new(RwLock::new(current_node));

    // Seccion servidor-servidor
    listen(server_socket, &current_node, handle_connection_server, show_s)?;

    // Seccion Gossip
    listen(gossip_socket, &current_node, handle_connection_gossip, show_g)?;

    // Seccion cliente-servidor
    listen(client_socket, &current_node, handle_connection_client, show_c)?;
    
    //empiezo gossip
    if let Some(neighbor_ip) = neighbor {
        let neighbor_gossip_socket = SocketAddr::new(neighbor_ip, 9000 as u16);
        if let Ok(mut s) = TcpStream::connect(neighbor_gossip_socket) {
            let gossip_msg = GossipMessage {
                ip: my_ip.to_string(),
                status: Status::Connected,
                ttl: 5_u8,
            };
            let _ = gossip_msg.send_server_through(&mut s);
        } else {
            return Err(MyError::TcpStreamError("No se pudo conectar al vecino inicial".to_string()));
        }
    }
    
    loop {
        thread::sleep(Duration::from_secs(2));
        
        let ips: Vec<IpAddr> = {
            let node = current_node.read().expect("Fallo al leer el nodo actual");
            node.neighbors
                .iter()
                .filter(|n| n.ip != node.ip)
                .map(|n| n.ip)
                .collect()
        };
        
        match connect_to_random_gossip(ips, show_g) {
            Some(mut s) => {
                let gossip_msg = GossipMessage {
                    ip: my_ip.to_string(),
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
        Err(e) => {
            return Err(MyError::FileError(format!(
                "Error 1 {:?}, en archivo: {:?}, dir: {:?}",
                e,
                filename,
                std::env::current_dir()
            )))
        }
    };
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| {
            let l_string = l.map_err(|e| MyError::FileError(format!("Error 2 {:?}", e)))?;
            l_string
                .parse::<u16>()
                .map_err(|e| MyError::FileError(format!("Error 3 {:?}", e)))
        })
        .collect()
}

/// Nos conectamos al adress indicado.
pub fn try_connect(socket: SocketAddr) -> Option<TcpStream> {
    match TcpStream::connect(socket) {
        Ok(stream) => Some(stream),
        Err(_) => None,
    }
}

fn listen<F>(
    socket: SocketAddr,
    node: &Arc<RwLock<Node>>,
    handler: F,
    show: bool,
) -> Result<JoinHandle<Result<(), MyError>>, MyError>
where
    F: Fn(Arc<RwLock<Node>>, &mut TcpStream, bool) -> Result<(), MyError> + Send + 'static + Copy,
{
    let current_node = Arc::clone(node);

    let t: JoinHandle<Result<(), MyError>> = thread::spawn(move || {
        let listener = match TcpListener::bind(socket) {
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
