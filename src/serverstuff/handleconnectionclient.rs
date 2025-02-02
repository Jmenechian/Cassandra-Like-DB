use crate::config::{self, RF, TABLE_DESTINATION_NAME, TABLE_ORIGIN_NAME, TABLE_PATH};
use crate::cql::query::Query;
use crate::errors::my_errors::*;
use crate::messagestuff::{
    message::*,
    message_cql::{
        cqlenvelope::*, errormessage::*, querymessage::QueryMessage, resultemptymessage::*,
        resultrowsmessage::*,
    },
    message_server::{
        queryservermessage::QueryServerMessage, readrepairmessage::ReadRepairMessage,
        resultservermessage::*, serverpacket::Packet, serverpacket::ServerMessage,
    },
};
use crate::partitioning::consistent_hashing::ConsistentHash;
use crate::partitioning::queries::get_table_name_from_path;
use crate::serverstuff::{neighbor_node::NeighborNode, node::Node, node_status::Status};
use chrono::NaiveDateTime;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

fn handle_message_query(
    node: &Arc<RwLock<Node>>,
    querymessage: QueryMessage,
    stream: &mut TcpStream,
    sender: Sender<String>,
) -> Result<(), MyError> {
    let table = match send_query_to_nodes(
        node,
        querymessage.query,
        querymessage.consistency,
        sender.clone(),
    ) {
        Ok(r) => r,
        Err(e) => {
            handle_message_error(
                stream,
                format!("Error: algo fallo en send_query_to_node: {:?}", e),
                sender,
            );
            return Err(e);
        }
    };
    if table.is_empty() {
        let void_msg = ResultEmptyMessage;
        let _ = void_msg.send_through(stream, Some(sender));
    } else {
        let content: Vec<String> = table.clone();
        let rows_msg = ResultRowsMessage {
            rows_count: content.len() as i32,
            rows_content: content,
        };
        let _ = rows_msg.send_through(stream, Some(sender));
    }
    Ok(())
}

fn handle_message_error(stream: &mut TcpStream, _error: String, sender: Sender<String>) {
    let msg = ErrorMessage {
        error: _error,
        code: 0_i32,
    };
    let _ = msg.send_through(stream, Some(sender));
}

/// Handler que se encarga de manejar la conexion del servidor con el cliente.
/// Funcion principal que recibe un nodo, un stream y un booleano.
pub fn handle_connection_client(
    node: Arc<RwLock<Node>>,
    stream: &mut TcpStream,
    sender: Sender<String>,
) -> Result<(), MyError> {
    while let Ok(env) = Envelope::from_stream(stream, Some(sender.clone())) {
        match env.body {
            CqlMessage::Query(querymessage) => {
                handle_message_query(&node, querymessage, stream, sender.clone())?;
            }
            _ => {
                handle_message_error(
                    stream,
                    "Error: Solo me puede llegar un QueryMessage".to_string(),
                    sender.clone(),
                );
            }
        }
    }

    Ok(())
}

/// Devuelve el resultado del SELECT segun el consistency level.
/// Hace el read-repair para las tabla de origen y destino de vuelos.
/// El simulador no tiene read-repair.
fn process_results(
    results: &mut HashMap<IpAddr, Vec<String>>,
    nodes_with_result: &Vec<NeighborNode>,
    table: &str,
    sender: Sender<String>,
) -> Vec<String> {
    if table.contains(TABLE_ORIGIN_NAME) && table.contains(TABLE_DESTINATION_NAME) {
        if let Some(node) = results.keys().collect::<Vec<&IpAddr>>().first() {
            if let Some(result) = results.get(*node) {
                return result.to_vec();
            }
            return vec![];
        }
    }

    match results.len() {
        0 => vec![],
        1 => {
            if let Some(result) = results.values().collect::<Vec<&Vec<String>>>().first() {
                return result.to_vec();
            }
            vec![]
        }
        _ => {
            if read_repair_needed(results.clone()) {
                let node = look_newer(results);
                if let Some(result) = results.get(&node) {
                    read_repair(
                        result,
                        nodes_with_result,
                        &node,
                        &get_table_name_from_path(table),
                        sender,
                    );
                    result.to_vec()
                } else {
                    vec![]
                }
            } else if let Some(result) = results.values().collect::<Vec<&Vec<String>>>().first() {
                result.to_vec()
            } else {
                vec![]
            }
        }
    }
}

fn look_newer(results: &HashMap<IpAddr, Vec<String>>) -> IpAddr {
    let mut newer = NaiveDateTime::MIN;
    let mut newest_node: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

    for (node, result) in results.iter() {
        for row in result {
            if let Some(timestamp) = row
                .split(",")
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .last()
            {
                if let Ok(parsed_datetime) =
                    NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S%.f %Z")
                {
                    if parsed_datetime >= newer {
                        newer = parsed_datetime;
                        newest_node = *node;
                    }
                };
            }
        }
    }
    newest_node
}

/// Chequea si es necesario hacer read repair
/// Solo se llama cuando hay por lo menos dos resultados
fn read_repair_needed(results: HashMap<IpAddr, Vec<String>>) -> bool {
    let mut results_vec = results.values().collect::<Vec<&Vec<String>>>();
    if results_vec.len() < 2 {
        return false;
    }

    if let Some(mut result1) = results_vec.pop() {
        while !results_vec.is_empty() {
            for result2 in &results_vec {
                if !same_results(result1, result2) {
                    return true;
                }
            }
            result1 = match results_vec.pop() {
                Some(r) => r,
                None => return true,
            };
        }
    }
    false
}

fn same_results(rows1: &[String], rows2: &[String]) -> bool {
    if rows1.is_empty() && rows2.is_empty() {
        return true;
    }

    if rows1.len() != rows2.len() {
        return false;
    }

    for (row1, row2) in rows1.iter().zip(rows2.iter()) {
        let mut row1_split = row1.split(",").collect::<Vec<&str>>();
        row1_split.pop(); // saco el timestamp

        let mut row2_split = row2.split(",").collect::<Vec<&str>>();
        row2_split.pop(); // saco el timestamp

        if row1_split != row2_split {
            return false;
        }
    }
    true
}

// Manda mensaje de read-repair a los nodos que no tienen la info correcta.
fn read_repair(
    result: &[String],
    nodes_with_result: &Vec<NeighborNode>,
    node: &IpAddr,
    table: &str,
    sender: Sender<String>,
) {
    for node_neighbor in nodes_with_result {
        if node_neighbor.ip == *node {
            continue;
        }
        if node_neighbor.status == Status::Disconnected {
            continue;
        }
        if let Ok(mut socket) = TcpStream::connect(node_neighbor.server_socket) {
            let msg = ReadRepairMessage {
                table: table.to_string(),
                rows_to_keep: result.to_vec(),
            };
            let _ = msg.send_server_through(&mut socket, sender.clone());
        }
    }
}

pub fn hinted_handoff(
    node_number: &String,
    neighbor_number: &String,
    query: &Query,
) -> Result<(), MyError> {
    let file = format!(
        "{}/logs/logs_{}_{}.csv",
        TABLE_PATH, node_number, neighbor_number
    );

    match OpenOptions::new().append(true).create(true).open(file) {
        Ok(mut f) => {
            write!(f, "{}", query.query).map_err(|e| MyError::FileError(e.to_string()))?;
        }
        Err(e) => {
            return Err(MyError::FileError(e.to_string()));
        }
    };

    Ok(())
}

/// Le envio la query a los nodos correspondientes para que la procesen y devuelvo el resultado.
/// Aplicamos Read-Repear si la consulta fue un SELECT.
pub fn send_query_to_nodes(
    node: &Arc<RwLock<Node>>,
    _query: String,
    consistency_level: u16,
    sender: Sender<String>,
) -> Result<Vec<String>, MyError> {
    let node_guard = match node.read() {
        Ok(n) => n,
        Err(e) => {
            let _error = format!("Error al tratar de usar el nodo: {:?}", e);
            return Err(MyError::LockingError(_error));
        }
    };
    let mut all_response_dict: HashMap<IpAddr, Vec<String>> = HashMap::new();
    let mut hash = ConsistentHash::new(RF);
    let mut query: Query = Query::new(_query.clone(), node_guard.ip.to_string());

    hash.add_nodes(node_guard.neighbors.to_vec());
    let key = query.get_key_representing_partition(&node_guard.ip.to_string());
    let nodes_with_result: Vec<NeighborNode> = if key == *"" {
        // Create table
        node_guard.neighbors.clone()
    } else {
        // otras queries
        hash.get_nodes(&key)
    };
    let is_select = query.query.contains("SELECT");

    for neighbor in &nodes_with_result {
        // El nodo original porque puede tener el resultado de la query:
        if neighbor.server_socket == node_guard.server_socket {
            match node_guard.process_query(&mut query) {
                Ok(result) => {
                    all_response_dict.insert(neighbor.ip, result.clone());

                    if consistency_level == config::ONE && all_response_dict.len() == 1 {
                        if is_select {
                            // Read-repair
                            return Ok(process_results(
                                &mut all_response_dict,
                                &nodes_with_result,
                                &query.get_table(),
                                sender.clone(),
                            ));
                        } else {
                            return Ok(result);
                        }
                    }
                }
                Err(error) => {
                    all_response_dict
                        .insert(neighbor.ip, vec![format!("Invalid Query: {}", error)]);
                }
            }
            continue;
        } else if neighbor.status == Status::Disconnected {
            if !query.query.contains("SELECT") {
                hinted_handoff(&node_guard.ip.to_string(), &neighbor.ip.to_string(), &query)?;
            }
            continue;
        } else if let Ok(mut socket) = TcpStream::connect(neighbor.server_socket) {
            // Mando mensaje y el IP del nodo que se conecto con el cliente y en ese mismo stream leo la respuesta que me envia como resultado de la consulta.

            let msg = QueryServerMessage {
                query: _query.clone(),
                consistency: 0,
            };
            let _ = msg.send_server_through(&mut socket, sender.clone());

            if let Ok(packet) = Packet::from_stream(&mut socket, sender.clone()) {
                if let ServerMessage::Result(resultmessage) = packet.body {
                    match resultmessage.body {
                        ResultServerBody::Rows(r) => {
                            all_response_dict.insert(neighbor.ip, r.rows_content.clone());
                        }
                        ResultServerBody::Void(_) => {
                            all_response_dict
                                .insert(neighbor.ip, vec!["Invalid Query".to_string()]);
                        }
                        ResultServerBody::Empty(_) => {}
                    }
                }
            }
        }
    }
    if is_select {
        Ok(process_results(
            &mut all_response_dict,
            &nodes_with_result,
            &query.get_table(),
            sender.clone(),
        ))
    } else {
        if let Some(result) = all_response_dict
            .values()
            .collect::<Vec<&Vec<String>>>()
            .first()
        {
            return Ok(result.to_vec());
        }
        Ok(vec![])
    }
}
