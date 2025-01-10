use crate::config::IPS;
use crate::errors::my_errors::MyError;
use crate::loggerstuff::logger::Logger;
use crate::messagestuff::{message::*, message_server::gossipmessage::*};
use crate::serverstuff::neighbor_node::NeighborNode;
use crate::serverstuff::node::Node;
use crate::serverstuff::node_status::Status;
use crate::serverstuff::{
    handleconnectionclient::handle_connection_client,
    handleconnectiongossip::handle_connection_gossip,
    handleconnectionserver::handle_connection_server,
};
use std::fs;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use super::handleconnectiongossip::connect_to_random_gossip;

/// Funcion de inicializacíon, donde se crea el nodo indicado y se escucha y responde por los puertos correspondientes.
pub fn node_initialize(ip: IpAddr) -> Result<(), MyError> {
    println!("IP: {}", ip);
    let mut current_node: Node = Node::new(ip)?;
    //me agrego a mi mismo a vecinos
    current_node.add_neighbor(NeighborNode::new(current_node.ip));
    let my_ip = current_node.ip;

    let server_socket = current_node.server_socket;
    let client_socket = current_node.client_socket;
    let gossip_socket = current_node.gossip_socket;

    let current_node: Arc<RwLock<Node>> = Arc::new(RwLock::new(current_node));

    let channel = mpsc::channel();
    let sender = channel.0;
    let receiver = channel.1;
    let mut logger = Logger::new(ip.to_string())?;
    thread::spawn(move || if logger.listen_log(receiver).is_ok() {});

    // Seccion servidor-servidor
    listen(
        server_socket,
        &current_node,
        sender.clone(),
        handle_connection_server,
    )?;

    // Seccion Gossip
    listen(
        gossip_socket,
        &current_node,
        sender.clone(),
        handle_connection_gossip,
    )?;

    // Seccion cliente-servidor
    listen(
        client_socket,
        &current_node,
        sender.clone(),
        handle_connection_client,
    )?;

    let channel = mpsc::channel();
    let sender_disengage: Sender<u8> = channel.0;
    let receiver_disengage: Receiver<u8> = channel.1;

    thread::spawn(move || {
        handle_disengage(sender_disengage);
    });

    //empiezo gossip
    let binding = fs::read_to_string(IPS).expect("Should have been able to read the file");
    let ips: Vec<&str> = binding.split("\n").collect();

    loop {
        let ips: Vec<IpAddr> = ips
            .iter()
            .filter(|n| n.parse() != Ok(my_ip))
            .map(|n| n.parse().unwrap())
            .collect();
        
        if let Some(mut stream) = connect_to_random_gossip(ips) {
            let mut status : Status = Status::Connected;
            let mut exit = false;
            
            if receiver_disengage.recv().unwrap() == 1 {
                status = Status::Disengaged;
                exit = true;
            }
            
            let gossip_msg = GossipMessage {
                ip: my_ip.to_string(),
                status,
                ttl: 5_u8,
            };
            let _ = gossip_msg.send_server_through(&mut stream, sender.clone());
            
            if exit{
                return Ok(());
            }
        }
        
        thread::sleep(Duration::from_secs(1));
    }
}

/// Vemos si el nodo manda una señal de Desconetado defitinitamente.
pub fn handle_disengage(sender_disengage: Sender<u8>){
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(1));
        let _ = sender_disengage.send(0);
    }

    let _ = sender_disengage.send(1);
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
    sender: Sender<String>,
    handler: F,
) -> Result<JoinHandle<Result<(), MyError>>, MyError>
where
    F: Fn(Arc<RwLock<Node>>, &mut TcpStream, Sender<String>) -> Result<(), MyError>
        + Send
        + 'static
        + Copy,
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
            let sender = sender.clone();

            thread::spawn(move || {
                let _ = handler(node_thread, &mut s, sender);
            });
        }
        Ok(())
    });
    Ok(t)
}
