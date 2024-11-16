use super::flight::Flight;

pub fn get_query_create_simulator_table() -> String {
    "CREATE TABLE flights_in_progress (flight STR, airport STR, destination STR, status STR, date DATETIME, 
        speed STR, latitude STR, longitude STR, height STR, timestamp TIMESTAMP, PRIMARY KEY (number, origin, date) 
        WITH CLUSTERING ORDER BY (timestamp ASC)".to_string()
}

pub fn get_query_insert_in_origin(flight: &Flight) -> String {
    format!("INSERT INTO origin (airport, destination, date, flight, status) VALUES ({} {}, {}, {}, {})", 
        flight.origin_airport.name, flight.return_airport.name, flight.departure_date, flight.number, flight.status.to_str())
}

pub fn get_query_insert_in_destination(flight: &Flight) -> String {
    format!("INSERT INTO destination (airport, origin, date, flight, status) VALUES ({} {}, {}, {}, {})", 
        flight.return_airport.name, flight.origin_airport.name, flight.departure_date, flight.number, flight.status.to_str())
}

pub fn get_query_insert_in_simulator(flight: &Flight) -> String {
    format!("INSERT INTO flights_in_progress (flight, airport, destination, status, date, speed, latitude, longitude, height) 
        VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {})", 
        flight.number, flight.origin_airport.name, flight.return_airport.name, flight.status.to_str(), flight.departure_date,
        flight.speed, flight.latitude, flight.longitude, flight.height)
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

pub fn get_query_update_in_origin(flight: &Flight) -> String {
    format!(
        "UPDATE origin SET status = {} WHERE flight = {} AND airport = {} AND date = {}",
        flight.status.to_str(),
        flight.number,
        flight.origin_airport.name,
        flight.departure_date
    )
}

pub fn get_query_update_in_destination(flight: &Flight) -> String {
    format!(
        "UPDATE destination SET status = {} WHERE flight = {} AND airport = {} AND date = {}",
        flight.status.to_str(),
        flight.number,
        flight.return_airport.name,
        flight.departure_date
    )
}

pub fn get_query_update_in_simulator(flight: &Flight) -> String {
    format!(
        "UPDATE flights_in_progress SET status = {}, speed = {}, latitude = {}, longitude = {}, height = {} WHERE flight = {} AND airport = {} AND date = {}",
        flight.status.to_str(),
        flight.speed,
        flight.latitude,
        flight.longitude,
        flight.height,
        flight.number,
        flight.origin_airport.name,
        flight.departure_date
    )
}
