use crate::config::TABLE_ORIGIN_NAME;
use crate::cql::query::Query;
use crate::errors::my_errors::*;
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
use crate::serverstuff::node::Node;
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

fn handle_message_query(
    node: Arc<RwLock<Node>>,
    query: QueryServerMessage,
    stream: &mut TcpStream,
    sender: Sender<String>,
) -> Result<(), MyError> {
    let node = match node.read() {
        Ok(n) => n,
        Err(e) => {
            let _error = format!("Error al tratar de usar el nodo: {:?}", e);
            return Err(MyError::LockingError(_error));
        }
    };

    let mut query = Query::new(query.query.clone(), node.ip.to_string());
    let msg = match node.process_query(&mut query) {
        Ok(s) => ResultRowsServerMessage {
            table: query.get_table(),
            rows_count: s.len() as i32,
            rows_content: s,
        }
        .into_result_server_message(),
        Err(_) => ResultVoidServerMessage.into_result_server_message(),
    };
    let _ = msg.send_server_through(stream, sender);
    Ok(())
}

fn get_primary_key(row: &str) -> Vec<String> {
    let row_vec = row.split(",").collect::<Vec<&str>>();
    let primary_keys = vec![
        row_vec[0].to_string(),
        row_vec[2].to_string(),
        row_vec[3].to_string(),
    ]; // airport, date, flight
    primary_keys
}

fn handle_message_read_repair(
    node: Arc<RwLock<Node>>,
    query: ReadRepairMessage,
) -> Result<(), MyError> {
    if query.rows_to_keep.is_empty() {
        return Ok(());
    }
    let primary_keys = get_primary_key(&query.rows_to_keep[0]);

    let query_str = format!(
        "SELECT * FROM {} WHERE airport = {} AND date = {}",
        query.table, primary_keys[0], primary_keys[1]
    );

    let node = match node.read() {
        Ok(n) => n,
        Err(e) => {
            return Err(MyError::LockingError(e.to_string()));
        }
    };

    let mut _query = Query::new(query_str, node.ip.to_string());
    let rows_to_update = node.process_query(&mut _query)?;

    let (ap, place, date, flight, stat) = (0, 1, 2, 3, 4);

    let column: &str = if query.table == TABLE_ORIGIN_NAME {
        "destination"
    } else {
        "origin"
    };

    for row in &query.rows_to_keep {
        let row_vec = row.split(",").collect::<Vec<&str>>();

        if contains_row(&rows_to_update, row) {
            let query_str = format!(
                    "UPDATE {} SET status = {} WHERE airport = {} AND {} = {} AND date = {} AND flight = {}",
                    query.table, row_vec[stat], row_vec[ap], column, row_vec[place], row_vec[date], row_vec[flight]);
            let mut query = Query::new(query_str, node.ip.to_string());
            node.process_query(&mut query)?;
        } else {
            let query_str = format!(
                "INSERT INTO {} (airport, {}, date, flight, status) VALUES ({}, {}, {}, {}, {})",
                query.table,
                column,
                row_vec[ap],
                row_vec[place],
                row_vec[date],
                row_vec[flight],
                row_vec[stat]
            );
            let mut query = Query::new(query_str, node.ip.to_string());
            node.process_query(&mut query)?;
        }
    }

    for row in rows_to_update {
        let row_vec = row.split(",").collect::<Vec<&str>>();

        if !contains_row(&query.rows_to_keep, &row) {
            let query_str = format!(
                "DELETE FROM {} WHERE airport = {} AND {} = {} AND date = {} AND flight = {}",
                query.table, row_vec[ap], column, row_vec[place], row_vec[date], row_vec[flight]
            );
            let mut query = Query::new(query_str, node.ip.to_string());
            node.process_query(&mut query)?;
        }
    }
    Ok(())
}

fn contains_row(rows: &Vec<String>, row: &str) -> bool {
    let row_to_compare = get_primary_key(row);
    for r in rows {
        let keep_row = get_primary_key(r);

        if row_to_compare == keep_row {
            return true;
        }
    }
    false
}

/// Handler que se encarga de manejar la conexion del servidor con si mismo.
/// Funcion principal que recibe un nodo, un stream y un booleano.
pub fn handle_connection_server(
    node: Arc<RwLock<Node>>,
    stream: &mut TcpStream,
    sender: Sender<String>,
) -> Result<(), MyError> {
    match Packet::from_stream(stream, sender.clone()) {
        Ok(packet) => match packet.body {
            ServerMessage::Query(query) => handle_message_query(node, query, stream, sender),
            ServerMessage::ReadRepair(msg) => handle_message_read_repair(node, msg),
            _ => {
                let _error = "Error entre server y servers".to_string();
                Err(MyError::ServerMessageError(_error))
            }
        },
        Err(_e) => Err(_e),
    }
}
