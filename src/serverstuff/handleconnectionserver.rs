// Handler que se encarga de manejar la conexion del servidor con si mismo.
//
// La funcion principal es `handle_connection_server` que recibe un nodo, un stream y un booleano.
// El resto son funciones auxiliares.

use crate::cql::query::{self, Query};
use crate::messagestuff::message_server::readrepairmessage::ReadRepairMessage;
use crate::messagestuff::message_server::resultservermessage::IntoResultServerMessage;
use crate::messagestuff::message_server::resultvoidservermsg::ResultVoidServerMessage;
use crate::messagestuff::{
    message::*,
    message_server::{
        queryservermessage::QueryServerMessage, resultrowsevermsg::ResultRowsServerMessage,
        serverpacket::Packet, serverpacket::ServerMessage,
    },
};
use crate::my_errors::*;
use crate::nodo::Node;
use std::io::Read;
use std::net::TcpStream;
use std::sync::{Arc, RwLock};

fn handle_message_query(
    node: Arc<RwLock<Node>>,
    query: QueryServerMessage,
    stream: &mut TcpStream,
    show: bool,
) -> Result<(), MyError> {
    let node = match node.read() {
        Ok(n) => n,
        Err(e) => {
            let _error = format!("Error al tratar de usar el nodo: {:?}", e);
            if show {
                println!("S   {:?}", _error);
            }
            return Err(MyError::LockingError(_error));
        }
    };
    let mut query = Query::new(query.query.clone(), node.server_ip.clone());
    let msg = match query.execute() {
        Ok(s) => {
            if show {
                println!("RESULTADO, TABLA: {:?}", s);
            }
            ResultRowsServerMessage {
                table: query.get_table(),
                rows_count: s.len() as i32,
                rows_content: s,
            }
            .into_result_server_message()
        }
        Err(e) => {
            println!("Error en query: {:?}", e);
            ResultVoidServerMessage.into_result_server_message()
        }
    };
    let _ = msg.send_server_through(stream);

    Ok(())
}

fn get_primary_key(row: String, table: &String) -> Vec<String>{
    let row_vec = row.split(",").collect::<Vec<&str>>();
    let mut primary_keys = Vec::new();
    primary_keys.push(row_vec[0].to_string());
    primary_keys.push(row_vec[2].to_string());
    primary_keys.push(row_vec[3].to_string());
    primary_keys
}

// Hay que revisar LOOK_NEWER para qu devuelva por FILAS y no por nodos
fn handle_message_read_repair (
    node: Arc<RwLock<Node>>,
    query: ReadRepairMessage,
    stream: &mut TcpStream,
    show: bool,) -> Result<(), MyError>{
        
        let primary_keys = get_primary_key(query.rows_to_keep[0].clone(), &query.table);
        
        let query_str = format!("SELECT * FROM {} WHERE airport = {} AND date = {} AND flight = {}", query.table, primary_keys[0], primary_keys[1], primary_keys[2]);
        
        let mut _query = Query::new(query_str, node.read().unwrap().server_ip.clone());
        
        let rows_to_update = _query.execute()?;

        for row in &query.rows_to_keep {
            let row_vec = row.split(",").collect::<Vec<&str>>();
        
            if contains_row(&rows_to_update, &row) {
                let query_str = format!("UPDATE {} SET status = {} WHERE airport = {} AND date = {} AND flight = {} ", query.table, row_vec[4], row_vec[0], row_vec[2], row_vec[3]);
                println!("\n\n{}\n\n", query_str);
                let mut query = Query::new(query_str, node.read().unwrap().server_ip.clone());
                query.execute()?;
            }
            else {
                let query_str = format!("INSERT INTO {} (airport, destination, date, flight, status) VALUES ({}, {}, {}, {}, {})", query.table, row_vec[0], row_vec[1], row_vec[2], row_vec[3], row_vec[4]);
                println!("\n\n{}\n\n", query_str);
                let mut query = Query::new(query_str, node.read().unwrap().server_ip.clone());
                query.execute()?;
            }
        }

        for row in rows_to_update {
            if !contains_row(&query.rows_to_keep, &row) {
                let query_str = format!("DELETE FROM {} WHERE airport = {} AND date = {} AND flight = {}", query.table, primary_keys[0], primary_keys[1], primary_keys[2]);
                println!("\n\n{}\n\n", query_str);
                let mut query = Query::new(query_str, node.read().unwrap().server_ip.clone());
                query.execute()?;
            }
        }
        Ok(())
}

fn contains_row(rows: &Vec<String>, row: &String) -> bool {
    let mut vec_row = row.split(",").collect::<Vec<&str>>();
    vec_row.pop();
    println!("ESTOY ACAAAA: {:?}, rows: {:?}", vec_row, rows);
    
    for r in rows {
        let mut vec_r = r.split(",").collect::<Vec<&str>>();
        vec_r.pop();
        println!("vec_r: {:?} vec_row: {:?}", vec_r, vec_row);

        if vec_r == vec_row {
            return true;
        }
    }
    false
}


pub fn handle_connection_server(
    node: Arc<RwLock<Node>>,
    stream: &mut TcpStream,
    show: bool,
) -> Result<(), MyError> {
    match Packet::from_stream(stream) {
        Ok(packet) => {
            if show {
                println!("S   El packet recibido es: {:?}", packet);
            }
            match packet.body {
                ServerMessage::Query(query) => {
                    handle_message_query(node.clone(), query, stream, show)
                }
                ServerMessage::ReadRepair(msg) => {
                    handle_message_read_repair(node.clone(), msg, stream, show)
                }
                _ => {
                    let _error = "Error entre server y servers".to_string();
                    if show {
                        println!("S   {:?}", _error);
                    }
                    return Err(MyError::ServerMessageError(_error));
                }
            }
        }
        Err(_e) => Err(_e),
    }
}
