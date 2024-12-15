use crate::config::{self, TABLE_PATH};
use crate::config::{RF, TABLE_DESTINATION_NAME, TABLE_ORIGIN_NAME};
use crate::cql::query::Query;
use crate::errors::my_errors::*;
use crate::messagestuff::message_cql::querymessage::QueryMessage;
use crate::messagestuff::message_server::readrepairmessage::ReadRepairMessage;
use crate::messagestuff::{
    message::*,
    message_cql::{cqlenvelope::*, errormessage::*, resultemptymessage::*, resultrowsmessage::*},
    message_server::{
        queryservermessage::QueryServerMessage, resultservermessage::*, serverpacket::Packet,
        serverpacket::ServerMessage,
    },
};
use crate::serverstuff::node_status::Status;
use crate::serverstuff::{neighbor_node::NeighborNode, node::Node};
use crate::ConsistentHash;
use chrono::NaiveDateTime;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::sync::{Arc, RwLock};

use std::fs::OpenOptions;
use std::io::Write;

fn handle_message_query(
    node: &Arc<RwLock<Node>>,
    querymessage: QueryMessage,
    stream: &mut TcpStream,
    show: bool,
) -> Result<(), MyError> {
    let table = match send_query_to_nodes(node, querymessage.query, querymessage.consistency, show)
    {
        Ok(r) => r,
        Err(e) => {
            handle_message_error(
                stream,
                "Error: algo fallo en send_query_to_node".to_string(),
                show,
            );
            return Err(e);
        }
    };
    if table.is_empty() {
        let void_msg = ResultEmptyMessage;
        if let Err(e) = void_msg.send_through(stream) {
            if show {
                println!("C   Error: {:?}", e);
            }
        }
    } else {
        let content: Vec<String> = table.clone();
        let rows_msg = ResultRowsMessage {
            rows_count: content.len() as i32,
            rows_content: content,
        };
        if let Err(e) = rows_msg.send_through(stream) {
            if show {
                println!("C   Error: {:?}", e);
            }
        }
    }
    Ok(())
}

fn handle_message_error(stream: &mut TcpStream, _error: String, show: bool) {
    let msg = ErrorMessage {
        error: _error,
        code: 0_i32,
    };
    if let Err(e) = msg.send_through(stream) {
        if show {
            println!("C   Error al enviarle un ErrorMessage al cliente: {:?}", e);
        }
    }
}

/// Handler que se encarga de manejar la conexion del servidor con el cliente.
/// Funcion principal que recibe un nodo, un stream y un booleano.
pub fn handle_connection_client(
    node: Arc<RwLock<Node>>,
    stream: &mut TcpStream,
    show: bool,
) -> Result<(), MyError> {
    loop {
        match Envelope::from_stream(stream) {
            Ok(env) => {
                if show {
                    println!("\nC   El env recibido es: {:?}", env);
                }
                match env.body {
                    CqlMessage::Query(querymessage) => {
                        handle_message_query(&node, querymessage, stream, show)?;
                    }
                    _ => {
                        handle_message_error(
                            stream,
                            "Error: Solo me puede llegar un QueryMessage".to_string(),
                            show,
                        );
                    }
                }
            }
            Err(e) => {
                if show {
                    println!("C    Error al leer el env: {:?}", e);
                }
                break;
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
) -> Vec<String> {
    println!("\n - pr ===> {:?}", results);
    let mut table_name = table.to_owned();
    table_name.pop();
    table_name.pop();
    if table_name != TABLE_ORIGIN_NAME && table_name != TABLE_DESTINATION_NAME {
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
                println!("Read repair needed");
                let node = look_newer(results);
                if let Some(result) = results.get(&node) {
                    read_repair(result, nodes_with_result, &node, &table_name);
                    result.to_vec()
                } else {
                    println!("ERROR EN READ REPAIR: No deberia llegar aca");
                    vec![]
                }
            } else {
                println!("Read repair not needed");
                if let Some(result) = results.values().collect::<Vec<&Vec<String>>>().first() {
                    result.to_vec()
                } else {
                    println!("ERROR EN READ REPAIR: No deberia llegar aca");
                    vec![]
                }
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
                        newest_node = node.clone();
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
            result1 = results_vec.pop().unwrap();
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
) {
    println!("Mandando mensaje de Read Repair");

    for node_neighbor in nodes_with_result {
        if node_neighbor.ip == *node {
            continue;
        }
        if node_neighbor.status == Status::Disconnected {
            continue;
        }
        if let Ok(mut socket) = TcpStream::connect(&node_neighbor.server_socket) {
            let msg = ReadRepairMessage {
                table: table.to_string(),
                rows_to_keep: result.to_vec(),
            };
            let _ = msg.send_server_through(&mut socket);
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
            println!("Error al abrir el archivo de logs: {:?}", e);
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
    show: bool,
) -> Result<Vec<String>, MyError> {
    let node = match node.read() {
        Ok(n) => n,
        Err(e) => {
            let _error = format!("Error al tratar de usar el nodo: {:?}", e);
            if show {
                println!("C   {:?}", _error);
            }
            return Err(MyError::LockingError(_error));
        }
    };
    let mut all_response: Vec<Vec<String>> = vec![];
    let mut all_response_dict: HashMap<IpAddr, Vec<String>> = HashMap::new();
    let mut hash = ConsistentHash::new(RF);
    let mut query: Query = Query::new(_query.clone(), node.ip.to_string()/*, node.client_socket.to_string()*/);

    hash.add_nodes(node.neighbors.to_vec());
    let key = query.get_key_representing_partition(&node.ip.to_string());
    let nodes_with_result: Vec<NeighborNode> = if key == *"" {
        // Create table
        node.neighbors.clone()
    } else {
        // otras queries
        hash.get_nodes(&key)
    };

    if show {
        println!("\n - La key es: {}", key);
        println!("\n - La query es: {:?}", query);
        println!(
            "
        C   vecinos: {:?}
        C   Los nodos que tienen la info son: {:?}\n",
            node.neighbors, nodes_with_result
        );
    }
    println!("nodes with result: {:?}", nodes_with_result);

    let is_select = query.query.contains("SELECT");

    for neighbor in &nodes_with_result {
        // El nodo original porque puede tener el resultado de la query:
        if neighbor.server_socket == node.server_socket {
            if show {
                println!(
                    "C   - Buscando el resultado en el nodo original: {}",
                    neighbor.server_socket
                );
            }
            match node.process_query(&mut query) {
                Ok(result) => {
                    // all_response.push(result);
                    all_response_dict.insert(neighbor.ip, result);

                    if consistency_level == config::ONE && all_response.len() == 1 {
                        if is_select {
                            return Ok(process_results(
                                &mut all_response_dict,
                                &nodes_with_result,
                                &query.get_table(),
                            ));
                        } else {
                            if let Some(result) = all_response.pop() {
                                return Ok(result);
                            }
                            return Ok(vec![]);
                        }
                    }
                }
                Err(error) => {
                    // all_response.push(vec![format!("Invalid Query: {}", error)]);
                    all_response_dict.insert(
                        neighbor.ip,
                        vec![format!("Invalid Query: {}", error)],
                    );
                }
            }
            continue;
        } else if neighbor.status == Status::Disconnected {
            if !query.query.contains("SELECT") {
                hinted_handoff(&node.ip.to_string(), &neighbor.ip.to_string(), &query)?;
            }
            if show {
                println!("C   - Nodo desconectado: {}", neighbor.server_socket);
            }
            continue;
        } else if let Ok(mut socket) = TcpStream::connect(&neighbor.server_socket) {
            // Mando mensaje y el IP del nodo que se conecto con el cliente y en ese mismo stream leo la respuesta que me envia como resultado de la consulta.

            let msg = QueryServerMessage {
                query: _query.clone(),
                consistency: 0,
            };
            let _ = msg.send_server_through(&mut socket);

            match Packet::from_stream(&mut socket) {
                Ok(packet) => {
                    if show {
                        println!("\nC   El packet recibido es: {:?}", packet);
                    }
                    match packet.body {
                        ServerMessage::Result(resultmessage) => match resultmessage.body {
                            ResultServerBody::Rows(r) => {
                                // all_response.push(r.rows_content.clone());
                                all_response_dict
                                    .insert(neighbor.ip, r.rows_content.clone());
                            }
                            ResultServerBody::Void(_) => {
                                all_response.push(vec!["Invalid Query".to_string()]);
                                all_response_dict.insert(
                                    neighbor.ip,
                                    vec!["Invalid Query".to_string()],
                                );
                            }
                            ResultServerBody::Empty(_) => {}
                        },
                        _ => {
                            if show {
                                println!("C   Error entre server y servers");
                            }
                        }
                    }
                }
                Err(e) => {
                    if show {
                        println!("C   Error al leer el paket: {:?}", e);
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
