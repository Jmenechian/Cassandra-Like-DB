// Handler que se encarga de manejar el servidor cuando se comunica por Gossip.
//
// La funcion principal es `handle_connection_gossip` que recibe un nodo, un stream y un booleano.
// El resto son funciones auxiliares.

use crate::errors::my_errors::*;
use crate::messagestuff::message_server::gossipmessage::GossipMessage;
use crate::messagestuff::{
    message::*,
    message_server::{serverpacket::Packet, serverpacket::ServerMessage},
};
use crate::serverstuff::neighbor_node::NeighborNode;
use crate::serverstuff::node::{try_connect, Node};
use rand::seq::SliceRandom;
use std::net::TcpStream;
use std::sync::{Arc, RwLock};

fn handle_message_gossip(
    node: &mut Node, //Arc<RwLock<Node>>,
    gossip: &mut GossipMessage,
    show: bool,
) -> Result<(), MyError> {
    let ip: String = gossip.ip.clone();
    // let mut node = match node.write() {
    //     Ok(n) => n,
    //     Err(e) => {
    //         if show {
    //             println!("G   Error al tratar de usar el nodo: {:?}", e);
    //         }
    //         return Err(MyError::LockingError("No puedo usar el nodo".to_string()));
    //     }
    // };

    if let Some(true) = gossip.decrease_ttl() {
        if let Some(n) = node.get_random_neighbor(node.number.clone()) {
            if let Some(mut stream) = try_connect(n.gossip_ip.clone()) {
                if let Err(e) = gossip.clone().send_server_through(&mut stream) {
                    if show {
                        println!("G   Error al eviarle mensaje gossip: {:?}", e);
                    }
                }
            } else {
                let _error = "Error al conectarse al vecino".to_string();
                if show {
                    println!("G   {:?}", _error);
                }
                // Desconectar el nodod de vecinos
                return Err(MyError::TcpStreamError(_error));
            }
        }
    }
    let port = match ip.chars().last() {
        Some(c) => c,
        None => return Err(MyError::Error("No puse obtener el puerto".to_string())),
    };
    let new = node.add_neighbor(NeighborNode::new(&port.to_string()));
    if show && new {
        println!("G   Mis vecinos son: {:?}", node.neighbors);
    }
    Ok(())
}

pub fn handle_connection_gossip(
    node: &mut Node,//Arc<RwLock<Node>>,
    stream: &mut TcpStream,
    show: bool,
) -> Result<(), MyError> {
    if show {
        println!("G   Entro al handle_connection_gossip");
    }
    match Packet::from_stream(stream) {
        Ok(packet) => {
            if show {
                println!("G   El packet recibido es: {:?}", packet);
            }
            match packet.body {
                ServerMessage::Gossip(mut gossip) => {
                    handle_message_gossip(node, &mut gossip, show)?;
                }
                _ => {
                    if show {
                        println!("G   Error entre gossip y gossip");
                    }
                }
            }
        }
        Err(e) => {
            let error = format!("G   Error al leer el packet: {:?}", e);
            if show {
                println!("{}", error);
            }
            return Err(MyError::Error(error));
        }
    }
    Ok(())
}

pub fn connect_to_random_gossip(
    ports: Vec<u16>,
    my_port: &String,
    show: bool,
) -> Option<(TcpStream, String)> {
    let mut shuffeld_ports = ports.clone();
    shuffeld_ports.shuffle(&mut rand::thread_rng());
    if show {
        println!(
            "MEZCLO y me queda: {:?}, siendo y el: {:?}",
            shuffeld_ports, my_port
        );
    }
    for &port in &shuffeld_ports {
        if &port.to_string() == my_port {
            continue;
        };
        let addr = format!("127.0.0.1:900{}", port);
        if let Some(stream) = try_connect(addr.clone()) {
            if show {
                println!("El primero que encontre fue el: {:?}", addr);
            }
            return Some((stream, addr));
        }
    }
    None
}
