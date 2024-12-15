use super::node_initializer::try_connect;
use crate::config::TABLE_PATH;
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
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::sync::{Arc, RwLock};

fn reenviar_gossip(
    node: &mut std::sync::RwLockWriteGuard<'_, Node>,
    gossip: GossipMessage,
    show: bool,
) {
    if let Some(n) = node.get_random_neighbor() {
        if show {
            println!(
                "G    El vecino aleatorio es: \n{:?}\nG    Le mando el mensaje: \n{:?}",
                n, gossip
            );
        }
        if let Some(mut stream) = try_connect(n.gossip_socket.clone()) {
            if let Err(e) = gossip.clone().send_server_through(&mut stream) {
                if show {
                    println!("G   Error al eviarle mensaje gossip: {:?}", e);
                }
            }
        } else {
            if show {
                println!(
                    "G   No me pude conectar con: {:?}, tiene que estar desconectado",
                    n.server_socket
                );
            }
            let gossip_msg = GossipMessage {
                ip: n.ip.to_string(),
                status: Status::Disconnected,
                ttl: 5_u8,
            };
            reenviar_gossip(node, gossip, show);
            reenviar_gossip(node, gossip_msg, show)
        }
    }
}

fn handle_send_backlog(node_name: &String, neighbor: &NeighborNode) {
    //4 ...
    let file = format!("logs/logs_{}_{}", node_name, neighbor.ip);
    let rdr = match get_reader(TABLE_PATH, &file) {
        Ok(r) => r,
        Err(_) => {
            return;
        }
    };
    for line in rdr.lines().map_while(Result::ok) {
        println!("line: {:?}", line);
        if let Ok(mut socket) = TcpStream::connect(&neighbor.server_socket) {
            let msg = QueryServerMessage {
                query: line.clone(),
                consistency: 0,
            };
            let _ = msg.send_server_through(&mut socket);

            let _ = Packet::from_stream(&mut socket);
        }
    }
    println!("borrando archivo");
    let _ = delete_file(&file, TABLE_PATH);
}

fn handle_message_gossip(
    node: Arc<RwLock<Node>>,
    gossip: &mut GossipMessage,
    show: bool,
) -> Result<(), MyError> {
    let mut node_guard: std::sync::RwLockWriteGuard<'_, Node> = match node.write() {
        Ok(n) => n,
        Err(e) => {
            if show {
                println!("G   Error al tratar de usar el nodo: {:?}", e);
            }
            return Err(MyError::LockingError("No puedo usar el nodo".to_string()));
        }
    };
    let ip: String = gossip.ip.clone();

    let ip_addr: IpAddr = match ip.parse() {
        Ok(ip) => ip,
        Err(_) => return Err(MyError::Error(format!("No pude obtener el ip: {:?}", ip))),
    };
    let mut new_neighbor = NeighborNode::new(ip_addr.clone());
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
            handle_send_backlog(&node_guard.ip.to_string(), &new_neighbor);
        }

        if show {
            println!("G   Mis vecinos son:");
            for neighbor in &node_guard.neighbors {
                println!("{:?}", neighbor);
            }
        }
    } else if let Some(false) = gossip.decrease_ttl() {
        return Ok(());
    }
    reenviar_gossip(&mut node_guard, gossip.clone(), show);
    Ok(())
}

/// Handler que se encarga de manejar el servidor cuando se comunica por Gossip.
/// Funcion principal que recibe un nodo, un stream y un booleano.
pub fn handle_connection_gossip(
    node: Arc<RwLock<Node>>,
    stream: &mut TcpStream,
    show: bool,
) -> Result<(), MyError> {
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

/// Nos conectamos con un nodo al azar y devolvemos el stream.
pub fn connect_to_random_gossip(
    ips: Vec<IpAddr>,
    show: bool,
) -> Option<TcpStream> {
    ips.clone().shuffle(&mut rand::thread_rng());
    
    for &ip in &ips {
        if let Some(stream) = try_connect(SocketAddr::new(ip , 9000 as u16)) {
            if show {
                println!("El primero que encontre fue el: {:?}", ip);
            }
            return Some(stream);
        }
    }
    None
}
