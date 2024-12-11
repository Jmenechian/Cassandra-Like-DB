use super::node_initializer::try_connect;
use crate::config::{TABLE_PATH, SHOW_GOSSIP};
use crate::errors::my_errors::*;
use crate::messagestuff::message_server::gossipmessage::GossipMessage;
use crate::messagestuff::{
    message::*,
    message_server::{
        queryservermessage::QueryServerMessage, serverpacket::Packet, serverpacket::ServerMessage,
    },
};
use crate::serverstuff::neighbor_node::NeighborNode;
use crate::serverstuff::node::Node;
use crate::serverstuff::node_status::Status;
use crate::utils::{delete_file, get_reader};
use rand::seq::SliceRandom;
use std::io::BufRead;
use std::net::TcpStream;
use std::sync::{Arc, RwLock};

fn reenviar_gossip(
    node: &mut std::sync::RwLockWriteGuard<'_, Node>,
    gossip: GossipMessage,
) {
    if let Some(n) = node.get_random_neighbor() {
        if SHOW_GOSSIP {
            println!(
                "G    El vecino aleatorio es: \n{:?}\nG    Le mando el mensaje: \n{:?}",
                n, gossip
            );
        }
        if let Some(mut stream) = try_connect(n.gossip_ip.clone()) {
            if let Err(e) = gossip.clone().send_server_through(&mut stream) {
                if SHOW_GOSSIP {
                    println!("G   Error al eviarle mensaje gossip: {:?}", e);
                }
            }
        } else {
            if SHOW_GOSSIP {
                println!(
                    "G   No me pude conectar con: {:?}, tiene que estar desconectado",
                    n.server_ip
                );
            }
            let gossip_msg = GossipMessage {
                ip: n.server_ip.clone(),
                status: Status::Disconnected,
                ttl: 5_u8,
            };
            reenviar_gossip(node, gossip);
            reenviar_gossip(node, gossip_msg)
        }
    }
}

fn handle_send_backlog(node_number: &String, neighbor: &NeighborNode, node: Arc<RwLock<Node>>) {
    //4 ...
    let file = format!("logs/logs_{}_{}", node_number, neighbor.number);
    let rdr = match get_reader(TABLE_PATH, &file) {
        Ok(r) => r,
        Err(_) => {
            return;
        }
    };
    for line in rdr.lines().map_while(Result::ok) {
        println!("line: {:?}", line);
        if let Ok(mut socket) = TcpStream::connect(&neighbor.server_ip) {
            let msg = QueryServerMessage {
                query: line.clone(),
                consistency: 0,
            };
            let _ = msg.send_server_through(&mut socket);

            let _ = Packet::from_stream(&mut socket, node.clone());
        }
    }
    println!("borrando archivo");
    let _ = delete_file(&file, TABLE_PATH);
}

fn handle_message_gossip(
    node: Arc<RwLock<Node>>,
    gossip: &mut GossipMessage,
) -> Result<(), MyError> {
    let mut node_guard: std::sync::RwLockWriteGuard<'_, Node> = match node.write() {
        Ok(n) => n,
        Err(e) => {
            if SHOW_GOSSIP {
                println!("G   Error al tratar de usar el nodo: {:?}", e);
            }
            return Err(MyError::LockingError("No puedo usar el nodo".to_string()));
        }
    };
    let ip: String = gossip.ip.clone();

    let port = match ip.chars().last() {
        Some(c) => c,
        None => return Err(MyError::Error("No puse obtener el puerto".to_string())),
    };
    let mut new_neighbor = NeighborNode::new(&port.to_string());
    match gossip.status {
        Status::Connected => {
            new_neighbor.connect_node();
        }
        Status::Disconnected => {
            new_neighbor.disconnect_node();
        }
    }
    if node_guard.add_neighbor(new_neighbor.clone()) {
        println!("{:?}", new_neighbor);
        if new_neighbor.status == Status::Connected {
            handle_send_backlog(&node_guard.number, &new_neighbor, node.clone());
        }

        if SHOW_GOSSIP {
            println!("G   Mis vecinos son:");
            for neighbor in &node_guard.neighbors {
                println!("{:?}", neighbor);
            }
        }
    } else if let Some(false) = gossip.decrease_ttl() {
        return Ok(());
    }
    reenviar_gossip(&mut node_guard, gossip.clone());
    Ok(())
}

/// Handler que se encarga de manejar el servidor cuando se comunica por Gossip.
/// Funcion principal que recibe un nodo, un stream y un booleano.
pub fn handle_connection_gossip(
    node: Arc<RwLock<Node>>,
    stream: &mut TcpStream,
) -> Result<(), MyError> {
    match Packet::from_stream(stream, node.clone()) {
        Ok(packet) => {
            if SHOW_GOSSIP {
                println!("G   El packet recibido es: {:?}", packet);
            }
            match packet.body {
                ServerMessage::Gossip(mut gossip) => {
                    handle_message_gossip(node, &mut gossip)?;
                }
                _ => {
                    if SHOW_GOSSIP {
                        println!("G   Error entre gossip y gossip");
                    }
                }
            }
        }
        Err(e) => {
            let error = format!("G   Error al leer el packet: {:?}", e);
            if SHOW_GOSSIP {
                println!("{}", error);
            }
            return Err(MyError::Error(error));
        }
    }
    Ok(())
}

/// Nos conectamos con un nodo al azar y devolvemos el stream.
pub fn connect_to_random_gossip(
    ports: Vec<u16>,
    my_port: &String,
) -> Option<(TcpStream, String)> {
    let mut shuffeld_ports = ports.clone();
    shuffeld_ports.shuffle(&mut rand::thread_rng());
    if SHOW_GOSSIP {
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
            if SHOW_GOSSIP {
                println!("El primero que encontre fue el: {:?}", addr);
            }
            return Some((stream, addr));
        }
    }
    None
}
