use std::net::TcpStream;

use crate::{client_cql::Client, errors::my_errors::MyError};

/// Query para borrar una partición determianda de una una tabla.
pub fn get_partition_delete_query(table: &str, airport: &str, date: &str) -> String {
    format!(
        "DELETE FROM {} WHERE airport = {} AND date = {}",
        table, airport, date
    )
}

/// Query para insertar nuevos registros.
pub fn get_insert_query(table: &str, headers: & mut Vec<String>, line: & mut Vec<String>) -> String {    
    headers.pop();
    line.pop();
    format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table,
        headers.join(", "),
        line.join(", ")
    )
}

/// Query para crear la tabla indicada.
pub fn get_create_query(table: &str) -> String {
    match table {
        "origin" => "CREATE TABLE origin (airport STR, destination STR, date DATETIME, flight STR, status STR, timestamp TIMESTAMP, PRIMARY KEY (airport, date, flight)) WITH CLUSTERING ORDER BY (timestamp ASC)".to_string(),
        "destination" => "CREATE TABLE destination (airport STR, origin STR, date DATETIME, flight STR, status STR, timestamp TIMESTAMP, PRIMARY KEY (airport, date, flight)) WITH CLUSTERING ORDER BY (timestamp ASC)".to_string(),
        "flights_in_progress" => "CREATE TABLE flights_in_progress (flight STR, airport STR, destination STR, status STR, date DATETIME, speed STR, latitude STR, longitude STR, height STR, timestamp TIMESTAMP, PRIMARY KEY (number, origin, date) WITH CLUSTERING ORDER BY (timestamp ASC)".to_string(),
        _ => "".to_string()
    }
}

pub fn get_query_select_in_origin(date: String, airport: String, flight: String) -> String {
    format!(
        "SELECT * FROM origin WHERE date = {} AND airport = {} AND flight = {}",
        date, airport, flight
    )
}

pub fn get_query_select_in_destination(date: String, airport: String, flight: String) -> String {
    format!(
        "SELECT * FROM destination WHERE date = {} AND airport = {} AND flight = {}",
        date, airport, flight
    )
}

pub fn get_query_select_in_simulator(date: String, airport: String, flight: String) -> String {
    format!(
        "SELECT * FROM flights_in_progress WHERE date = {} AND airport = {} AND flight = {}",
        date, airport, flight
    )
}

pub fn make_query(query: String, consistency: u16, port: String) -> Result<Vec<String>, MyError> {
    let mut client = Client::initialize(port);
    let mut stream_server = if let Ok(stream) = client.connect() {
        stream
    } else {
        return Err(MyError::TcpStreamError("Error connecting  c client to server".to_string()));
    };
    if Client::send_message(&mut stream_server, query, consistency).is_ok() {
        if let Ok(response) = Client::read_response(&mut stream_server) {
            return Ok(response)
            }
        }
    Err(MyError::Error("Error al hacer la query".to_string()))
}