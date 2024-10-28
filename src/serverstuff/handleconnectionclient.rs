// Handler que se encarga de manejar la conexion del servidor con el cliente.
//
// La funcion principal es `handle_connection_client` que recibe un nodo, un stream y un booleano.
// El resto son funciones auxiliares.

use crate::config;
use crate::config::RF;
use crate::cql::query::Query;
use crate::messagestuff::message_cql::querymessage::QueryMessage;
use crate::messagestuff::{
    message::*,
    message_cql::{cqlenvelope::*, errormessage::*, resultrowsmessage::*, resultvoidmessage::*},
    message_server::{
        queryservermessage::QueryServerMessage, resultservermessage::*, serverpacket::Packet,
        serverpacket::ServerMessage,
    },
};
use crate::my_errors::*;
use crate::nodo::Node;
use crate::nodo::Status;
use crate::ConsistentHash;
use chrono::NaiveDateTime;
use std::net::TcpStream;
use std::sync::{Arc, RwLock, RwLockReadGuard};

fn handle_message_query(
    node: RwLockReadGuard<'_, Node>,
    querymessage: QueryMessage,
    stream: &mut TcpStream,
    show: bool,
) -> Result<(), MyError> {
    let table = match send_query_to_nodes(
        &node,
        querymessage.query.clone(),
        querymessage.consistency,
        show,
    ) {
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
        let void_msg = ResultVoidMessage;
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

pub fn handle_connection_client(
    node: Arc<RwLock<Node>>,
    stream: &mut TcpStream,
    show: bool,
) -> Result<(), MyError> {
    loop {
        match Envelope::from_stream(stream) {
            Ok(env) => {
                let node: RwLockReadGuard<'_, Node> = match node.read() {
                    Ok(n) => n,
                    Err(e) => {
                        let _error = format!("Error al tratar de usar el nodo: {:?}", e);
                        if show {
                            println!("C   {:?}", _error);
                        }
                        return Err(MyError::LockingError(_error));
                    }
                };
                if show {
                    println!("C   El env recibido es: {:?}", env);
                }
                match env.body {
                    CqlMessage::Query(querymessage) => {
                        handle_message_query(node, querymessage, stream, show)?;
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

fn process_results(results: &Vec<Vec<String>>) -> Vec<String> {
    println!("pr ===> {:?}", results);

    match results.len() {
        0 => vec![],
        1 => results[0].clone(),
        _ => {
            let index = look_newer(results);
            results[index].clone()
        }
    }
}

pub fn look_newer(results: &[Vec<String>]) -> usize {
    //falta testear en process results
    let mut newer = NaiveDateTime::MIN;
    let mut index = 0;
    for (i, result) in results.iter().enumerate() {
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
                    if parsed_datetime > newer {
                        newer = parsed_datetime;
                        index = i;
                    }
                };
            }
        }
    }
    index
}

pub fn send_query_to_nodes(
    node: &RwLockReadGuard<'_, Node>,
    _query: String,
    consistency_level: u16,
    show: bool,
) -> Result<Vec<String>, MyError> {
    let mut all_response: Vec<Vec<String>> = vec![];
    let mut hash = ConsistentHash::new(RF);
    let mut query: Query = Query::new(_query.clone(), node.client_ip.clone());

    // Si es un create table, lo ejecuto y devuelvo vacio
    if query.query.contains("CREATE TABLE") {
        match query.execute() {
            Ok(_) => return Ok(vec!["Se crean las tablas.".to_string()]),
            Err(e) => {
                let _error = format!("Error al procesar la query: {:?}", e);
                if show {
                    println!("C   {:?}", _error);
                }
                return Err(MyError::Error(_error));
            }
        }
    }

    hash.add_nodes(node.neighbors.to_vec());
    let key = query.get_key_representing_partition(&node.server_ip);
    println!("La key es: {}", key);
    println!("La query es: {:?}", query);
    let nodes_with_result = hash.get_nodes(&key);

    if show {
        println!(
            "
        C   vecinos: {:?}
        C   Los nodos que tienen la info son: {:?}\n",
            node.neighbors, nodes_with_result
        );
    }
    println!("nodes with result: {:?}", nodes_with_result);
    for neighbor in nodes_with_result {
        // El nodo original porque puede tener el resultado de la query:
        if neighbor.server_ip == node.server_ip {
            if show {
                println!(
                    "C   - Buscando el resultado en el nodo original: {}",
                    neighbor.server_ip
                );
            }
            if let Ok(result) = node.process_query(&mut query) {
                all_response.push(result);

                if consistency_level == config::ONE && all_response.len() == 1 {
                    return Ok(process_results(&all_response));
                }
            }
            continue;
        } else if neighbor.status == Status::Disconnected {
            if show {
                println!("C   - Nodo desconectado: {}", neighbor.server_ip);
            }
            continue;
        } else if let Ok(mut socket) = TcpStream::connect(&neighbor.server_ip) {
            // Mando mensaje y el IP del nodo que se conecto con el cliente y en ese mismo stream leo la respuesta que me envia como resultado de la consulta.
            // - Mando mensaje:
            let msg = QueryServerMessage {
                query: _query.clone(),
                consistency: 0,
            };
            let _ = msg.send_server_through(&mut socket);
            match Packet::from_stream(&mut socket) {
                Ok(packet) => {
                    if show {
                        println!("C   El packet recibido es: {:?}", packet);
                    }
                    match packet.body {
                        ServerMessage::Result(resultmessage) => match resultmessage.body {
                            ResultServerBody::Rows(r) => {
                                all_response.push(r.rows_content.clone());
                            }
                            ResultServerBody::Void(_) => {}
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
    Ok(process_results(&all_response))
}
