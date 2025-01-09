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
use crate::partitioning::new_node_aggregator::NewNodeAgregator;
use crate::serverstuff::neighbor_node::NeighborNode;
use crate::serverstuff::node::Node;
use crate::serverstuff::node_status::Status;
use crate::utils::{date_is_recent, delete_file, get_reader};
use rand::seq::SliceRandom;
use std::io::BufRead;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

fn reenviar_gossip(node: &mut std::sync::RwLockWriteGuard<'_, Node>, gossip: GossipMessage, sender: Sender<String>,) {
    if let Some(n) = node.clone().get_random_neighbor() {
        if let Some(mut stream) = try_connect(n.gossip_socket) {
            let _ = gossip
                .clone()
                .send_server_through(&mut stream, sender);
        } else {
            let status = match n.status {
                Status::Disconnected => Status::Disconnected,
                Status::Disengaged => Status::Disengaged,
                Status::Connected => Status::Disconnected,
            };
            if let Some(_n) = node.neighbors.iter_mut().find(|x| x.ip == n.ip) {
                _n.status = status;
            }
            let gossip_msg = GossipMessage {
                ip: n.ip.to_string(),
                status,
                ttl: 3_u8,
            };
            reenviar_gossip(node, gossip, sender.clone());
            reenviar_gossip(node, gossip_msg, sender.clone())
        }
    }
}

fn handle_send_backlog(node_number: &String, neighbor: &NeighborNode, sender: Sender<String>) {
    //4 ...
    let file = format!("logs/logs_{}_{}", node_number, neighbor.ip);
    let rdr = match get_reader(TABLE_PATH, &file) {
        Ok(r) => r,
        Err(_) => {
            return;
        }
    };
    for line in rdr.lines().map_while(Result::ok) {
        if let Ok(mut socket) = TcpStream::connect(neighbor.server_socket) {
            let msg = QueryServerMessage {
                query: line.clone(),
                consistency: 0,
            };
            let _ = msg.send_server_through(&mut socket, sender.clone());

            let _ = Packet::from_stream(&mut socket, sender.clone());
        }
    }
    let _ = delete_file(&file, TABLE_PATH);
}

fn handle_message_gossip(
    node: Arc<RwLock<Node>>,
    gossip: &mut GossipMessage,
    sender: Sender<String>,
) -> Result<(), MyError> {
    let mut node_guard: std::sync::RwLockWriteGuard<'_, Node> = match node.write() {
        Ok(n) => n,
        Err(_) => {
            println!("No pude obtener el nodo");
            return Err(MyError::LockingError("No puedo usar el nodo".to_string()));
        }
    };

    let ip: String = gossip.ip.clone();

    let ip_addr: IpAddr = match ip.parse() {
        Ok(ip) => ip,
        Err(_) => return Err(MyError::Error(format!("No pude obtener el ip: {:?}", ip))),
    };
    let mut new_neighbor = NeighborNode::new(ip_addr);
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
            handle_send_backlog(&node_guard.ip.to_string(), &new_neighbor, sender.clone());
        }

        if res == 2 && !date_is_recent(node_guard.connection_date) {
            let mut agg_node = NewNodeAgregator::new(new_neighbor, node_guard.clone());
            agg_node.new_node_in_ring(sender.clone());
        }
    } else if let Some(false) = gossip.decrease_ttl() {
        return Ok(());
    }
    reenviar_gossip(&mut node_guard, gossip.clone(), sender);
    Ok(())
}

/// Handler que se encarga de manejar el servidor cuando se comunica por Gossip.
/// Funcion principal que recibe un nodo, un stream y un booleano.
pub fn handle_connection_gossip(
    node: Arc<RwLock<Node>>,
    stream: &mut TcpStream,
    sender: Sender<String>,
) -> Result<(), MyError> {
    match Packet::from_stream(stream, sender.clone()) {
        Ok(packet) => {
            if let ServerMessage::Gossip(mut gossip) = packet.body {
                handle_message_gossip(node, &mut gossip, sender)?;
            }
        }
        Err(e) => {
            let error = format!("G   Error al leer el packet: {:?}", e);
            return Err(MyError::Error(error));
        }
    }
    Ok(())
}

/// Nos conectamos con un nodo al azar y devolvemos el stream.
pub fn connect_to_random_gossip(ips: Vec<IpAddr>) -> Option<TcpStream> {
    ips.clone().shuffle(&mut rand::thread_rng());

    for &ip in &ips {
        if let Some(stream) = try_connect(SocketAddr::new(ip, 9000_u16)) {
            return Some(stream);
        }
    }
    None
}
