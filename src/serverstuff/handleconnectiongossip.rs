use super::node_initializer::try_connect;
use crate::config::{SHOW_GOSSIP, TABLE_PATH};
use crate::errors::my_errors::*;
use crate::loggerstuff::logger::Logger;
use crate::messagestuff::message_server::gossipmessage::GossipMessage;
use crate::messagestuff::{
    message::*,
    message_server::{
        queryservermessage::QueryServerMessage, serverpacket::Packet, serverpacket::ServerMessage,
    },
};
use crate::partitioning::new_node_aggregator::NewNodeAgregator;
use crate::serverstuff::neighbor_node::NeighborNode;
use crate::serverstuff::node::Node;
use crate::serverstuff::node_status::Status;
use crate::utils::{date_is_recent, delete_file, get_reader};
use rand::seq::SliceRandom;
use std::io::BufRead;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::sync::{Arc, RwLock};

fn reenviar_gossip(node: &mut std::sync::RwLockWriteGuard<'_, Node>, gossip: GossipMessage) {
    if let Some(n) = node.get_random_neighbor() {
        if SHOW_GOSSIP {
            println!(
                "G    El vecino aleatorio es: \n{:?}\nG    Le mando el mensaje: \n{:?}",
                n, gossip
            );
        }
        if let Some(mut stream) = try_connect(n.gossip_socket.clone()) {
            if let Err(e) = gossip
                .clone()
                .send_server_through(&mut stream, node.logger.clone())
            {
                if SHOW_GOSSIP {
                    println!("G   Error al eviarle mensaje gossip: {:?}", e);
                }
            }
        } else {
            if SHOW_GOSSIP {
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
            reenviar_gossip(node, gossip);
            reenviar_gossip(node, gossip_msg)
        }
    }
}

fn handle_send_backlog(node_number: &String, neighbor: &NeighborNode, logger: Arc<RwLock<Logger>>) {

    //4 ...
    let file = format!("logs/logs_{}_{}", node_number, neighbor.ip);
    let rdr = match get_reader(TABLE_PATH, &file) {
        Ok(r) => r,
        Err(_) => {
            return;
        }
    };
    for line in rdr.lines().map_while(Result::ok) {
        println!("line: {:?}", line);
        let logger_clone = Arc::clone(&logger);
        let logger_clone_sender = Arc::clone(&logger);
        if let Ok(mut socket) = TcpStream::connect(&neighbor.server_socket) {
            let msg = QueryServerMessage {
                query: line.clone(),
                consistency: 0,
            };
            let _ = msg.send_server_through(&mut socket, logger_clone_sender);

            let _ = Packet::from_stream(&mut socket, logger_clone);
        }
    }
    println!("borrando archivo");
    let _ = delete_file(&file, TABLE_PATH);
}

fn handle_message_gossip(
    node: Arc<RwLock<Node>>,
    gossip: &mut GossipMessage,
    logger: Arc<RwLock<Logger>>,
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
        Status::Disengaged => {
            new_neighbor.disengaged_node();
        }
    }
    let res = node_guard.add_neighbor(new_neighbor.clone());
    if res > 0 {
        if res == 1 && new_neighbor.status == Status::Connected {
            handle_send_backlog(&node_guard.ip.to_string(), &new_neighbor, logger);
        }

        if res == 2 &&  !date_is_recent(node_guard.connection_date) {
            println!("G   Me llego un nuevo vecino: {:?}", new_neighbor);
            let mut agg_node = NewNodeAgregator::new(new_neighbor, node_guard.clone());
            agg_node.new_node_in_ring();
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
    logger: Arc<RwLock<Logger>>,
) -> Result<(), MyError> {
    let logger_clone = Arc::clone(&logger);
    match Packet::from_stream(stream, logger) {
        Ok(packet) => {
            // if SHOW_GOSSIP {
            //     println!("G   El packet recibido es: {:?}", packet);
            // }
            match packet.body {
                ServerMessage::Gossip(mut gossip) => {
                    handle_message_gossip(node, &mut gossip, logger_clone)?;
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
    ips: Vec<IpAddr>,
) -> Option<TcpStream> {
    ips.clone().shuffle(&mut rand::thread_rng());
    
    for &ip in &ips {
        if let Some(stream) = try_connect(SocketAddr::new(ip , 9000 as u16)) {
            if SHOW_GOSSIP {
                println!("El primero que encontre fue el: {:?}", ip);
            }
            return Some(stream);
        }
    }
    None
}
/*
#[cfg(test)]
mod tests {
    use crate::partitioning::consistent_hashing::ConsistentHash;

    use super::*;

    #[test]
    pub fn test() {
        let key = "MEX,25-12-24".to_string();
        let rf = 3;

        let node_1 = NeighborNode::new("1");
        let node_2 = NeighborNode::new("2");
        let node_3 = NeighborNode::new("3");
        let node_4 = NeighborNode::new("4");
        let node_5 = NeighborNode::new("5");
        let node_6 = NeighborNode::new("6");
        let node_7 = NeighborNode::new("7");
        let node_8 = NeighborNode::new("8");
        let node_9 = NeighborNode::new("9");

        let nodes: Vec<NeighborNode> = vec![
            node_1, node_2, node_3, node_4, node_5, node_6, node_7, node_8, node_9,
        ];

        let mut hashing = ConsistentHash::new(rf);
        hashing.add_nodes(nodes.clone());

        let nodes_result_before = hashing.get_nodes(&key);
        println!("{:?}", nodes_result_before);
    }
}
*/