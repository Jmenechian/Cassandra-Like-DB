use super::{airport::Airport, flight::Flight};
use std::fs::read_to_string;
use crate::config::AIRPORTS_PATH;

pub fn get_airport_coordinates() -> Vec<Airport> {
    let mut airports: Vec<Airport> = vec![];
    for (index, line) in read_to_string(AIRPORTS_PATH).unwrap().lines().enumerate() {
        if index == 0 {
            continue;
        }
        let airport_data: Vec<&str> = line.split(",").collect();
        let airport_name = airport_data[0].to_string();
        let longitude: f32 = airport_data[2].parse::<f32>().unwrap();
        let latitude: f32 = airport_data[3].parse::<f32>().unwrap();
        let airport: Airport = Airport::new(airport_name, (latitude, longitude));
        airports.push(airport);
    }
    airports
}

// AGREGAR En las tablas de origen un campo que sea el destino y en las de destino una columna de origen

pub fn get_origin_airport_of(flights: &Vec<Flight>) -> Vec<Airport>{
    vec![]
}

pub fn get_return_airport_of(flights: &Vec<Flight>) -> Vec<Airport> {
    vec![]
}
